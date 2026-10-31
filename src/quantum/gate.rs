use super::complex::{Complex, I, ONE, SQRT1_2, ZERO};
use std::f64::consts::FRAC_PI_4;

/// A 2x2 complex unitary matrix representing a single-qubit quantum gate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix2x2 {
    pub m: [[Complex; 2]; 2],
}

impl Matrix2x2 {
    pub const fn new(m00: Complex, m01: Complex, m10: Complex, m11: Complex) -> Self {
        Self {
            m: [[m00, m01], [m10, m11]],
        }
    }

    pub fn identity() -> Self {
        Self::new(ONE, ZERO, ZERO, ONE)
    }

    pub fn pauli_x() -> Self {
        Self::new(ZERO, ONE, ONE, ZERO)
    }

    pub fn pauli_y() -> Self {
        Self::new(ZERO, -I, I, ZERO)
    }

    pub fn pauli_z() -> Self {
        Self::new(ONE, ZERO, ZERO, -ONE)
    }

    pub fn hadamard() -> Self {
        let inv_sqrt2 = Complex::new(SQRT1_2, 0.0);
        Self::new(inv_sqrt2, inv_sqrt2, inv_sqrt2, -inv_sqrt2)
    }

    pub fn s() -> Self {
        Self::new(ONE, ZERO, ZERO, I)
    }

    pub fn s_dagger() -> Self {
        Self::new(ONE, ZERO, ZERO, -I)
    }

    pub fn t() -> Self {
        let phase = Complex::from_polar(1.0, FRAC_PI_4);
        Self::new(ONE, ZERO, ZERO, phase)
    }

    pub fn t_dagger() -> Self {
        let phase = Complex::from_polar(1.0, -FRAC_PI_4);
        Self::new(ONE, ZERO, ZERO, phase)
    }

    pub fn phase(lambda: f64) -> Self {
        Self::new(ONE, ZERO, ZERO, Complex::from_polar(1.0, lambda))
    }

    pub fn rx(theta: f64) -> Self {
        let half = theta / 2.0;
        let c = Complex::new(half.cos(), 0.0);
        let s = Complex::new(0.0, -half.sin());
        Self::new(c, s, s, c)
    }

    pub fn ry(theta: f64) -> Self {
        let half = theta / 2.0;
        let c = Complex::new(half.cos(), 0.0);
        let s = Complex::new(half.sin(), 0.0);
        Self::new(c, -s, s, c)
    }

    pub fn rz(theta: f64) -> Self {
        let half = theta / 2.0;
        let p_neg = Complex::from_polar(1.0, -half);
        let p_pos = Complex::from_polar(1.0, half);
        Self::new(p_neg, ZERO, ZERO, p_pos)
    }

    pub fn u3(theta: f64, phi: f64, lambda: f64) -> Self {
        let half_t = theta / 2.0;
        let cos_half = half_t.cos();
        let sin_half = half_t.sin();

        let m00 = Complex::new(cos_half, 0.0);
        let m01 = -Complex::from_polar(1.0, lambda) * sin_half;
        let m10 = Complex::from_polar(1.0, phi) * sin_half;
        let m11 = Complex::from_polar(1.0, phi + lambda) * cos_half;

        Self::new(m00, m01, m10, m11)
    }

    /// Multiply two 2x2 matrices
    pub fn matmul(&self, other: &Self) -> Self {
        let mut res = [[ZERO; 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    res[i][j] += self.m[i][k] * other.m[k][j];
                }
            }
        }
        Self { m: res }
    }

    /// Conjugate transpose (Hermitian adjoint)
    pub fn dagger(&self) -> Self {
        Self::new(
            self.m[0][0].conj(),
            self.m[1][0].conj(),
            self.m[0][1].conj(),
            self.m[1][1].conj(),
        )
    }
}

