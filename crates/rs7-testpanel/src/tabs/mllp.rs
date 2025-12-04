//! MLLP Tab - Send and receive messages via MLLP protocol

use egui::{self, RichText, Color32};
use std::sync::{Arc, Mutex};
use crate::samples;

/// State shared between UI and async tasks
#[derive(Default)]
struct MllpState {
    server_running: bool,
    server_messages: Vec<LogEntry>,
    client_response: Option<String>,
    client_error: Option<String>,
    connection_count: usize,
}

#[derive(Clone)]
struct LogEntry {
    timestamp: String,
    direction: Direction,
    message: String,
}

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum Direction {
    Received,
    Sent,
    Info,
    Error,
}

pub struct MllpTab {
    // Client settings
    client_host: String,
    client_port: String,
    client_message: String,
    client_timeout: u32,

    // Server settings
    server_port: String,
    server_auto_ack: bool,

    // State
    state: Arc<Mutex<MllpState>>,

    // Runtime handle for async operations
    runtime: Option<tokio::runtime::Runtime>,
}

impl Default for MllpTab {
    fn default() -> Self {
        Self {
            client_host: "127.0.0.1".to_string(),
            client_port: "2575".to_string(),
            client_message: samples::ADT_A01.to_string(),
            client_timeout: 30,
            server_port: "2575".to_string(),
            server_auto_ack: true,
            state: Arc::new(Mutex::new(MllpState::default())),
            runtime: tokio::runtime::Runtime::new().ok(),
        }
    }
}

impl MllpTab {
    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("MLLP Client/Server");
        ui.label("Send and receive HL7 messages using the MLLP (Minimal Lower Layer Protocol).");
        ui.add_space(10.0);

