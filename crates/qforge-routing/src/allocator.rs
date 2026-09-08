// Qubit allocation: map logical circuit qubits to physical hardware qubits.
// Strategy: graph-based initial mapping that minimises the total distance
// between qubits that interact in the circuit.

use crate::topology::HardwareTopology;
use qforge_ir::{Circuit, Gate};
use std::collections::HashMap;

/// Returns a mapping from logical qubit index to physical qubit index.
pub fn allocate_qubits(circuit: &Circuit, topology: &HardwareTopology) -> Vec<usize> {
    let logical_count = circuit.qubit_count();
    assert!(
        logical_count <= topology.qubit_count,
        "circuit requires {} qubits but topology only has {}",
        logical_count,
        topology.qubit_count
    );

    // Build interaction frequency map: how often do logical qubits interact?
    let mut interactions: HashMap<(usize, usize), usize> = HashMap::new();
    let logical_index = build_logical_index(circuit);

    for gate in &circuit.gates {
        if let Some((q0, q1)) = two_qubit_logical_indices(gate, &logical_index) {
            let key = (q0.min(q1), q0.max(q1));
            *interactions.entry(key).or_insert(0) += 1;
        }
    }

    // Greedy allocation: place most-interacting pairs on adjacent physical qubits.
    let dist = topology.distance_matrix();
    let mut mapping = vec![usize::MAX; logical_count];
    let mut used: std::collections::HashSet<usize> = std::collections::HashSet::new();

    // Sort interactions by frequency descending
    let mut pairs: Vec<((usize, usize), usize)> = interactions.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1));

    for ((l0, l1), _) in &pairs {
        if mapping[*l0] != usize::MAX && mapping[*l1] != usize::MAX { continue; }

        if mapping[*l0] == usize::MAX && mapping[*l1] == usize::MAX {
            // Find the best adjacent physical pair
            let best = best_adjacent_pair(topology, &used);
            mapping[*l0] = best.0;
            mapping[*l1] = best.1;
            used.insert(best.0);
            used.insert(best.1);
        } else if mapping[*l0] != usize::MAX {
            // l0 is placed, find best neighbour for l1
            let p0 = mapping[*l0];
            let nb = topology.neighbours(p0)
                .into_iter()
                .filter(|p| !used.contains(p))
                .min_by_key(|p| dist[p0][*p])
                .unwrap_or_else(|| first_free(topology.qubit_count, &used));
            mapping[*l1] = nb;
            used.insert(nb);
        } else {
            // l1 is placed, find best neighbour for l0
            let p1 = mapping[*l1];
            let nb = topology.neighbours(p1)
                .into_iter()
                .filter(|p| !used.contains(p))
                .min_by_key(|p| dist[p1][*p])
                .unwrap_or_else(|| first_free(topology.qubit_count, &used));
            mapping[*l0] = nb;
            used.insert(nb);
        }
    }

    // Fill any unmapped logical qubits with remaining physical qubits
    for m in &mut mapping {
        if *m == usize::MAX {
            *m = first_free(topology.qubit_count, &used);
            used.insert(*m);
        }
    }

    mapping
}

fn best_adjacent_pair(topology: &HardwareTopology, used: &std::collections::HashSet<usize>) -> (usize, usize) {
    for edge in &topology.coupling_map {
        if !used.contains(&edge.source) && !used.contains(&edge.target) {
            return (edge.source, edge.target);
        }
    }
    // Fallback: any two free qubits
    let free: Vec<usize> = (0..topology.qubit_count).filter(|q| !used.contains(q)).collect();
    (free[0], if free.len() > 1 { free[1] } else { free[0] })
}

fn first_free(n: usize, used: &std::collections::HashSet<usize>) -> usize {
    (0..n).find(|q| !used.contains(q)).unwrap_or(0)
}

pub fn build_logical_index(circuit: &Circuit) -> HashMap<String, usize> {
    let mut idx = 0;
    let mut map = HashMap::new();
    for (reg, size) in &circuit.qregs {
        for i in 0..*size {
            map.insert(format!("{}_{}", reg, i), idx);
            idx += 1;
        }
    }
    map
}

fn two_qubit_logical_indices(
    gate: &Gate,
    logical_index: &HashMap<String, usize>,
) -> Option<(usize, usize)> {
    let key = |r: &str, i: usize| format!("{}_{}", r, i);
    match gate {
        Gate::Cx(c, t) | Gate::Cz(c, t) | Gate::Swap(c, t) => {
            let l0 = *logical_index.get(&key(&c.register, c.index))?;
            let l1 = *logical_index.get(&key(&t.register, t.index))?;
            Some((l0, l1))
        }
        Gate::Ccx(a, b, c) => {
            let l0 = *logical_index.get(&key(&a.register, a.index))?;
            let l1 = *logical_index.get(&key(&b.register, b.index))?;
            let _ = logical_index.get(&key(&c.register, c.index))?;
            Some((l0, l1))
        }
        _ => None,
    }
}