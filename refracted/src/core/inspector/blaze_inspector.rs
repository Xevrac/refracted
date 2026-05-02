// Blaze Inspector - UI for viewing captured Blaze packets with TDF parsing

use crate::blaze::tdf::TdfTreeNode;
use crate::core::inspector::inspector_module::*;
use egui;
use std::collections::HashSet;

/// State for the Blaze inspector UI
pub struct BlazeInspectorState {
    pub selected_packet_index: Option<usize>,
    pub tdf_tree: Option<Vec<TdfTreeNode>>,
    pub selected_tdf_path: Vec<usize>,
    pub expanded_tdf_nodes: HashSet<Vec<usize>>,
    pub tdf_parse_error: Option<String>,
    /// Set when user clicks **Inspect**; parent toolkit consumes and jumps to Make.
    pub open_make_from_index: Option<usize>,
}

impl BlazeInspectorState {
    pub fn new() -> Self {
        Self {
            selected_packet_index: None,
            tdf_tree: None,
            selected_tdf_path: Vec::new(),
            expanded_tdf_nodes: HashSet::new(),
            tdf_parse_error: None,
            open_make_from_index: None,
        }
    }
}

/// Convert bytes to plaintext, falling back to hex if invalid UTF-8
#[allow(dead_code)]
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

