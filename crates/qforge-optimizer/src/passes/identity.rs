// Identity elimination pass.
// Remove gates that are provably identity operations.
// Phase 2 deliverable.

use crate::pass::{Pass, PassReport};
use qforge_ir::{Circuit, Gate};

pub struct IdentityElimination;

impl Pass for IdentityElimination {
    fn name(&self) -> &str {
        "identity_elimination"
    }

    fn run(&self, mut circuit: Circuit) -> (Circuit, PassReport) {
        let before = circuit.gate_count();
        circuit.gates.retain(|g| !is_identity(g));
        let removed = before as i64 - circuit.gate_count() as i64;
        (
            circuit,
            PassReport {
                pass_name: self.name().into(),
                gates_removed: removed,
                reason: format!("removed {} identity gate(s)", removed),
            },
        )
    }
}

fn is_identity(gate: &Gate) -> bool {
    match gate {
        Gate::Rz(a, _) | Gate::Rx(a, _) | Gate::Ry(a, _) => a.abs() < 1e-10,
        Gate::U1(a, _) => a.abs() < 1e-10,
        _ => false,
    }
}
