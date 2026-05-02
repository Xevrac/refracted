// LSX Inspector - UI for viewing captured LSX requests/responses

use crate::core::inspector::inspector_module::*;
use egui::Color32;

/// State for LSX inspector UI
pub struct LsxInspectorState {
    pub selected_index: Option<usize>,
    pub show_plaintext: bool,
}

impl LsxInspectorState {
    pub fn new() -> Self {
        Self {
            selected_index: None,
            show_plaintext: false,
        }
    }
}

/// Convert bytes to plaintext, falling back to hex if invalid UTF-8
fn bytes_to_plaintext(data: &[u8]) -> String {
    // Try to decode as UTF-8
    if let Ok(text) = std::str::from_utf8(data) {
        // Check if it's mostly printable ASCII or valid UTF-8
        if text.chars().all(|c| c.is_ascii() || !c.is_control() || c == '\n' || c == '\r' || c == '\t') {
            return text.to_string();
        }
    }
    // Fall back to hex dump
    format_hex_dump(data, 4096)
}

/// Render LSX inspector UI
pub fn render_lsx_inspector(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    state: &mut LsxInspectorState,
    buffer: LsxBuffer,
) {
    let lsx_list = {
        let buf = buffer.lock();
        buf.iter()
            .enumerate()
            .rev()
            .map(|(i, h)| (i, h.clone()))
            .collect::<Vec<_>>()
    };
    let count = lsx_list.len();

    // Top toolbar
    ui.horizontal(|ui| {
        ui.label(format!("LSX Requests/Responses: {}", count));
        ui.checkbox(&mut state.show_plaintext, "Plaintext");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Copy to clipboard button
            if ui.button("📋").clicked() {
                let buffer = buffer.lock();
                let mut output = String::new();
                output.push_str("=== LSX Inspection Data ===\n\n");

                for (idx, lsx) in buffer.iter().enumerate().rev() {
                    output.push_str(&format!("LSX #{}:\n", idx));
                    output.push_str(&format!("  Direction: {}\n", lsx.direction.to_string()));
                    output.push_str(&format!("  Method: {}\n", lsx.method));
                    output.push_str(&format!("  Path: {}\n", lsx.path));
                    output.push_str(&format!("  Host: {}\n", lsx.host));
                    output.push_str(&format!("  Body Size: {} bytes\n", lsx.body_size));
                    if let Some(status) = lsx.status_code {
                        output.push_str(&format!("  Status Code: {}\n", status));
                    }
                    output.push_str("\n  Headers:\n");
                    for (key, value) in &lsx.headers {
                        output.push_str(&format!("    {}: {}\n", key, value));
                    }
                    output.push_str("\n  Body:\n");
                    let body_text = bytes_to_plaintext(&lsx.body);
                    output.push_str(&format!("    {}\n", body_text));
                    output.push_str("\n");
                }

                ctx.copy_text(output);
            }

            // Save As button
            if ui.button("Save As...").clicked() {
                let buffer = buffer.lock();
                let mut output = String::new();
                output.push_str("=== LSX Inspection Data ===\n\n");

                for (idx, lsx) in buffer.iter().enumerate().rev() {
                    output.push_str(&format!("LSX #{}:\n", idx));
                    output.push_str(&format!("  Direction: {}\n", lsx.direction.to_string()));
                    output.push_str(&format!("  Method: {}\n", lsx.method));
                    output.push_str(&format!("  Path: {}\n", lsx.path));
                    output.push_str(&format!("  Host: {}\n", lsx.host));
                    output.push_str(&format!("  Body Size: {} bytes\n", lsx.body_size));
                    if let Some(status) = lsx.status_code {
                        output.push_str(&format!("  Status Code: {}\n", status));
                    }
                    output.push_str("\n  Headers:\n");
                    for (key, value) in &lsx.headers {
                        output.push_str(&format!("    {}: {}\n", key, value));
                    }
                    output.push_str("\n  Body:\n");
                    let body_text = bytes_to_plaintext(&lsx.body);
                    output.push_str(&format!("    {}\n", body_text));
                    output.push_str("\n");
                }
                drop(buffer);

                let file = rfd::FileDialog::new()
                    .add_filter("Text files", &["txt"])
                    .add_filter("All files", &["*"])
                    .set_file_name("lsx.txt")
                    .save_file();

                if let Some(path) = file {
                    if let Err(e) = std::fs::write(&path, output) {
                        eprintln!("Failed to save LSX data: {}", e);
                    }
                }
            }

            if ui.button("Clear").clicked() {
                let mut buf = buffer.lock();
                buf.clear();
                state.selected_index = None;
            }
        });
    });

    ui.separator();

    // Two-panel layout: List | Details
    ui.columns(2, |columns| {
        // Left panel: LSX list
        columns[0].vertical(|ui| {
            ui.heading("LSX List");
            ui.separator();

            egui::ScrollArea::vertical()
                .id_source("lsx_list_scroll")
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for (idx, lsx) in &lsx_list {
                        let is_selected = state.selected_index == Some(*idx);
                        let direction_color = match lsx.direction {
                            LsxDirection::ClientToServer => Color32::from_rgb(100, 150, 255),
                            LsxDirection::ServerToClient => Color32::from_rgb(255, 150, 100),
                        };

                        let status_str = if let Some(status) = lsx.status_code {
                            format!(" | Status: {}", status)
                        } else {
                            String::new()
                        };

                        let response = ui.selectable_label(
                            is_selected,
                            format!(
                                "[{}] {} {} | {} bytes{}",
                                lsx.direction.to_string(),
                                lsx.method,
                                lsx.path,
                                lsx.body_size,
                                status_str
                            ),
                        );

                        if response.clicked() {
                            state.selected_index = Some(*idx);
                        }

                        if is_selected {
                            ui.painter().rect_filled(
                                response.rect,
                                0.0,
                                direction_color.linear_multiply(0.2),
                            );
                        }
                    }
                });
        });

        // Right panel: Details
        columns[1].vertical(|ui| {
            ui.heading("Details");
            ui.separator();

            if let Some(idx) = state.selected_index {
                if let Some(lsx) = buffer.lock().get(idx) {
                    render_lsx_details(ui, lsx, state.show_plaintext);
                }
            } else {
                ui.label("Select an LSX request/response to view details");
            }
        });
    });
}

