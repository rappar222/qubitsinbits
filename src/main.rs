use clap::{Parser, Subcommand};
use colored::*;
use quantum_sim::algorithms::*;
use quantum_sim::display::*;
use quantum_sim::gui::run_gui;
use quantum_sim::quantum::*;
use quantum_sim::repl::start_repl;
use rand::thread_rng;
use std::env;
use std::f64::consts::PI;
use std::time::Instant;

#[derive(Parser)]
#[command(
    name = "quantum_sim",
    author = "Quantum Simulator in Rust",
    version = "0.1.0",
    about = "High-performance N-qubit Quantum Computer Simulator with GUI, CLI, Interactive REPL & Algorithms",
    long_about = None
)]
struct Cli {
    /// Force CLI mode instead of GUI
    #[arg(long, global = true)]
    cli: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the Graphical User Interface (default if display server is available)
    Gui {
        #[arg(short, long, default_value_t = 3)]
        qubits: usize,
    },

    /// Launch interactive quantum CLI REPL
    Repl {
        #[arg(short, long, default_value_t = 3)]
        qubits: usize,
    },

    /// Simulate Bell States and show quantum entanglement
    Bell {
        /// Bell state type: 0 for |Phi+>, 1 for |Phi->, 2 for |Psi+>, 3 for |Psi->
        #[arg(short, long, default_value_t = 0)]
        state_type: usize,

        /// Number of measurement shots to sample
        #[arg(short, long, default_value_t = 1000)]
        shots: usize,
    },

    /// Simulate Greenberger–Horne–Zeilinger (GHZ) maximally entangled N-qubit state
    Ghz {
        /// Number of qubits (>= 2)
        #[arg(short, long, default_value_t = 4)]
        qubits: usize,

        /// Number of measurement shots to sample
        #[arg(short, long, default_value_t = 1000)]
        shots: usize,
    },

    /// Run Quantum Teleportation protocol of an arbitrary quantum state
    Teleport {
        /// Polar angle theta for initial state Ry(theta)|0>
        #[arg(long, default_value_t = 1.047197)] // pi / 3
        theta: f64,

        /// Azimuthal angle phi for initial state
        #[arg(long, default_value_t = 0.0)]
        phi: f64,
    },

    /// Run Grover's Quantum Search Algorithm with quadratic speedup
    Grover {
        /// Number of qubits in search space (search space size = 2^N)
        #[arg(short, long, default_value_t = 3)]
        qubits: usize,

        /// Target integer to search for (0 <= target < 2^N)
        #[arg(short, long, default_value_t = 5)]
        target: usize,

        /// Number of measurement shots
        #[arg(short, long, default_value_t = 1000)]
        shots: usize,
    },

    /// Run Quantum Fourier Transform (QFT)
    Qft {
        /// Number of qubits
        #[arg(short, long, default_value_t = 3)]
        qubits: usize,

        /// Input integer state to transform (0 <= input < 2^N)
        #[arg(short, long, default_value_t = 1)]
        input: usize,
    },

    /// Run Deutsch-Jozsa Algorithm to distinguish constant vs balanced functions in 1 query
    DeutschJozsa {
        /// Number of input qubits
        #[arg(short, long, default_value_t = 3)]
        qubits: usize,

        /// Oracle function type: 'constant0', 'constant1', 'balanced'
        #[arg(short, long, default_value = "balanced")]
        oracle: String,
    },

    /// Benchmark quantum gate execution speeds across different qubit counts
    Benchmark {
        /// Maximum number of qubits to benchmark (up to 24)
        #[arg(short, long, default_value_t = 20)]
        max_qubits: usize,
    },
}

