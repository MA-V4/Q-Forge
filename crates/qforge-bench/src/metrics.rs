use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name:                String,
    pub circuit_file:        String,
    pub input_qubits:        usize,
    pub input_gates:         usize,
    pub input_depth:         usize,
    pub input_2q:            usize,
    pub output_gates:        usize,
    pub output_depth:        usize,
    pub output_2q:           usize,
    pub gate_reduction_pct:  f64,
    pub depth_reduction_pct: f64,
    pub fidelity_pct:        f64,
    pub compile_ms:          u128,
    pub status:              String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuite {
    pub results:             Vec<BenchmarkResult>,
    pub total_circuits:      usize,
    pub avg_gate_reduction:  f64,
    pub avg_fidelity:        f64,
    pub total_compile_ms:    u128,
    pub qforge_version:      String,
}

impl BenchmarkSuite {
    pub fn new(results: Vec<BenchmarkResult>) -> Self {
        let n = results.len() as f64;
        let avg_gate_reduction = if n > 0.0 {
            results.iter().map(|r| r.gate_reduction_pct).sum::<f64>() / n
        } else { 0.0 };

        let avg_fidelity = if n > 0.0 {
            results.iter().map(|r| r.fidelity_pct).sum::<f64>() / n
        } else { 0.0 };

        let total_compile_ms = results.iter().map(|r| r.compile_ms).sum();
        let total_circuits   = results.len();

        Self {
            results,
            total_circuits,
            avg_gate_reduction,
            avg_fidelity,
            total_compile_ms,
            qforge_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}