/// Gate operation representation in a circuit
#[derive(Debug, Clone, PartialEq)]
pub enum GateOp {
    H(usize),
    X(usize),
    Y(usize),
    Z(usize),
    S(usize),
    Sdg(usize),
    T(usize),
    Tdg(usize),
    Rx(usize, f64),
    Ry(usize, f64),
    Rz(usize, f64),
    Phase(usize, f64),
    U3(usize, f64, f64, f64),
    Custom1Q(usize, Matrix2x2, String),
    CX(usize, usize),            // control, target
    CY(usize, usize),
    CZ(usize, usize),
    CH(usize, usize),
    SWAP(usize, usize),
    CCX(usize, usize, usize),    // control1, control2, target (Toffoli)
    CSWAP(usize, usize, usize),  // control, target1, target2 (Fredkin)
    CP(usize, usize, f64),       // control, target, angle (Controlled Phase)
    MCX(Vec<usize>, usize),      // multiple controls, target
    MCZ(Vec<usize>, usize),      // multiple controls, target
    Measure(usize),
}

impl GateOp {
    pub fn name(&self) -> String {
        match self {
            GateOp::H(_) => "H".to_string(),
            GateOp::X(_) => "X".to_string(),
            GateOp::Y(_) => "Y".to_string(),
            GateOp::Z(_) => "Z".to_string(),
            GateOp::S(_) => "S".to_string(),
            GateOp::Sdg(_) => "S†".to_string(),
            GateOp::T(_) => "T".to_string(),
            GateOp::Tdg(_) => "T†".to_string(),
            GateOp::Rx(_, th) => format!("Rx({:.3})", th),
            GateOp::Ry(_, th) => format!("Ry({:.3})", th),
            GateOp::Rz(_, th) => format!("Rz({:.3})", th),
            GateOp::Phase(_, l) => format!("P({:.3})", l),
            GateOp::U3(_, t, p, l) => format!("U({:.2},{:.2},{:.2})", t, p, l),
            GateOp::Custom1Q(_, _, name) => name.clone(),
            GateOp::CX(_, _) => "CX".to_string(),
            GateOp::CY(_, _) => "CY".to_string(),
            GateOp::CZ(_, _) => "CZ".to_string(),
            GateOp::CH(_, _) => "CH".to_string(),
            GateOp::SWAP(_, _) => "SWAP".to_string(),
            GateOp::CCX(_, _, _) => "CCX".to_string(),
            GateOp::CSWAP(_, _, _) => "CSWAP".to_string(),
            GateOp::CP(_, _, p) => format!("CP({:.3})", p),
            GateOp::MCX(c, _) => format!("MCX({})", c.len()),
            GateOp::MCZ(c, _) => format!("MCZ({})", c.len()),
            GateOp::Measure(_) => "M".to_string(),
        }
    }

    pub fn qubits_involved(&self) -> Vec<usize> {
        match self {
            GateOp::H(q)
            | GateOp::X(q)
            | GateOp::Y(q)
            | GateOp::Z(q)
            | GateOp::S(q)
            | GateOp::Sdg(q)
            | GateOp::T(q)
            | GateOp::Tdg(q)
            | GateOp::Rx(q, _)
            | GateOp::Ry(q, _)
            | GateOp::Rz(q, _)
            | GateOp::Phase(q, _)
            | GateOp::U3(q, _, _, _)
            | GateOp::Custom1Q(q, _, _)
            | GateOp::Measure(q) => vec![*q],
            GateOp::CX(c, t)
            | GateOp::CY(c, t)
            | GateOp::CZ(c, t)
            | GateOp::CH(c, t)
            | GateOp::SWAP(c, t)
            | GateOp::CP(c, t, _) => vec![*c, *t],
            GateOp::CCX(c1, c2, t) => vec![*c1, *c2, *t],
            GateOp::CSWAP(c, t1, t2) => vec![*c, *t1, *t2],
            GateOp::MCX(c, t) | GateOp::MCZ(c, t) => {
                let mut v = c.clone();
                v.push(*t);
                v
            }
        }
    }
}
