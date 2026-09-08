use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QubitRef {
    pub register: String,
    pub index:    usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CbitRef {
    pub register: String,
    pub index:    usize,
}

impl QubitRef {
    pub fn new(register: impl Into<String>, index: usize) -> Self {
        Self { register: register.into(), index }
    }
}

impl CbitRef {
    pub fn new(register: impl Into<String>, index: usize) -> Self {
        Self { register: register.into(), index }
    }
}

impl std::fmt::Display for QubitRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}]", self.register, self.index)
    }
}
