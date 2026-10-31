use super::complex::{format_complex, Complex, ZERO};
use super::gate::{GateOp, Matrix2x2};
use rand::Rng;
use rayon::prelude::*;
use std::collections::BTreeMap;

/// Threshold for multi-threading operations with Rayon
const PARALLEL_THRESHOLD: usize = 1 << 12; // 4096 amplitudes (12 qubits)

/// State vector representation of an N-qubit quantum state.
#[derive(Debug, Clone, PartialEq)]
pub struct QubitState {
    pub num_qubits: usize,
    pub amplitudes: Vec<Complex>,
}

impl QubitState {
    /// Initialize state with all qubits in |0...0>
    pub fn new(num_qubits: usize) -> Self {
        assert!(num_qubits > 0, "Number of qubits must be at least 1");
        assert!(
            num_qubits <= 28,
            "Simulating more than 28 qubits requires > 4GB RAM"
        );
        let dim = 1 << num_qubits;
        let mut amplitudes = vec![ZERO; dim];
        amplitudes[0] = Complex::new(1.0, 0.0);
        Self {
            num_qubits,
            amplitudes,
        }
    }

    /// Initialize from custom amplitudes (must have length 2^N and be normalized)
    pub fn from_amplitudes(num_qubits: usize, amplitudes: Vec<Complex>) -> Result<Self, String> {
        let expected_dim = 1 << num_qubits;
        if amplitudes.len() != expected_dim {
            return Err(format!(
                "Expected {} amplitudes for {} qubits, got {}",
                expected_dim,
                num_qubits,
                amplitudes.len()
            ));
        }

        let norm_sq: f64 = amplitudes.iter().map(|c| c.norm_sqr()).sum();
        if (norm_sq - 1.0).abs() > 1e-4 {
            return Err(format!("Amplitudes must sum of squares to 1.0, got {:.6}", norm_sq));
        }

        Ok(Self {
            num_qubits,
            amplitudes,
        })
    }

    /// Dimension of the state space (2^N)
    #[inline]
    pub fn dim(&self) -> usize {
        self.amplitudes.len()
    }

    /// Reset state to |0...0>
    pub fn reset(&mut self) {
        for a in self.amplitudes.iter_mut() {
            *a = ZERO;
        }
        self.amplitudes[0] = Complex::new(1.0, 0.0);
    }

    /// Apply a single-qubit unitary gate on target qubit
    pub fn apply_single_gate(&mut self, target: usize, u: &Matrix2x2) {
        assert!(target < self.num_qubits, "Target qubit out of range");
        let half_dim = self.dim() / 2;
        let t_bit = 1 << target;
        let low_mask = t_bit - 1;

        let u00 = u.m[0][0];
        let u01 = u.m[0][1];
        let u10 = u.m[1][0];
        let u11 = u.m[1][1];

        if self.dim() >= PARALLEL_THRESHOLD {
            // Parallel execution using disjoint slice chunks
            let step = 1 << (target + 1);
            let half_step = 1 << target;
            self.amplitudes
                .par_chunks_exact_mut(step)
                .for_each(|block| {
                    let (low_half, high_half) = block.split_at_mut(half_step);
                    for j in 0..half_step {
                        let c0 = low_half[j];
                        let c1 = high_half[j];
                        low_half[j] = u00 * c0 + u01 * c1;
                        high_half[j] = u10 * c0 + u11 * c1;
                    }
                });
        } else {
            for k in 0..half_dim {
                let low = k & low_mask;
                let high = (k >> target) << (target + 1);
                let i0 = high | low;
                let i1 = i0 | t_bit;

                let c0 = self.amplitudes[i0];
                let c1 = self.amplitudes[i1];

                self.amplitudes[i0] = u00 * c0 + u01 * c1;
                self.amplitudes[i1] = u10 * c0 + u11 * c1;
            }
        }
    }

    /// Apply an arbitrary multi-controlled single-qubit gate
    pub fn apply_controlled_gate(&mut self, controls: &[usize], target: usize, u: &Matrix2x2) {
        assert!(target < self.num_qubits, "Target qubit out of range");
        for &c in controls {
            assert!(c < self.num_qubits, "Control qubit out of range");
            assert_ne!(c, target, "Control and target qubits cannot be identical");
        }

        if controls.is_empty() {
            self.apply_single_gate(target, u);
            return;
        }

        let half_dim = self.dim() / 2;
        let t_bit = 1 << target;
        let low_mask = t_bit - 1;

        let mut control_mask = 0usize;
        for &c in controls {
            control_mask |= 1 << c;
        }

        let u00 = u.m[0][0];
        let u01 = u.m[0][1];
        let u10 = u.m[1][0];
        let u11 = u.m[1][1];

        for k in 0..half_dim {
            let low = k & low_mask;
            let high = (k >> target) << (target + 1);
            let i0 = high | low;

            // Only apply if all control bits are 1
            if (i0 & control_mask) == control_mask {
                let i1 = i0 | t_bit;
                let c0 = self.amplitudes[i0];
                let c1 = self.amplitudes[i1];

                self.amplitudes[i0] = u00 * c0 + u01 * c1;
                self.amplitudes[i1] = u10 * c0 + u11 * c1;
            }
        }
    }

