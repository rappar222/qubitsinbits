use super::gate::GateOp;
use super::state::QubitState;

#[derive(Debug, Clone)]
pub struct QuantumCircuit {
    pub num_qubits: usize,
    pub operations: Vec<GateOp>,
}

impl QuantumCircuit {
    pub fn new(num_qubits: usize) -> Self {
        assert!(num_qubits > 0, "Number of qubits must be greater than 0");
        Self {
            num_qubits,
            operations: Vec::new(),
        }
    }

    pub fn add_op(&mut self, op: GateOp) -> &mut Self {
        for &q in &op.qubits_involved() {
            assert!(
                q < self.num_qubits,
                "Qubit index {} out of range (num_qubits: {})",
                q,
                self.num_qubits
            );
        }
        self.operations.push(op);
        self
    }

    pub fn h(&mut self, q: usize) -> &mut Self {
        self.add_op(GateOp::H(q))
    }

    pub fn x(&mut self, q: usize) -> &mut Self {
        self.add_op(GateOp::X(q))
    }

    pub fn y(&mut self, q: usize) -> &mut Self {
        self.add_op(GateOp::Y(q))
    }

    pub fn z(&mut self, q: usize) -> &mut Self {
        self.add_op(GateOp::Z(q))
    }

    pub fn s(&mut self, q: usize) -> &mut Self {
        self.add_op(GateOp::S(q))
    }

    pub fn sdg(&mut self, q: usize) -> &mut Self {
        self.add_op(GateOp::Sdg(q))
    }

    pub fn t(&mut self, q: usize) -> &mut Self {
        self.add_op(GateOp::T(q))
    }

    pub fn tdg(&mut self, q: usize) -> &mut Self {
        self.add_op(GateOp::Tdg(q))
    }

    pub fn rx(&mut self, q: usize, theta: f64) -> &mut Self {
        self.add_op(GateOp::Rx(q, theta))
    }

    pub fn ry(&mut self, q: usize, theta: f64) -> &mut Self {
        self.add_op(GateOp::Ry(q, theta))
    }

    pub fn rz(&mut self, q: usize, theta: f64) -> &mut Self {
        self.add_op(GateOp::Rz(q, theta))
    }

    pub fn phase(&mut self, q: usize, lambda: f64) -> &mut Self {
        self.add_op(GateOp::Phase(q, lambda))
    }

    pub fn cx(&mut self, c: usize, t: usize) -> &mut Self {
        self.add_op(GateOp::CX(c, t))
    }

    pub fn cz(&mut self, c: usize, t: usize) -> &mut Self {
        self.add_op(GateOp::CZ(c, t))
    }

    pub fn swap(&mut self, q1: usize, q2: usize) -> &mut Self {
        self.add_op(GateOp::SWAP(q1, q2))
    }

    pub fn ccx(&mut self, c1: usize, c2: usize, t: usize) -> &mut Self {
        self.add_op(GateOp::CCX(c1, c2, t))
    }

    pub fn cswap(&mut self, c: usize, q1: usize, q2: usize) -> &mut Self {
        self.add_op(GateOp::CSWAP(c, q1, q2))
    }

    pub fn cp(&mut self, c: usize, t: usize, lambda: f64) -> &mut Self {
        self.add_op(GateOp::CP(c, t, lambda))
    }

    pub fn mcx(&mut self, controls: Vec<usize>, target: usize) -> &mut Self {
        self.add_op(GateOp::MCX(controls, target))
    }

    pub fn mcz(&mut self, controls: Vec<usize>, target: usize) -> &mut Self {
        self.add_op(GateOp::MCZ(controls, target))
    }

    pub fn measure(&mut self, q: usize) -> &mut Self {
        self.add_op(GateOp::Measure(q))
    }

    /// Execute the circuit starting from |0...0> and return final state
    pub fn run(&self) -> QubitState {
        let mut state = QubitState::new(self.num_qubits);
        for op in &self.operations {
            state.apply_op(op);
        }
        state
    }