/// Render the Blaze inspector UI
pub fn render_blaze_inspector(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    state: &mut BlazeInspectorState,
    packet_buffer: PacketBuffer,
) {
    use crate::blaze::tdf::TdfTreeParser;

    let packets: Vec<CapturedPacket> = packet_buffer.lock().clone();
    let packet_count = packets.len();

    if let Some(idx) = state.selected_packet_index {
        if let Some(packet) = packets.get(idx) {
            if state.tdf_tree.is_none() {
                if !packet.payload.is_empty() {
                    let payload_clone = packet.payload.clone();
                    let parse_result = std::panic::catch_unwind(|| {
                        TdfTreeParser::parse_packet(&payload_clone)
                    });

                    match parse_result {
                        Ok(Ok(tree)) => {
                            state.tdf_tree = Some(tree);
                            state.tdf_parse_error = None;
                            state.selected_tdf_path.clear();
                        }
                        Ok(Err(e)) => {
                            let error_msg = format!(
                                "Failed to parse TDF:\nPayload size: {} bytes\nMetadata size: {} bytes\nRaw size: {} bytes\nError: {:?}",
                                packet.payload.len(),
                                packet.metadata_size,
                                packet.raw_packet.len(),
                                e
                            );
                            eprintln!("{}", error_msg);
                            state.tdf_tree = None;
                            state.tdf_parse_error = Some(error_msg);
                        }
                        Err(_) => {
                            let error_msg = format!(
                                "Panic detected!\nPayload size: {} bytes\nMetadata size: {} bytes\nRaw size: {} bytes\n\nThis indicates an unexpected panic in the parser. Please report this issue.",
                                packet.payload.len(),
                                packet.metadata_size,
                                packet.raw_packet.len()
                            );
                            eprintln!("{}", error_msg);
                            state.tdf_tree = None;
                            state.tdf_parse_error = Some(error_msg);
                        }
                    }
                } else {
                    state.tdf_tree = None;
                    state.tdf_parse_error = Some("Empty payload (0 bytes) found".to_string());
                }
            }
        }
    } else {
        state.tdf_tree = None;
        state.tdf_parse_error = None;
        state.selected_tdf_path.clear();
    }

    let (packet_list, selected_packet_data, tdf_tree_clone) = {
        let packet_list: Vec<(usize, CapturedPacket)> = packets
            .iter()
            .enumerate()
            .rev()
            .map(|(i, p)| (i, p.clone()))
            .collect();

        let selected_packet_data = if let Some(idx) = state.selected_packet_index {
            packets.get(idx).map(|p| {
                (
                    p.direction,
                    p.component,
                    p.command,
                    p.command_name.clone(),
                    p.msg_num,
                    p.msg_type.clone(),
                    p.payload_size,
                    p.payload.clone(),
                )
            })
        } else {
            None
        };

        let tdf_tree_clone = state.tdf_tree.clone();
        (packet_list, selected_packet_data, tdf_tree_clone)
    };

    // Top toolbar
    ui.horizontal(|ui| {
        ui.label(format!("Packets: {}", packet_count));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Copy to clipboard button
            if ui.button("📋").clicked() {
                let buffer = packet_buffer.lock();
                let mut output = String::new();
                output.push_str("=== Packet Inspection Data ===\n\n");

                for (idx, packet) in buffer.iter().enumerate().rev() {
                    output.push_str(&format!("Packet #{}:\n", idx));
                    output.push_str(&format!("  Direction: {}\n", packet.direction.to_string()));
                    output.push_str(&format!("  Component: 0x{:04x} ({})\n", packet.component, packet.component));
                    output.push_str(&format!("  Command: 0x{:04x} ({})\n", packet.command, packet.command));
                    if let Some(ref cmd_name) = packet.command_name {
                        output.push_str(&format!("  Command Name: {}\n", cmd_name));
                    }
                    output.push_str(&format!("  Message Number: {}\n", packet.msg_num));
                    output.push_str(&format!("  Message Type: {}\n", packet.msg_type));
                    output.push_str(&format!("  Payload Size: {} bytes\n", packet.payload_size));
                    output.push_str(&format!("  Timestamp: {:.3}\n", packet.timestamp));
                    output.push_str("\n  Hex:\n");
                    let hex_dump = format_hex_dump(&packet.payload, 4096);
                    for line in hex_dump.lines() {
                        output.push_str(&format!("    {}\n", line));
                    }
                    output.push_str("\n");
                }

                ctx.copy_text(output);
            }

            // Save As button
            if ui.button("Save As...").clicked() {
                let buffer = packet_buffer.lock();
                let mut output = String::new();
                output.push_str("=== Packet Inspection Data ===\n\n");

                for (idx, packet) in buffer.iter().enumerate().rev() {
                    output.push_str(&format!("Packet #{}:\n", idx));
                    output.push_str(&format!("  Direction: {}\n", packet.direction.to_string()));
                    output.push_str(&format!("  Component: 0x{:04x} ({})\n", packet.component, packet.component));
                    output.push_str(&format!("  Command: 0x{:04x} ({})\n", packet.command, packet.command));
                    if let Some(ref cmd_name) = packet.command_name {
                        output.push_str(&format!("  Command Name: {}\n", cmd_name));
                    }
                    output.push_str(&format!("  Message Number: {}\n", packet.msg_num));
                    output.push_str(&format!("  Message Type: {}\n", packet.msg_type));
                    output.push_str(&format!("  Payload Size: {} bytes\n", packet.payload_size));
                    output.push_str(&format!("  Timestamp: {:.3}\n", packet.timestamp));
                    output.push_str("\n  Hex:\n");
                    let hex_dump = format_hex_dump(&packet.payload, 4096);
                    for line in hex_dump.lines() {
                        output.push_str(&format!("    {}\n", line));
                    }
                    output.push_str("\n");
                }
                drop(buffer);

                // Use native file dialog
                let file = rfd::FileDialog::new()
                    .add_filter("Text files", &["txt"])
                    .add_filter("All files", &["*"])
                    .set_file_name("packets.txt")
                    .save_file();

                if let Some(path) = file {
                    if let Err(e) = std::fs::write(&path, output) {
                        eprintln!("Failed to save packet data: {}", e);
                    }
                }
            }

            if ui.button("Clear").clicked() {
                let mut buf = packet_buffer.lock();
                buf.clear();
                state.selected_packet_index = None;
                state.tdf_tree = None;
                state.selected_tdf_path.clear();
                state.expanded_tdf_nodes.clear();
                state.open_make_from_index = None;
                return;
            }

            ui.separator();

            let can_inspect = state.selected_packet_index.is_some();
            let inspect_btn = ui.add_enabled(
                can_inspect,
                egui::Button::new(egui::RichText::new("Inspect").strong()),
            );
            if inspect_btn.clicked() {
                state.open_make_from_index = state.selected_packet_index;
            }
            inspect_btn.on_hover_text(
                "Open Make → Blaze with this packet’s component, command, msg #, type, and payload.",
            );
        });
    });

    ui.separator();

    // Three-panel horizontal layout: Packet list | TDF Tree | Field Details
    ui.columns(3, |columns| {
        // Left panel: Packet list
        render_packet_list(&mut columns[0], &packet_list, state);

        // Middle panel: TDF Tree
        render_tdf_tree_panel(&mut columns[1], &tdf_tree_clone, state, &packets);

        // Right panel: Field Details
        render_field_details_panel(&mut columns[2], &tdf_tree_clone, &selected_packet_data, state);
    });
}

