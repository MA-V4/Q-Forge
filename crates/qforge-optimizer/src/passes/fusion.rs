// Gate fusion pass.
// Finds maximal runs of single-qubit gates on the same qubit,
// multiplies their 2x2 unitary matrices, and collapses them into one U3.
// Runs after decomposition to clean up Rz/SX sequences.

use crate::pass::{Pass, PassReport};
use num_complex::Complex64;
use qforge_ir::{Circuit, Gate, QubitRef};
use std::f64::consts::FRAC_1_SQRT_2;

type M2 = [[Complex64; 2]; 2];

pub struct GateFusion;

impl Pass for GateFusion {
    fn name(&self) -> &str {
        "gate_fusion"
    }

    fn run(&self, mut circuit: Circuit) -> (Circuit, PassReport) {
        let before = circuit.gate_count();
        let gates = std::mem::take(&mut circuit.gates);
        let mut out: Vec<Gate> = Vec::with_capacity(gates.len());

        let mut i = 0;
        while i < gates.len() {
            let gate = &gates[i];

            // Only start fusion on single-qubit gates (not measurements, barriers)
            if let Some(qubit) = single_qubit_target(gate) {
                // Find the maximal run of single-qubit gates on this qubit
                // with no intervening two-qubit gates touching it
                let mut j = i;
                let mut mat = identity();

                while j < gates.len() {
                    let g = &gates[j];
                    if affects_qubit(g, &qubit) {
                        if let Some(m) = gate_matrix(g) {
                            mat = mat_mul(m, mat); // right-multiply (later gate on left)
                            j += 1;
                        } else {
                            // Measurement or reset — stop fusion
                            break;
                        }
                    } else if blocks_qubit(g, &qubit) {
                        // Two-qubit gate touching our qubit — stop
                        break;
                    } else {
                        // Gate on a different qubit — skip past it
                        j += 1;
                        if is_two_qubit(g) {
                            // We passed a 2Q gate not touching our qubit — safe
                        }
                        // Add the skipped gate to output immediately if before our run end
                        // We handle this by just noting we need to output it
                        // Actually: collect all skipped non-qubit gates
                        break; // simpler: stop fusion at any non-qubit gate
                    }
                }

                let fused_count = j - i;

                if fused_count <= 1 {
                    // Nothing to fuse
                    out.push(gates[i].clone());
                    i += 1;
                } else {
                    // Emit fused gate
                    if let Some(fused) = mat_to_gate(mat, qubit) {
                        out.push(fused);
                    }
                    i = j;
                }
            } else {
                out.push(gates[i].clone());
                i += 1;
            }
        }

        circuit.gates = out;
        let after = circuit.gate_count();
        let removed = before as i64 - after as i64;
        (
            circuit,
            PassReport {
                pass_name: self.name().into(),
                gates_removed: removed,
                reason: format!("fused {} single-qubit gate(s)", removed),
            },
        )
    }
}

fn c(re: f64, im: f64) -> Complex64 {
    Complex64::new(re, im)
}
fn r(re: f64) -> Complex64 {
    Complex64::new(re, 0.0)
}

fn identity() -> M2 {
    [[r(1.0), r(0.0)], [r(0.0), r(1.0)]]
}

