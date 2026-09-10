use crate::gate::Gate;
use crate::qubit::QubitRef;
use std::collections::HashMap;

/// Compute the true circuit depth via critical-path analysis on the gate DAG.
/// Depth = length of the longest chain of gates with data dependencies.
pub fn compute_depth(gates: &[Gate]) -> usize {
    if gates.is_empty() {
        return 0;
    }

    let mut qubit_layer: HashMap<String, usize> = HashMap::new();
    let mut max_depth = 0usize;

    for gate in gates {
        if matches!(gate, Gate::Barrier(_)) {
            continue;
        }

        let qubits = gate_qubits(gate);
        if qubits.is_empty() {
            continue;
        }

        let earliest = qubits
            .iter()
            .map(|q| qubit_layer.get(&qubit_key(q)).copied().unwrap_or(0))
            .max()
            .unwrap_or(0);

        let layer = earliest + 1;
        for q in &qubits {
            qubit_layer.insert(qubit_key(q), layer);
        }
        if layer > max_depth {
            max_depth = layer;
        }
    }

    max_depth
}

fn qubit_key(q: &QubitRef) -> String {
    format!("{}_{}", q.register, q.index)
}

fn gate_qubits(gate: &Gate) -> Vec<QubitRef> {
    match gate {
        Gate::H(q)
        | Gate::X(q)
        | Gate::Y(q)
        | Gate::Z(q)
        | Gate::S(q)
        | Gate::Sdg(q)
        | Gate::T(q)
        | Gate::Tdg(q)
        | Gate::Sx(q)
        | Gate::Sxdg(q)
        | Gate::Rx(_, q)
        | Gate::Ry(_, q)
        | Gate::Rz(_, q)
        | Gate::U1(_, q)
        | Gate::Reset(q) => vec![q.clone()],

        Gate::U2(_, _, q) | Gate::U3(_, _, _, q) => vec![q.clone()],

        Gate::Cx(c, t) | Gate::Cz(c, t) | Gate::Swap(c, t) => vec![c.clone(), t.clone()],

        Gate::Ccx(a, b, c) => vec![a.clone(), b.clone(), c.clone()],

        Gate::Measure(q, _) => vec![q.clone()],

        Gate::Barrier(qs) => qs.clone(),

        Gate::Custom { qubits, .. } => qubits.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate::Gate;
    use crate::qubit::QubitRef;

    fn q(i: usize) -> QubitRef {
        QubitRef::new("q", i)
    }

    #[test]
    fn empty_circuit_depth_zero() {
        assert_eq!(compute_depth(&[]), 0);
    }

    #[test]
    fn single_gate_depth_one() {
        assert_eq!(compute_depth(&[Gate::H(q(0))]), 1);
    }

    #[test]
    fn parallel_gates_depth_one() {
        let gates = vec![Gate::H(q(0)), Gate::H(q(1))];
        assert_eq!(compute_depth(&gates), 1);
    }

    #[test]
    fn serial_gates_same_qubit() {
        let gates = vec![Gate::H(q(0)), Gate::X(q(0))];
        assert_eq!(compute_depth(&gates), 2);
    }

    #[test]
    fn cx_depends_on_both_qubits() {
        let gates = vec![Gate::H(q(0)), Gate::H(q(1)), Gate::Cx(q(0), q(1))];
        assert_eq!(compute_depth(&gates), 2);
    }

    #[test]
    fn bell_state_depth() {
        let gates = vec![Gate::H(q(0)), Gate::Cx(q(0), q(1))];
        assert_eq!(compute_depth(&gates), 2);
    }

    #[test]
    fn barriers_ignored_in_depth() {
        let gates = vec![
            Gate::H(q(0)),
            Gate::Barrier(vec![q(0), q(1)]),
            Gate::H(q(1)),
        ];
        assert_eq!(compute_depth(&gates), 1);
    }

    #[test]
    fn ghz_depth() {
        let gates = vec![Gate::H(q(0)), Gate::Cx(q(0), q(1)), Gate::Cx(q(0), q(2))];
        assert_eq!(compute_depth(&gates), 3);
    }
}