    /// Apply SWAP gate between qubit1 and qubit2
    pub fn apply_swap(&mut self, q1: usize, q2: usize) {
        assert!(q1 < self.num_qubits && q2 < self.num_qubits, "Qubit out of range");
        if q1 == q2 {
            return;
        }
        let bit1 = 1 << q1;
        let bit2 = 1 << q2;
        let diff_mask = bit1 | bit2;

        for i in 0..self.dim() {
            // Only swap when bit1 = 1 and bit2 = 0 to avoid double swapping
            if (i & bit1) != 0 && (i & bit2) == 0 {
                let j = i ^ diff_mask;
                self.amplitudes.swap(i, j);
            }
        }
    }

    /// Apply Controlled SWAP (Fredkin gate)
    pub fn apply_cswap(&mut self, control: usize, q1: usize, q2: usize) {
        assert!(control < self.num_qubits && q1 < self.num_qubits && q2 < self.num_qubits);
        if q1 == q2 {
            return;
        }
        let ctrl_bit = 1 << control;
        let bit1 = 1 << q1;
        let bit2 = 1 << q2;
        let diff_mask = bit1 | bit2;

        for i in 0..self.dim() {
            if (i & ctrl_bit) != 0 && (i & bit1) != 0 && (i & bit2) == 0 {
                let j = i ^ diff_mask;
                self.amplitudes.swap(i, j);
            }
        }
    }

    /// Apply a gate operation
    pub fn apply_op(&mut self, op: &GateOp) {
        match *op {
            GateOp::H(q) => self.apply_single_gate(q, &Matrix2x2::hadamard()),
            GateOp::X(q) => self.apply_single_gate(q, &Matrix2x2::pauli_x()),
            GateOp::Y(q) => self.apply_single_gate(q, &Matrix2x2::pauli_y()),
            GateOp::Z(q) => self.apply_single_gate(q, &Matrix2x2::pauli_z()),
            GateOp::S(q) => self.apply_single_gate(q, &Matrix2x2::s()),
            GateOp::Sdg(q) => self.apply_single_gate(q, &Matrix2x2::s_dagger()),
            GateOp::T(q) => self.apply_single_gate(q, &Matrix2x2::t()),
            GateOp::Tdg(q) => self.apply_single_gate(q, &Matrix2x2::t_dagger()),
            GateOp::Rx(q, th) => self.apply_single_gate(q, &Matrix2x2::rx(th)),
            GateOp::Ry(q, th) => self.apply_single_gate(q, &Matrix2x2::ry(th)),
            GateOp::Rz(q, th) => self.apply_single_gate(q, &Matrix2x2::rz(th)),
            GateOp::Phase(q, l) => self.apply_single_gate(q, &Matrix2x2::phase(l)),
            GateOp::U3(q, t, p, l) => self.apply_single_gate(q, &Matrix2x2::u3(t, p, l)),
            GateOp::Custom1Q(q, m, _) => self.apply_single_gate(q, &m),
            GateOp::CX(c, t) => self.apply_controlled_gate(&[c], t, &Matrix2x2::pauli_x()),
            GateOp::CY(c, t) => self.apply_controlled_gate(&[c], t, &Matrix2x2::pauli_y()),
            GateOp::CZ(c, t) => self.apply_controlled_gate(&[c], t, &Matrix2x2::pauli_z()),
            GateOp::CH(c, t) => self.apply_controlled_gate(&[c], t, &Matrix2x2::hadamard()),
            GateOp::SWAP(q1, q2) => self.apply_swap(q1, q2),
            GateOp::CCX(c1, c2, t) => self.apply_controlled_gate(&[c1, c2], t, &Matrix2x2::pauli_x()),
            GateOp::CSWAP(c, q1, q2) => self.apply_cswap(c, q1, q2),
            GateOp::CP(c, t, p) => self.apply_controlled_gate(&[c], t, &Matrix2x2::phase(p)),
            GateOp::MCX(ref c, t) => self.apply_controlled_gate(c, t, &Matrix2x2::pauli_x()),
            GateOp::MCZ(ref c, t) => self.apply_controlled_gate(c, t, &Matrix2x2::pauli_z()),
            GateOp::Measure(q) => {
                let mut rng = rand::thread_rng();
                self.measure_qubit(q, &mut rng);
            }
        }
    }

