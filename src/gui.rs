use crate::algorithms::*;
use crate::quantum::complex::format_complex;
use crate::quantum::gate::GateOp;
use crate::quantum::state::QubitState;
use crate::quantum::QuantumCircuit;
use eframe::egui;
use std::collections::BTreeMap;
use std::f64::consts::PI;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GuiTab {
    CircuitAndState,
    BlochSpheres,
    Sampling,
    Qasm,
}

pub struct QuantumApp {
    pub num_qubits: usize,
    pub circuit: QuantumCircuit,
    pub state: QubitState,
    pub step_index: usize,

    // UI state
    pub active_tab: GuiTab,
    pub selected_target: usize,
    pub selected_control: usize,
    pub selected_control2: usize,
    pub angle_param: f64,

    // Monte Carlo sampling
    pub shots_count: usize,
    pub sample_results: Option<BTreeMap<usize, usize>>,

    // Grover parameters
    pub grover_target: usize,

    // Status / measurement log
    pub status_message: String,
}

impl QuantumApp {
    pub fn new(num_qubits: usize) -> Self {
        let n = num_qubits.clamp(1, 12);
        let circuit = QuantumCircuit::new(n);
        let state = circuit.run();

        Self {
            num_qubits: n,
            circuit,
            state,
            step_index: 0,
            active_tab: GuiTab::CircuitAndState,
            selected_target: 0,
            selected_control: 0,
            selected_control2: 1,
            angle_param: PI / 2.0,
            shots_count: 1024,
            sample_results: None,
            grover_target: 0,
            status_message: format!("Initialized with {} qubits.", n),
        }
    }

    pub fn recompute_state(&mut self) {
        let mut st = QubitState::new(self.num_qubits);
        let end = self.step_index.min(self.circuit.operations.len());
        for op in &self.circuit.operations[0..end] {
            st.apply_op(op);
        }
        self.state = st;
    }

    pub fn add_op(&mut self, op: GateOp) {
        self.circuit.add_op(op.clone());
        self.step_index = self.circuit.operations.len();
        self.state.apply_op(&op);
        self.sample_results = None;
        self.status_message = format!("Added gate: {}", op.name());
    }

    pub fn reset_circuit(&mut self) {
        self.circuit = QuantumCircuit::new(self.num_qubits);
        self.state = self.circuit.run();
        self.step_index = 0;
        self.sample_results = None;
        self.status_message = "Circuit cleared.".to_string();
    }

    pub fn set_num_qubits(&mut self, n: usize) {
        self.num_qubits = n.clamp(1, 12);
        self.selected_target = self.selected_target.min(self.num_qubits - 1);
        self.selected_control = self.selected_control.min(self.num_qubits - 1);
        self.selected_control2 = self.selected_control2.min(self.num_qubits - 1);
        self.reset_circuit();
    }

    pub fn load_algorithm(&mut self, name: &str) {
        match name {
            "bell" => {
                self.num_qubits = 2;
                self.circuit = create_bell_circuit(0);
                self.step_index = self.circuit.operations.len();
                self.recompute_state();
                self.status_message = "Loaded Bell State |Φ⁺⟩.".to_string();
            }
            "ghz" => {
                let n = self.num_qubits.max(3);
                self.num_qubits = n;
                self.circuit = create_ghz_circuit(n);
                self.step_index = self.circuit.operations.len();
                self.recompute_state();
                self.status_message = format!("Loaded {}-Qubit GHZ State.", n);
            }
            "teleport" => {
                self.num_qubits = 3;
                self.circuit = create_teleportation_circuit(PI / 3.0, 0.0);
                self.step_index = self.circuit.operations.len();
                self.recompute_state();
                self.status_message = "Loaded Quantum Teleportation Protocol.".to_string();
            }
            "superdense" => {
                self.num_qubits = 2;
                self.circuit = create_superdense_circuit(true, true);
                self.step_index = self.circuit.operations.len();
                self.recompute_state();
                self.status_message = "Loaded Superdense Coding Protocol (transmitting '11').".to_string();
            }
            "grover" => {
                let n = self.num_qubits.max(2).min(5);
                self.num_qubits = n;
                let target = self.grover_target % (1 << n);
                self.circuit = create_grover_circuit(n, target, None);
                self.step_index = self.circuit.operations.len();
                self.recompute_state();
                self.status_message = format!("Loaded Grover's Search for target |{:0width$b}⟩.", target, width = n);
            }
            "qft" => {
                let n = self.num_qubits.max(2).min(6);
                self.num_qubits = n;
                self.circuit = create_qft_circuit(n);
                self.step_index = self.circuit.operations.len();
                self.recompute_state();
                self.status_message = format!("Loaded {}-Qubit Quantum Fourier Transform.", n);
            }
            "dj" => {
                let n = self.num_qubits.max(2).min(5);
                self.num_qubits = n;
                self.circuit = create_deutsch_jozsa_circuit(n - 1, OracleType::BalancedXor);
                self.step_index = self.circuit.operations.len();
                self.recompute_state();
                self.status_message = "Loaded Deutsch-Jozsa Algorithm (Balanced XOR Oracle).".to_string();
            }
            _ => {}
        }
        self.sample_results = None;
    }
}

