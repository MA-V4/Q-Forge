// Gate fusion pass. Phase 2 deliverable.
pub struct GateFusion;
use crate::pass::{Pass, PassReport};
use qforge_ir::Circuit;

impl Pass for GateFusion {
    fn name(&self) -> &str { "gate_fusion" }
    fn run(&self, circuit: Circuit) -> (Circuit, PassReport) {
        (circuit, PassReport {
            pass_name: self.name().into(),
            gates_removed: 0,
            reason: "Phase 2 — gate fusion".into(),
        })
    }
}