/// Render the packet list panel
fn render_packet_list(
    column: &mut egui::Ui,
    packet_list: &[(usize, CapturedPacket)],
    state: &mut BlazeInspectorState,
) {
    column.vertical(|ui| {
        ui.heading("Packet List");
        ui.separator();

        egui::ScrollArea::vertical()
            .id_source("packet_list_scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for (idx, packet) in packet_list {
                    let is_selected = state.selected_packet_index == Some(*idx);

                    let direction_color = match packet.direction {
                        PacketDirection::ClientToBlaze => egui::Color32::from_rgb(100, 150, 255),
                        PacketDirection::BlazeToClient => egui::Color32::from_rgb(255, 150, 100),
                    };

                    let cmd_display = if let Some(ref cmd_name) = packet.command_name {
                        format!("{}", cmd_name)
                    } else {
                        format!("Component={}, Command={}", packet.component, packet.command)
                    };

                    // Format direction with proper arrow character
                    let direction_str = packet.direction.to_string();

                    let response = ui.selectable_label(
                        is_selected,
                        format!(
                            "[{}] {} | {} | Size: {} bytes | MsgNum: {}",
                            direction_str, cmd_display, packet.msg_type, packet.payload_size, packet.msg_num
                        ),
                    );

                    if response.clicked() {
                        state.selected_packet_index = Some(*idx);
                        state.tdf_tree = None; // Force re-parse
                        state.selected_tdf_path.clear();
                        state.expanded_tdf_nodes.clear();
                    }

                    // Highlight with direction color
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
}

/// Render the TDF tree panel
fn render_tdf_tree_panel(
    column: &mut egui::Ui,
    tdf_tree_clone: &Option<Vec<TdfTreeNode>>,
    state: &mut BlazeInspectorState,
    packets: &[CapturedPacket],
) {
    column.vertical(|ui| {
        // Header with clipboard button
        ui.horizontal(|ui| {
            ui.heading("TDF Structure");
            if let Some(ref tree) = tdf_tree_clone {
                if ui.button("📋").on_hover_text("Copy TDF Structure to clipboard").clicked() {
                    let text = format_tdf_tree_as_text(tree);
                    ui.output_mut(|o| o.copied_text = text);
                }
            }
        });
        ui.separator();

        if let Some(ref tree) = tdf_tree_clone {
            egui::ScrollArea::vertical()
                .id_source("tdf_tree_scroll")
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    // Use monospace font for tree display
                    ui.style_mut().text_styles.insert(
                        egui::TextStyle::Body,
                        egui::FontId::monospace(11.0),
                    );

                    // Clone expanded state to avoid borrow issues
                    let expanded_nodes = state.expanded_tdf_nodes.clone();
                    let selected_path = state.selected_tdf_path.clone();

                    // Render tree without mutating state during rendering
                    // We'll collect the changes and apply them after
                    let mut new_expanded = expanded_nodes.clone();
                    let mut new_selected = selected_path.clone();

                    render_tdf_tree_static(
                        ui,
                        tree,
                        Vec::new(),
                        &expanded_nodes,
                        &selected_path,
                        &mut new_expanded,
                        &mut new_selected,
                    );

                    // Apply changes after rendering
                    state.expanded_tdf_nodes = new_expanded;
                    state.selected_tdf_path = new_selected;
                });
        } else if state.selected_packet_index.is_some() {
            // Show parsing status, error, or empty payload message
            ui.vertical(|ui| {
                // Check if there's a parse error or info message
                if let Some(ref error_msg) = state.tdf_parse_error {
                    let is_info = error_msg.starts_with("Message:");
                    let is_empty_payload = error_msg == "Empty payload (0 bytes) found";
                    if is_info {
                        ui.add_space(5.0);
                        egui::ScrollArea::vertical()
                            .id_source("tdf_info_scroll")
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(error_msg)
                                        .family(egui::FontFamily::Monospace)
                                        .size(10.0)
                                        .color(egui::Color32::from_rgb(100, 150, 255)),
                                );
                            });
                    } else if is_empty_payload {
                        ui.add_space(5.0);
                        egui::ScrollArea::vertical()
                            .id_source("tdf_error_scroll")
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(error_msg)
                                        .family(egui::FontFamily::Monospace)
                                        .size(10.0)
                                        .color(egui::Color32::YELLOW),
                                );
                            });
                    } else {
                        ui.add_space(5.0);
                        egui::ScrollArea::vertical()
                            .id_source("tdf_error_scroll")
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(error_msg)
                                        .family(egui::FontFamily::Monospace)
                                        .size(10.0)
                                        .color(egui::Color32::RED),
                                );
                            });
                    }
                    ui.add_space(5.0);
                    if ui.button("Clear Selection").clicked() {
                        state.selected_packet_index = None;
                        state.tdf_tree = None;
                        state.tdf_parse_error = None;
                    }
                } else {
                    // Show parsing status or empty payload
                    ui.label(egui::RichText::new("Parsing data..").color(egui::Color32::YELLOW));
                    ui.add_space(5.0);

                    // Show packet info
                    if let Some(idx) = state.selected_packet_index {
                        if let Some(packet) = packets.get(idx) {
                            ui.label(format!("Payload size: {} bytes", packet.payload.len()));
                            ui.label(format!("Metadata size: {} bytes", packet.metadata_size));

                            if packet.payload.is_empty() {
                                ui.add_space(5.0);
                                ui.label(egui::RichText::new("Empty payload (0 bytes) found")
                                    .color(egui::Color32::from_rgb(100, 150, 255)));
                            } else {
                                ui.add_space(5.0);
                                ui.label("Packet may contain invalid or corrupted TDF data..");
                            }
                            ui.add_space(5.0);
                            if ui.button("Cancel & Clear Selection").clicked() {
                                state.selected_packet_index = None;
                                state.tdf_tree = None;
                                state.tdf_parse_error = None;
                            }
                        }
                    }
                }
            });
        } else {
            ui.label("Select a packet to view TDF structure");
        }
    });
}

