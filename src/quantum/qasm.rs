use super::circuit::QuantumCircuit;
use super::gate::GateOp;
use std::collections::HashMap;
use std::f64::consts::PI;

#[derive(Debug, Clone)]
pub struct QasmInstruction {
    pub line_number: usize,
    pub raw_text: String,
    pub op: Option<GateOp>,
}

pub struct QasmParser;

impl QasmParser {
    pub fn parse(qasm_code: &str) -> Result<(QuantumCircuit, Vec<QasmInstruction>), String> {
        let mut qregs: HashMap<String, usize> = HashMap::new(); // reg_name -> size
        let mut qreg_offsets: HashMap<String, usize> = HashMap::new(); // reg_name -> start_index
        let mut total_qubits = 0;

        let mut lines_to_process = Vec::new();

        // First pass: find register declarations and strip comments
        for (line_idx, raw_line) in qasm_code.lines().enumerate() {
            let line_num = line_idx + 1;
            let line = raw_line.split("//").next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with("OPENQASM") || line.starts_with("include") {
                continue;
            }

            if line.starts_with("qreg ") {
                let parts: Vec<&str> = line.trim_start_matches("qreg ").trim_end_matches(';').trim().split('[').collect();
                if parts.len() != 2 {
                    return Err(format!("Line {}: Invalid qreg syntax '{}'", line_num, line));
                }
                let name = parts[0].trim().to_string();
                let size_str = parts[1].trim_end_matches(']').trim();
                let size: usize = size_str.parse().map_err(|_| format!("Line {}: Invalid register size '{}'", line_num, size_str))?;

                if size == 0 {
                    return Err(format!("Line {}: qreg size must be > 0", line_num));
                }

                qreg_offsets.insert(name.clone(), total_qubits);
                qregs.insert(name, size);
                total_qubits += size;
                continue;
            }

            if line.starts_with("creg ") {
                // Classical registers parsed but handled conceptually
                continue;
            }

            lines_to_process.push((line_num, raw_line.to_string(), line.to_string()));
        }

        if total_qubits == 0 {
            total_qubits = 1; // Default to 1 qubit if none declared
            qregs.insert("q".to_string(), 1);
            qreg_offsets.insert("q".to_string(), 0);
        }

        if total_qubits > 26 {
            return Err(format!("Simulating {} qubits exceeds maximum capacity (26)", total_qubits));
        }

        let mut circuit = QuantumCircuit::new(total_qubits);
        let mut instructions = Vec::new();

        // Helper to resolve qubit string like "q[0]" or "q"
        let resolve_qubit = |s: &str, line_num: usize| -> Result<usize, String> {
            let s = s.trim();
            if let Some(open) = s.find('[') {
                if let Some(close) = s.find(']') {
                    let name = &s[..open];
                    let idx_str = &s[open + 1..close];
                    let idx: usize = idx_str.parse().map_err(|_| format!("Line {}: Invalid qubit index '{}'", line_num, idx_str))?;

                    let offset = qreg_offsets.get(name).ok_or_else(|| format!("Line {}: Unknown quantum register '{}'", line_num, name))?;
                    let size = qregs.get(name).unwrap();

                    if idx >= *size {
                        return Err(format!("Line {}: Qubit index {} out of range for register '{}' of size {}", line_num, idx, name, size));
                    }
                    return Ok(offset + idx);
                }
            }
            // If just a number was passed
            if let Ok(idx) = s.parse::<usize>() {
                if idx < total_qubits {
                    return Ok(idx);
                }
            }
            Err(format!("Line {}: Cannot resolve qubit identifier '{}'", line_num, s))
        };

        // Second pass: parse gate instructions
        for (line_num, raw_text, line) in lines_to_process {
            let line = line.trim_end_matches(';').trim();
            if line.is_empty() || line.starts_with("barrier") {
                continue;
            }

            // Measurement: measure q[0] -> c[0]
            if line.starts_with("measure ") {
                let rest = line.trim_start_matches("measure ").trim();
                let parts: Vec<&str> = rest.split("->").collect();
                if parts.is_empty() {
                    return Err(format!("Line {}: Invalid measure syntax", line_num));
                }
                let q = resolve_qubit(parts[0], line_num)?;
                circuit.measure(q);
                instructions.push(QasmInstruction {
                    line_number: line_num,
                    raw_text,
                    op: Some(GateOp::Measure(q)),
                });
                continue;
            }

            // Gate parse: split name and args
            let mut name_part: &str;
            let args_part: &str;
            let mut param_str = "";

            if let Some(space_idx) = line.find(|c: char| c.is_whitespace()) {
                name_part = line[..space_idx].trim();
                args_part = line[space_idx..].trim();
            } else {
                name_part = line;
                args_part = "";
            }

            if let Some(param_start) = name_part.find('(') {
                if let Some(param_end) = name_part.find(')') {
                    param_str = &name_part[param_start + 1..param_end];
                    name_part = &name_part[..param_start];
                }
            }

            let args: Vec<&str> = args_part.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            let gate_name = name_part.to_lowercase();

            let op: Option<GateOp> = match gate_name.as_str() {
                "h" => {
                    if args.len() != 1 { return Err(format!("Line {}: H requires 1 qubit argument", line_num)); }
                    let q = resolve_qubit(args[0], line_num)?;
                    circuit.h(q);
                    Some(GateOp::H(q))
                }
                "x" => {
                    if args.len() != 1 { return Err(format!("Line {}: X requires 1 qubit argument", line_num)); }
                    let q = resolve_qubit(args[0], line_num)?;
                    circuit.x(q);
                    Some(GateOp::X(q))
                }
                "y" => {
                    if args.len() != 1 { return Err(format!("Line {}: Y requires 1 qubit argument", line_num)); }
                    let q = resolve_qubit(args[0], line_num)?;
                    circuit.y(q);
                    Some(GateOp::Y(q))
                }
                "z" => {
                    if args.len() != 1 { return Err(format!("Line {}: Z requires 1 qubit argument", line_num)); }
                    let q = resolve_qubit(args[0], line_num)?;
                    circuit.z(q);
                    Some(GateOp::Z(q))
                }
                "s" => {
                    if args.len() != 1 { return Err(format!("Line {}: S requires 1 qubit argument", line_num)); }
                    let q = resolve_qubit(args[0], line_num)?;
                    circuit.s(q);
                    Some(GateOp::S(q))
                }
                "sdg" => {
                    if args.len() != 1 { return Err(format!("Line {}: SDG requires 1 qubit argument", line_num)); }
                    let q = resolve_qubit(args[0], line_num)?;
                    circuit.sdg(q);
                    Some(GateOp::Sdg(q))
                }
                "t" => {
                    if args.len() != 1 { return Err(format!("Line {}: T requires 1 qubit argument", line_num)); }
                    let q = resolve_qubit(args[0], line_num)?;
                    circuit.t(q);
                    Some(GateOp::T(q))
                }
                "tdg" => {
                    if args.len() != 1 { return Err(format!("Line {}: TDG requires 1 qubit argument", line_num)); }
                    let q = resolve_qubit(args[0], line_num)?;
                    circuit.tdg(q);
                    Some(GateOp::Tdg(q))
                }
                "cx" | "cnot" => {
                    if args.len() != 2 { return Err(format!("Line {}: CX requires 2 qubit arguments (control, target)", line_num)); }
                    let c = resolve_qubit(args[0], line_num)?;
                    let t = resolve_qubit(args[1], line_num)?;
                    circuit.cx(c, t);
                    Some(GateOp::CX(c, t))
                }
                "cz" => {
                    if args.len() != 2 { return Err(format!("Line {}: CZ requires 2 qubit arguments", line_num)); }
                    let c = resolve_qubit(args[0], line_num)?;
                    let t = resolve_qubit(args[1], line_num)?;
                    circuit.cz(c, t);
                    Some(GateOp::CZ(c, t))
                }
                "cy" => {
                    if args.len() != 2 { return Err(format!("Line {}: CY requires 2 qubit arguments", line_num)); }
                    let c = resolve_qubit(args[0], line_num)?;
                    let t = resolve_qubit(args[1], line_num)?;
                    circuit.add_op(GateOp::CY(c, t));
                    Some(GateOp::CY(c, t))
                }
                "ch" => {
                    if args.len() != 2 { return Err(format!("Line {}: CH requires 2 qubit arguments", line_num)); }
                    let c = resolve_qubit(args[0], line_num)?;
                    let t = resolve_qubit(args[1], line_num)?;
                    circuit.add_op(GateOp::CH(c, t));
                    Some(GateOp::CH(c, t))
                }
                "swap" => {
                    if args.len() != 2 { return Err(format!("Line {}: SWAP requires 2 qubit arguments", line_num)); }
                    let q1 = resolve_qubit(args[0], line_num)?;
                    let q2 = resolve_qubit(args[1], line_num)?;
                    circuit.swap(q1, q2);
                    Some(GateOp::SWAP(q1, q2))
                }
                "ccx" => {
                    if args.len() != 3 { return Err(format!("Line {}: CCX requires 3 qubit arguments", line_num)); }
                    let c1 = resolve_qubit(args[0], line_num)?;
                    let c2 = resolve_qubit(args[1], line_num)?;
                    let t = resolve_qubit(args[2], line_num)?;
                    circuit.ccx(c1, c2, t);
                    Some(GateOp::CCX(c1, c2, t))
                }
                "cswap" => {
                    if args.len() != 3 { return Err(format!("Line {}: CSWAP requires 3 qubit arguments", line_num)); }
                    let c = resolve_qubit(args[0], line_num)?;
                    let t1 = resolve_qubit(args[1], line_num)?;
                    let t2 = resolve_qubit(args[2], line_num)?;
                    circuit.cswap(c, t1, t2);
                    Some(GateOp::CSWAP(c, t1, t2))
                }
                "rx" => {
                    if args.len() != 1 { return Err(format!("Line {}: Rx requires 1 qubit argument", line_num)); }
                    let theta = parse_math_expr(param_str).map_err(|e| format!("Line {}: {}", line_num, e))?;
                    let q = resolve_qubit(args[0], line_num)?;
                    circuit.rx(q, theta);
                    Some(GateOp::Rx(q, theta))
                }
                "ry" => {
                    if args.len() != 1 { return Err(format!("Line {}: Ry requires 1 qubit argument", line_num)); }
                    let theta = parse_math_expr(param_str).map_err(|e| format!("Line {}: {}", line_num, e))?;
                    let q = resolve_qubit(args[0], line_num)?;
                    circuit.ry(q, theta);
                    Some(GateOp::Ry(q, theta))
                }
                "rz" => {
                    if args.len() != 1 { return Err(format!("Line {}: Rz requires 1 qubit argument", line_num)); }
                    let theta = parse_math_expr(param_str).map_err(|e| format!("Line {}: {}", line_num, e))?;
                    let q = resolve_qubit(args[0], line_num)?;
                    circuit.rz(q, theta);
                    Some(GateOp::Rz(q, theta))
                }
                "p" | "u1" => {
                    if args.len() != 1 { return Err(format!("Line {}: Phase/U1 requires 1 qubit argument", line_num)); }
                    let lambda = parse_math_expr(param_str).map_err(|e| format!("Line {}: {}", line_num, e))?;
                    let q = resolve_qubit(args[0], line_num)?;
                    circuit.phase(q, lambda);
                    Some(GateOp::Phase(q, lambda))
                }
                "cp" => {
                    if args.len() != 2 { return Err(format!("Line {}: CP requires 2 qubit arguments (control, target)", line_num)); }
                    let lambda = parse_math_expr(param_str).map_err(|e| format!("Line {}: {}", line_num, e))?;
                    let c = resolve_qubit(args[0], line_num)?;
                    let t = resolve_qubit(args[1], line_num)?;
                    circuit.cp(c, t, lambda);
                    Some(GateOp::CP(c, t, lambda))
                }
                "u3" => {
                    if args.len() != 1 { return Err(format!("Line {}: U3 requires 1 qubit argument", line_num)); }
                    let params: Vec<&str> = param_str.split(',').collect();
                    if params.len() != 3 { return Err(format!("Line {}: U3 requires 3 parameters: (theta, phi, lambda)", line_num)); }
                    let theta = parse_math_expr(params[0])?;
                    let phi = parse_math_expr(params[1])?;
                    let lambda = parse_math_expr(params[2])?;
                    let q = resolve_qubit(args[0], line_num)?;
                    circuit.add_op(GateOp::U3(q, theta, phi, lambda));
                    Some(GateOp::U3(q, theta, phi, lambda))
                }
                "id" | "i" => None,
                _ => return Err(format!("Line {}: Unrecognized gate or command '{}'", line_num, gate_name)),
            };

            instructions.push(QasmInstruction {
                line_number: line_num,
                raw_text,
                op,
            });
        }

        Ok((circuit, instructions))
    }
}

