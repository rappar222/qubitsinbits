# High-Performance Quantum Computer Simulator in Rust

A state-of-the-art $N$-qubit quantum computer simulator written in Rust. It features statevector manipulation via direct bit-mask indexing, multi-threaded parallel execution (via `rayon`), an interactive terminal REPL, circuit visualization, and pre-built quantum algorithms.

---

## Features

- **Built-In OpenQASM 2.0 IDE & Debugger**:
  - **Full OpenQASM 2.0 Parser**: Run real OpenQASM code directly inside the simulator without external tools.
  - **IDE Code Editor**: Interactive code editor with line-by-line syntax checking, instant compiler feedback, and error reporting.
  - **Step-by-Step Instruction Stepper**: Step line-by-line forward/backward through your QASM program and watch the quantum state evolve after each gate!
  - **Side-by-Side Live Dashboard**: Instant live statevector table, Dirac notation, Bloch spheres, and measurement shots simulator while typing code.
  - **Code Templates & Quick Insert**: Pre-loaded with Bell states, GHZ, Teleportation, Superdense coding, Grover's search, QFT, and quick gate insert buttons (`+ H`, `+ CX`, `+ X`, `+ Rz`, `+ Measure`).
  - **File I/O**: Open and save `.qasm` files directly from the IDE.
  - **CLI QASM Runner**: Execute `.qasm` scripts directly from your terminal with `qubitbit run file.qasm`.

- **Dual GUI & CLI Interfaces**:
  - **Native Desktop GUI** (powered by `eframe` / `egui`):
    - Visual interactive quantum circuit track.
    - Gate palette: click to apply single-qubit ($H, X, Y, Z, S, S^\dagger, T, T^\dagger$), rotation ($R_x, R_y, R_z$, Phase), and multi-qubit ($CX, CZ, SWAP, CP, CCX$) gates.
    - Live statevector table with Dirac notation, complex numbers, magnitude, phase, and probability progress bars.
    - 2D Bloch sphere projections for each individual qubit showing state vectors, spherical angles $(\theta, \phi)$, and purity to visualize entanglement.
    - Monte Carlo measurement simulator with configurable shots and interactive bar charts.
    - Live OpenQASM 2.0 viewer and one-click clipboard exporter.
    - One-click algorithm presets: Bell states, GHZ, Teleportation, Superdense Coding, Grover's Search, and QFT.
  - **Interactive Terminal REPL & CLI**: Full command-line interface with step-by-step evaluation, ASCII diagrams, and subcommands.
  - Full $2^N$ complex amplitude state space (`num-complex::Complex64`).
  - Bit-stride gate application without allocating sparse/dense matrix operators.
  - Multi-threaded parallel acceleration for registers $\ge 12$ qubits.
- **Comprehensive Quantum Gate Library**:
  - **Single-Qubit**: Pauli-$X$, $Y$, $Z$, Hadamard ($H$), Phase ($S$, $S^\dagger$), $T$, $T^\dagger$, arbitrary single-qubit unitaries $U(\theta, \phi, \lambda)$, and rotations $R_x(\theta)$, $R_y(\theta)$, $R_z(\theta)$, $P(\lambda)$.
  - **Two-Qubit / Entangling**: CNOT ($CX$), $CY$, $CZ$, Controlled-$H$, $SWAP$, Controlled-Phase $CP(\lambda)$.
  - **Multi-Qubit**: Toffoli ($CCX$), Fredkin ($CSWAP$), Multi-Controlled $X$ ($MCX$), Multi-Controlled $Z$ ($MCZ$).
- **Wavefunction Measurements & Diagnostics**:
  - Projective single-qubit measurement with state collapse and renormalization.
  - Non-destructive statistical shot sampling (e.g. 1000+ shots) with ASCII bar histograms.
  - Expectation value calculation $\langle Z_i \rangle$.
  - Partial trace and single-qubit reduced density matrix calculation.
  - Bloch sphere 3D vector $(r_x, r_y, r_z)$ and state purity ($1.0$ = pure, $0.5$ = maximally entangled).
- **Quantum Circuit Tools**:
  - ASCII / Unicode circuit diagram generation.
  - OpenQASM 2.0 export.
