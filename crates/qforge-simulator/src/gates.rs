// Unitary matrices for all standard gates.
// Each gate is represented as a 2x2 or 4x4 complex matrix.

use num_complex::Complex64;
use std::f64::consts::FRAC_1_SQRT_2;

pub type Matrix2 = [[Complex64; 2]; 2];
pub type Matrix4 = [[Complex64; 4]; 4];

fn c(re: f64, im: f64) -> Complex64 {
    Complex64::new(re, im)
}
fn r(re: f64) -> Complex64 {
    Complex64::new(re, 0.0)
}

pub fn h() -> Matrix2 {
    let s = FRAC_1_SQRT_2;
    [[r(s), r(s)], [r(s), r(-s)]]
}

pub fn x() -> Matrix2 {
    [[r(0.0), r(1.0)], [r(1.0), r(0.0)]]
}

pub fn y() -> Matrix2 {
    [[r(0.0), c(0.0, -1.0)], [c(0.0, 1.0), r(0.0)]]
}

pub fn z() -> Matrix2 {
    [[r(1.0), r(0.0)], [r(0.0), r(-1.0)]]
}

pub fn s() -> Matrix2 {
    [[r(1.0), r(0.0)], [r(0.0), c(0.0, 1.0)]]
}

pub fn sdg() -> Matrix2 {
    [[r(1.0), r(0.0)], [r(0.0), c(0.0, -1.0)]]
}

pub fn t() -> Matrix2 {
    let v = c(FRAC_1_SQRT_2, FRAC_1_SQRT_2);
    [[r(1.0), r(0.0)], [r(0.0), v]]
}

pub fn tdg() -> Matrix2 {
    let v = c(FRAC_1_SQRT_2, -FRAC_1_SQRT_2);
    [[r(1.0), r(0.0)], [r(0.0), v]]
}

pub fn rx(theta: f64) -> Matrix2 {
    let cos = r((theta / 2.0).cos());
    let sin = c(0.0, -(theta / 2.0).sin());
    [[cos, sin], [sin, cos]]
}

pub fn ry(theta: f64) -> Matrix2 {
    let cos = r((theta / 2.0).cos());
    let sin_p = r((theta / 2.0).sin());
    let sin_n = r(-(theta / 2.0).sin());
    [[cos, sin_n], [sin_p, cos]]
}

pub fn rz(theta: f64) -> Matrix2 {
    [
        [c((-theta / 2.0).cos(), (-theta / 2.0).sin()), r(0.0)],
        [r(0.0), c((theta / 2.0).cos(), (theta / 2.0).sin())],
    ]
}

pub fn u1(lambda: f64) -> Matrix2 {
    [[r(1.0), r(0.0)], [r(0.0), c(lambda.cos(), lambda.sin())]]
}

pub fn u2(phi: f64, lambda: f64) -> Matrix2 {
    let s = FRAC_1_SQRT_2;
    [
        [r(s), c(-s * lambda.cos(), -s * lambda.sin())],
        [
            c(s * phi.cos(), s * phi.sin()),
            c(s * (phi + lambda).cos(), s * (phi + lambda).sin()),
        ],
    ]
}

pub fn u3(theta: f64, phi: f64, lambda: f64) -> Matrix2 {
    [
        [
            r((theta / 2.0).cos()),
            c(
                -(theta / 2.0).sin() * lambda.cos(),
                -(theta / 2.0).sin() * lambda.sin(),
            ),
        ],
        [
            c(
                (theta / 2.0).sin() * phi.cos(),
                (theta / 2.0).sin() * phi.sin(),
            ),
            c(
                (theta / 2.0).cos() * (phi + lambda).cos(),
                (theta / 2.0).cos() * (phi + lambda).sin(),
            ),
        ],
    ]
}

/// CX (CNOT) as a 4x4 matrix over the |ctrl, tgt> basis.
pub fn cx() -> Matrix4 {
    [
        [r(1.0), r(0.0), r(0.0), r(0.0)],
        [r(0.0), r(1.0), r(0.0), r(0.0)],
        [r(0.0), r(0.0), r(0.0), r(1.0)],
        [r(0.0), r(0.0), r(1.0), r(0.0)],
    ]
}

pub fn cz() -> Matrix4 {
    [
        [r(1.0), r(0.0), r(0.0), r(0.0)],
        [r(0.0), r(1.0), r(0.0), r(0.0)],
        [r(0.0), r(0.0), r(1.0), r(0.0)],
        [r(0.0), r(0.0), r(0.0), r(-1.0)],
    ]
}

pub fn swap() -> Matrix4 {
    [
        [r(1.0), r(0.0), r(0.0), r(0.0)],
        [r(0.0), r(0.0), r(1.0), r(0.0)],
        [r(0.0), r(1.0), r(0.0), r(0.0)],
        [r(0.0), r(0.0), r(0.0), r(1.0)],
    ]
}
