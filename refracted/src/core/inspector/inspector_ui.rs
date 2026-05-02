//! Toolkit UI: protocol listeners, message builders, and optional research proxies.

use crate::core::inspector::blaze_inspector::*;
use crate::core::inspector::grpc_inspector::*;
use crate::core::inspector::http_inspector::*;
use crate::core::inspector::inspector_module::*;
use crate::core::inspector::lsx_inspector::*;
use crate::core::inspector::toolkit_make_blaze;
use crate::core::inspector::toolkit_make_grpc;
use std::sync::Arc;

/// Inspector UI state
pub struct InspectorUiState {
    pub mode: InspectorMode,
    pub selected_type: InspectorType,
    pub blaze_state: BlazeInspectorState,
    pub grpc_state: GrpcInspectorState,
    pub http_state: HttpInspectorState,
    pub lsx_state: LsxInspectorState,
    pub proxy_state: Arc<ProxyState>,
    pub proxy_config_http_listen: String,
    pub proxy_config_https_listen: String,
    pub proxy_config_grpc_listen: String,
    pub proxy_config_blaze_listen: String,
    pub proxy_config_lsx_listen: String,
    pub proxy_config_target_host: String,
    pub proxy_config_target_http: String,
    pub proxy_config_target_https: String,
    pub proxy_config_target_grpc: String,
    pub proxy_config_target_blaze: String,
    pub proxy_config_target_lsx: String,
    pub proxy_config_enable_http: bool,
    pub proxy_config_enable_https: bool,
    pub proxy_config_enable_grpc: bool,
    pub proxy_config_enable_blaze: bool,
    pub proxy_config_enable_lsx: bool,

    pub toolkit_workbench: ToolkitWorkbenchMode,
    pub toolkit_make_tab: ToolkitMakeTab,
    pub blaze_make: toolkit_make_blaze::BlazeMakeWorkbenchState,
    pub grpc_make: toolkit_make_grpc::GrpcMakeWorkbenchState,
}

impl InspectorUiState {
    pub fn new() -> Self {
        let proxy_state = Arc::new(ProxyState::new());
        init_global_proxy_state(proxy_state.clone());
        
        let default_config = ProxyConfig::default();
        Self {
            mode: InspectorMode::Emulator,
            selected_type: InspectorType::Blaze,
            blaze_state: BlazeInspectorState::new(),
            grpc_state: GrpcInspectorState::new(),
            http_state: HttpInspectorState::new(),
            lsx_state: LsxInspectorState::new(),
            proxy_state,
            proxy_config_http_listen: default_config.http_listen_port.to_string(),
            proxy_config_https_listen: default_config.https_listen_port.to_string(),
            proxy_config_grpc_listen: default_config.grpc_listen_port.to_string(),
            proxy_config_blaze_listen: default_config.blaze_listen_port.to_string(),
            proxy_config_lsx_listen: default_config.lsx_listen_port.to_string(),
            proxy_config_target_host: default_config.target_host.clone(),
            proxy_config_target_http: default_config.target_http_port.to_string(),
            proxy_config_target_https: default_config.target_https_port.to_string(),
            proxy_config_target_grpc: default_config.target_grpc_port.to_string(),
            proxy_config_target_blaze: default_config.target_blaze_port.to_string(),
            proxy_config_target_lsx: default_config.target_lsx_port.to_string(),
            proxy_config_enable_http: default_config.enable_http,
            proxy_config_enable_https: default_config.enable_https,
            proxy_config_enable_grpc: default_config.enable_grpc,
            proxy_config_enable_blaze: default_config.enable_blaze,
            proxy_config_enable_lsx: default_config.enable_lsx,
            toolkit_workbench: ToolkitWorkbenchMode::Listen,
            toolkit_make_tab: ToolkitMakeTab::Blaze,
            blaze_make: toolkit_make_blaze::BlazeMakeWorkbenchState::default(),
            grpc_make: toolkit_make_grpc::GrpcMakeWorkbenchState::default(),
        }
    }
}

