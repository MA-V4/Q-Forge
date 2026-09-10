// Statevector simulator.
// Represents |psi> as a Vec<Complex64> with 2^n amplitudes.
// Single-qubit gates use tensor product indexing.
// Two-qubit gates operate on the joint Hilbert space of the qubit pair.

use crate::gates;
use num_complex::Complex64;
use qforge_ir::{Circuit, Gate, QubitRef};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub shots: usize,
    pub counts: HashMap<String, usize>,
    pub probabilities: HashMap<String, f64>,
    pub qubit_count: usize,
}

impl SimulationResult {
    pub fn most_probable(&self) -> Option<(&str, f64)> {
        self.probabilities
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(k, v)| (k.as_str(), *v))
    }
}

pub fn simulate(circuit: &Circuit, shots: usize) -> SimulationResult {
    let n = circuit.qubit_count();
    if n == 0 {
        return SimulationResult {
            shots,
            counts: HashMap::new(),
            probabilities: HashMap::new(),
            qubit_count: 0,
        };
    }

    let dim = 1usize << n;
    let mut state = vec![Complex64::new(0.0, 0.0); dim];
    state[0] = Complex64::new(1.0, 0.0); // |000...0>

    let logical_map = build_logical_map(circuit);

    for gate in &circuit.gates {
        apply_gate(&mut state, gate, n, &logical_map);
    }

    // Compute probabilities
    let probs: Vec<f64> = state.iter().map(|a| a.norm_sqr()).collect();

    // Sample shots
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut rng_state = 12345u64; // simple LCG
    for _ in 0..shots {
        let r = lcg_random(&mut rng_state);
        let mut cumulative = 0.0;
        let mut outcome = dim - 1;
        for (i, p) in probs.iter().enumerate() {
            cumulative += p;
            if r < cumulative {
                outcome = i;
                break;
            }
        }
        let bitstring = format!("{:0>width$b}", outcome, width = n);
        *counts.entry(bitstring).or_insert(0) += 1;
    }

    let probabilities: HashMap<String, f64> = probs
        .iter()
        .enumerate()
        .filter(|(_, p)| **p > 1e-10)
        .map(|(i, p)| (format!("{:0>width$b}", i, width = n), *p))
        .collect();

    SimulationResult {
        shots,
        counts,
        probabilities,
        qubit_count: n,
    }
}

fn lcg_random(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (*state >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
}

fn build_logical_map(circuit: &Circuit) -> HashMap<String, usize> {
    let mut idx = 0;
    let mut map = HashMap::new();
    let mut regs: Vec<(&String, &usize)> = circuit.qregs.iter().collect();
    regs.sort_by_key(|(name, _)| name.as_str());
    for (reg, size) in regs {
        for i in 0..*size {
            map.insert(format!("{}_{}", reg, i), idx);
            idx += 1;
        }
    }
    map
}

fn qubit_idx(q: &QubitRef, map: &HashMap<String, usize>) -> usize {
    *map.get(&format!("{}_{}", q.register, q.index))
        .unwrap_or(&q.index)
}

fn apply_gate(state: &mut Vec<Complex64>, gate: &Gate, n: usize, map: &HashMap<String, usize>) {
    match gate {
        Gate::H(q) => apply_1q(state, n, qubit_idx(q, map), gates::h()),
        Gate::X(q) => apply_1q(state, n, qubit_idx(q, map), gates::x()),
        Gate::Y(q) => apply_1q(state, n, qubit_idx(q, map), gates::y()),
        Gate::Z(q) => apply_1q(state, n, qubit_idx(q, map), gates::z()),
        Gate::S(q) => apply_1q(state, n, qubit_idx(q, map), gates::s()),
        Gate::Sdg(q) => apply_1q(state, n, qubit_idx(q, map), gates::sdg()),
        Gate::T(q) => apply_1q(state, n, qubit_idx(q, map), gates::t()),
        Gate::Tdg(q) => apply_1q(state, n, qubit_idx(q, map), gates::tdg()),
        Gate::Rx(a, q) => apply_1q(state, n, qubit_idx(q, map), gates::rx(*a)),
        Gate::Ry(a, q) => apply_1q(state, n, qubit_idx(q, map), gates::ry(*a)),
        Gate::Rz(a, q) => apply_1q(state, n, qubit_idx(q, map), gates::rz(*a)),
        Gate::U1(a, q) => apply_1q(state, n, qubit_idx(q, map), gates::u1(*a)),
        Gate::U2(a, b, q) => apply_1q(state, n, qubit_idx(q, map), gates::u2(*a, *b)),
        Gate::U3(a, b, c, q) => apply_1q(state, n, qubit_idx(q, map), gates::u3(*a, *b, *c)),
        Gate::Cx(c, t) => apply_2q(state, n, qubit_idx(c, map), qubit_idx(t, map), gates::cx()),
        Gate::Cz(c, t) => apply_2q(state, n, qubit_idx(c, map), qubit_idx(t, map), gates::cz()),
        Gate::Swap(a, b) => apply_2q(
            state,
            n,
            qubit_idx(a, map),
            qubit_idx(b, map),
            gates::swap(),
        ),
        Gate::Measure(_, _) | Gate::Barrier(_) | Gate::Reset(_) => {}
        Gate::Custom { .. } => {}
        _ => {}
    }
}

/// Apply a single-qubit gate matrix to qubit `target` in the full statevector.
fn apply_1q(state: &mut Vec<Complex64>, n: usize, target: usize, gate: gates::Matrix2) {
    let dim = 1 << n;
    let stride = 1 << target;

    for i in 0..dim {
        if i & stride == 0 {
            let j = i | stride;
            let a = state[i];
            let b = state[j];
            state[i] = gate[0][0] * a + gate[0][1] * b;
            state[j] = gate[1][0] * a + gate[1][1] * b;
        }
    }
}

/// Apply a two-qubit gate matrix to qubits `q0` (control) and `q1` (target).
fn apply_2q(state: &mut Vec<Complex64>, n: usize, q0: usize, q1: usize, gate: gates::Matrix4) {
    let dim = 1 << n;
    let s0 = 1 << q0;
    let s1 = 1 << q1;

    for i in 0..dim {
        if i & s0 == 0 && i & s1 == 0 {
            let i00 = i;
            let i01 = i | s1;
            let i10 = i | s0;
            let i11 = i | s0 | s1;

            let a00 = state[i00];
            let a01 = state[i01];
            let a10 = state[i10];
            let a11 = state[i11];

            state[i00] = gate[0][0] * a00 + gate[0][1] * a01 + gate[0][2] * a10 + gate[0][3] * a11;
            state[i01] = gate[1][0] * a00 + gate[1][1] * a01 + gate[1][2] * a10 + gate[1][3] * a11;
            state[i10] = gate[2][0] * a00 + gate[2][1] * a01 + gate[2][2] * a10 + gate[2][3] * a11;
            state[i11] = gate[3][0] * a00 + gate[3][1] * a01 + gate[3][2] * a10 + gate[3][3] * a11;
        }
    }
}