/// Render the field details panel
fn render_field_details_panel(
    column: &mut egui::Ui,
    tdf_tree_clone: &Option<Vec<TdfTreeNode>>,
    selected_packet_data: &Option<(
        PacketDirection,
        u16,
        u16,
        Option<String>,
        u32,
        String,
        usize,
        Vec<u8>,
    )>,
    state: &mut BlazeInspectorState,
) {
    column.vertical(|ui| {
        ui.heading("Field Details");
        ui.separator();

        // Check if a TDF field is selected - if so, show field details
        // Otherwise, show packet information
        if !state.selected_tdf_path.is_empty() {
            if let Some(ref tree) = tdf_tree_clone {
                // Safety: get node with error handling
                let node_opt = get_selected_tdf_node(&state.selected_tdf_path, tree);

                if let Some(node) = node_opt {
                    render_field_details(ui, node);
                } else {
                    // Invalid field selection - fall back to packet details if available
                    if selected_packet_data.is_some() {
                        // Clear invalid selection and show packet details instead
                        state.selected_tdf_path.clear();
                        // Recursively call to show packet details (will be shown on next frame)
                        ui.label(egui::RichText::new("Invalid field selection - showing packet details")
                            .color(egui::Color32::YELLOW));
                    } else {
                        ui.label(egui::RichText::new("Error: Invalid field selection")
                            .color(egui::Color32::RED));
                        ui.add_space(5.0);
                        if ui.button("Clear Invalid Selection").clicked() {
                            state.selected_tdf_path.clear();
                        }
                    }
                }
            } else {
                // TDF tree not available - show packet details if available
                if let Some((direction, component, command, cmd_name, msg_num, msg_type, payload_size, payload)) =
                    selected_packet_data
                {
                    render_packet_details(ui, *direction, *component, *command, cmd_name, *msg_num, msg_type, *payload_size, payload);
                } else {
                    ui.label("TDF tree not available");
                }
            }
        } else if let Some((direction, component, command, cmd_name, msg_num, msg_type, payload_size, payload)) =
            selected_packet_data
        {
            render_packet_details(ui, *direction, *component, *command, cmd_name, *msg_num, msg_type, *payload_size, payload);
        } else {
            // No packet selected - show message
            ui.label("Select a packet to view details");
        }
    });
}