/// Renders the toolkit: **Listen** (live captures) or **Make** (compose or decode payloads).
pub fn render_toolkit(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    state: &mut InspectorUiState,
    packet_buffer: PacketBuffer,
    grpc_buffer: GrpcBuffer,
    http_buffer: HttpBuffer,
    lsx_buffer: LsxBuffer,
) {
    // Proxy settings while research-mode proxies are running
    if state.mode == InspectorMode::Research && state.proxy_state.is_running() {
        render_proxy_configuration(ui, state);
        ui.separator();
    }

    ui.horizontal(|ui| {
        if ui
            .selectable_label(state.toolkit_workbench == ToolkitWorkbenchMode::Listen, "Listen")
            .clicked()
        {
            state.toolkit_workbench = ToolkitWorkbenchMode::Listen;
        }
        if ui
            .selectable_label(state.toolkit_workbench == ToolkitWorkbenchMode::Make, "Make")
            .clicked()
        {
            state.toolkit_workbench = ToolkitWorkbenchMode::Make;
        }
    });
    ui.separator();

    match state.toolkit_workbench {
        ToolkitWorkbenchMode::Listen => {
            ui.horizontal(|ui| {
                ui.label("Protocol:");
                ui.separator();

                if ui
                    .selectable_label(state.selected_type == InspectorType::Blaze, "Blaze")
                    .clicked()
                {
                    state.selected_type = InspectorType::Blaze;
                }

                if ui
                    .selectable_label(state.selected_type == InspectorType::Grpc, "gRPC")
                    .clicked()
                {
                    state.selected_type = InspectorType::Grpc;
                }

                if ui
                    .selectable_label(state.selected_type == InspectorType::Http, "HTTP")
                    .clicked()
                {
                    state.selected_type = InspectorType::Http;
                }

                if ui
                    .selectable_label(state.selected_type == InspectorType::Lsx, "LSX")
                    .clicked()
                {
                    state.selected_type = InspectorType::Lsx;
                }
            });

            ui.separator();

            match state.selected_type {
                InspectorType::Blaze => {
                    render_blaze_inspector(ctx, ui, &mut state.blaze_state, packet_buffer.clone());
                    if let Some(idx) = state.blaze_state.open_make_from_index.take() {
                        if let Some(packet) = packet_buffer.lock().get(idx).cloned() {
                            toolkit_make_blaze::prefill_from_captured_packet(
                                &mut state.blaze_make,
                                &packet,
                            );
                            state.toolkit_workbench = ToolkitWorkbenchMode::Make;
                            state.toolkit_make_tab = ToolkitMakeTab::Blaze;
                            ctx.request_repaint();
                        }
                    }
                }
                InspectorType::Grpc => {
                    render_grpc_inspector(ctx, ui, &mut state.grpc_state, grpc_buffer.clone());
                    if let Some(ix) = state.grpc_state.open_make_from_index.take() {
                        if let Some(g) = grpc_buffer.lock().get(ix).cloned() {
                            toolkit_make_grpc::prefill_capture_body(&mut state.grpc_make, &g.body);
                            state.toolkit_workbench = ToolkitWorkbenchMode::Make;
                            state.toolkit_make_tab = ToolkitMakeTab::Grpc;
                            ctx.request_repaint();
                        }
                    }
                }
                InspectorType::Http => {
                    render_http_inspector(ctx, ui, &mut state.http_state, http_buffer);
                }
                InspectorType::Lsx => {
                    render_lsx_inspector(ctx, ui, &mut state.lsx_state, lsx_buffer);
                }
            }
        }
        ToolkitWorkbenchMode::Make => {
            ui.horizontal(|ui| {
                ui.label("Protocol:");
                ui.separator();

                if ui
                    .selectable_label(state.toolkit_make_tab == ToolkitMakeTab::Blaze, "Blaze")
                    .clicked()
                {
                    state.toolkit_make_tab = ToolkitMakeTab::Blaze;
                }
                if ui
                    .selectable_label(state.toolkit_make_tab == ToolkitMakeTab::Grpc, "gRPC")
                    .clicked()
                {
                    state.toolkit_make_tab = ToolkitMakeTab::Grpc;
                }
            });
            ui.separator();
            match state.toolkit_make_tab {
                ToolkitMakeTab::Blaze => {
                    toolkit_make_blaze::render_blaze_make(ui, &mut state.blaze_make, ctx);
                }
                ToolkitMakeTab::Grpc => {
                    toolkit_make_grpc::render_grpc_make(ui, &mut state.grpc_make, ctx);
                }
            }
        }
    }
}

/// Backwards-compatible name for embedders.
pub fn render_inspector(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    state: &mut InspectorUiState,
    packet_buffer: PacketBuffer,
    grpc_buffer: GrpcBuffer,
    http_buffer: HttpBuffer,
    lsx_buffer: LsxBuffer,
) {
    render_toolkit(
        ctx,
        ui,
        state,
        packet_buffer,
        grpc_buffer,
        http_buffer,
        lsx_buffer,
    );
}