impl eframe::App for QuantumApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- TOP MENU BAR ---
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.heading("⚛ QubitsInBits");
                ui.separator();

                ui.label("Qubits:");
                let n = self.num_qubits;
                if ui.button("➖").clicked() && n > 1 {
                    self.set_num_qubits(n - 1);
                }
                ui.monospace(format!(" {} ", self.num_qubits));
                if ui.button("➕").clicked() && n < 12 {
                    self.set_num_qubits(n + 1);
                }

                ui.separator();
                if ui.button("🔄 Reset / Clear").clicked() {
                    self.reset_circuit();
                }

                ui.separator();
                ui.label("Step:");
                let total_ops = self.circuit.operations.len();
                let can_step_back = self.step_index > 0;
                let can_step_forward = self.step_index < total_ops;

                if ui.add_enabled(can_step_back, egui::Button::new("⏮ Back")).clicked() {
                    self.step_index = self.step_index.saturating_sub(1);
                    self.recompute_state();
                }
                ui.monospace(format!("{}/{}", self.step_index, total_ops));
                if ui.add_enabled(can_step_forward, egui::Button::new("⏭ Next")).clicked() {
                    self.step_index = (self.step_index + 1).min(total_ops);
                    self.recompute_state();
                }
                if ui.button("⏩ Run All").clicked() {
                    self.step_index = total_ops;
                    self.recompute_state();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::widgets::global_dark_light_mode_buttons(ui);
                });
            });
        });

        // --- BOTTOM STATUS BAR ---
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.colored_label(egui::Color32::from_rgb(100, 200, 255), &self.status_message);
            });
        });

        // --- LEFT PANEL: GATE PALETTE & CONTROLS ---
        egui::SidePanel::left("left_palette").min_width(220.0).show(ctx, |ui| {
            ui.heading("Gate Palette");
            ui.separator();

            // Target & Control Selectors
            ui.label("Target Qubit:");
            egui::ComboBox::from_id_source("target_qubit_combo")
                .selected_text(format!("q[{}]", self.selected_target))
                .show_ui(ui, |ui| {
                    for q in 0..self.num_qubits {
                        ui.selectable_value(&mut self.selected_target, q, format!("q[{}]", q));
                    }
                });

            ui.label("Control Qubit (for 2Q):");
            egui::ComboBox::from_id_source("control_qubit_combo")
                .selected_text(format!("q[{}]", self.selected_control))
                .show_ui(ui, |ui| {
                    for q in 0..self.num_qubits {
                        ui.selectable_value(&mut self.selected_control, q, format!("q[{}]", q));
                    }
                });

            ui.label("Control 2 (for CCX):");
            egui::ComboBox::from_id_source("control2_qubit_combo")
                .selected_text(format!("q[{}]", self.selected_control2))
                .show_ui(ui, |ui| {
                    for q in 0..self.num_qubits {
                        ui.selectable_value(&mut self.selected_control2, q, format!("q[{}]", q));
                    }
                });

            ui.add_space(8.0);
            ui.label("Rotation Angle (θ / λ):");
            ui.add(egui::Slider::new(&mut self.angle_param, -PI..=PI).text("rad"));
            ui.horizontal(|ui| {
                if ui.button("π/4").clicked() {
                    self.angle_param = PI / 4.0;
                }
                if ui.button("π/2").clicked() {
                    self.angle_param = PI / 2.0;
                }
                if ui.button("π").clicked() {
                    self.angle_param = PI;
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.label(egui::RichText::new("Single-Qubit Gates").strong());
            let t = self.selected_target;

            ui.horizontal(|ui| {
                if ui.button(" H ").on_hover_text("Hadamard gate").clicked() {
                    self.add_op(GateOp::H(t));
                }
                if ui.button(" X ").on_hover_text("Pauli-X (NOT)").clicked() {
                    self.add_op(GateOp::X(t));
                }
                if ui.button(" Y ").on_hover_text("Pauli-Y").clicked() {
                    self.add_op(GateOp::Y(t));
                }
                if ui.button(" Z ").on_hover_text("Pauli-Z").clicked() {
                    self.add_op(GateOp::Z(t));
                }
            });

            ui.horizontal(|ui| {
                if ui.button(" S ").on_hover_text("Phase gate S").clicked() {
                    self.add_op(GateOp::S(t));
                }
                if ui.button(" S†").on_hover_text("S dagger").clicked() {
                    self.add_op(GateOp::Sdg(t));
                }
                if ui.button(" T ").on_hover_text("T gate (π/8)").clicked() {
                    self.add_op(GateOp::T(t));
                }
                if ui.button(" T†").on_hover_text("T dagger").clicked() {
                    self.add_op(GateOp::Tdg(t));
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Rx").clicked() {
                    self.add_op(GateOp::Rx(t, self.angle_param));
                }
                if ui.button("Ry").clicked() {
                    self.add_op(GateOp::Ry(t, self.angle_param));
                }
                if ui.button("Rz").clicked() {
                    self.add_op(GateOp::Rz(t, self.angle_param));
                }
                if ui.button("Phase").clicked() {
                    self.add_op(GateOp::Phase(t, self.angle_param));
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.label(egui::RichText::new("Multi-Qubit Gates").strong());
            let c = self.selected_control;
            let c2 = self.selected_control2;

            ui.horizontal(|ui| {
                let valid_cx = c != t;
                if ui.add_enabled(valid_cx, egui::Button::new("CX (CNOT)")).clicked() {
                    self.add_op(GateOp::CX(c, t));
                }
                let valid_cz = c != t;
                if ui.add_enabled(valid_cz, egui::Button::new("CZ")).clicked() {
                    self.add_op(GateOp::CZ(c, t));
                }
            });

            ui.horizontal(|ui| {
                let valid_swap = c != t;
                if ui.add_enabled(valid_swap, egui::Button::new("SWAP")).clicked() {
                    self.add_op(GateOp::SWAP(c, t));
                }
                let valid_cp = c != t;
                if ui.add_enabled(valid_cp, egui::Button::new("CP")).clicked() {
                    self.add_op(GateOp::CP(c, t, self.angle_param));
                }
            });

            let valid_ccx = c != c2 && c != t && c2 != t;
            if ui.add_enabled(valid_ccx, egui::Button::new("CCX (Toffoli)")).clicked() {
                self.add_op(GateOp::CCX(c, c2, t));
            }

            ui.add_space(8.0);
            ui.separator();
            ui.label(egui::RichText::new("Measurement").strong());
            if ui.button(format!("Measure q[{}] (Collapse)", t)).clicked() {
                let mut rng = rand::thread_rng();
                let (outcome, prob) = self.state.measure_qubit(t, &mut rng);
                self.circuit.measure(t);
                self.step_index = self.circuit.operations.len();
                self.status_message = format!(
                    "Measured q[{}]: {} (Prob: {:.1}%)",
                    t,
                    if outcome { "1" } else { "0" },
                    prob * 100.0
                );
            }

            ui.add_space(10.0);
            ui.separator();
            ui.label(egui::RichText::new("Preset Algorithms").strong());
            if ui.button("Bell State (|Φ⁺⟩)").clicked() {
                self.load_algorithm("bell");
            }
            if ui.button("GHZ Entanglement").clicked() {
                self.load_algorithm("ghz");
            }
            if ui.button("Quantum Teleportation").clicked() {
                self.load_algorithm("teleport");
            }
            if ui.button("Superdense Coding").clicked() {
                self.load_algorithm("superdense");
            }

            ui.horizontal(|ui| {
                if ui.button("Grover Search").clicked() {
                    self.load_algorithm("grover");
                }
                ui.add(egui::DragValue::new(&mut self.grover_target).prefix("tgt: "));
            });

            if ui.button("Quantum Fourier (QFT)").clicked() {
                self.load_algorithm("qft");
            }
            if ui.button("Deutsch-Jozsa").clicked() {
                self.load_algorithm("dj");
            }
        });

        // --- CENTRAL PANEL: TABS & VISUALIZATION ---
        egui::CentralPanel::default().show(ctx, |ui| {
            // Tab Selector
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, GuiTab::CircuitAndState, "📊 Circuit & State");
                ui.selectable_value(&mut self.active_tab, GuiTab::BlochSpheres, "🌐 Bloch Spheres");
                ui.selectable_value(&mut self.active_tab, GuiTab::Sampling, "🎲 Shots Sampling");
                ui.selectable_value(&mut self.active_tab, GuiTab::Qasm, "📝 OpenQASM");
            });
            ui.separator();

            match self.active_tab {
                GuiTab::CircuitAndState => {
                    self.render_circuit_and_state(ui);
                }
                GuiTab::BlochSpheres => {
                    self.render_bloch_spheres(ui);
                }
                GuiTab::Sampling => {
                    self.render_sampling(ui);
                }
                GuiTab::Qasm => {
                    self.render_qasm(ui);
                }
            }
        });
    }
}

impl QuantumApp {
    fn render_circuit_and_state(&mut self, ui: &mut egui::Ui) {
        ui.heading("Quantum Circuit Track");
        egui::ScrollArea::horizontal().show(ui, |ui| {
            let font_id = egui::TextStyle::Monospace.resolve(ui.style());
            let diagram = self.circuit.to_ascii_diagram();
            ui.label(egui::RichText::new(diagram).font(font_id).color(egui::Color32::from_rgb(180, 240, 180)));
        });

        ui.add_space(8.0);
        ui.separator();

        ui.heading("Quantum State Vector");
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Dirac Notation:").strong());
            ui.label(
                egui::RichText::new(self.state.to_dirac_string(1e-4))
                    .strong()
                    .color(egui::Color32::from_rgb(255, 220, 100)),
            );
        });

        ui.add_space(4.0);

        egui::ScrollArea::vertical().max_height(350.0).show(ui, |ui| {
            egui::Grid::new("state_vector_grid")
                .striped(true)
                .min_col_width(80.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Basis").strong());
                    ui.label(egui::RichText::new("Dirac").strong());
                    ui.label(egui::RichText::new("Amplitude (Re + Im)").strong());
                    ui.label(egui::RichText::new("Magnitude").strong());
                    ui.label(egui::RichText::new("Phase (rad)").strong());
                    ui.label(egui::RichText::new("Probability").strong());
                    ui.label(egui::RichText::new("Probability Bar").strong());
                    ui.end_row();

                    for (idx, &amp) in self.state.amplitudes.iter().enumerate() {
                        let prob = amp.norm_sqr();
                        if prob < 1e-5 && self.num_qubits > 4 {
                            continue; // hide negligible amplitudes on larger states to keep UI responsive
                        }
                        let bin_str = format!("{:0width$b}", idx, width = self.num_qubits);
                        let dirac_str = format!("|{}⟩", bin_str);
                        let complex_str = format_complex(amp, 4);
                        let mag = amp.norm();
                        let phase = amp.arg();

                        ui.monospace(format!("{}", idx));
                        ui.label(egui::RichText::new(dirac_str).strong().color(egui::Color32::LIGHT_GREEN));
                        ui.monospace(complex_str);
                        ui.monospace(format!("{:.4}", mag));
                        ui.monospace(format!("{:.4}", phase));
                        ui.monospace(format!("{:>5.1}%", prob * 100.0));

                        // Visual progress bar
                        ui.add(egui::ProgressBar::new(prob as f32).show_percentage());
                        ui.end_row();
                    }
                });
        });
    }

    fn render_bloch_spheres(&mut self, ui: &mut egui::Ui) {
        ui.heading("Single-Qubit Bloch Sphere Representation");
        ui.label("For entangled qubits, the Bloch vector shrinks inside the unit sphere (purity < 1.0).");
        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for q in 0..self.num_qubits {
                    let (rx, ry, rz, purity) = self.state.bloch_vector(q);
                    let r = (rx * rx + ry * ry + rz * rz).sqrt();
                    let theta = if r > 1e-7 { (rz / r).clamp(-1.0, 1.0).acos() } else { 0.0 };
                    let phi = ry.atan2(rx);
                    let phi_pos = if phi < 0.0 { phi + 2.0 * PI } else { phi };

                    ui.group(|ui| {
                        ui.set_width(220.0);
                        ui.heading(format!("Qubit q[{}]", q));

                        // 2D Bloch Projection Painter
                        let (rect, _response) = ui.allocate_exact_size(egui::vec2(160.0, 160.0), egui::Sense::hover());
                        let painter = ui.painter_at(rect);
                        let center = rect.center();
                        let radius = 65.0_f32;

                        // Draw sphere circle & axes
                        painter.circle_stroke(center, radius, egui::Stroke::new(1.5_f32, egui::Color32::GRAY));
                        painter.line_segment([center - egui::vec2(radius, 0.0), center + egui::vec2(radius, 0.0)], egui::Stroke::new(0.5_f32, egui::Color32::DARK_GRAY));
                        painter.line_segment([center - egui::vec2(0.0, radius), center + egui::vec2(0.0, radius)], egui::Stroke::new(0.5_f32, egui::Color32::DARK_GRAY));

                        // Labels |0> (top) and |1> (bottom)
                        painter.text(center - egui::vec2(0.0, radius + 10.0), egui::Align2::CENTER_CENTER, "|0⟩", egui::FontId::monospace(12.0), egui::Color32::WHITE);
                        painter.text(center + egui::vec2(0.0, radius + 10.0), egui::Align2::CENTER_CENTER, "|1⟩", egui::FontId::monospace(12.0), egui::Color32::WHITE);
                        painter.text(center + egui::vec2(radius + 12.0, 0.0), egui::Align2::CENTER_CENTER, "|+⟩", egui::FontId::monospace(10.0), egui::Color32::GRAY);

                        // State vector arrow (X vs Z projection)
                        let arrow_tip = center + egui::vec2((rx as f32) * radius, (-rz as f32) * radius);
                        painter.line_segment([center, arrow_tip], egui::Stroke::new(2.5_f32, egui::Color32::from_rgb(255, 100, 100)));
                        painter.circle_filled(arrow_tip, 4.0, egui::Color32::from_rgb(255, 50, 50));

                        ui.add_space(4.0);
                        ui.label(format!("r = ({:.2}, {:.2}, {:.2})", rx, ry, rz));
                        ui.label(format!("θ = {:.2} rad ({:.0}°)", theta, theta * 180.0 / PI));
                        ui.label(format!("ϕ = {:.2} rad ({:.0}°)", phi_pos, phi_pos * 180.0 / PI));

                        let is_pure = (purity - 1.0).abs() < 1e-3;
                        if is_pure {
                            ui.colored_label(egui::Color32::LIGHT_GREEN, format!("Purity: {:.3} (Pure)", purity));
                        } else {
                            ui.colored_label(egui::Color32::YELLOW, format!("Purity: {:.3} (Entangled)", purity));
                        }
                    });
                }
            });
        });
    }

    fn render_sampling(&mut self, ui: &mut egui::Ui) {
        ui.heading("Monte Carlo Measurement Simulator");
        ui.horizontal(|ui| {
            ui.label("Shots:");
            ui.add(egui::Slider::new(&mut self.shots_count, 100..=10000).logarithmic(true));
            if ui.button("🎲 Run Shots").clicked() {
                let mut rng = rand::thread_rng();
                self.sample_results = Some(self.state.sample_shots(self.shots_count, &mut rng));
                self.status_message = format!("Sampled {} measurement shots.", self.shots_count);
            }
        });
        ui.separator();

        if let Some(counts) = &self.sample_results {
            ui.label(format!("Total Shots: {}", self.shots_count));
            let max_count = counts.values().cloned().max().unwrap_or(1);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (&idx, &count) in counts {
                    let bin_str = format!("{:0width$b}", idx, width = self.num_qubits);
                    let pct = (count as f64 / self.shots_count as f64) * 100.0;

                    ui.horizontal(|ui| {
                        ui.monospace(format!("|{}⟩ ({:>3}):", bin_str, idx));
                        ui.monospace(format!("{:>5} ({:>5.1}%)", count, pct));
                        let frac = count as f32 / max_count as f32;
                        ui.add(egui::ProgressBar::new(frac).text(format!("{:.1}%", pct)));
                    });
                }
            });
        } else {
            ui.label("Click 'Run Shots' to sample outcomes from the current quantum state.");
        }
    }

    fn render_qasm(&mut self, ui: &mut egui::Ui) {
        ui.heading("OpenQASM 2.0 Representation");
        let qasm_text = self.circuit.to_qasm();

        ui.horizontal(|ui| {
            if ui.button("📋 Copy to Clipboard").clicked() {
                ui.output_mut(|o| o.copied_text = qasm_text.clone());
                self.status_message = "OpenQASM copied to clipboard.".to_string();
            }
        });
        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.text_edit_multiline(&mut qasm_text.as_str());
        });
    }
}

pub fn run_gui(initial_qubits: usize) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 720.0])
            .with_min_inner_size([720.0, 500.0])
            .with_title("⚛ QubitsInBits - Quantum Computer Simulator"),
        ..Default::default()
    };

    eframe::run_native(
        "QubitsInBits",
        options,
        Box::new(move |_cc| Ok(Box::new(QuantumApp::new(initial_qubits)))),
    )
}
