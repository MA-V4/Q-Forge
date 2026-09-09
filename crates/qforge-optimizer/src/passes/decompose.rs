use crate::pass::{Pass, PassReport};
use qforge_ir::{Circuit, Gate, QubitRef};
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

pub struct NativeGateDecomposition;

impl Pass for NativeGateDecomposition {
    fn name(&self) -> &str { "native_gate_decomposition" }

    fn run(&self, mut circuit: Circuit) -> (Circuit, PassReport) {
        let before = circuit.gate_count();
        let gates  = std::mem::take(&mut circuit.gates);
        let mut out: Vec<Gate> = Vec::with_capacity(gates.len() * 2);
        for gate in gates { decompose_gate(gate, &mut out); }
        circuit.gates = out;
        let after = circuit.gate_count();
        let delta = after as i64 - before as i64;
        (circuit, PassReport {
            pass_name:     self.name().into(),
            gates_removed: -delta,
            reason:        "decomposed to IBM native gates (CX, RZ, SX, X)".into(),
        })
    }
}

fn rz(theta: f64, q: &QubitRef) -> Gate { Gate::Rz(theta, q.clone()) }
fn sx(q: &QubitRef)              -> Gate { Gate::Sx(q.clone()) }
fn sxdg(q: &QubitRef)            -> Gate { Gate::Sxdg(q.clone()) }
fn x(q: &QubitRef)               -> Gate { Gate::X(q.clone()) }
fn cx(c: &QubitRef, t: &QubitRef) -> Gate { Gate::Cx(c.clone(), t.clone()) }

fn decompose_gate(gate: Gate, out: &mut Vec<Gate>) {
    match gate {
        // Already native
        Gate::Cx(_, _) | Gate::Rz(_, _) | Gate::X(_)
        | Gate::Sx(_) | Gate::Sxdg(_) => {
            out.push(gate);
        }

        // H = Rz(π/2) · SX · Rz(π/2)
        Gate::H(q) => {
            out.push(rz(FRAC_PI_2, &q));
            out.push(sx(&q));
            out.push(rz(FRAC_PI_2, &q));
        }

        // Y = Rz(π) · X
        Gate::Y(q) => {
            out.push(rz(PI, &q));
            out.push(x(&q));
        }

        // Z = Rz(π)
        Gate::Z(q) => { out.push(rz(PI, &q)); }

        // S = Rz(π/2)
        Gate::S(q) => { out.push(rz(FRAC_PI_2, &q)); }

        // Sdg = Rz(-π/2)
        Gate::Sdg(q) => { out.push(rz(-FRAC_PI_2, &q)); }

        // T = Rz(π/4)
        Gate::T(q) => { out.push(rz(FRAC_PI_4, &q)); }

        // Tdg = Rz(-π/4)
        Gate::Tdg(q) => { out.push(rz(-FRAC_PI_4, &q)); }

        // Rx(θ) = Rz(-π/2) · SX · Rz(θ) · SX · Rz(-π/2)
        Gate::Rx(theta, q) => {
            out.push(rz(-FRAC_PI_2, &q));
            out.push(sx(&q));
            out.push(rz(theta, &q));
            out.push(sx(&q));
            out.push(rz(-FRAC_PI_2, &q));
        }

        // Ry(θ) = SX · Rz(θ) · Sxdg
        Gate::Ry(theta, q) => {
            out.push(sx(&q));
            out.push(rz(theta, &q));
            out.push(sxdg(&q));
        }

        // SWAP = CX(a,b) · CX(b,a) · CX(a,b)
        Gate::Swap(a, b) => {
            out.push(cx(&a, &b));
            out.push(cx(&b, &a));
            out.push(cx(&a, &b));
        }

        // CZ = H(tgt) · CX · H(tgt)
        Gate::Cz(ctrl, tgt) => {
            out.push(rz(FRAC_PI_2, &tgt));
            out.push(sx(&tgt));
            out.push(rz(FRAC_PI_2, &tgt));
            out.push(cx(&ctrl, &tgt));
            out.push(rz(FRAC_PI_2, &tgt));
            out.push(sx(&tgt));
            out.push(rz(FRAC_PI_2, &tgt));
        }

        // U1(λ) = Rz(λ)
        Gate::U1(lambda, q) => { out.push(rz(lambda, &q)); }

        // U2(φ, λ) = Rz(φ + π/2) · SX · Rz(λ - π/2)
        Gate::U2(phi, lambda, q) => {
            out.push(rz(phi + FRAC_PI_2, &q));
            out.push(sx(&q));
            out.push(rz(lambda - FRAC_PI_2, &q));
        }

        // U3(θ, φ, λ) = Rz(φ + π) · SX · Rz(θ + π) · SX · Rz(λ)
        Gate::U3(theta, phi, lambda, q) => {
            out.push(rz(phi + PI, &q));
            out.push(sx(&q));
            out.push(rz(theta + PI, &q));
            out.push(sx(&q));
            out.push(rz(lambda, &q));
        }

        // CCX (Toffoli) — standard 6-CX decomposition
        Gate::Ccx(a, b, c) => {
            out.push(rz(FRAC_PI_2,  &c));
            out.push(sx(&c));
            out.push(rz(FRAC_PI_2,  &c));
            out.push(cx(&b, &c));
            out.push(rz(-FRAC_PI_4, &c));
            out.push(cx(&a, &c));
            out.push(rz(FRAC_PI_4,  &c));
            out.push(cx(&b, &c));
            out.push(rz(-FRAC_PI_4, &c));
            out.push(cx(&a, &c));
            out.push(rz(FRAC_PI_4,  &b));
            out.push(rz(FRAC_PI_4,  &c));
            out.push(cx(&a, &b));
            out.push(rz(FRAC_PI_4,  &a));
            out.push(rz(-FRAC_PI_4, &b));
            out.push(cx(&a, &b));
            out.push(rz(FRAC_PI_2,  &c));
            out.push(sx(&c));
            out.push(rz(FRAC_PI_2,  &c));
        }

        Gate::Barrier(_) | Gate::Measure(_, _) | Gate::Reset(_) => {
            out.push(gate);
        }

        Gate::Custom { .. } => { out.push(gate); }
    }
}