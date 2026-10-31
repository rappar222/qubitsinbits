use crate::quantum::QuantumCircuit;

/// Quantum Teleportation Protocol
///
/// Qubit 0: Alice's state |psi> = alpha|0> + beta|1> to teleport
/// Qubit 1: Alice's half of the Bell pair
/// Qubit 2: Bob's half of the Bell pair
///
/// Input angles (theta, phi) allow preparing an arbitrary initial state on qubit 0:
/// |psi> = cos(theta/2)|0> + e^(i*phi) sin(theta/2)|1>
pub fn create_teleportation_circuit(theta: f64, phi: f64) -> QuantumCircuit {
    let mut qc = QuantumCircuit::new(3);

    // 1. Prepare arbitrary state on qubit 0
    qc.ry(0, theta);
    if phi.abs() > 1e-6 {
        qc.rz(0, phi);
    }

    // 2. Create EPR Bell pair between Qubit 1 and Qubit 2
    qc.h(1);
    qc.cx(1, 2);

    // 3. Alice performs Bell-state measurement on Qubit 0 and Qubit 1
    qc.cx(0, 1);
    qc.h(0);

    // 4. Bob applies conditional corrections based on measurement outcomes:
    // If Q1 is 1 -> Bob applies X on Q2 (equivalent to CX(1, 2))
    // If Q0 is 1 -> Bob applies Z on Q2 (equivalent to CZ(0, 2))
    qc.cx(1, 2);
    qc.cz(0, 2);

    qc
}

/// Superdense Coding Protocol
///
/// Encodes 2 classical bits (b1, b0) into 1 qubit of an entangled pair,
/// and transmits it so the receiver recovers both classical bits.
pub fn create_superdense_circuit(bit1: bool, bit0: bool) -> QuantumCircuit {
    let mut qc = QuantumCircuit::new(2);

    // 1. Prepare entangled Bell pair |Phi+> = (|00> + |11>) / sqrt(2)
    qc.h(0);
    qc.cx(0, 1);

    // 2. Alice encodes two classical bits on her qubit (qubit 0)
    // 00 -> I
    // 01 -> Z (sets bit 0)
    // 10 -> X (sets bit 1)
    // 11 -> X then Z (sets bits 0 and 1)
    if bit0 {
        qc.z(0);
    }
    if bit1 {
        qc.x(0);
    }

    // 3. Bob decodes by applying CX and H, then measuring
    qc.cx(0, 1);
    qc.h(0);

    qc
}
