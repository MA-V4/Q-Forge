// Rotation merging pass.
// Sequential Rz(a) + Rz(b) on the same qubit -> Rz(a+b).
// Phase 2 deliverable.

use crate::pass::{Pass, PassReport};
use qforge_ir::{Circuit, Gate};

pub struct RotationMerging;

impl Pass for RotationMerging {
    fn name(&self) -> &str {
        "rotation_merging"
    }

    fn run(&self, mut circuit: Circuit) -> (Circuit, PassReport) {
        let before = circuit.gate_count();
        let gates = std::mem::take(&mut circuit.gates);
        let mut out: Vec<Gate> = Vec::with_capacity(gates.len());

        for gate in gates {
            if let Some(last) = out.last_mut() {
                match (&gate, &*last) {
                    (Gate::Rz(b, q2), Gate::Rz(a, q1)) if q1 == q2 => {
                        let merged = a + b;
                        *last = Gate::Rz(merged, q2.clone());
                        continue;
                    }
                    (Gate::Rx(b, q2), Gate::Rx(a, q1)) if q1 == q2 => {
                        *last = Gate::Rx(a + b, q2.clone());
                        continue;
                    }
                    (Gate::Ry(b, q2), Gate::Ry(a, q1)) if q1 == q2 => {
                        *last = Gate::Ry(a + b, q2.clone());
                        continue;
                    }
                    _ => {}
                }
            }
            out.push(gate);
        }

        // Remove zero rotations
        let out: Vec<Gate> = out
            .into_iter()
            .filter(|g| match g {
                Gate::Rz(a, _) | Gate::Rx(a, _) | Gate::Ry(a, _) => a.abs() > 1e-10,
                _ => true,
            })
            .collect();

        circuit.gates = out;
        let removed = before as i64 - circuit.gate_count() as i64;
        (
            circuit,
            PassReport {
                pass_name: self.name().into(),
                gates_removed: removed,
                reason: format!("merged {} rotation(s)", removed),
            },
        )
    }
}
