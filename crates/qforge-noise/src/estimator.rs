// Fidelity estimator.
// Given a compiled circuit and a noise model, estimates the probability
// that the circuit executes correctly (process fidelity approximation).
//
// Model: F = product of (1 - error_i) for each gate and measurement.
// This is a lower bound — real fidelity also depends on coherence time
// and crosstalk, which we approximate via T1/T2 and circuit duration.

use crate::model::NoiseModel;
use qforge_ir::{Circuit, Gate};

#[derive(Debug, Clone)]
pub struct FidelityEstimate {
    pub fidelity:          f64,
    pub gate_error_budget: f64,
    pub readout_error:     f64,
    pub decoherence_error: f64,
    pub total_duration_ns: f64,
}

pub fn estimate_fidelity(circuit: &Circuit, noise: &NoiseModel) -> FidelityEstimate {
    let mut fidelity          = 1.0f64;
    let mut gate_error_budget = 1.0f64;
    let mut total_duration    = 0.0f64;

    for gate in &circuit.gates {
        let (error, duration) = gate_error(gate, noise);
        fidelity          *= 1.0 - error;
        gate_error_budget *= 1.0 - error;
        total_duration    += duration;
    }

    // Readout error — applied once per measured qubit
    let mut readout_fidelity = 1.0f64;
    let measured_qubits: Vec<usize> = circuit.gates.iter()
        .filter_map(|g| if let Gate::Measure(q, _) = g {
            q.register.parse::<usize>().ok().or(Some(q.index))
        } else { None })
        .collect();

    for qubit in &measured_qubits {
        let err = noise.readout_error(*qubit);
        readout_fidelity *= 1.0 - err;
        fidelity         *= 1.0 - err;
    }

    // Decoherence: approximate T2 decay over circuit duration
    // F_decohere = product of exp(-t_circuit / T2_i) for active qubits
    let mut decoherence = 1.0f64;
    let n = circuit.qubit_count().min(noise.qubit_count);
    for i in 0..n {
        if let Some(qe) = noise.qubits.get(i) {
            let t2_ns = qe.t2_us * 1000.0;
            if t2_ns > 0.0 {
                decoherence *= (-total_duration / t2_ns).exp();
            }
        }
    }
    fidelity *= decoherence;

    FidelityEstimate {
        fidelity:          fidelity.max(0.0),
        gate_error_budget: gate_error_budget.max(0.0),
        readout_error:     1.0 - readout_fidelity,
        decoherence_error: 1.0 - decoherence,
        total_duration_ns: total_duration,
    }
}

fn gate_error(gate: &Gate, noise: &NoiseModel) -> (f64, f64) {
    match gate {
        Gate::Cx(c, t) => {
            let err = noise.two_qubit_error(c.index, t.index);
            (err, 533.0)
        }
        Gate::Cz(c, t) => {
            let err = noise.two_qubit_error(c.index, t.index);
            (err, 533.0)
        }
        Gate::Swap(a, b) => {
            // SWAP = 3 CX gates
            let err = noise.two_qubit_error(a.index, b.index);
            let total_err = 1.0 - (1.0 - err).powi(3);
            (total_err, 1599.0)
        }
        Gate::Rz(_, _) => (0.0, 0.0), // virtual gate
        Gate::H(q) | Gate::X(q) | Gate::Y(q) | Gate::Z(q)
        | Gate::S(q) | Gate::Sdg(q) | Gate::T(q) | Gate::Tdg(q) => {
            let err = noise.single_qubit_error(q.index);
            (err, 35.5)
        }
        Gate::Rx(_, q) | Gate::Ry(_, q) => {
            let err = noise.single_qubit_error(q.index);
            (err, 35.5)
        }
        Gate::Measure(_, _) => (0.0, 1000.0),
        Gate::Barrier(_)    => (0.0, 0.0),
        _ => (0.001, 35.5),
    }
}