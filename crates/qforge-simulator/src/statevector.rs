// Statevector simulator.
// Represents |psi> as a Vec<Complex64> with 2^n amplitudes.
// Phase 5 deliverable.

use qforge_ir::Circuit;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub shots:    usize,
    pub counts:   std::collections::HashMap<String, usize>,
    pub fidelity: Option<f64>,
}

pub fn simulate(_circuit: &Circuit, _shots: usize) -> SimulationResult {
    todo!("Phase 5 — statevector simulation")
}