    /// Execute step-by-step and return each state snapshot
    pub fn run_with_history(&self) -> Vec<(GateOp, QubitState)> {
        let mut state = QubitState::new(self.num_qubits);
        let mut history = Vec::with_capacity(self.operations.len());
        for op in &self.operations {
            state.apply_op(op);
            history.push((op.clone(), state.clone()));
        }
        history
    }

    /// Generate an ASCII / Unicode circuit diagram
    pub fn to_ascii_diagram(&self) -> String {
        if self.operations.is_empty() {
            let mut out = String::new();
            for q in 0..self.num_qubits {
                out.push_str(&format!("q[{}]: ───\n", q));
            }
            return out;
        }

        // Each column represents one time slice / gate step
        // We'll build a grid of strings for each qubit wire and intermediate vertical wire
        let n_qubits = self.num_qubits;
        let mut qubit_lines: Vec<String> = (0..n_qubits)
            .map(|q| format!("q[{}]: ─", q))
            .collect();

        // Intermediate lines between qubits (for vertical connections)
        let mut inter_lines: Vec<String> = (0..n_qubits.saturating_sub(1))
            .map(|_| "       ".to_string())
            .collect();

        for op in &self.operations {
            let mut col_items = vec!["───".to_string(); n_qubits];
            let mut inter_items = vec!["   ".to_string(); n_qubits.saturating_sub(1)];

            match op {
                GateOp::H(q) => col_items[*q] = "[H]─".to_string(),
                GateOp::X(q) => col_items[*q] = "[X]─".to_string(),
                GateOp::Y(q) => col_items[*q] = "[Y]─".to_string(),
                GateOp::Z(q) => col_items[*q] = "[Z]─".to_string(),
                GateOp::S(q) => col_items[*q] = "[S]─".to_string(),
                GateOp::Sdg(q) => col_items[*q] = "[S†]".to_string(),
                GateOp::T(q) => col_items[*q] = "[T]─".to_string(),
                GateOp::Tdg(q) => col_items[*q] = "[T†]".to_string(),
                GateOp::Rx(q, _) => col_items[*q] = "[Rx]".to_string(),
                GateOp::Ry(q, _) => col_items[*q] = "[Ry]".to_string(),
                GateOp::Rz(q, _) => col_items[*q] = "[Rz]".to_string(),
                GateOp::Phase(q, _) => col_items[*q] = "[P]─".to_string(),
                GateOp::U3(q, _, _, _) => col_items[*q] = "[U]─".to_string(),
                GateOp::Custom1Q(q, _, name) => {
                    let mut s = format!("[{}]─", name);
                    if s.len() < 4 {
                        s.push('─');
                    }
                    col_items[*q] = s;
                }
                GateOp::CX(c, t) => {
                    col_items[*c] = "─●──".to_string();
                    col_items[*t] = "─⊕──".to_string();
                    let (min_q, max_q) = if c < t { (*c, *t) } else { (*t, *c) };
                    for i in min_q..max_q {
                        inter_items[i] = " │  ".to_string();
                    }
                    for i in (min_q + 1)..max_q {
                        col_items[i] = "─┼──".to_string();
                    }
                }
                GateOp::CZ(c, t) => {
                    col_items[*c] = "─●──".to_string();
                    col_items[*t] = "─●──".to_string();
                    let (min_q, max_q) = if c < t { (*c, *t) } else { (*t, *c) };
                    for i in min_q..max_q {
                        inter_items[i] = " │  ".to_string();
                    }
                    for i in (min_q + 1)..max_q {
                        col_items[i] = "─┼──".to_string();
                    }
                }
                GateOp::CH(c, t) => {
                    col_items[*c] = "─●──".to_string();
                    col_items[*t] = "[H]─".to_string();
                    let (min_q, max_q) = if c < t { (*c, *t) } else { (*t, *c) };
                    for i in min_q..max_q {
                        inter_items[i] = " │  ".to_string();
                    }
                    for i in (min_q + 1)..max_q {
                        col_items[i] = "─┼──".to_string();
                    }
                }
                GateOp::SWAP(q1, q2) => {
                    col_items[*q1] = "─✕──".to_string();
                    col_items[*q2] = "─✕──".to_string();
                    let (min_q, max_q) = if q1 < q2 { (*q1, *q2) } else { (*q2, *q1) };
                    for i in min_q..max_q {
                        inter_items[i] = " │  ".to_string();
                    }
                    for i in (min_q + 1)..max_q {
                        col_items[i] = "─┼──".to_string();
                    }
                }
                GateOp::CCX(c1, c2, t) => {
                    col_items[*c1] = "─●──".to_string();
                    col_items[*c2] = "─●──".to_string();
                    col_items[*t] = "─⊕──".to_string();
                    let min_q = (*c1).min(*c2).min(*t);
                    let max_q = (*c1).max(*c2).max(*t);
                    for i in min_q..max_q {
                        inter_items[i] = " │  ".to_string();
                    }
                    for i in (min_q + 1)..max_q {
                        if i != *c1 && i != *c2 && i != *t {
                            col_items[i] = "─┼──".to_string();
                        }
                    }
                }
                GateOp::CSWAP(c, q1, q2) => {
                    col_items[*c] = "─●──".to_string();
                    col_items[*q1] = "─✕──".to_string();
                    col_items[*q2] = "─✕──".to_string();
                    let min_q = (*c).min(*q1).min(*q2);
                    let max_q = (*c).max(*q1).max(*q2);
                    for i in min_q..max_q {
                        inter_items[i] = " │  ".to_string();
                    }
                    for i in (min_q + 1)..max_q {
                        if i != *c && i != *q1 && i != *q2 {
                            col_items[i] = "─┼──".to_string();
                        }
                    }
                }
                GateOp::MCX(ref ctrls, t) => {
                    for &c in ctrls {
                        col_items[c] = "─●──".to_string();
                    }
                    col_items[*t] = "─⊕──".to_string();
                    let min_q = *ctrls.iter().chain(std::iter::once(t)).min().unwrap();
                    let max_q = *ctrls.iter().chain(std::iter::once(t)).max().unwrap();
                    for i in min_q..max_q {
                        inter_items[i] = " │  ".to_string();
                    }
                    for i in (min_q + 1)..max_q {
                        if !ctrls.contains(&i) && i != *t {
                            col_items[i] = "─┼──".to_string();
                        }
                    }
                }
                GateOp::MCZ(ref ctrls, t) => {
                    for &c in ctrls {
                        col_items[c] = "─●──".to_string();
                    }
                    col_items[*t] = "─●──".to_string();
                    let min_q = *ctrls.iter().chain(std::iter::once(t)).min().unwrap();
                    let max_q = *ctrls.iter().chain(std::iter::once(t)).max().unwrap();
                    for i in min_q..max_q {
                        inter_items[i] = " │  ".to_string();
                    }
                    for i in (min_q + 1)..max_q {
                        if !ctrls.contains(&i) && i != *t {
                            col_items[i] = "─┼──".to_string();
                        }
                    }
                }
                GateOp::CP(c, t, _) => {
                    col_items[*c] = "─●──".to_string();
                    col_items[*t] = "[P]─".to_string();
                    let (min_q, max_q) = if c < t { (*c, *t) } else { (*t, *c) };
                    for i in min_q..max_q {
                        inter_items[i] = " │  ".to_string();
                    }
                    for i in (min_q + 1)..max_q {
                        col_items[i] = "─┼──".to_string();
                    }
                }
                GateOp::CY(c, t) => {
                    col_items[*c] = "─●──".to_string();
                    col_items[*t] = "[Y]─".to_string();
                    let (min_q, max_q) = if c < t { (*c, *t) } else { (*t, *c) };
                    for i in min_q..max_q {
                        inter_items[i] = " │  ".to_string();
                    }
                    for i in (min_q + 1)..max_q {
                        col_items[i] = "─┼──".to_string();
                    }
                }
                GateOp::Measure(q) => {
                    col_items[*q] = "[M]─".to_string();
                }
            }

            for q in 0..n_qubits {
                qubit_lines[q].push_str(&col_items[q]);
            }
            for i in 0..n_qubits.saturating_sub(1) {
                inter_lines[i].push_str(&inter_items[i]);
            }
        }

        let mut out = String::new();
        for q in 0..n_qubits {
            out.push_str(&qubit_lines[q]);
            out.push_str("─┤\n");
            if q < n_qubits.saturating_sub(1) {
                out.push_str(&inter_lines[q]);
                out.push('\n');
            }
        }
        out
    }