    /// Calculate probability of finding target qubit in state |0> or |1>
    pub fn probability_zero(&self, target: usize) -> f64 {
        assert!(target < self.num_qubits, "Target qubit out of range");
        let t_bit = 1 << target;
        let mut p0 = 0.0;
        for (i, &amp) in self.amplitudes.iter().enumerate() {
            if (i & t_bit) == 0 {
                p0 += amp.norm_sqr();
            }
        }
        p0.clamp(0.0, 1.0)
    }

    /// Measure a single qubit with wavefunction collapse
    /// Returns (measured_bit, probability_of_that_outcome)
    pub fn measure_qubit<R: Rng>(&mut self, target: usize, rng: &mut R) -> (bool, f64) {
        let p0 = self.probability_zero(target);
        let p1 = 1.0 - p0;
        let r: f64 = rng.gen();
        let outcome = r >= p0; // false = 0, true = 1
        let t_bit = 1 << target;

        let norm_factor = if !outcome {
            1.0 / p0.max(1e-15).sqrt()
        } else {
            1.0 / p1.max(1e-15).sqrt()
        };

        for (i, amp) in self.amplitudes.iter_mut().enumerate() {
            let bit_is_one = (i & t_bit) != 0;
            if bit_is_one == outcome {
                *amp *= norm_factor;
            } else {
                *amp = ZERO;
            }
        }

        (outcome, if !outcome { p0 } else { p1 })
    }

    /// Compute probabilities of all 2^N computational basis states
    pub fn probabilities(&self) -> Vec<f64> {
        self.amplitudes.iter().map(|c| c.norm_sqr()).collect()
    }

    /// Sample measurements across all qubits (shots) without collapsing the original state
    pub fn sample_shots<R: Rng>(&self, shots: usize, rng: &mut R) -> BTreeMap<usize, usize> {
        let probs = self.probabilities();
        let mut cumulative = Vec::with_capacity(probs.len());
        let mut running = 0.0;
        for &p in &probs {
            running += p;
            cumulative.push(running);
        }

        let mut counts = BTreeMap::new();
        for _ in 0..shots {
            let r: f64 = rng.gen_range(0.0..1.0);
            let idx = match cumulative.binary_search_by(|&val| {
                val.partial_cmp(&r).unwrap_or(std::cmp::Ordering::Less)
            }) {
                Ok(i) => i,
                Err(i) => i.min(probs.len() - 1),
            };
            *counts.entry(idx).or_insert(0) += 1;
        }

        counts
    }

    /// Calculate expectation value <Z_i> for qubit i
    pub fn expectation_z(&self, target: usize) -> f64 {
        let p0 = self.probability_zero(target);
        let p1 = 1.0 - p0;
        p0 - p1
    }

    /// Calculate reduced density matrix 2x2 for single qubit by partial trace
    pub fn reduced_density_matrix(&self, qubit: usize) -> [[Complex; 2]; 2] {
        assert!(qubit < self.num_qubits);
        let q_bit = 1 << qubit;
        let mut rho00 = Complex::new(0.0, 0.0);
        let mut rho11 = Complex::new(0.0, 0.0);
        let mut rho01 = Complex::new(0.0, 0.0);

        for (i, &amp) in self.amplitudes.iter().enumerate() {
            if (i & q_bit) == 0 {
                let partner = i | q_bit;
                let amp_partner = self.amplitudes[partner];
                rho00 += amp.norm_sqr();
                rho01 += amp * amp_partner.conj();
            } else {
                rho11 += amp.norm_sqr();
            }
        }

        let rho10 = rho01.conj();
        [[rho00, rho01], [rho10, rho11]]
    }

    /// Calculate Bloch vector (rx, ry, rz) and purity for a single qubit
    pub fn bloch_vector(&self, qubit: usize) -> (f64, f64, f64, f64) {
        let rho = self.reduced_density_matrix(qubit);
        let rx = 2.0 * rho[0][1].re;
        let ry = 2.0 * rho[1][0].im;
        let rz = rho[0][0].re - rho[1][1].re;
        let purity = (1.0 + rx * rx + ry * ry + rz * rz) / 2.0;
        (rx, ry, rz, purity)
    }

    /// Format state in Dirac bra-ket notation, e.g. "0.7071|00> + 0.7071|11>"
    pub fn to_dirac_string(&self, eps: f64) -> String {
        let mut terms = Vec::new();
        for (idx, &c) in self.amplitudes.iter().enumerate() {
            if c.norm_sqr() >= eps {
                let bin_str = format!("{:0width$b}", idx, width = self.num_qubits);
                let coeff = format_complex(c, 4);
                terms.push(format!("({})|{}>", coeff, bin_str));
            }
        }
        if terms.is_empty() {
            "|0>".to_string()
        } else {
            terms.join(" + ")
        }
    }
}