fn render_lsx_details(ui: &mut egui::Ui, lsx: &CapturedLsx, show_plaintext: bool) {
    egui::ScrollArea::vertical()
        .id_source("lsx_details_scroll")
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Direction:").heading());
            let direction_color = match lsx.direction {
                LsxDirection::ClientToServer => Color32::from_rgb(100, 150, 255),
                LsxDirection::ServerToClient => Color32::from_rgb(255, 150, 100),
            };
            ui.label(egui::RichText::new(lsx.direction.to_string()).color(direction_color));
            ui.add_space(5.0);

            ui.label(egui::RichText::new("Method:").heading());
            ui.label(&lsx.method);
            ui.add_space(5.0);

            ui.label(egui::RichText::new("Path:").heading());
            ui.label(&lsx.path);
            ui.add_space(5.0);

            ui.label(egui::RichText::new("Host:").heading());
            ui.label(&lsx.host);
            ui.add_space(5.0);

            if let Some(status) = lsx.status_code {
                ui.label(egui::RichText::new("Status Code:").heading());
                ui.label(format!("{}", status));
                ui.add_space(5.0);
            }

            ui.label(egui::RichText::new("Body Size:").heading());
            ui.label(format!("{} bytes", lsx.body_size));
            ui.add_space(10.0);

            ui.separator();
            ui.add_space(5.0);

            ui.label(egui::RichText::new("Headers:").heading());
            ui.add_space(5.0);
            for (key, value) in &lsx.headers {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("{}:", key))
                        .family(egui::FontFamily::Monospace)
                        .size(10.0));
                    ui.label(egui::RichText::new(value)
                        .family(egui::FontFamily::Monospace)
                        .size(10.0));
                });
            }
            ui.add_space(10.0);

            ui.separator();
            ui.add_space(5.0);

            ui.label(egui::RichText::new(if show_plaintext { "Body:" } else { "Hex:" }).heading());
            ui.add_space(5.0);
            egui::ScrollArea::vertical()
                .id_source("body_scroll")
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let display_text = if show_plaintext {
                        bytes_to_plaintext(&lsx.body)
                    } else {
                        format_hex_dump(&lsx.body, 4096)
                    };
                    ui.label(egui::RichText::new(display_text)
                        .family(egui::FontFamily::Monospace)
                        .size(10.0));
                });
        });
}





