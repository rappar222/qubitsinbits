use crate::algorithms::{
    create_bell_circuit, create_deutsch_jozsa_circuit, create_ghz_circuit, create_grover_circuit,
    create_qft_circuit, create_superdense_circuit, create_teleportation_circuit, OracleType,
};
use crate::display::{print_bloch_sphere, print_histogram, print_state_table};
use crate::quantum::QuantumCircuit;
use colored::*;
use std::f64::consts::PI;
use std::io::{self, Write};

fn parse_angle(s: &str) -> Result<f64, String> {
    let s = s.trim().to_lowercase();
    if s == "pi" {
        return Ok(PI);
    }
    if s == "-pi" {
        return Ok(-PI);
    }
    if s == "pi/2" {
        return Ok(PI / 2.0);
    }
    if s == "-pi/2" {
        return Ok(-PI / 2.0);
    }
    if s == "pi/4" {
        return Ok(PI / 4.0);
    }
    if s == "-pi/4" {
        return Ok(-PI / 4.0);
    }
    if s == "2*pi" || s == "2pi" {
        return Ok(2.0 * PI);
    }
    s.parse::<f64>()
        .map_err(|_| format!("Invalid angle number: '{}'", s))
}

pub fn start_repl(initial_qubits: usize) {
    let mut num_qubits = initial_qubits.max(1).min(26);
    let mut circuit = QuantumCircuit::new(num_qubits);
    let mut state = circuit.run();

    println!("{}", "=================================================".bright_blue());
    println!("{}", "       Quantum Computing Simulator REPL          ".bold().cyan());
    println!("{}", "=================================================".bright_blue());
    println!("Simulating {} qubits (state space: {} amplitudes).", num_qubits, 1 << num_qubits);
    println!("Type {} for a list of commands, or {} to leave.\n", "help".yellow().bold(), "quit".yellow().bold());

    let stdin = io::stdin();
    let mut line_buffer = String::new();

    loop {
        print!("{} ", format!("quantum[{}]>", num_qubits).cyan().bold());
        io::stdout().flush().unwrap();

        line_buffer.clear();
        if stdin.read_line(&mut line_buffer).is_err() {
            break;
        }

        let line = line_buffer.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        let cmd = parts[0].to_lowercase();

        match cmd.as_str() {
            "help" | "?" => {
                println!("{}", "Available Commands:".bold().yellow());
                println!("  {} <N>                 - Create a new quantum register with N qubits", "new".green());
                println!("  {}                       - Show state amplitudes table & Dirac notation", "state / show".green());
                println!("  {}                       - Show circuit diagram in ASCII", "diagram / circuit".green());
                println!("  {}                       - Show Bloch sphere coordinates & purity", "bloch".green());
                println!("  {} [shots]               - Sample shots (default: 1000) & plot histogram", "sample".green());
                println!("  {} <q>                 - Projective measurement on qubit q (collapses state)", "measure".green());
                println!("  {} <q>                   - Calculate expectation value <Z_q>", "expz".green());
                println!("  {}                        - Output OpenQASM 2.0 code", "qasm".green());
                println!("  {}                       - Reset state back to |00...0>", "reset".green());
                println!("  {}                       - Clear circuit gates", "clear".green());
                println!("  {} <algo>               - Load algorithm: bell, ghz, grover, teleport, qft", "load".green());
                println!("  {}", "Gates:".bold().yellow());
                println!("    h <q>                  - Hadamard gate");
                println!("    x <q>, y <q>, z <q>    - Pauli X, Y, Z gates");
                println!("    s <q>, sdg <q>         - Phase S and S† gates");
                println!("    t <q>, tdg <q>         - T (π/8) and T† gates");
                println!("    rx <q> <theta>         - Rotation around X axis (e.g. 'rx 0 pi/2')");
                println!("    ry <q> <theta>         - Rotation around Y axis (e.g. 'ry 0 pi')");
                println!("    rz <q> <theta>         - Rotation around Z axis");
                println!("    p <q> <lambda>         - Phase shift gate");
                println!("    cx/cnot <ctrl> <tgt>   - Controlled-NOT gate");
                println!("    cz <ctrl> <tgt>        - Controlled-Z gate");
                println!("    swap <q1> <q2>         - SWAP gate");
                println!("    ccx <c1> <c2> <tgt>    - Toffoli (CCX) gate");
                println!("    cswap <c> <q1> <q2>    - Fredkin (CSWAP) gate");
                println!("    cp <ctrl> <tgt> <rad>  - Controlled-Phase gate");
                println!("  {} / {}              - Exit the REPL", "quit".green(), "exit".green());
                println!();
            }
            "quit" | "exit" => {
                println!("Goodbye!");
                break;
            }
            "new" => {
                if parts.len() < 2 {
                    println!("{}", "Usage: new <num_qubits>".red());
                    continue;
                }
                match parts[1].parse::<usize>() {
                    Ok(n) if n > 0 && n <= 26 => {
                        num_qubits = n;
                        circuit = QuantumCircuit::new(num_qubits);
                        state = circuit.run();
                        println!("Initialized fresh state with {} qubits.", num_qubits);
                    }
                    Ok(_) => println!("{}", "Number of qubits must be between 1 and 26".red()),
                    Err(_) => println!("{}", "Invalid integer for qubits".red()),
                }
            }
            "reset" => {
                state = circuit.run();
                println!("Reset state back to initial circuit evaluation.");
            }
            "clear" => {
                circuit = QuantumCircuit::new(num_qubits);
                state = circuit.run();
                println!("Cleared circuit and reset state to |0...0>.");
            }
            "state" | "show" => {
                print_state_table(&state, 1e-4);
            }
            "diagram" | "circuit" => {
                println!("{}", circuit.to_ascii_diagram());
            }
            "bloch" => {
                print_bloch_sphere(&state);
            }
            "qasm" => {
                println!("{}", "--- OpenQASM 2.0 ---".bold().cyan());
                println!("{}", circuit.to_qasm());
            }
            "sample" => {
                let shots = if parts.len() > 1 {
                    parts[1].parse::<usize>().unwrap_or(1000)
                } else {
                    1000
                };
                let mut rng = rand::thread_rng();
                let counts = state.sample_shots(shots, &mut rng);
                print_histogram(&counts, num_qubits, shots);
            }
            "measure" => {
                if parts.len() < 2 {
                    println!("{}", "Usage: measure <qubit>".red());
                    continue;
                }
                match parts[1].parse::<usize>() {
                    Ok(q) if q < num_qubits => {
                        let mut rng = rand::thread_rng();
                        let (outcome, prob) = state.measure_qubit(q, &mut rng);
                        circuit.measure(q);
                        let bit_val = if outcome { 1 } else { 0 };
                        println!(
                            "Measured qubit {}: {} (collapse probability: {:.2}%)",
                            q,
                            bit_val.to_string().green().bold(),
                            prob * 100.0
                        );
                        println!("Updated state Dirac: {}", state.to_dirac_string(1e-4).yellow());
                    }
                    _ => println!("{}", format!("Qubit must be between 0 and {}", num_qubits - 1).red()),
                }
            }
            "expz" => {
                if parts.len() < 2 {
                    println!("{}", "Usage: expz <qubit>".red());
                    continue;
                }
                match parts[1].parse::<usize>() {
                    Ok(q) if q < num_qubits => {
                        let ez = state.expectation_z(q);
                        println!("<Z_{}> = {:.5}", q, ez);
                    }
                    _ => println!("{}", "Qubit out of range".red()),
                }
            }
            "h" => {
                if let Some(q) = parse_qubit(parts.get(1), num_qubits) {
                    circuit.h(q);
                    state.apply_op(&crate::quantum::GateOp::H(q));
                    println!("Applied H on q[{}]. State: {}", q, state.to_dirac_string(1e-4));
                }
            }
            "x" => {
                if let Some(q) = parse_qubit(parts.get(1), num_qubits) {
                    circuit.x(q);
                    state.apply_op(&crate::quantum::GateOp::X(q));
                    println!("Applied X on q[{}]. State: {}", q, state.to_dirac_string(1e-4));
                }
            }
            "y" => {
                if let Some(q) = parse_qubit(parts.get(1), num_qubits) {
                    circuit.y(q);
                    state.apply_op(&crate::quantum::GateOp::Y(q));
                    println!("Applied Y on q[{}]. State: {}", q, state.to_dirac_string(1e-4));
                }
            }
            "z" => {
                if let Some(q) = parse_qubit(parts.get(1), num_qubits) {
                    circuit.z(q);
                    state.apply_op(&crate::quantum::GateOp::Z(q));
                    println!("Applied Z on q[{}]. State: {}", q, state.to_dirac_string(1e-4));
                }
            }
            "s" => {
                if let Some(q) = parse_qubit(parts.get(1), num_qubits) {
                    circuit.s(q);
                    state.apply_op(&crate::quantum::GateOp::S(q));
                    println!("Applied S on q[{}]. State: {}", q, state.to_dirac_string(1e-4));
                }
            }
            "sdg" => {
                if let Some(q) = parse_qubit(parts.get(1), num_qubits) {
                    circuit.sdg(q);
                    state.apply_op(&crate::quantum::GateOp::Sdg(q));
                    println!("Applied S† on q[{}]. State: {}", q, state.to_dirac_string(1e-4));
                }
            }
            "t" => {
                if let Some(q) = parse_qubit(parts.get(1), num_qubits) {
                    circuit.t(q);
                    state.apply_op(&crate::quantum::GateOp::T(q));
                    println!("Applied T on q[{}]. State: {}", q, state.to_dirac_string(1e-4));
                }
            }
            "tdg" => {
                if let Some(q) = parse_qubit(parts.get(1), num_qubits) {
                    circuit.tdg(q);
                    state.apply_op(&crate::quantum::GateOp::Tdg(q));
                    println!("Applied T† on q[{}]. State: {}", q, state.to_dirac_string(1e-4));
                }
            }
            "rx" => {
                if parts.len() < 3 {
                    println!("{}", "Usage: rx <qubit> <theta>".red());
                    continue;
                }
                if let (Some(q), Ok(theta)) = (parse_qubit(parts.get(1), num_qubits), parse_angle(parts[2])) {
                    circuit.rx(q, theta);
                    state.apply_op(&crate::quantum::GateOp::Rx(q, theta));
                    println!("Applied Rx({:.4}) on q[{}]. State: {}", theta, q, state.to_dirac_string(1e-4));
                }
            }
            "ry" => {
                if parts.len() < 3 {
                    println!("{}", "Usage: ry <qubit> <theta>".red());
                    continue;
                }
                if let (Some(q), Ok(theta)) = (parse_qubit(parts.get(1), num_qubits), parse_angle(parts[2])) {
                    circuit.ry(q, theta);
                    state.apply_op(&crate::quantum::GateOp::Ry(q, theta));
                    println!("Applied Ry({:.4}) on q[{}]. State: {}", theta, q, state.to_dirac_string(1e-4));
                }
            }
            "rz" => {
                if parts.len() < 3 {
                    println!("{}", "Usage: rz <qubit> <theta>".red());
                    continue;
                }
                if let (Some(q), Ok(theta)) = (parse_qubit(parts.get(1), num_qubits), parse_angle(parts[2])) {
                    circuit.rz(q, theta);
                    state.apply_op(&crate::quantum::GateOp::Rz(q, theta));
                    println!("Applied Rz({:.4}) on q[{}]. State: {}", theta, q, state.to_dirac_string(1e-4));
                }
            }
            "p" => {
                if parts.len() < 3 {
                    println!("{}", "Usage: p <qubit> <lambda>".red());
                    continue;
                }
                if let (Some(q), Ok(lambda)) = (parse_qubit(parts.get(1), num_qubits), parse_angle(parts[2])) {
                    circuit.phase(q, lambda);
                    state.apply_op(&crate::quantum::GateOp::Phase(q, lambda));
                    println!("Applied Phase({:.4}) on q[{}]. State: {}", lambda, q, state.to_dirac_string(1e-4));
                }
            }
            "cx" | "cnot" => {
                if parts.len() < 3 {
                    println!("{}", "Usage: cx <control> <target>".red());
                    continue;
                }
                let c = parse_qubit(parts.get(1), num_qubits);
                let t = parse_qubit(parts.get(2), num_qubits);
                if let (Some(c), Some(t)) = (c, t) {
                    if c == t {
                        println!("{}", "Control and target cannot be the same qubit".red());
                        continue;
                    }
                    circuit.cx(c, t);
                    state.apply_op(&crate::quantum::GateOp::CX(c, t));
                    println!("Applied CX (ctrl: q[{}], tgt: q[{}]). State: {}", c, t, state.to_dirac_string(1e-4));
                }
            }
            "cz" => {
                if parts.len() < 3 {
                    println!("{}", "Usage: cz <control> <target>".red());
                    continue;
                }
                let c = parse_qubit(parts.get(1), num_qubits);
                let t = parse_qubit(parts.get(2), num_qubits);
                if let (Some(c), Some(t)) = (c, t) {
                    if c == t {
                        println!("{}", "Control and target cannot be the same qubit".red());
                        continue;
                    }
                    circuit.cz(c, t);
                    state.apply_op(&crate::quantum::GateOp::CZ(c, t));
                    println!("Applied CZ (ctrl: q[{}], tgt: q[{}]). State: {}", c, t, state.to_dirac_string(1e-4));
                }
            }
            "swap" => {
                if parts.len() < 3 {
                    println!("{}", "Usage: swap <q1> <q2>".red());
                    continue;
                }
                let q1 = parse_qubit(parts.get(1), num_qubits);
                let q2 = parse_qubit(parts.get(2), num_qubits);
                if let (Some(q1), Some(q2)) = (q1, q2) {
                    circuit.swap(q1, q2);
                    state.apply_op(&crate::quantum::GateOp::SWAP(q1, q2));
                    println!("Applied SWAP (q[{}], q[{}]). State: {}", q1, q2, state.to_dirac_string(1e-4));
                }
            }
            "ccx" => {
                if parts.len() < 4 {
                    println!("{}", "Usage: ccx <c1> <c2> <target>".red());
                    continue;
                }
                let c1 = parse_qubit(parts.get(1), num_qubits);
                let c2 = parse_qubit(parts.get(2), num_qubits);
                let t = parse_qubit(parts.get(3), num_qubits);
                if let (Some(c1), Some(c2), Some(t)) = (c1, c2, t) {
                    if c1 == c2 || c1 == t || c2 == t {
                        println!("{}", "Controls and target must be distinct qubits".red());
                        continue;
                    }
                    circuit.ccx(c1, c2, t);
                    state.apply_op(&crate::quantum::GateOp::CCX(c1, c2, t));
                    println!("Applied CCX (ctrls: q[{}], q[{}], tgt: q[{}]). State: {}", c1, c2, t, state.to_dirac_string(1e-4));
                }
            }
            "cswap" => {
                if parts.len() < 4 {
                    println!("{}", "Usage: cswap <control> <q1> <q2>".red());
                    continue;
                }
                let c = parse_qubit(parts.get(1), num_qubits);
                let q1 = parse_qubit(parts.get(2), num_qubits);
                let q2 = parse_qubit(parts.get(3), num_qubits);
                if let (Some(c), Some(q1), Some(q2)) = (c, q1, q2) {
                    circuit.cswap(c, q1, q2);
                    state.apply_op(&crate::quantum::GateOp::CSWAP(c, q1, q2));
                    println!("Applied CSWAP. State: {}", state.to_dirac_string(1e-4));
                }
            }
            "cp" => {
                if parts.len() < 4 {
                    println!("{}", "Usage: cp <control> <target> <angle>".red());
                    continue;
                }
                let c = parse_qubit(parts.get(1), num_qubits);
                let t = parse_qubit(parts.get(2), num_qubits);
                let p = parse_angle(parts[3]);
                if let (Some(c), Some(t), Ok(lambda)) = (c, t, p) {
                    circuit.cp(c, t, lambda);
                    state.apply_op(&crate::quantum::GateOp::CP(c, t, lambda));
                    println!("Applied CP({:.4}) on ctrl: {}, tgt: {}. State: {}", lambda, c, t, state.to_dirac_string(1e-4));
                }
            }
            "load" => {
                if parts.len() < 2 {
                    println!("{}", "Usage: load <algo_name> (bell, ghz, teleport, superdense, grover, qft, dj)".red());
                    continue;
                }
                let algo = parts[1].to_lowercase();
                match algo.as_str() {
                    "bell" => {
                        num_qubits = 2;
                        circuit = create_bell_circuit(0);
                        state = circuit.run();
                        println!("Loaded Bell State |Phi+> circuit.");
                    }
                    "ghz" => {
                        let n = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(3);
                        num_qubits = n;
                        circuit = create_ghz_circuit(n);
                        state = circuit.run();
                        println!("Loaded {}-qubit GHZ state circuit.", n);
                    }
                    "teleport" => {
                        num_qubits = 3;
                        circuit = create_teleportation_circuit(PI / 3.0, 0.0);
                        state = circuit.run();
                        println!("Loaded Quantum Teleportation circuit (|psi> = Ry(pi/3)|0>).");
                    }
                    "superdense" => {
                        num_qubits = 2;
                        circuit = create_superdense_circuit(true, true);
                        state = circuit.run();
                        println!("Loaded Superdense Coding circuit (transmitting '11').");
                    }
                    "grover" => {
                        let n = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(3);
                        let target = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(5);
                        num_qubits = n;
                        circuit = create_grover_circuit(n, target, None);
                        state = circuit.run();
                        println!("Loaded Grover Search circuit for {} qubits searching for target {}.", n, target);
                    }
                    "qft" => {
                        let n = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(3);
                        num_qubits = n;
                        circuit = create_qft_circuit(n);
                        state = circuit.run();
                        println!("Loaded Quantum Fourier Transform (QFT) on {} qubits.", n);
                    }
                    "dj" => {
                        num_qubits = 3;
                        circuit = create_deutsch_jozsa_circuit(2, OracleType::BalancedXor);
                        state = circuit.run();
                        println!("Loaded Deutsch-Jozsa circuit with balanced XOR oracle.");
                    }
                    _ => {
                        println!("{}", format!("Unknown algorithm '{}'. Available: bell, ghz, teleport, superdense, grover, qft, dj", algo).red());
                    }
                }
                print_state_table(&state, 1e-4);
            }
            _ => {
                println!("{}", format!("Unknown command: '{}'. Type 'help' for instructions.", cmd).red());
            }
        }
    }
}

fn parse_qubit(opt: Option<&&str>, max: usize) -> Option<usize> {
    match opt {
        Some(s) => match s.parse::<usize>() {
            Ok(q) if q < max => Some(q),
            Ok(q) => {
                println!("{}", format!("Qubit index {} out of range (num_qubits: {})", q, max).red());
                None
            }
            Err(_) => {
                println!("{}", format!("Invalid qubit index: '{}'", s).red());
                None
            }
        },
        None => {
            println!("{}", "Missing qubit argument".red());
            None
        }
    }
}