/// Simple parser for parameter math expressions like "pi", "pi/2", "-pi/4", "3.1415", "0.5*pi"
fn parse_math_expr(expr: &str) -> Result<f64, String> {
    let s = expr.trim().replace(' ', "");
    if s.is_empty() {
        return Err("Empty angle parameter expression".to_string());
    }

    if s.eq_ignore_ascii_case("pi") {
        return Ok(PI);
    }
    if s.eq_ignore_ascii_case("-pi") {
        return Ok(-PI);
    }

    // Handle fractions like "pi/2", "3*pi/4"
    if let Some(slash_idx) = s.find('/') {
        let num_str = &s[..slash_idx];
        let denom_str = &s[slash_idx + 1..];

        let num = parse_math_expr(num_str)?;
        let denom: f64 = denom_str.parse().map_err(|_| format!("Invalid denominator '{}'", denom_str))?;
        if denom.abs() < 1e-12 {
            return Err("Division by zero in angle expression".to_string());
        }
        return Ok(num / denom);
    }

    // Handle multiplication like "2*pi", "0.5*pi"
    if let Some(star_idx) = s.find('*') {
        let left_str = &s[..star_idx];
        let right_str = &s[star_idx + 1..];

        let left = parse_math_expr(left_str)?;
        let right = parse_math_expr(right_str)?;
        return Ok(left * right);
    }

    // Otherwise standard float
    s.parse::<f64>().map_err(|_| format!("Cannot parse numeric angle '{}'", s))
}