fn mat_mul(a: M2, b: M2) -> M2 {
    let mut out = [[c(0.0, 0.0); 2]; 2];
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                out[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    out
}

fn gate_matrix(gate: &Gate) -> Option<M2> {
    let s = FRAC_1_SQRT_2;
    Some(match gate {
        Gate::H(_) => [[r(s), r(s)], [r(s), r(-s)]],
        Gate::X(_) => [[r(0.0), r(1.0)], [r(1.0), r(0.0)]],
        Gate::Y(_) => [[r(0.0), c(0.0, -1.0)], [c(0.0, 1.0), r(0.0)]],
        Gate::Z(_) => [[r(1.0), r(0.0)], [r(0.0), r(-1.0)]],
        Gate::S(_) => [[r(1.0), r(0.0)], [r(0.0), c(0.0, 1.0)]],
        Gate::Sdg(_) => [[r(1.0), r(0.0)], [r(0.0), c(0.0, -1.0)]],
        Gate::T(_) => [[r(1.0), r(0.0)], [r(0.0), c(s, s)]],
        Gate::Tdg(_) => [[r(1.0), r(0.0)], [r(0.0), c(s, -s)]],
        Gate::Sx(_) => [[c(0.5, 0.5), c(0.5, -0.5)], [c(0.5, -0.5), c(0.5, 0.5)]],
        Gate::Sxdg(_) => [[c(0.5, -0.5), c(0.5, 0.5)], [c(0.5, 0.5), c(0.5, -0.5)]],
        Gate::Rx(t, _) => {
            let cos = r((t / 2.0).cos());
            let sin = c(0.0, -(t / 2.0).sin());
            [[cos, sin], [sin, cos]]
        }
        Gate::Ry(t, _) => {
            let c2 = r((t / 2.0).cos());
            let sp = r((t / 2.0).sin());
            let sn = r(-(t / 2.0).sin());
            [[c2, sn], [sp, c2]]
        }
        Gate::Rz(t, _) => {
            let e_neg = c((-t / 2.0).cos(), (-t / 2.0).sin());
            let e_pos = c((t / 2.0).cos(), (t / 2.0).sin());
            [[e_neg, r(0.0)], [r(0.0), e_pos]]
        }
        Gate::U1(l, _) => [[r(1.0), r(0.0)], [r(0.0), c(l.cos(), l.sin())]],
        Gate::U3(t, p, l, _) => {
            let cos = (t / 2.0).cos();
            let sin = (t / 2.0).sin();
            [
                [r(cos), c(-sin * l.cos(), -sin * l.sin())],
                [
                    c(sin * p.cos(), sin * p.sin()),
                    c(cos * (p + l).cos(), cos * (p + l).sin()),
                ],
            ]
        }
        // Cannot fuse measurements, barriers, 2Q gates
        _ => return None,
    })
}

/// Convert a 2x2 unitary matrix to a U3 gate via ZYZ decomposition.
/// Returns None if the matrix is approximately identity (gate can be dropped).
fn mat_to_gate(mat: M2, qubit: QubitRef) -> Option<Gate> {
    // Global phase doesn't matter — normalize by mat[0][0] if nonzero
    let norm = mat[0][0].norm();
    let phase = if norm > 1e-10 {
        mat[0][0] / norm
    } else {
        c(1.0, 0.0)
    };
    let u = [
        [mat[0][0] / phase, mat[0][1] / phase],
        [mat[1][0] / phase, mat[1][1] / phase],
    ];

    // ZYZ: U = Rz(phi) Ry(theta) Rz(lambda)
    // theta = 2 * arccos(|u[0][0]|)
    let a00_norm = u[0][0].norm().min(1.0);
    let theta = 2.0 * a00_norm.acos();

    // If theta is near zero, it's approximately Rz
    if theta.abs() < 1e-10 {
        let lambda = u[1][1].arg() - u[0][0].arg();
        if lambda.abs() < 1e-10 {
            return None; // identity
        }
        return Some(Gate::Rz(lambda, qubit));
    }

    let phi = u[1][0].arg() - u[0][0].arg();
    let lambda = (-u[0][1]).arg() - u[0][0].arg();

    // Normalize angles to [-π, π]
    let normalize = |a: f64| -> f64 {
        let mut v = a % (2.0 * std::f64::consts::PI);
        if v > std::f64::consts::PI {
            v -= 2.0 * std::f64::consts::PI;
        }
        if v < -std::f64::consts::PI {
            v += 2.0 * std::f64::consts::PI;
        }
        v
    };

    let theta = normalize(theta);
    let phi = normalize(phi);
    let lambda = normalize(lambda);

    // If U3 reduces to something simpler, emit the simpler form
    if theta.abs() < 1e-10 {
        let angle = normalize(phi + lambda);
        if angle.abs() < 1e-10 {
            return None;
        }
        return Some(Gate::Rz(angle, qubit));
    }

    Some(Gate::U3(theta, phi, lambda, qubit))
}

fn single_qubit_target(gate: &Gate) -> Option<QubitRef> {
    match gate {
        Gate::H(q)
        | Gate::X(q)
        | Gate::Y(q)
        | Gate::Z(q)
        | Gate::S(q)
        | Gate::Sdg(q)
        | Gate::T(q)
        | Gate::Tdg(q)
        | Gate::Sx(q)
        | Gate::Sxdg(q)
        | Gate::Rx(_, q)
        | Gate::Ry(_, q)
        | Gate::Rz(_, q)
        | Gate::U1(_, q)
        | Gate::U3(_, _, _, q) => Some(q.clone()),
        _ => None,
    }
}

fn affects_qubit(gate: &Gate, qubit: &QubitRef) -> bool {
    single_qubit_target(gate).as_ref() == Some(qubit)
}

fn blocks_qubit(gate: &Gate, qubit: &QubitRef) -> bool {
    match gate {
        Gate::Cx(c, t) | Gate::Cz(c, t) | Gate::Swap(c, t) => c == qubit || t == qubit,
        Gate::Ccx(a, b, c) => a == qubit || b == qubit || c == qubit,
        Gate::Barrier(_) => true,
        Gate::Measure(q, _) | Gate::Reset(q) => q == qubit,
        _ => false,
    }
}

fn is_two_qubit(gate: &Gate) -> bool {
    matches!(
        gate,
        Gate::Cx(_, _) | Gate::Cz(_, _) | Gate::Swap(_, _) | Gate::Ccx(_, _, _)
    )
}
