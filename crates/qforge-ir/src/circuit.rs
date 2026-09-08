use crate::dag::compute_depth;
use crate::gate::Gate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circuit {
    pub name:     String,
    pub qregs:    HashMap<String, usize>,
    pub cregs:    HashMap<String, usize>,
    pub gates:    Vec<Gate>,
    pub metadata: CircuitMetadata,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CircuitMetadata {
    pub source:  Option<String>,
    pub version: Option<String>,
}

impl Circuit {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name:     name.into(),
            qregs:    HashMap::new(),
            cregs:    HashMap::new(),
            gates:    Vec::new(),
            metadata: CircuitMetadata::default(),
        }
    }

    pub fn qubit_count(&self) -> usize {
        self.qregs.values().sum()
    }

    pub fn gate_count(&self) -> usize {
        self.gates.iter().filter(|g| !matches!(g, Gate::Barrier(_))).count()
    }

    pub fn two_qubit_gate_count(&self) -> usize {
        self.gates.iter().filter(|g| g.is_two_qubit()).count()
    }

    pub fn depth(&self) -> usize {
        compute_depth(&self.gates)
    }

    pub fn push(&mut self, gate: Gate) {
        self.gates.push(gate);
    }
}