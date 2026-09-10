// Commutation analysis pass.
// Reorders gates that commute to bring cancellable pairs adjacent.
// After this pass, gate_cancellation finds pairs it would have missed.
//
// Commutation rules:
//   - Two single-qubit gates on DIFFERENT qubits always commute.
//   - Diagonal gates (Z, S, T, Rz) commute with each other on the SAME qubit.
//   - Diagonal gates commute past the CONTROL of a CX (not the target).
//   - X, Rx commute past the TARGET of a CX (not the control).

use crate::pass::{Pass, PassReport};
use qforge_ir::{Circuit, Gate, QubitRef};
use std::collections::HashSet;

pub struct CommutationAnalysis;

impl Pass for CommutationAnalysis {
    fn name(&self) -> &str {
        "commutation_analysis"
    }

    fn run(&self, mut circuit: Circuit) -> (Circuit, PassReport) {
        let before = circuit.gate_count();
        let mut changed = true;

        while changed {
            changed = false;
            let gates = std::mem::take(&mut circuit.gates);
            let n = gates.len();
            let mut skip: HashSet<usize> = HashSet::new();
            let mut out: Vec<Gate> = Vec::with_capacity(n);

            let mut i = 0;
            while i < n {
                if skip.contains(&i) {
                    i += 1;
                    continue;
                }

                // Try to find a later gate j that:
                // (a) commutes with gates[i]
                // (b) together with gates[i] forms a cancellable pair
                let mut found = false;
                let mut j = i + 1;

                while j < n && j < i + 8 {
                    // limit lookahead to 8 gates
                    if skip.contains(&j) {
                        j += 1;
                        continue;
                    }

                    if are_inverse(&gates[i], &gates[j]) {
                        // Check all gates between i and j commute with gates[i]
                        let path_clear = (i + 1..j)
                            .filter(|k| !skip.contains(k))
                            .all(|k| commutes(&gates[i], &gates[k]));

                        if path_clear {
                            // Cancel gates[i] and gates[j] — skip both
                            skip.insert(i);
                            skip.insert(j);
                            found = true;
                            changed = true;
                            break;
                        }
                    }

                    // If gates[i] does NOT commute with gates[j], stop lookahead
                    if !commutes(&gates[i], &gates[j]) {
                        break;
                    }

                    j += 1;
                }

                if !found {
                    out.push(gates[i].clone());
                }
                i += 1;
            }

            circuit.gates = out;
        }

        let removed = before as i64 - circuit.gate_count() as i64;
        (
            circuit,
            PassReport {
                pass_name: self.name().into(),
                gates_removed: removed,
                reason: format!("commuted and cancelled {} gate(s)", removed),
            },
        )
    }
}

fn are_inverse(a: &Gate, b: &Gate) -> bool {
    use Gate::*;
    match (a, b) {
        (H(q1), H(q2)) => q1 == q2,
        (X(q1), X(q2)) => q1 == q2,
        (Y(q1), Y(q2)) => q1 == q2,
        (Z(q1), Z(q2)) => q1 == q2,
        (S(q1), Sdg(q2)) => q1 == q2,
        (Sdg(q1), S(q2)) => q1 == q2,
        (T(q1), Tdg(q2)) => q1 == q2,
        (Tdg(q1), T(q2)) => q1 == q2,
        (Cx(c1, t1), Cx(c2, t2)) => c1 == c2 && t1 == t2,
        (Cz(a1, b1), Cz(a2, b2)) => a1 == a2 && b1 == b2,
        (Rz(a, q1), Rz(b, q2)) => q1 == q2 && (a + b).abs() < 1e-10,
        (Rx(a, q1), Rx(b, q2)) => q1 == q2 && (a + b).abs() < 1e-10,
        (Ry(a, q1), Ry(b, q2)) => q1 == q2 && (a + b).abs() < 1e-10,
        _ => false,
    }
}

/// Returns true if gate a and gate b can be swapped without changing circuit semantics.
pub fn commutes(a: &Gate, b: &Gate) -> bool {
    let qa = qubit_set(a);
    let qb = qubit_set(b);

    // Disjoint qubit sets: always commute
    if qa.is_disjoint(&qb) {
        return true;
    }

    use Gate::*;
    match (a, b) {
        // Diagonal gates commute with each other on same qubit
        (Rz(_, q1), Rz(_, q2)) if q1 == q2 => true,
        (Z(q1), Z(q2)) if q1 == q2 => true,
        (Z(q1), Rz(_, q2)) if q1 == q2 => true,
        (Rz(_, q1), Z(q2)) if q1 == q2 => true,
        (S(q1), T(q2)) if q1 == q2 => true,
        (S(q1), Rz(_, q2)) if q1 == q2 => true,
        (T(q1), Rz(_, q2)) if q1 == q2 => true,

        // Diagonal gates commute past the CONTROL of CX
        (Rz(_, q1), Cx(ctrl, _)) if q1 == ctrl => true,
        (Cx(ctrl, _), Rz(_, q1)) if q1 == ctrl => true,
        (Z(q1), Cx(ctrl, _)) if q1 == ctrl => true,
        (Cx(ctrl, _), Z(q1)) if q1 == ctrl => true,

        // X and Rx commute past the TARGET of CX
        (X(q1), Cx(_, tgt)) if q1 == tgt => true,
        (Cx(_, tgt), X(q1)) if q1 == tgt => true,
        (Rx(_, q1), Cx(_, tgt)) if q1 == tgt => true,
        (Cx(_, tgt), Rx(_, q1)) if q1 == tgt => true,

        // Same gate applied twice commutes trivially
        _ if std::mem::discriminant(a) == std::mem::discriminant(b)
            && qubit_set(a) == qubit_set(b) =>
        {
            true
        }

        _ => false,
    }
}

fn qubit_set(gate: &Gate) -> HashSet<String> {
    gate_qubits(gate)
        .into_iter()
        .map(|q| format!("{}_{}", q.register, q.index))
        .collect()
}

fn gate_qubits(gate: &Gate) -> Vec<QubitRef> {
    use Gate::*;
    match gate {
        H(q)
        | X(q)
        | Y(q)
        | Z(q)
        | S(q)
        | Sdg(q)
        | T(q)
        | Tdg(q)
        | Sx(q)
        | Sxdg(q)
        | Rx(_, q)
        | Ry(_, q)
        | Rz(_, q)
        | U1(_, q)
        | Reset(q) => vec![q.clone()],
        U2(_, _, q) | U3(_, _, _, q) => vec![q.clone()],
        Cx(c, t) | Cz(c, t) | Swap(c, t) => vec![c.clone(), t.clone()],
        Ccx(a, b, c) => vec![a.clone(), b.clone(), c.clone()],
        Measure(q, _) => vec![q.clone()],
        Barrier(qs) => qs.clone(),
        Custom { qubits, .. } => qubits.clone(),
    }
}