/// Render proxy configuration UI
fn render_proxy_configuration(ui: &mut egui::Ui, state: &mut InspectorUiState) {
    ui.collapsing("Proxy Configuration", |ui| {
        let is_running = state.proxy_state.is_running();
        
        // Status indicator
        ui.horizontal(|ui| {
            ui.label("Status:");
            if is_running {
                ui.label(egui::RichText::new("● Running").color(egui::Color32::GREEN));
            } else {
                ui.label(egui::RichText::new("○ Stopped").color(egui::Color32::GRAY));
            }
        });

        ui.separator();

        // Listen ports configuration (read-only when running)
        ui.heading("Listen Ports (Proxy)");
        ui.horizontal(|ui| {
            ui.label("HTTP:");
            ui.add(egui::TextEdit::singleline(&mut state.proxy_config_http_listen)
                .interactive(!is_running));
            ui.label("HTTPS:");
            ui.add(egui::TextEdit::singleline(&mut state.proxy_config_https_listen)
                .interactive(!is_running));
        });
        ui.horizontal(|ui| {
            ui.label("gRPC:");
            ui.add(egui::TextEdit::singleline(&mut state.proxy_config_grpc_listen)
                .interactive(!is_running));
            ui.label("Blaze:");
            ui.add(egui::TextEdit::singleline(&mut state.proxy_config_blaze_listen)
                .interactive(!is_running));
        });
        if !is_running {
            ui.horizontal(|ui| {
                ui.label("LSX:");
                ui.add(egui::TextEdit::singleline(&mut state.proxy_config_lsx_listen)
                    .interactive(true));
            });
        }

        ui.separator();

        // Target server configuration (Blaze and LSX - HTTP/HTTPS/gRPC extract from Host header)
        ui.heading("Target Server");
        ui.label(egui::RichText::new("Note: HTTP, HTTPS, and gRPC proxies automatically extract the destination from the Host header. Blaze and LSX require manual target configuration.")
            .size(11.0)
            .color(egui::Color32::GRAY));
        ui.horizontal(|ui| {
            ui.label("Host:");
            ui.text_edit_singleline(&mut state.proxy_config_target_host);
        });
        ui.horizontal(|ui| {
            ui.label("Blaze Port:");
            ui.text_edit_singleline(&mut state.proxy_config_target_blaze);
            ui.label("LSX Port:");
            ui.text_edit_singleline(&mut state.proxy_config_target_lsx);
        });

        ui.separator();

        // Note: Start/Stop is now controlled from Actions menu
        // Just show status and configuration here
        
        if is_running {
            ui.add_space(5.0);
        }

        ui.label(egui::RichText::new("Note: Configure your client to connect to the proxy listen ports instead of the target server directly.")
            .size(12.0)
            .color(egui::Color32::GRAY));
    });
}

/// Start all proxy servers (only enabled ones)
pub fn start_proxy_servers(proxy_state: Arc<ProxyState>) {
    use crate::core::inspector::proxy::*;
    
    proxy_state.start();
    let running = proxy_state.running.clone();

    // Start HTTP proxy (if enabled)
    {
        let config = proxy_state.config.lock().clone();
        if config.enable_http {
            let config = proxy_state.config.clone();
            let running_clone = running.clone();
            tokio::spawn(async move {
                let config = config.lock().clone();
                let _ = start_http_proxy(
                    config.http_listen_port,
                    config.target_host.clone(),
                    config.target_http_port,
                    running_clone.clone(),
                ).await;
            });
        }
    }

    // Start HTTPS proxy (if enabled)
    {
        let config = proxy_state.config.lock().clone();
        if config.enable_https {
            let config = proxy_state.config.clone();
            let running_clone = running.clone();
            tokio::spawn(async move {
                let config = config.lock().clone();
                let _ = start_https_proxy(
                    config.https_listen_port,
                    config.target_host.clone(),
                    config.target_https_port,
                    running_clone.clone(),
                ).await;
            });
        }
    }

    // Start gRPC proxy (if enabled)
    {
        let config = proxy_state.config.lock().clone();
        if config.enable_grpc {
            let config = proxy_state.config.clone();
            let running_clone = running.clone();
            tokio::spawn(async move {
                let config = config.lock().clone();
                let _ = start_grpc_proxy(
                    config.grpc_listen_port,
                    config.target_host.clone(),
                    config.target_grpc_port,
                    running_clone.clone(),
                ).await;
            });
        }
    }

    // Start Blaze proxy (if enabled)
    {
        let config = proxy_state.config.lock().clone();
        if config.enable_blaze {
            let config = proxy_state.config.clone();
            let running_clone = running.clone();
            tokio::spawn(async move {
                let config = config.lock().clone();
                let _ = start_blaze_proxy(
                    config.blaze_listen_port,
                    config.target_host.clone(),
                    config.target_blaze_port,
                    running_clone.clone(),
                ).await;
            });
        }
    }

    // Start LSX proxy (if enabled)
    {
        let config = proxy_state.config.lock().clone();
        if config.enable_lsx {
            let config = proxy_state.config.clone();
            let running_clone = running.clone();
            tokio::spawn(async move {
                let config = config.lock().clone();
                // LSX proxy - use HTTP proxy as LSX is typically HTTP-based
                let _ = start_http_proxy(
                    config.lsx_listen_port,
                    config.target_host.clone(),
                    config.target_lsx_port,
                    running_clone.clone(),
                ).await;
            });
        }
    }
}


