use crate::quantum::QuantumCircuit;

/// Creates a circuit that produces one of the 4 Bell states on 2 qubits
/// index:
/// 0: |Phi+> = (|00> + |11>) / sqrt(2)
/// 1: |Phi-> = (|00> - |11>) / sqrt(2)
/// 2: |Psi+> = (|01> + |10>) / sqrt(2)
/// 3: |Psi-> = (|01> - |10>) / sqrt(2)
pub fn create_bell_circuit(index: usize) -> QuantumCircuit {
    let mut qc = QuantumCircuit::new(2);
    match index {
        0 => {
            // |Phi+>
            qc.h(0);
            qc.cx(0, 1);
        }
        1 => {
            // |Phi->
            qc.x(0);
            qc.h(0);
            qc.cx(0, 1);
        }
        2 => {
            // |Psi+>
            qc.x(1);
            qc.h(0);
            qc.cx(0, 1);
        }
        3 => {
            // |Psi->
            qc.x(0);
            qc.x(1);
            qc.h(0);
            qc.cx(0, 1);
        }
        _ => {
            qc.h(0);
            qc.cx(0, 1);
        }
    }
    qc
}

/// Creates a Greenberger–Horne–Zeilinger (GHZ) state on n qubits:
/// |GHZ> = (|00...0> + |11...1>) / sqrt(2)
pub fn create_ghz_circuit(num_qubits: usize) -> QuantumCircuit {
    assert!(num_qubits >= 2, "GHZ state requires at least 2 qubits");
    let mut qc = QuantumCircuit::new(num_qubits);
    qc.h(0);
    for i in 0..(num_qubits - 1) {
        qc.cx(i, i + 1);
    }
    qc
}

/// Creates a 3-qubit W-state:
/// |W> = (|001> + |010> + |100>) / sqrt(3)
pub fn create_w_state_3qubit() -> QuantumCircuit {
    let mut qc = QuantumCircuit::new(3);
    // theta1 = 2 * arccos(1/sqrt(3))
    let theta1 = 2.0 * (1.0 / 3.0_f64.sqrt()).acos();
    qc.ry(0, theta1);
    qc.cx(0, 1);
    // theta2 = 2 * arccos(1/sqrt(2)) = pi/2
    let theta2 = std::f64::consts::FRAC_PI_2;
    qc.ry(0, -theta2); // rotation to split remaining amplitude
    qc.cx(1, 0);
    qc.ry(0, theta2);
    qc.cx(0, 2);
    qc.cx(1, 0);
    qc
}