    /// Export to OpenQASM 2.0 format
    pub fn to_qasm(&self) -> String {
        let mut qasm = String::new();
        qasm.push_str("OPENQASM 2.0;\ninclude \"qelib1.inc\";\n\n");
        qasm.push_str(&format!("qreg q[{}];\n", self.num_qubits));
        qasm.push_str(&format!("creg c[{}];\n\n", self.num_qubits));

        for op in &self.operations {
            match op {
                GateOp::H(q) => qasm.push_str(&format!("h q[{}];\n", q)),
                GateOp::X(q) => qasm.push_str(&format!("x q[{}];\n", q)),
                GateOp::Y(q) => qasm.push_str(&format!("y q[{}];\n", q)),
                GateOp::Z(q) => qasm.push_str(&format!("z q[{}];\n", q)),
                GateOp::S(q) => qasm.push_str(&format!("s q[{}];\n", q)),
                GateOp::Sdg(q) => qasm.push_str(&format!("sdg q[{}];\n", q)),
                GateOp::T(q) => qasm.push_str(&format!("t q[{}];\n", q)),
                GateOp::Tdg(q) => qasm.push_str(&format!("tdg q[{}];\n", q)),
                GateOp::Rx(q, th) => qasm.push_str(&format!("rx({}) q[{}];\n", th, q)),
                GateOp::Ry(q, th) => qasm.push_str(&format!("ry({}) q[{}];\n", th, q)),
                GateOp::Rz(q, th) => qasm.push_str(&format!("rz({}) q[{}];\n", th, q)),
                GateOp::Phase(q, l) => qasm.push_str(&format!("p({}) q[{}];\n", l, q)),
                GateOp::U3(q, t, p, l) => {
                    qasm.push_str(&format!("u3({},{},{}) q[{}];\n", t, p, l, q))
                }
                GateOp::CX(c, t) => qasm.push_str(&format!("cx q[{}],q[{}];\n", c, t)),
                GateOp::CY(c, t) => qasm.push_str(&format!("cy q[{}],q[{}];\n", c, t)),
                GateOp::CZ(c, t) => qasm.push_str(&format!("cz q[{}],q[{}];\n", c, t)),
                GateOp::CH(c, t) => qasm.push_str(&format!("ch q[{}],q[{}];\n", c, t)),
                GateOp::SWAP(q1, q2) => qasm.push_str(&format!("swap q[{}],q[{}];\n", q1, q2)),
                GateOp::CCX(c1, c2, t) => {
                    qasm.push_str(&format!("ccx q[{}],q[{}],q[{}];\n", c1, c2, t))
                }
                GateOp::CSWAP(c, q1, q2) => {
                    qasm.push_str(&format!("cswap q[{}],q[{}],q[{}];\n", c, q1, q2))
                }
                GateOp::MCX(ref c, t) => {
                    qasm.push_str(&format!("// mcx on controls {:?} target {}\n", c, t));
                }
                GateOp::MCZ(ref c, t) => {
                    qasm.push_str(&format!("// mcz on controls {:?} target {}\n", c, t));
                }
                GateOp::CP(c, t, p) => qasm.push_str(&format!("cp({}) q[{}],q[{}];\n", p, c, t)),
                GateOp::Measure(q) => qasm.push_str(&format!("measure q[{}] -> c[{}];\n", q, q)),
                GateOp::Custom1Q(q, _, name) => {
                    qasm.push_str(&format!("// custom gate {} on q[{}]\n", name, q))
                }
            }
        }
        qasm
    }
}
