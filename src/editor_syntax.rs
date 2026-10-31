use eframe::egui::{self, text::LayoutJob, Color32, FontId, TextFormat};
use std::sync::Arc;

pub fn highlight_qasm(ui: &egui::Ui, text: &str, wrap_width: f32) -> Arc<egui::Galley> {
    let font_id = FontId::monospace(13.0);
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;

    let comment_color = Color32::from_rgb(106, 153, 85);    // #6A9955 VS Code green
    let keyword_color = Color32::from_rgb(197, 134, 192);    // #C586C0 purple
    let gate_color = Color32::from_rgb(86, 156, 214);       // #569CD6 blue
    let number_color = Color32::from_rgb(181, 206, 168);     // #B5CEA8 sage green
    let string_color = Color32::from_rgb(206, 145, 120);     // #CE9178 orange
    let punct_color = Color32::from_rgb(140, 140, 140);      // gray
    let default_color = Color32::from_rgb(220, 220, 220);    // white

    for line in text.split_inclusive('\n') {
        if let Some(comment_start) = line.find("//") {
            let code_part = &line[..comment_start];
            let comment_part = &line[comment_start..];
            highlight_code_slice(&mut job, code_part, &font_id, keyword_color, gate_color, number_color, string_color, punct_color, default_color);
            job.append(comment_part, 0.0, TextFormat {
                font_id: font_id.clone(),
                color: comment_color,
                ..Default::default()
            });
        } else {
            highlight_code_slice(&mut job, line, &font_id, keyword_color, gate_color, number_color, string_color, punct_color, default_color);
        }
    }

    ui.fonts(|f| f.layout_job(job))
}

fn highlight_code_slice(
    job: &mut LayoutJob,
    code: &str,
    font_id: &FontId,
    keyword_color: Color32,
    gate_color: Color32,
    number_color: Color32,
    string_color: Color32,
    punct_color: Color32,
    default_color: Color32,
) {
    let mut chars = code.char_indices().peekable();
    let mut last_idx = 0;

    while let Some(&(i, ch)) = chars.peek() {
        if ch == '"' {
            if i > last_idx {
                job.append(&code[last_idx..i], 0.0, TextFormat { font_id: font_id.clone(), color: default_color, ..Default::default() });
            }
            chars.next(); // consume quote
            let mut end_idx = code.len();
            while let Some(&(j, c)) = chars.peek() {
                chars.next();
                if c == '"' {
                    end_idx = j + 1;
                    break;
                }
            }
            job.append(&code[i..end_idx], 0.0, TextFormat { font_id: font_id.clone(), color: string_color, ..Default::default() });
            last_idx = end_idx;
        } else if ch.is_alphabetic() || ch == '_' {
            if i > last_idx {
                job.append(&code[last_idx..i], 0.0, TextFormat { font_id: font_id.clone(), color: default_color, ..Default::default() });
            }
            let start = i;
            let mut end = i;
            while let Some(&(j, c)) = chars.peek() {
                if c.is_alphanumeric() || c == '_' {
                    end = j + c.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
            let word = &code[start..end];
            let lower = word.to_lowercase();
            let col = match lower.as_str() {
                "openqasm" | "include" | "qreg" | "creg" | "barrier" | "reset" | "measure" => keyword_color,
                "h" | "x" | "y" | "z" | "s" | "sdg" | "t" | "tdg" | "cx" | "cnot" | "cz" | "cy"
                | "ch" | "swap" | "ccx" | "cswap" | "rx" | "ry" | "rz" | "p" | "cp" | "u1" | "u2" | "u3" | "id" => gate_color,
                "pi" => number_color,
                _ => default_color,
            };
            job.append(word, 0.0, TextFormat { font_id: font_id.clone(), color: col, ..Default::default() });
            last_idx = end;
        } else if ch.is_ascii_digit() {
            if i > last_idx {
                job.append(&code[last_idx..i], 0.0, TextFormat { font_id: font_id.clone(), color: default_color, ..Default::default() });
            }
            let start = i;
            let mut end = i;
            while let Some(&(j, c)) = chars.peek() {
                if c.is_ascii_digit() || c == '.' {
                    end = j + c.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
            job.append(&code[start..end], 0.0, TextFormat { font_id: font_id.clone(), color: number_color, ..Default::default() });
            last_idx = end;
        } else if matches!(ch, ';' | ',' | '[' | ']' | '(' | ')' | '-' | '>' | '+' | '*' | '/') {
            if i > last_idx {
                job.append(&code[last_idx..i], 0.0, TextFormat { font_id: font_id.clone(), color: default_color, ..Default::default() });
            }
            chars.next();
            let end = i + ch.len_utf8();
            job.append(&code[i..end], 0.0, TextFormat { font_id: font_id.clone(), color: punct_color, ..Default::default() });
            last_idx = end;
        } else {
            chars.next();
        }
    }

    if last_idx < code.len() {
        job.append(&code[last_idx..], 0.0, TextFormat { font_id: font_id.clone(), color: default_color, ..Default::default() });
    }
}