/// Render field details for a selected TDF node
fn render_field_details(ui: &mut egui::Ui, node: &TdfTreeNode) {
    ui.vertical(|ui| {
        // Safety: sanitize all strings before display
        let tag_display = sanitize_for_display(&node.tag);
        let type_display = sanitize_for_display(&node.value_type);
        let value_display = sanitize_for_display(&node.value_display);

        ui.label(egui::RichText::new("Tag:").heading());
        ui.label(
            egui::RichText::new(tag_display)
                .family(egui::FontFamily::Monospace)
                .size(11.0),
        );
        ui.add_space(5.0);

        ui.label(egui::RichText::new("Type:").heading());
        ui.label(
            egui::RichText::new(type_display)
                .family(egui::FontFamily::Monospace)
                .size(11.0),
        );
        ui.add_space(5.0);

        ui.label(egui::RichText::new("Name:").heading());
        let name_display = sanitize_for_display(&node.name);
        ui.label(
            egui::RichText::new(name_display)
                .family(egui::FontFamily::Monospace)
                .size(11.0),
        );
        ui.add_space(5.0);

        // Show children count for structs/lists
        if !node.children.is_empty() {
            ui.label(egui::RichText::new("Children:").heading());
            ui.label(format!("{} items", node.children.len()));
            ui.add_space(5.0);
        }

        ui.label(egui::RichText::new("Value:").heading());
        ui.add_space(5.0);

        egui::ScrollArea::vertical()
            .id_source("field_details_scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // Limit value display length to prevent UI freezing
                let display_text = if value_display.len() > 10000 {
                    format!(
                        "{}...\n\n(Truncated - {} more characters)",
                        &value_display[..10000],
                        value_display.len() - 10000
                    )
                } else {
                    value_display
                };

                ui.label(
                    egui::RichText::new(display_text)
                        .family(egui::FontFamily::Monospace)
                        .size(10.0),
                );
            });
    });
}

/// Render packet details
fn render_packet_details(
    ui: &mut egui::Ui,
    direction: PacketDirection,
    component: u16,
    command: u16,
    cmd_name: &Option<String>,
    msg_num: u32,
    msg_type: &str,
    payload_size: usize,
    payload: &[u8],
) {
    ui.vertical(|ui| {
        ui.label(egui::RichText::new("Direction:").heading());
        let direction_color = match direction {
            PacketDirection::ClientToBlaze => egui::Color32::from_rgb(100, 150, 255),
            PacketDirection::BlazeToClient => egui::Color32::from_rgb(255, 150, 100),
        };
        let direction_str = direction.to_string();
        ui.label(egui::RichText::new(direction_str).color(direction_color));
        ui.add_space(5.0);

        ui.label(egui::RichText::new("Component:").heading());
        ui.label(format!("0x{:04x} ({})", component, component));
        ui.add_space(5.0);

        ui.label(egui::RichText::new("Command:").heading());
        ui.label(format!("0x{:04x} ({})", command, command));
        ui.add_space(5.0);

        if let Some(ref cmd_name) = cmd_name {
            ui.label(egui::RichText::new("Command Name:").heading());
            ui.label(cmd_name);
            ui.add_space(5.0);
        }

        ui.label(egui::RichText::new("Message Number:").heading());
        ui.label(format!("{}", msg_num));
        ui.add_space(5.0);

        ui.label(egui::RichText::new("Message Type:").heading());
        ui.label(msg_type);
        ui.add_space(5.0);

        ui.label(egui::RichText::new("Payload Size:").heading());
        ui.label(format!("{} bytes", payload_size));
        ui.add_space(10.0);

        ui.separator();
        ui.add_space(5.0);

        ui.label(egui::RichText::new("Hex:").heading());
        ui.add_space(5.0);

        egui::ScrollArea::vertical()
            .id_source("hex_dump_scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let display_text = format_hex_dump(payload, 4096);
                ui.label(
                    egui::RichText::new(display_text)
                        .family(egui::FontFamily::Monospace)
                        .size(10.0),
                );
            });
    });
}

