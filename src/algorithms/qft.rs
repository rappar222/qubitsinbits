use crate::quantum::QuantumCircuit;
use std::f64::consts::PI;

/// Appends Quantum Fourier Transform (QFT) gates to the circuit for the specified qubits
pub fn append_qft(qc: &mut QuantumCircuit, qubits: &[usize]) {
    let n = qubits.len();
    for i in 0..n {
        let target = qubits[i];
        qc.h(target);
        for j in (i + 1)..n {
            let control = qubits[j];
            let k = j - i + 1;
            let angle = 2.0 * PI / (1 << k) as f64;
            qc.cp(control, target, angle);
        }
    }

    // Reverse qubit order with SWAPs
    for i in 0..(n / 2) {
        qc.swap(qubits[i], qubits[n - 1 - i]);
    }
}

/// Appends Inverse Quantum Fourier Transform (IQFT) gates
pub fn append_iqft(qc: &mut QuantumCircuit, qubits: &[usize]) {
    let n = qubits.len();

    // Reverse SWAPs first
    for i in 0..(n / 2) {
        qc.swap(qubits[i], qubits[n - 1 - i]);
    }

    // Reverse gates in backwards order with negative phase
    for i in (0..n).rev() {
        let target = qubits[i];
        for j in ((i + 1)..n).rev() {
            let control = qubits[j];
            let k = j - i + 1;
            let angle = -2.0 * PI / (1 << k) as f64;
            qc.cp(control, target, angle);
        }
        qc.h(target);
    }
}

/// Create a standalone QFT circuit for `num_qubits`
pub fn create_qft_circuit(num_qubits: usize) -> QuantumCircuit {
    let mut qc = QuantumCircuit::new(num_qubits);
    let qubits: Vec<usize> = (0..num_qubits).collect();
    append_qft(&mut qc, &qubits);
    qc
}
