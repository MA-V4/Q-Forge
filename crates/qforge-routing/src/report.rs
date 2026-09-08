use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingReport {
    pub swaps_inserted:    usize,
    pub swap_cost_in_cx:   usize,
    pub initial_mapping:   Vec<usize>,
    pub final_gate_count:  usize,
}