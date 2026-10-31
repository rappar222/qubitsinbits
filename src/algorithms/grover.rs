use crate::quantum::QuantumCircuit;
use std::f64::consts::PI;

/// Create a Grover's Search circuit for `num_qubits` searching for `target` (|w>).
/// `iterations`: if None, uses the theoretical optimal iterations round(pi/4 * sqrt(2^n)).
pub fn create_grover_circuit(num_qubits: usize, target: usize, iterations: Option<usize>) -> QuantumCircuit {
    let dim = 1 << num_qubits;
    assert!(target < dim, "Target {} is out of range for {} qubits", target, num_qubits);
    assert!(num_qubits >= 2, "Grover search requires at least 2 qubits");

    let num_iters = iterations.unwrap_or_else(|| {
        let iters = (PI / 4.0 * (dim as f64).sqrt()).round() as usize;
        iters.max(1)
    });

    let mut qc = QuantumCircuit::new(num_qubits);

    // 1. Initialize uniform superposition |s> = H^{\otimes n} |0...0>
    for q in 0..num_qubits {
        qc.h(q);
    }

    let all_controls: Vec<usize> = (0..(num_qubits - 1)).collect();
    let last_qubit = num_qubits - 1;

    for _ in 0..num_iters {
        // --- ORACLE for target |w> ---
        // Flip bits that are 0 in target
        for q in 0..num_qubits {
            if (target & (1 << q)) == 0 {
                qc.x(q);
            }
        }

        // Multi-controlled Z to flip phase of |11...1>
        if num_qubits == 2 {
            qc.cz(0, 1);
        } else {
            qc.mcz(all_controls.clone(), last_qubit);
        }

        // Uncompute X gates
        for q in 0..num_qubits {
            if (target & (1 << q)) == 0 {
                qc.x(q);
            }
        }

        // --- DIFFUSION OPERATOR (Inversion about the mean) ---
        // H^{\otimes n}
        for q in 0..num_qubits {
            qc.h(q);
        }
        // X^{\otimes n}
        for q in 0..num_qubits {
            qc.x(q);
        }
        // Multi-controlled Z
        if num_qubits == 2 {
            qc.cz(0, 1);
        } else {
            qc.mcz(all_controls.clone(), last_qubit);
        }
        // X^{\otimes n}
        for q in 0..num_qubits {
            qc.x(q);
        }
        // H^{\otimes n}
        for q in 0..num_qubits {
            qc.h(q);
        }
    }

    qc
}
