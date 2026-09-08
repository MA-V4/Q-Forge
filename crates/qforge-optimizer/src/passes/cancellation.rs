// Gate cancellation pass.
// Consecutive inverse gate pairs collapse to identity: H H, X X, CX CX etc.
// Phase 2 deliverable.

use crate::pass::{Pass, PassReport};
use qforge_ir::{Circuit, Gate};

pub struct GateCancellation;

impl Pass for GateCancellation {
    fn name(&self) -> &str { "gate_cancellation" }

    fn run(&self, mut circuit: Circuit) -> (Circuit, PassReport) {
        let before = circuit.gate_count();
        let mut changed = true;
        while changed {
            changed = false;
            let gates = std::mem::take(&mut circuit.gates);
            let mut out: Vec<Gate> = Vec::with_capacity(gates.len());
            let mut i = 0;
            while i < gates.len() {
                if i + 1 < gates.len() && are_inverse(&gates[i], &gates[i + 1]) {
                    i += 2;
                    changed = true;
                } else {
                    out.push(gates[i].clone());
                    i += 1;
                }
            }
            circuit.gates = out;
        }
        let after   = circuit.gate_count();
        let removed = before as i64 - after as i64;
        (circuit, PassReport {
            pass_name:     self.name().into(),
            gates_removed: removed,
            reason:        format!("cancelled {} inverse gate pair(s)", removed / 2),
        })
    }
}

fn are_inverse(a: &Gate, b: &Gate) -> bool {
    use Gate::*;
    match (a, b) {
        (H(q1),    H(q2))    => q1 == q2,
        (X(q1),    X(q2))    => q1 == q2,
        (Y(q1),    Y(q2))    => q1 == q2,
        (Z(q1),    Z(q2))    => q1 == q2,
        (S(q1),    Sdg(q2))  => q1 == q2,
        (Sdg(q1),  S(q2))    => q1 == q2,
        (T(q1),    Tdg(q2))  => q1 == q2,
        (Tdg(q1),  T(q2))    => q1 == q2,
        (Cx(c1,t1), Cx(c2,t2)) => c1 == c2 && t1 == t2,
        (Cz(q1a,q1b), Cz(q2a,q2b)) => q1a == q2a && q1b == q2b,
        (Rz(a, q1), Rz(b, q2)) => q1 == q2 && (a + b).abs() < 1e-10,
        (Rx(a, q1), Rx(b, q2)) => q1 == q2 && (a + b).abs() < 1e-10,
        (Ry(a, q1), Ry(b, q2)) => q1 == q2 && (a + b).abs() < 1e-10,
        _ => false,
    }
}
