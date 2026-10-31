use crate::quantum::QuantumCircuit;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OracleType {
    Constant0,
    Constant1,
    BalancedXor,
    BalancedFirstBit,
}

/// Builds the Deutsch-Jozsa circuit for `n` input qubits + 1 ancilla qubit (total n + 1 qubits).
pub fn create_deutsch_jozsa_circuit(num_input_qubits: usize, oracle: OracleType) -> QuantumCircuit {
    let total_qubits = num_input_qubits + 1;
    let ancilla = num_input_qubits;
    let mut qc = QuantumCircuit::new(total_qubits);

    // 1. Prepare ancilla in |-> = (|0> - |1>)/sqrt(2)
    qc.x(ancilla);
    qc.h(ancilla);

    // 2. Prepare input qubits in uniform superposition |+>
    for q in 0..num_input_qubits {
        qc.h(q);
    }

    // 3. Apply Oracle U_f: |x>|y> -> |x>|y + f(x)>
    match oracle {
        OracleType::Constant0 => {
            // f(x) = 0: do nothing
        }
        OracleType::Constant1 => {
            // f(x) = 1: invert ancilla
            qc.x(ancilla);
        }
        OracleType::BalancedFirstBit => {
            // f(x) = x_0: CNOT from input 0 to ancilla
            qc.cx(0, ancilla);
        }
        OracleType::BalancedXor => {
            // f(x) = x_0 ^ x_1 ^ ... ^ x_{n-1}
            for q in 0..num_input_qubits {
                qc.cx(q, ancilla);
            }
        }
    }

    // 4. Apply Hadamard to all input qubits
    for q in 0..num_input_qubits {
        qc.h(q);
    }

    qc
}