- **Pre-Built Quantum Algorithms**:
  - **Bell States** ($\Phi^\pm, \Psi^\pm$) & **GHZ States** ($N \ge 2$).
  - **Quantum Teleportation** (arbitrary state teleportation with 100% fidelity).
  - **Superdense Coding** (transmitting 2 classical bits using 1 entangled qubit).
  - **Deutsch-Jozsa Algorithm** (distinguishes constant vs balanced functions in 1 query).
  - **Grover's Search Algorithm** (arbitrary $N$-qubit search with quadratic speedup).
  - **Quantum Fourier Transform (QFT)** & Inverse QFT (IQFT).
- **Interactive REPL**:
  - Live shell to create qubits, apply gates, view state vectors, sample measurements, inspect Bloch vectors, and load algorithms.

---

## Installation & Build

Ensure Rust and Cargo are installed (1.70+):

```bash
cargo build --release
```

Run tests:

```bash
cargo test
```

---

## Usage Guide

### 1. Interactive REPL

Launch the interactive REPL with 3 qubits (or any $N$):

```bash
qubitbit --cli
# or
qubitbit repl --qubits 3
```

Within the REPL:
```text
quantum[3]> h 0
quantum[3]> cx 0 1
quantum[3]> cx 1 2
quantum[3]> state
quantum[3]> bloch
quantum[3]> sample 1000
quantum[3]> diagram
quantum[3]> qasm
quantum[3]> quit
```

#### REPL Commands:
| Command | Description | Example |
|---|---|---|
| `new <N>` | Re-initialize register with $N$ qubits | `new 4` |
| `h <q>` | Apply Hadamard gate | `h 0` |
| `x`, `y`, `z` | Apply Pauli gates | `x 1` |
| `rx`, `ry`, `rz` | Rotation gates (supports `pi`, `pi/2`, floats) | `ry 0 pi/3` |
| `cx` / `cnot` | Controlled-NOT | `cx 0 1` |
| `cz` | Controlled-Z | `cz 0 1` |
| `swap` | Swap two qubits | `swap 0 1` |
| `ccx` | Toffoli gate (3 qubits) | `ccx 0 1 2` |
| `measure <q>` | Projective measurement (collapses state) | `measure 0` |
| `state` / `show` | Display amplitude table & Dirac notation | `state` |
| `bloch` | Display Bloch sphere coordinates and purity | `bloch` |
| `sample [shots]` | Sample shots and display ASCII histogram | `sample 1000` |
| `diagram` | Display ASCII circuit diagram | `diagram` |
| `qasm` | Export OpenQASM 2.0 code | `qasm` |
| `load <algo>` | Load built-in algorithm (`bell`, `ghz`, `grover`, `teleport`, `qft`, `dj`) | `load grover 3 5` |
| `reset` | Re-evaluate circuit to initial state | `reset` |
| `clear` | Clear all gates | `clear` |

---

### 2. CLI Subcommands

#### Bell State Entanglement
```bash
cargo run --release -- bell --state-type 0 --shots 1000
```

#### N-Qubit GHZ State
```bash
cargo run --release -- ghz --qubits 5 --shots 1000
```

#### Quantum Teleportation
```bash
cargo run --release -- teleport --theta 1.047197 --phi 0.0
```

#### Grover's Search Algorithm
Search a database of $2^3 = 8$ items for target `5`:
```bash
cargo run --release -- grover --qubits 3 --target 5 --shots 1000
```

#### Quantum Fourier Transform (QFT)
Apply QFT to basis state $|1\rangle$:
```bash
cargo run --release -- qft --qubits 3 --input 1
```

#### Deutsch-Jozsa Algorithm
Query balanced or constant oracles:
```bash
cargo run --release -- deutsch-jozsa --qubits 3 --oracle balanced
cargo run --release -- deutsch-jozsa --qubits 3 --oracle constant0
```

#### Simulator Benchmark
Benchmark gate execution from 4 to 20 qubits:
```bash
cargo run --release -- benchmark --max-qubits 20
```

---

## Programmatic Library Usage

You can also use `quantum_sim` as a library in your own Rust applications:

```rust
use quantum_sim::quantum::QuantumCircuit;
use quantum_sim::display::{print_state_table, print_bloch_sphere};

fn main() {
    let mut qc = QuantumCircuit::new(2);
    qc.h(0);
    qc.cx(0, 1);

    let state = qc.run();
    print_state_table(&state, 1e-4);
    print_bloch_sphere(&state);
}
```
