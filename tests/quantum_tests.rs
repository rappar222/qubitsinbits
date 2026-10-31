use num_complex::Complex64;
use quantum_sim::algorithms::*;
use quantum_sim::quantum::*;
use std::f64::consts::{FRAC_1_SQRT_2, PI};

#[test]
fn test_hadamard_properties() {
    let mut state = QubitState::new(1);
    assert_eq!(state.amplitudes[0], Complex64::new(1.0, 0.0));
    assert_eq!(state.amplitudes[1], Complex64::new(0.0, 0.0));

    // H|0> = |+> = 1/sqrt(2) (|0> + |1>)
    state.apply_single_gate(0, &Matrix2x2::hadamard());
    assert!((state.amplitudes[0].re - FRAC_1_SQRT_2).abs() < 1e-6);
    assert!((state.amplitudes[1].re - FRAC_1_SQRT_2).abs() < 1e-6);

    // H|+> = |0> (H^2 = I)
    state.apply_single_gate(0, &Matrix2x2::hadamard());
    assert!((state.amplitudes[0].re - 1.0).abs() < 1e-6);
    assert!(state.amplitudes[1].norm() < 1e-6);
}

#[test]
fn test_pauli_gates() {
    let mut state = QubitState::new(1);
    // X|0> = |1>
    state.apply_single_gate(0, &Matrix2x2::pauli_x());
    assert!(state.amplitudes[0].norm() < 1e-6);
    assert!((state.amplitudes[1].re - 1.0).abs() < 1e-6);

    // Z|1> = -|1>
    state.apply_single_gate(0, &Matrix2x2::pauli_z());
    assert!((state.amplitudes[1].re + 1.0).abs() < 1e-6);

    // Y(-|1>) = +i|0>
    state.apply_single_gate(0, &Matrix2x2::pauli_y());
    assert!((state.amplitudes[0].im - 1.0).abs() < 1e-6);
    assert!(state.amplitudes[1].norm() < 1e-6);
}

#[test]
fn test_bell_state_phi_plus() {
    let qc = create_bell_circuit(0);
    let state = qc.run();

    // (|00> + |11>) / sqrt(2)
    let p = state.probabilities();
    assert!((p[0] - 0.5).abs() < 1e-6, "p[00] should be 0.5");
    assert!(p[1] < 1e-6, "p[01] should be 0.0");
    assert!(p[2] < 1e-6, "p[10] should be 0.0");
    assert!((p[3] - 0.5).abs() < 1e-6, "p[11] should be 0.5");

    // Single qubit reduced density matrix on entangled state has purity = 0.5
    let (_, _, _, purity0) = state.bloch_vector(0);
    let (_, _, _, purity1) = state.bloch_vector(1);
    assert!((purity0 - 0.5).abs() < 1e-6);
    assert!((purity1 - 0.5).abs() < 1e-6);
}

#[test]
fn test_ghz_state() {
    let n = 5;
    let qc = create_ghz_circuit(n);
    let state = qc.run();

    let p = state.probabilities();
    let dim = 1 << n;
    assert!((p[0] - 0.5).abs() < 1e-6, "p[00000] should be 0.5");
    assert!((p[dim - 1] - 0.5).abs() < 1e-6, "p[11111] should be 0.5");

    for i in 1..(dim - 1) {
        assert!(p[i] < 1e-6, "Non-GHZ components must be zero");
    }
}

#[test]
fn test_superdense_coding() {
    for &(b1, b0) in &[(false, false), (false, true), (true, false), (true, true)] {
        let qc = create_superdense_circuit(b1, b0);
        let state = qc.run();
        let p = state.probabilities();

        let expected_idx = (if b0 { 1 } else { 0 }) | (if b1 { 2 } else { 0 });
        assert!(
            (p[expected_idx] - 1.0).abs() < 1e-6,
            "Superdense coding failed for bits b1={}, b0={}",
            b1,
            b0
        );
    }
}

#[test]
fn test_quantum_teleportation() {
    // Teleport state with theta = PI/3, phi = PI/4
    let theta = PI / 3.0;
    let phi = PI / 4.0;
    let qc = create_teleportation_circuit(theta, phi);
    let state = qc.run();

    // Verify Bob's qubit (qubit 2)
    let (rx, ry, rz, purity) = state.bloch_vector(2);

    // Expected Bloch vector for |psi> = cos(theta/2)|0> + e^(i*phi)sin(theta/2)|1>
    let expected_rx = theta.sin() * phi.cos();
    let expected_ry = theta.sin() * phi.sin();
    let expected_rz = theta.cos();

    assert!((rx - expected_rx).abs() < 1e-5, "rx mismatch");
    assert!((ry - expected_ry).abs() < 1e-5, "ry mismatch");
    assert!((rz - expected_rz).abs() < 1e-5, "rz mismatch");
    assert!((purity - 1.0).abs() < 1e-5, "Bob's state must be pure");
}

#[test]
fn test_qft_inverse_identity() {
    let n = 3;
    let mut qc = QuantumCircuit::new(n);
    let q_indices: Vec<usize> = (0..n).collect();

    // Prepare arbitrary state
    qc.ry(0, 0.7);
    qc.h(1);
    qc.cx(1, 2);

    let initial_state = qc.run();

    // Append QFT then IQFT
    append_qft(&mut qc, &q_indices);
    append_iqft(&mut qc, &q_indices);

    let final_state = qc.run();

    for i in 0..(1 << n) {
        let diff = (initial_state.amplitudes[i] - final_state.amplitudes[i]).norm();
        assert!(diff < 1e-6, "QFT followed by IQFT must be Identity");
    }
}

#[test]
fn test_grover_search() {
    let n = 3; // 8 items
    let target = 6;
    let qc = create_grover_circuit(n, target, None);
    let state = qc.run();

    let p = state.probabilities();
    // In 3 qubits with 2 iterations, target probability should be > 90%
    assert!(
        p[target] > 0.90,
        "Grover search failed to amplify target: prob was {:.4}",
        p[target]
    );
}

#[test]
fn test_projective_measurement_collapse() {
    let mut rng = rand::thread_rng();
    let mut state = QubitState::new(2);

    // Create Bell pair
    state.apply_single_gate(0, &Matrix2x2::hadamard());
    state.apply_controlled_gate(&[0], 1, &Matrix2x2::pauli_x());

    // Measure qubit 0
    let (m0, _) = state.measure_qubit(0, &mut rng);

    // After collapse, measuring qubit 0 again must give m0 with probability 1.0
    let p0_second = state.probability_zero(0);
    if !m0 {
        assert!((p0_second - 1.0).abs() < 1e-6);
    } else {
        assert!(p0_second < 1e-6);
    }

    // Due to entanglement, qubit 1 must also now have collapsed to m0!
    let p1_zero = state.probability_zero(1);
    if !m0 {
        assert!((p1_zero - 1.0).abs() < 1e-6);
    } else {
        assert!(p1_zero < 1e-6);
    }
}