/// Render TDF tree recursively
fn render_tdf_tree_static(
    ui: &mut egui::Ui,
    nodes: &[TdfTreeNode],
    path: Vec<usize>,
    expanded_nodes: &HashSet<Vec<usize>>,
    selected_path: &Vec<usize>,
    new_expanded: &mut HashSet<Vec<usize>>,
    new_selected: &mut Vec<usize>,
) {
    // Safety: limit recursion depth to prevent stack overflow
    const MAX_DEPTH: usize = 100;
    if path.len() > MAX_DEPTH {
        ui.label(egui::RichText::new("... (max depth reached)").color(egui::Color32::GRAY));
        return;
    }

    // Safety: limit number of nodes to render
    const MAX_NODES: usize = 5000;
    let nodes_to_render = nodes.len().min(MAX_NODES);

    for (idx, node) in nodes.iter().take(nodes_to_render).enumerate() {
        // Safety: limit path length
        if path.len() > 50 {
            continue;
        }
        let mut current_path = path.clone();
        current_path.push(idx);
        let is_selected = selected_path == &current_path;
        let is_expanded = expanded_nodes.contains(&current_path);

        if !node.children.is_empty() {
            // Expandable node - use simple ASCII arrows for better compatibility
            let expand_icon = if is_expanded { "-" } else { "+" };
            // Sanitize node name to ensure it displays correctly
            // Allow all printable ASCII characters, only escape control chars and non-ASCII
            let mut node_name = String::new();
            for c in node.name.chars().take(200) {
                // Limit name length
                let byte = c as u32;
                if c.is_ascii() && byte >= 0x20 && byte <= 0x7E {
                    // Printable ASCII (space through tilde) - allow all
                    node_name.push(c);
                } else if c.is_control() && c != '\n' && c != '\t' && c != '\r' {
                    // Control characters (except common whitespace) - escape
                    if byte <= 0xFF {
                        node_name.push_str(&format!("\\x{:02X}", byte as u8));
                    } else {
                        node_name.push_str(&format!("\\u{:04X}", byte));
                    }
                } else if byte > 0x7E {
                    // Non-ASCII - escape
                    if byte <= 0xFF {
                        node_name.push_str(&format!("\\x{:02X}", byte as u8));
                    } else {
                        node_name.push_str(&format!("\\u{:04X}", byte));
                    }
                } else {
                    // Allow newline, tab, carriage return
                    node_name.push(c);
                }
            }

            let response = ui.selectable_label(
                is_selected,
                egui::RichText::new(format!("[{}] {}", expand_icon, node_name))
                    .family(egui::FontFamily::Monospace)
                    .size(11.0),
            );

            if response.clicked() {
                // Safety: validate path before setting selection
                let path_valid = if current_path.len() == 1 {
                    current_path[0] < nodes.len()
                } else {
                    // For nested paths, we'll validate when accessing
                    true
                };

                if path_valid {
                    if is_expanded {
                        new_expanded.remove(&current_path);
                    } else {
                        new_expanded.insert(current_path.clone());
                    }
                    *new_selected = current_path.clone();
                }
            }

            // Render children if expanded
            if is_expanded {
                // Safety: limit recursion and validate children
                if node.children.len() > 10000 {
                    ui.label(egui::RichText::new(format!("... ({} children - too many to display)", node.children.len()))
                        .color(egui::Color32::GRAY)
                        .size(10.0));
                } else {
                    // Create unique ID from path
                    let indent_id = format!("tdf_indent_{}", current_path.iter().map(|i| i.to_string()).collect::<Vec<_>>().join("_"));

                    // Safety: render children with validation
                    ui.indent(indent_id, |ui| {
                        // Validate children before rendering
                        if node.children.len() <= 10000 {
                            render_tdf_tree_static(
                                ui,
                                &node.children,
                                current_path,
                                expanded_nodes,
                                selected_path,
                                new_expanded,
                                new_selected,
                            );
                        } else {
                            ui.label(egui::RichText::new("Too many children to display safely")
                                .color(egui::Color32::YELLOW)
                                .size(10.0));
                        }
                    });
                }
            }
        } else {
            // Leaf node - sanitize name for display
            // Allow most printable ASCII characters, only escape control chars and non-ASCII
            let mut node_name = String::new();
            for c in node.name.chars() {
                let byte = c as u32;
                if c.is_ascii() && byte >= 0x20 && byte <= 0x7E {
                    // Printable ASCII (space through tilde) - allow all
                    node_name.push(c);
                } else if c.is_control() && c != '\n' && c != '\t' && c != '\r' {
                    // Control characters (except common whitespace) - escape
                    if byte <= 0xFF {
                        node_name.push_str(&format!("\\x{:02X}", byte as u8));
                    } else {
                        node_name.push_str(&format!("\\u{:04X}", byte));
                    }
                } else if byte > 0x7E {
                    // Non-ASCII - escape
                    if byte <= 0xFF {
                        node_name.push_str(&format!("\\x{:02X}", byte as u8));
                    } else {
                        node_name.push_str(&format!("\\u{:04X}", byte));
                    }
                } else {
                    // Allow newline, tab, carriage return
                    node_name.push(c);
                }
            }

            let response = ui.selectable_label(
                is_selected,
                egui::RichText::new(format!("  {}", node_name))
                    .family(egui::FontFamily::Monospace)
                    .size(11.0),
            );
            if response.clicked() {
                *new_selected = current_path;
            }
        }
    }

    // Show message if nodes were truncated
    if nodes.len() > MAX_NODES {
        ui.label(egui::RichText::new(format!("... ({} more nodes not shown)", nodes.len() - MAX_NODES))
            .color(egui::Color32::GRAY)
            .size(10.0));
    }
}

