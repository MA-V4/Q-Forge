use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationReport {
    pub input_gates:  usize,
    pub output_gates: usize,
    pub input_depth:  usize,
    pub output_depth: usize,
    pub input_2q:     usize,
    pub output_2q:    usize,
    pub passes:       Vec<PassSummary>,
    pub time_ms:      u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassSummary {
    pub name:          String,
    pub gates_removed: i64,
    pub reason:        String,
}

impl OptimizationReport {
    pub fn gate_reduction_pct(&self) -> f64 {
        if self.input_gates == 0 { return 0.0; }
        ((self.input_gates as f64 - self.output_gates as f64) / self.input_gates as f64) * 100.0
    }

    pub fn depth_reduction_pct(&self) -> f64 {
        if self.input_depth == 0 { return 0.0; }
        ((self.input_depth as f64 - self.output_depth as f64) / self.input_depth as f64) * 100.0
    }
}
