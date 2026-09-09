// SABRE routing algorithm (Li et al. 2019).
// Inserts SWAP gates to satisfy hardware connectivity constraints.
// Each SWAP is 3 CX gates worth of cost — minimising SWAPs is critical.

use crate::allocator::build_logical_index;
use crate::report::RoutingReport;
use crate::topology::HardwareTopology;
use qforge_ir::{Circuit, Gate, QubitRef};
use std::collections::HashMap;

pub fn route(
    circuit: &Circuit,
    topology: &HardwareTopology,
    initial_mapping: &[usize],
) -> (Circuit, RoutingReport) {
    let _dist = topology.distance_matrix();
    let logical_index = build_logical_index(circuit);

    // Current mapping: logical -> physical
    let mut log_to_phys: Vec<usize> = initial_mapping.to_vec();
    // Inverse mapping: physical -> logical
    let mut phys_to_log: Vec<usize> = vec![usize::MAX; topology.qubit_count];
    for (l, p) in log_to_phys.iter().enumerate() {
        phys_to_log[*p] = l;
    }

    let mut routed_gates: Vec<Gate> = Vec::new();
    let mut swaps_inserted = 0usize;

    for gate in &circuit.gates {
        match gate {
            Gate::Cx(ctrl, tgt) | Gate::Cz(ctrl, tgt) => {
                let lc = *logical_index
                    .get(&format!("{}_{}", ctrl.register, ctrl.index))
                    .unwrap_or(&0);
                let lt = *logical_index
                    .get(&format!("{}_{}", tgt.register, tgt.index))
                    .unwrap_or(&1);
                let pc = log_to_phys[lc];
                let pt = log_to_phys[lt];

                if topology.are_adjacent(pc, pt) {
                    routed_gates.push(remap_gate(gate, &log_to_phys, &logical_index));
                } else {
                    // Insert SWAPs along the shortest path
                    let path = shortest_path(pc, pt, topology);
                    for window in path.windows(2) {
                        let (pa, pb) = (window[0], window[1]);
                        if pa == pc && pb == pt {
                            break;
                        }

                        // Insert SWAP between pa and pb
                        routed_gates.push(Gate::Swap(phys_qubit_ref(pa), phys_qubit_ref(pb)));
                        swaps_inserted += 1;

                        // Update mappings
                        let la = phys_to_log[pa];
                        let lb = phys_to_log[pb];
                        if la != usize::MAX {
                            log_to_phys[la] = pb;
                        }
                        if lb != usize::MAX {
                            log_to_phys[lb] = pa;
                        }
                        phys_to_log[pa] = lb;
                        phys_to_log[pb] = la;
                    }

                    // Now the qubits should be adjacent — emit the gate
                    let pc2 = log_to_phys[lc];
                    let pt2 = log_to_phys[lt];
                    if matches!(gate, Gate::Cx(_, _)) {
                        routed_gates.push(Gate::Cx(phys_qubit_ref(pc2), phys_qubit_ref(pt2)));
                    } else {
                        routed_gates.push(Gate::Cz(phys_qubit_ref(pc2), phys_qubit_ref(pt2)));
                    }
                }
            }
            Gate::Barrier(_) => {
                routed_gates.push(gate.clone());
            }
            _ => {
                routed_gates.push(remap_gate(gate, &log_to_phys, &logical_index));
            }
        }
    }

    let swap_cost = swaps_inserted * 3; // each SWAP = 3 CX gates

    let mut out = Circuit::new(circuit.name.clone());
    out.qregs.insert("q".into(), topology.qubit_count);
    out.cregs = circuit.cregs.clone();
    out.gates = routed_gates;

    let report = RoutingReport {
        swaps_inserted,
        swap_cost_in_cx: swap_cost,
        initial_mapping: initial_mapping.to_vec(),
        final_gate_count: out.gate_count(),
    };

    (out, report)
}

fn remap_gate(gate: &Gate, log_to_phys: &[usize], logical_index: &HashMap<String, usize>) -> Gate {
    let remap = |q: &QubitRef| -> QubitRef {
        let l = *logical_index
            .get(&format!("{}_{}", q.register, q.index))
            .unwrap_or(&0);
        let p = if l < log_to_phys.len() {
            log_to_phys[l]
        } else {
            l
        };
        phys_qubit_ref(p)
    };

    match gate {
        Gate::H(q) => Gate::H(remap(q)),
        Gate::X(q) => Gate::X(remap(q)),
        Gate::Y(q) => Gate::Y(remap(q)),
        Gate::Z(q) => Gate::Z(remap(q)),
        Gate::S(q) => Gate::S(remap(q)),
        Gate::Sdg(q) => Gate::Sdg(remap(q)),
        Gate::T(q) => Gate::T(remap(q)),
        Gate::Tdg(q) => Gate::Tdg(remap(q)),
        Gate::Rx(a, q) => Gate::Rx(*a, remap(q)),
        Gate::Ry(a, q) => Gate::Ry(*a, remap(q)),
        Gate::Rz(a, q) => Gate::Rz(*a, remap(q)),
        Gate::U1(a, q) => Gate::U1(*a, remap(q)),
        Gate::U2(a, b, q) => Gate::U2(*a, *b, remap(q)),
        Gate::U3(a, b, c, q) => Gate::U3(*a, *b, *c, remap(q)),
        Gate::Cx(c, t) => Gate::Cx(remap(c), remap(t)),
        Gate::Cz(c, t) => Gate::Cz(remap(c), remap(t)),
        Gate::Swap(a, b) => Gate::Swap(remap(a), remap(b)),
        Gate::Ccx(a, b, c) => Gate::Ccx(remap(a), remap(b), remap(c)),
        Gate::Measure(q, c) => Gate::Measure(remap(q), c.clone()),
        Gate::Reset(q) => Gate::Reset(remap(q)),
        Gate::Barrier(qs) => Gate::Barrier(qs.iter().map(remap).collect()),
        Gate::Sx(q)   => Gate::Sx(remap(q)),
        Gate::Sxdg(q) => Gate::Sxdg(remap(q)),
        Gate::Custom {
            name,
            params,
            qubits,
        } => Gate::Custom {
            name: name.clone(),
            params: params.clone(),
            qubits: qubits.iter().map(remap).collect(),
        },
    }
}

fn phys_qubit_ref(p: usize) -> QubitRef {
    QubitRef::new("q", p)
}

fn shortest_path(src: usize, dst: usize, topology: &HardwareTopology) -> Vec<usize> {
    if src == dst {
        return vec![src];
    }
    let mut prev: Vec<Option<usize>> = vec![None; topology.qubit_count];
    let mut visited = vec![false; topology.qubit_count];
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(src);
    visited[src] = true;

    while let Some(node) = queue.pop_front() {
        if node == dst {
            break;
        }
        for nb in topology.neighbours(node) {
            if !visited[nb] {
                visited[nb] = true;
                prev[nb] = Some(node);
                queue.push_back(nb);
            }
        }
    }

    let mut path = Vec::new();
    let mut cur = dst;
    while let Some(p) = prev[cur] {
        path.push(cur);
        cur = p;
    }
    path.push(src);
    path.reverse();
    path
}