pub fn tdf_tree_to_plaintext(nodes: &[TdfTreeNode]) -> String {
    format_tdf_tree_as_text(nodes)
}

/// Format TDF tree as plain text for clipboard
fn format_tdf_tree_as_text(nodes: &[TdfTreeNode]) -> String {
    let mut output = String::new();
    format_tdf_tree_recursive(nodes, &mut output, 0);
    output
}

/// Recursively format TDF tree nodes as text
fn format_tdf_tree_recursive(nodes: &[TdfTreeNode], output: &mut String, indent: usize) {
    for node in nodes {
        let indent_str = "  ".repeat(indent);
        
        if !node.children.is_empty() {
            // Node with children - include type info
            output.push_str(&format!("{}{} ({})\n", indent_str, node.name, node.value_type));
            format_tdf_tree_recursive(&node.children, output, indent + 1);
        } else {
            // Leaf: listener tree lists `node.name` only. Many parsers set name to `{tag}: {value}`;
            // appending `: value_display` again duplicates plain strings (e.g. F00: BAR: BAR).
            let same_as_name_suffix = format!("{}: {}", node.tag, node.value_display) == node.name;
            let redundant = node.value_display.is_empty()
                || node.value_display == "Unknown"
                || node.name == node.value_display
                || same_as_name_suffix;
            if redundant {
                output.push_str(&format!("{}{}\n", indent_str, node.name));
            } else {
                output.push_str(&format!("{}{}: {}\n", indent_str, node.name, node.value_display));
            }
        }
    }
}

/// Get the selected TDF node from the tree
fn get_selected_tdf_node<'a>(
    selected_path: &[usize],
    tree: &'a [TdfTreeNode],
) -> Option<&'a TdfTreeNode> {
    // Safety: validate path is not empty
    if selected_path.is_empty() {
        return None;
    }

    // Safety: validate first index
    let first_idx = *selected_path.get(0)?;
    if first_idx >= tree.len() {
        return None;
    }

    // Safety: get first node
    let mut current = match tree.get(first_idx) {
        Some(node) => node,
        None => return None,
    };

    // Safety: traverse path with validation
    for &idx in selected_path.iter().skip(1) {
        if idx >= current.children.len() {
            return None; // Invalid path
        }
        current = match current.children.get(idx) {
            Some(node) => node,
            None => return None, // Unsupported path
        };
    }

    Some(current)
}

/// Sanitize string for safe display in UI
/// Only escapes actual control characters, preserves all printable ASCII (0x20-0x7E)
fn sanitize_for_display(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars().take(10000) {
        // Limit total length
        let byte = c as u32;
        
        // Allow all printable ASCII (0x20 space through 0x7E tilde)
        if byte >= 0x20 && byte <= 0x7E {
            result.push(c);
        } else if c.is_control() && c != '\n' && c != '\t' && c != '\r' {
            // Replace control characters (except common whitespace) with escape sequence
            if byte <= 0xFF {
                result.push_str(&format!("\\x{:02X}", byte as u8));
            } else {
                result.push_str(&format!("\\u{:04X}", byte));
            }
        } else if byte > 0x7E && byte <= 0x10FFFF {
            // Non-ASCII but valid Unicode - allow it
            result.push(c);
        } else if byte > 0x10FFFF {
            // Invalid Unicode, skip
            continue;
        } else {
            // Allow newline, tab, carriage return
            result.push(c);
        }
    }
    result
}
