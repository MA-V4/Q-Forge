use serde::{Deserialize, Serialize};

/// Per-qubit error characteristics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QubitError {
    pub qubit: usize,
    pub t1_us: f64,       // T1 relaxation time in microseconds
    pub t2_us: f64,       // T2 dephasing time in microseconds
    pub readout_err: f64, // Measurement error probability
}

/// Per-gate error rate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateError {
    pub gate: String,
    pub qubits: Vec<usize>,
    pub error: f64,       // Average gate error probability
    pub duration_ns: f64, // Gate duration in nanoseconds
}

/// Noise model for a quantum hardware backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseModel {
    pub name: String,
    pub qubit_count: usize,
    pub qubits: Vec<QubitError>,
    pub gates: Vec<GateError>,
}

impl NoiseModel {
    /// IBM Brisbane-like noise model for demonstration.
    pub fn ibm_brisbane_like(n_qubits: usize) -> Self {
        let mut qubits = Vec::new();
        for i in 0..n_qubits {
            qubits.push(QubitError {
                qubit: i,
                t1_us: (150.0 + (i as f64 * 7.3) % 80.0),
                t2_us: (90.0 + (i as f64 * 4.1) % 60.0),
                readout_err: 0.01 + (i as f64 * 0.003) % 0.04,
            });
        }

        let mut gates = Vec::new();

        // Single-qubit gates
        for i in 0..n_qubits {
            gates.push(GateError {
                gate: "sx".into(),
                qubits: vec![i],
                error: 0.0002 + (i as f64 * 0.00003) % 0.0003,
                duration_ns: 35.5,
            });
            gates.push(GateError {
                gate: "rz".into(),
                qubits: vec![i],
                error: 0.0, // virtual gate, zero error
                duration_ns: 0.0,
            });
        }

        // Two-qubit CX gates (linear coupling)
        for i in 0..n_qubits.saturating_sub(1) {
            gates.push(GateError {
                gate: "cx".into(),
                qubits: vec![i, i + 1],
                error: 0.005 + (i as f64 * 0.0008) % 0.008,
                duration_ns: 533.0,
            });
        }

        Self {
            name: format!("ibm-brisbane-like-{}", n_qubits),
            qubit_count: n_qubits,
            qubits,
            gates,
        }
    }

    /// Load from IBM Quantum JSON format.
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn single_qubit_error(&self, qubit: usize) -> f64 {
        self.gates
            .iter()
            .filter(|g| g.gate == "sx" && g.qubits == vec![qubit])
            .map(|g| g.error)
            .next()
            .unwrap_or(0.001)
    }

    pub fn two_qubit_error(&self, q0: usize, q1: usize) -> f64 {
        self.gates
            .iter()
            .filter(|g| {
                g.gate == "cx" && ((g.qubits == vec![q0, q1]) || (g.qubits == vec![q1, q0]))
            })
            .map(|g| g.error)
            .next()
            .unwrap_or(0.01)
    }

    pub fn readout_error(&self, qubit: usize) -> f64 {
        self.qubits
            .iter()
            .find(|q| q.qubit == qubit)
            .map(|q| q.readout_err)
            .unwrap_or(0.02)
    }
}
