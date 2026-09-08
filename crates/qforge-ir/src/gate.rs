use crate::qubit::{CbitRef, QubitRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Gate {
    // Single-qubit Clifford
    H(QubitRef),
    X(QubitRef),
    Y(QubitRef),
    Z(QubitRef),
    S(QubitRef),
    Sdg(QubitRef),
    T(QubitRef),
    Tdg(QubitRef),
    // Parameterized single-qubit
    Rx(f64, QubitRef),
    Ry(f64, QubitRef),
    Rz(f64, QubitRef),
    U1(f64, QubitRef),
    U2(f64, f64, QubitRef),
    U3(f64, f64, f64, QubitRef),
    // Two-qubit
    Cx(QubitRef, QubitRef),
    Cz(QubitRef, QubitRef),
    Swap(QubitRef, QubitRef),
    // Three-qubit
    Ccx(QubitRef, QubitRef, QubitRef),
    // Measurement
    Measure(QubitRef, CbitRef),
    // Structural
    Barrier(Vec<QubitRef>),
    Reset(QubitRef),
    // Custom gate call
    Custom {
        name: String,
        params: Vec<f64>,
        qubits: Vec<QubitRef>,
    },
}

impl Gate {
    pub fn qubit_count(&self) -> usize {
        match self {
            Gate::H(_)
            | Gate::X(_)
            | Gate::Y(_)
            | Gate::Z(_)
            | Gate::S(_)
            | Gate::Sdg(_)
            | Gate::T(_)
            | Gate::Tdg(_)
            | Gate::Rx(_, _)
            | Gate::Ry(_, _)
            | Gate::Rz(_, _)
            | Gate::U1(_, _)
            | Gate::U2(_, _, _)
            | Gate::U3(_, _, _, _)
            | Gate::Measure(_, _)
            | Gate::Reset(_) => 1,
            Gate::Cx(_, _) | Gate::Cz(_, _) | Gate::Swap(_, _) => 2,
            Gate::Ccx(_, _, _) => 3,
            Gate::Barrier(qs) => qs.len(),
            Gate::Custom { qubits, .. } => qubits.len(),
        }
    }

    pub fn is_two_qubit(&self) -> bool {
        matches!(self, Gate::Cx(_, _) | Gate::Cz(_, _) | Gate::Swap(_, _))
    }

    pub fn is_measurement(&self) -> bool {
        matches!(self, Gate::Measure(_, _))
    }

    pub fn name(&self) -> &str {
        match self {
            Gate::H(_) => "h",
            Gate::X(_) => "x",
            Gate::Y(_) => "y",
            Gate::Z(_) => "z",
            Gate::S(_) => "s",
            Gate::Sdg(_) => "sdg",
            Gate::T(_) => "t",
            Gate::Tdg(_) => "tdg",
            Gate::Rx(_, _) => "rx",
            Gate::Ry(_, _) => "ry",
            Gate::Rz(_, _) => "rz",
            Gate::U1(_, _) => "u1",
            Gate::U2(_, _, _) => "u2",
            Gate::U3(_, _, _, _) => "u3",
            Gate::Cx(_, _) => "cx",
            Gate::Cz(_, _) => "cz",
            Gate::Swap(_, _) => "swap",
            Gate::Ccx(_, _, _) => "ccx",
            Gate::Measure(_, _) => "measure",
            Gate::Barrier(_) => "barrier",
            Gate::Reset(_) => "reset",
            Gate::Custom { name, .. } => name,
        }
    }
}