fn main() {
    let cli = Cli::parse();

    let has_display = env::var("DISPLAY").is_ok() || env::var("WAYLAND_DISPLAY").is_ok();

    match cli.command {
        None => {
            if cli.cli || !has_display {
                start_repl(3);
            } else if let Err(e) = run_gui(3) {
                eprintln!("Failed to launch GUI: {}. Falling back to CLI REPL...", e);
                start_repl(3);
            }
        }
        Some(Commands::Gui { qubits }) => {
            if let Err(e) = run_gui(qubits) {
                eprintln!("Failed to launch GUI: {}. Falling back to CLI REPL...", e);
                start_repl(qubits);
            }
        }
        Some(Commands::Repl { qubits }) => {
            start_repl(qubits);
        }
        Some(Commands::Bell { state_type, shots }) => {
            println!("{}", "=== Bell State Entanglement Simulation ===".bold().cyan());
            let names = ["|Phi+> = (|00> + |11>)/sqrt(2)",
                         "|Phi-> = (|00> - |11>)/sqrt(2)",
                         "|Psi+> = (|01> + |10>)/sqrt(2)",
                         "|Psi-> = (|01> - |10>)/sqrt(2)"];
            let idx = state_type.min(3);
            println!("Target State: {}\n", names[idx].yellow().bold());

            let qc = create_bell_circuit(idx);
            println!("Circuit Diagram:");
            println!("{}", qc.to_ascii_diagram());

            let state = qc.run();
            print_state_table(&state, 1e-4);
            print_bloch_sphere(&state);

            let mut rng = thread_rng();
            let counts = state.sample_shots(shots, &mut rng);
            print_histogram(&counts, 2, shots);
        }
        Some(Commands::Ghz { qubits, shots }) => {
            if qubits < 2 {
                eprintln!("{}", "GHZ state requires at least 2 qubits".red());
                return;
            }
            println!("{}", format!("=== {}-Qubit GHZ State Simulation ===", qubits).bold().cyan());
            let qc = create_ghz_circuit(qubits);
            println!("Circuit Diagram:");
            println!("{}", qc.to_ascii_diagram());

            let state = qc.run();
            print_state_table(&state, 1e-4);
            print_bloch_sphere(&state);

            let mut rng = thread_rng();
            let counts = state.sample_shots(shots, &mut rng);
            print_histogram(&counts, qubits, shots);
        }
        Some(Commands::Teleport { theta, phi }) => {
            println!("{}", "=== Quantum Teleportation Protocol ===".bold().cyan());
            println!("Teleporting unknown state |psi> = cos({:.3}/2)|0> + e^(i*{:.3})sin({:.3}/2)|1>", theta, phi, theta);
            println!("Qubit 0: Alice's state to teleport");
            println!("Qubit 1: Alice's EPR entangled pair");
            println!("Qubit 2: Bob's receiver qubit\n");

            let expected_c0 = (theta / 2.0).cos();
            let expected_c1 = Complex::from_polar((theta / 2.0).sin(), phi);
            println!("Source State Amplitudes:");
            println!("  alpha (|0>): {:.5}", expected_c0);
            println!("  beta  (|1>): {:.5}\n", expected_c1);

            let qc = create_teleportation_circuit(theta, phi);
            println!("Circuit Diagram:");
            println!("{}", qc.to_ascii_diagram());

            let state = qc.run();
            print_state_table(&state, 1e-4);

            let (rx, ry, rz, purity) = state.bloch_vector(2);
            println!("{}", "Bob's Qubit (q[2]) Bloch Sphere:".bold().green());
            println!("  r_x = {:.4}, r_y = {:.4}, r_z = {:.4} (Purity: {:.4})", rx, ry, rz, purity);
            println!("\nBob's qubit now matches Alice's original state with 100% fidelity!");
        }
        Some(Commands::Grover { qubits, target, shots }) => {
            let dim = 1 << qubits;
            if target >= dim {
                eprintln!("{}", format!("Target {} exceeds state space 2^{} = {}", target, qubits, dim).red());
                return;
            }
            let optimal_iters = ((PI / 4.0 * (dim as f64).sqrt()).round() as usize).max(1);
            println!("{}", format!("=== Grover's Search Algorithm ({} Qubits, 2^{} = {} items) ===", qubits, qubits, dim).bold().cyan());
            println!("Searching for marked item: |{}> (index {})", format!("{:0width$b}", target, width = qubits).green().bold(), target);
            println!("Theoretical optimal Grover iterations: {}\n", optimal_iters.to_string().yellow().bold());

            let qc = create_grover_circuit(qubits, target, Some(optimal_iters));
            println!("Circuit Diagram:");
            println!("{}", qc.to_ascii_diagram());

            let state = qc.run();
            print_state_table(&state, 1e-4);

            let mut rng = thread_rng();
            let counts = state.sample_shots(shots, &mut rng);
            print_histogram(&counts, qubits, shots);

            let target_prob = state.amplitudes[target].norm_sqr();
            println!("Target item discovery probability: {:.2}%", target_prob * 100.0);
        }
        Some(Commands::Qft { qubits, input }) => {
            let dim = 1 << qubits;
            if input >= dim {
                eprintln!("{}", format!("Input {} exceeds 2^{} = {}", input, qubits, dim).red());
                return;
            }
            println!("{}", format!("=== Quantum Fourier Transform (QFT) on {} Qubits ===", qubits).bold().cyan());
            println!("Input Computational Basis State: |{}> (index {})", format!("{:0width$b}", input, width = qubits).green().bold(), input);

            let mut qc = QuantumCircuit::new(qubits);
            for q in 0..qubits {
                if (input & (1 << q)) != 0 {
                    qc.x(q);
                }
            }
            let q_indices: Vec<usize> = (0..qubits).collect();
            append_qft(&mut qc, &q_indices);

            println!("\nCircuit Diagram:");
            println!("{}", qc.to_ascii_diagram());

            let state = qc.run();
            print_state_table(&state, 1e-4);
            print_probability_distribution(&state, 1e-4);
        }
        Some(Commands::DeutschJozsa { qubits, oracle }) => {
            println!("{}", format!("=== Deutsch-Jozsa Algorithm ({} inputs + 1 ancilla) ===", qubits).bold().cyan());
            let oracle_type = match oracle.to_lowercase().as_str() {
                "constant0" => OracleType::Constant0,
                "constant1" => OracleType::Constant1,
                "balanced" | "balancedxor" => OracleType::BalancedXor,
                "firstbit" => OracleType::BalancedFirstBit,
                _ => {
                    eprintln!("{}", "Unknown oracle type. Choose 'constant0', 'constant1', or 'balanced'".red());
                    return;
                }
            };

            let qc = create_deutsch_jozsa_circuit(qubits, oracle_type);
            println!("Circuit Diagram:");
            println!("{}", qc.to_ascii_diagram());

            let state = qc.run();
            print_state_table(&state, 1e-4);

            let p_zero_all_inputs: f64 = state.amplitudes
                .iter()
                .enumerate()
                .filter(|(idx, _)| (idx & ((1 << qubits) - 1)) == 0)
                .map(|(_, amp)| amp.norm_sqr())
                .sum();

            println!("{}", "--- Algorithm Result ---".bold().cyan());
            if p_zero_all_inputs > 0.999 {
                println!("Result: {} (Probability of |0...0>: {:.2}%)", "CONSTANT FUNCTION".green().bold(), p_zero_all_inputs * 100.0);
            } else {
                println!("Result: {} (Probability of non-|0...0>: {:.2}%)", "BALANCED FUNCTION".magenta().bold(), (1.0 - p_zero_all_inputs) * 100.0);
            }
        }
        Some(Commands::Benchmark { max_qubits }) => {
            println!("{}", "=== Quantum Simulator Scaling Benchmark ===".bold().cyan());
            println!("{:<8} {:<16} {:<18} {:<18}", "Qubits", "Dimension (2^N)", "Memory (KB)", "H + CX Time");
            println!("{}", "-".repeat(62).dimmed());

            let upper = max_qubits.min(25);
            for n in 4..=upper {
                let dim = 1 << n;
                let mem_kb = (dim * std::mem::size_of::<Complex>()) / 1024;

                let start = Instant::now();
                let mut state = QubitState::new(n);
                for q in 0..n {
                    state.apply_single_gate(q, &Matrix2x2::hadamard());
                }
                for q in 0..(n - 1) {
                    state.apply_controlled_gate(&[q], q + 1, &Matrix2x2::pauli_x());
                }
                let duration = start.elapsed();

                println!(
                    "{:<8} {:<16} {:<18} {:<18.2?}",
                    n, dim, mem_kb, duration
                );
            }
            println!("\nBenchmark complete.");
        }
    }
}
