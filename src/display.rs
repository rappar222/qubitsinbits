use crate::quantum::complex::format_complex;
use crate::quantum::state::QubitState;
use colored::*;
use std::collections::BTreeMap;
use std::f64::consts::PI;

/// Print statevector amplitudes table
pub fn print_state_table(state: &QubitState, min_prob: f64) {
    println!("{}", "=== Quantum State Vector ===".bold().cyan());
    println!(
        "{:<8} {:<12} {:<24} {:<12} {:<12} {:<10}",
        "Basis", "Dirac", "Amplitude (Complex)", "Magnitude", "Phase (rad)", "Prob (%)"
    );
    println!("{}", "-".repeat(80).dimmed());

    let mut printed = 0;
    for (idx, &amp) in state.amplitudes.iter().enumerate() {
        let prob = amp.norm_sqr();
        if prob >= min_prob {
            let bin_str = format!("{:0width$b}", idx, width = state.num_qubits);
            let dirac_str = format!("|{}>", bin_str);
            let complex_str = format_complex(amp, 4);
            let mag = amp.norm();
            let phase = amp.arg();

            let prob_pct = prob * 100.0;
            println!(
                "{:<8} {:<12} {:<24} {:<12.4} {:<12.4} {:<10.2}%",
                idx,
                dirac_str.green().bold(),
                complex_str,
                mag,
                phase,
                prob_pct
            );
            printed += 1;
        }
    }

    if printed == 0 {
        println!("(No states with probability >= {:.4})", min_prob);
    }
    println!("{}", "-".repeat(80).dimmed());
    println!(
        "State Dirac notation: {}",
        state.to_dirac_string(min_prob).yellow().bold()
    );
    println!();
}

/// Print ASCII bar histogram of measurement counts
pub fn print_histogram(counts: &BTreeMap<usize, usize>, num_qubits: usize, total_shots: usize) {
    println!("{}", "=== Measurement Outcome Histogram ===".bold().cyan());
    let bar_max_width = 40;
    let max_count = counts.values().cloned().max().unwrap_or(1);

    for (state_idx, &count) in counts {
        let bin_str = format!("{:0width$b}", state_idx, width = num_qubits);
        let pct = (count as f64 / total_shots as f64) * 100.0;
        let bar_len = ((count as f64 / max_count as f64) * bar_max_width as f64).round() as usize;
        let bar = "█".repeat(bar_len);

        println!(
            "|{}> ({:>3}): {:>6} shots ({:>5.1}%) | {}",
            bin_str.green().bold(),
            state_idx,
            count.to_string().cyan(),
            pct,
            bar.magenta()
        );
    }
    println!();
}

/// Print ASCII bar chart of theoretical probabilities
pub fn print_probability_distribution(state: &QubitState, min_prob: f64) {
    println!("{}", "=== Probability Distribution ===".bold().cyan());
    let bar_max_width = 40;
    let probs = state.probabilities();

    for (idx, &prob) in probs.iter().enumerate() {
        if prob >= min_prob {
            let bin_str = format!("{:0width$b}", idx, width = state.num_qubits);
            let bar_len = (prob * bar_max_width as f64).round() as usize;
            let bar = "■".repeat(bar_len);
            println!(
                "|{}>: {:>6.2}% | {}",
                bin_str.green().bold(),
                prob * 100.0,
                bar.cyan()
            );
        }
    }
    println!();
}

/// Print Bloch sphere coordinates for all qubits
pub fn print_bloch_sphere(state: &QubitState) {
    println!("{}", "=== Single-Qubit Bloch Sphere & Purity ===".bold().cyan());
    println!(
        "{:<8} {:<10} {:<10} {:<10} {:<10} {:<10} {:<12}",
        "Qubit", "r_x", "r_y", "r_z", "Theta (rad)", "Phi (rad)", "Purity"
    );
    println!("{}", "-".repeat(74).dimmed());

    for q in 0..state.num_qubits {
        let (rx, ry, rz, purity) = state.bloch_vector(q);
        let r = (rx * rx + ry * ry + rz * rz).sqrt();
        let theta = if r > 1e-7 { (rz / r).clamp(-1.0, 1.0).acos() } else { 0.0 };
        let phi = ry.atan2(rx);
        let phi_pos = if phi < 0.0 { phi + 2.0 * PI } else { phi };

        let purity_desc = if (purity - 1.0).abs() < 1e-3 {
            format!("{:.4} (Pure)", purity).green()
        } else {
            format!("{:.4} (Entangled)", purity).yellow()
        };

        println!(
            "q[{}]      {:<10.4} {:<10.4} {:<10.4} {:<10.4} {:<10.4} {}",
            q, rx, ry, rz, theta, phi_pos, purity_desc
        );
    }
    println!();
}