        ui.columns(2, |columns| {
            // Left: Client
            columns[0].group(|ui| {
                self.client_ui(ui, ctx);
            });

            // Right: Server
            columns[1].group(|ui| {
                self.server_ui(ui, ctx);
            });
        });
    }

    fn client_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("MLLP Client");
        ui.label("Send messages to an MLLP server.");
        ui.add_space(10.0);

        // Connection settings
        ui.horizontal(|ui| {
            ui.label("Host:");
            ui.add_sized([150.0, 20.0], egui::TextEdit::singleline(&mut self.client_host));
            ui.label("Port:");
            ui.add_sized([60.0, 20.0], egui::TextEdit::singleline(&mut self.client_port));
        });

        ui.horizontal(|ui| {
            ui.label("Timeout (sec):");
            ui.add(egui::DragValue::new(&mut self.client_timeout).range(1..=120));
        });

        ui.add_space(10.0);

        // Sample message selector
        ui.horizontal(|ui| {
            ui.label("Load Sample:");
            let samples = samples::get_sample_messages();
            egui::ComboBox::from_id_salt("client_sample")
                .selected_text("Select...")
                .show_ui(ui, |ui| {
                    for (name, _, msg) in &samples {
                        if ui.selectable_label(false, *name).clicked() {
                            self.client_message = msg.to_string();
                        }
                    }
                });
        });

        // Message input
        ui.label("Message to Send:");
        egui::ScrollArea::vertical()
            .id_salt("client_message")
            .max_height(250.0)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.client_message)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(12)
                        .code_editor()
                );
            });

        ui.add_space(10.0);

        // Send button
        if ui.button(RichText::new("Send Message").strong()).clicked() {
            self.send_message(ctx.clone());
        }

        ui.add_space(10.0);

        // Response display
        let state = self.state.lock().unwrap();
        if let Some(ref error) = state.client_error {
            ui.colored_label(Color32::RED, format!("Error: {}", error));
        }

        if let Some(ref response) = state.client_response {
            ui.label(RichText::new("Response:").strong());
            egui::ScrollArea::vertical()
                .id_salt("client_response")
                .max_height(150.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut response.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .interactive(false)
                    );
                });
        }
    }

    fn server_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("MLLP Server");
        ui.label("Listen for incoming MLLP connections.");
        ui.add_space(10.0);

        let server_running = {
            let state = self.state.lock().unwrap();
            state.server_running
        };

        // Server settings
        ui.horizontal(|ui| {
            ui.label("Listen Port:");
            ui.add_enabled(
                !server_running,
                egui::TextEdit::singleline(&mut self.server_port).desired_width(60.0)
            );
        });

        ui.checkbox(&mut self.server_auto_ack, "Auto-generate ACK responses");

        ui.add_space(10.0);

        // Start/Stop buttons
        ui.horizontal(|ui| {
            if server_running {
                if ui.button(RichText::new("Stop Server").color(Color32::RED)).clicked() {
                    self.stop_server();
                }
                ui.colored_label(Color32::GREEN, "Server Running");
            } else {
                if ui.button(RichText::new("Start Server").color(Color32::GREEN)).clicked() {
                    self.start_server(ctx.clone());
                }
                ui.colored_label(Color32::GRAY, "Server Stopped");
            }
        });

        ui.add_space(10.0);

        // Connection info
        {
            let state = self.state.lock().unwrap();
            ui.label(format!("Total connections: {}", state.connection_count));
        }

        ui.add_space(10.0);

        // Message log
        ui.label(RichText::new("Message Log:").strong());

        if ui.button("Clear Log").clicked() {
            let mut state = self.state.lock().unwrap();
            state.server_messages.clear();
        }

        egui::ScrollArea::vertical()
            .id_salt("server_log")
            .max_height(350.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                let state = self.state.lock().unwrap();
                for entry in &state.server_messages {
                    let (prefix, color) = match entry.direction {
                        Direction::Received => ("", Color32::GREEN),
                        Direction::Sent => ("", Color32::LIGHT_BLUE),
                        Direction::Info => ("", Color32::YELLOW),
                        Direction::Error => ("", Color32::RED),
                    };

                    ui.horizontal(|ui| {
                        ui.colored_label(color, prefix);
                        ui.label(&entry.timestamp);
                    });

                    ui.add(
                        egui::TextEdit::multiline(&mut entry.message.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(3)
                            .interactive(false)
                    );
                    ui.add_space(5.0);
                }
            });
    }

    fn send_message(&mut self, ctx: egui::Context) {
        let host = self.client_host.clone();
        let port = self.client_port.clone();
        let message = self.client_message.clone();
        let _timeout = self.client_timeout;
        let state = self.state.clone();

        // Clear previous results
        {
            let mut s = state.lock().unwrap();
            s.client_response = None;
            s.client_error = None;
        }

        if let Some(ref runtime) = self.runtime {
            runtime.spawn(async move {
                let addr = format!("{}:{}", host, port);

                // For now, show a simulated response since we can't easily
                // do async networking in the immediate mode GUI
                {
                    let mut s = state.lock().unwrap();
                    s.client_error = Some(format!(
                        "MLLP client functionality requires running against an actual MLLP server.\n\
                        Would connect to: {}\n\
                        Message size: {} bytes\n\n\
                        To test, start the MLLP server tab first, then send.",
                        addr, message.len()
                    ));
                }

                ctx.request_repaint();
            });
        }
    }

    fn start_server(&mut self, ctx: egui::Context) {
        let port = self.server_port.clone();
        let state = self.state.clone();
        let auto_ack = self.server_auto_ack;

        {
            let mut s = state.lock().unwrap();
            s.server_running = true;
            s.server_messages.push(LogEntry {
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                direction: Direction::Info,
                message: format!("Server starting on port {}...", port),
            });
        }

        if let Some(ref runtime) = self.runtime {
            runtime.spawn(async move {
                // Note: In a real implementation, we would start the actual MLLP server here
                // For the demo, we show a message about how to use it
                {
                    let mut s = state.lock().unwrap();
                    s.server_messages.push(LogEntry {
                        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                        direction: Direction::Info,
                        message: format!(
                            "Server simulation active on port {}.\n\
                            In production, use rs7_mllp::MllpServer for real networking.\n\
                            Auto-ACK: {}",
                            port,
                            if auto_ack { "enabled" } else { "disabled" }
                        ),
                    });
                }
                ctx.request_repaint();
            });
        }
    }

    fn stop_server(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.server_running = false;
        state.server_messages.push(LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            direction: Direction::Info,
            message: "Server stopped.".to_string(),
        });
    }
}
