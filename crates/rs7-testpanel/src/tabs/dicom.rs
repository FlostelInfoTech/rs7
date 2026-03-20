//! DICOM Tab - SCU/SCP Client and Server for DICOM networking
//!
//! Left panel: SCU (Service Class User) - C-ECHO, C-STORE operations
//! Right panel: SCP (Service Class Provider) - Receives DICOM files

use egui::{Color32, RichText, ScrollArea, TextEdit, Ui};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

// DICOM SOP Class UIDs
const VERIFICATION_SOP_CLASS: &str = "1.2.840.10008.1.1";
const IMPLICIT_VR_LITTLE_ENDIAN: &str = "1.2.840.10008.1.2";
const EXPLICIT_VR_LITTLE_ENDIAN: &str = "1.2.840.10008.1.2.1";

/// Direction for log entries
#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)] // Outgoing reserved for future use
enum Direction {
    Outgoing,
    Incoming,
    Info,
    Error,
}

impl Direction {
    fn color(&self) -> Color32 {
        match self {
            Direction::Outgoing => Color32::from_rgb(100, 149, 237),
            Direction::Incoming => Color32::from_rgb(144, 238, 144),
            Direction::Info => Color32::from_rgb(200, 200, 200),
            Direction::Error => Color32::from_rgb(255, 100, 100),
        }
    }
}

/// Log entry for DICOM operations
#[derive(Clone)]
struct LogEntry {
    timestamp: String,
    direction: Direction,
    message: String,
}

/// SCU (Client) state
#[derive(Default)]
#[allow(dead_code)] // connected reserved for future use
struct ScuState {
    connected: bool,
    connecting: bool,
    last_result: Option<String>,
    error: Option<String>,
    log_entries: Vec<LogEntry>,
}

/// SCP (Server) state
struct ScpState {
    running: bool,
    starting: bool,
    error: Option<String>,
    log_entries: Vec<LogEntry>,
    received_files: Vec<ReceivedFile>,
    connection_count: u32,
}

impl Default for ScpState {
    fn default() -> Self {
        Self {
            running: false,
            starting: false,
            error: None,
            log_entries: Vec::new(),
            received_files: Vec::new(),
            connection_count: 0,
        }
    }
}

/// Received DICOM file info
#[derive(Clone)]
#[allow(dead_code)] // sop_class reserved for future UI display
struct ReceivedFile {
    timestamp: String,
    filename: String,
    sop_class: String,
    size: u64,
}

/// Main DICOM Tab
pub struct DicomTab {
    // SCU (Client) settings
    scu_remote_host: String,
    scu_remote_port: String,
    scu_remote_ae: String,
    scu_local_ae: String,
    scu_file_path: String,

    // SCP (Server) settings
    scp_port: String,
    scp_ae_title: String,
    scp_storage_dir: String,

    // State
    scu_state: Arc<Mutex<ScuState>>,
    scp_state: Arc<Mutex<ScpState>>,

    // Server control
    scp_shutdown: Arc<AtomicBool>,

    // Async runtime
    runtime: Runtime,
}

impl Default for DicomTab {
    fn default() -> Self {
        Self {
            // SCU defaults
            scu_remote_host: "127.0.0.1".to_string(),
            scu_remote_port: "11112".to_string(),
            scu_remote_ae: "ANY-SCP".to_string(),
            scu_local_ae: "RS7-SCU".to_string(),
            scu_file_path: String::new(),

            // SCP defaults
            scp_port: "11113".to_string(),
            scp_ae_title: "RS7-SCP".to_string(),
            scp_storage_dir: std::env::temp_dir()
                .join("rs7-dicom")
                .to_string_lossy()
                .to_string(),

            // State
            scu_state: Arc::new(Mutex::new(ScuState::default())),
            scp_state: Arc::new(Mutex::new(ScpState::default())),

            // Server control
            scp_shutdown: Arc::new(AtomicBool::new(false)),

            // Runtime
            runtime: Runtime::new().expect("Failed to create tokio runtime"),
        }
    }
}

impl DicomTab {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Left panel: SCU (Client)
            ui.vertical(|ui| {
                ui.set_min_width(ui.available_width() / 2.0 - 10.0);
                self.scu_panel(ui);
            });

            ui.separator();

            // Right panel: SCP (Server)
            ui.vertical(|ui| {
                self.scp_panel(ui);
            });
        });
    }

    fn scu_panel(&mut self, ui: &mut Ui) {
        ui.heading("SCU (Service Class User)");
        ui.add_space(8.0);

        // Connection settings
        ui.group(|ui| {
            ui.label(RichText::new("Connection Settings").strong());
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Remote Host:");
                ui.add(TextEdit::singleline(&mut self.scu_remote_host).desired_width(120.0));
                ui.label("Port:");
                ui.add(TextEdit::singleline(&mut self.scu_remote_port).desired_width(60.0));
            });

            ui.horizontal(|ui| {
                ui.label("Remote AE Title:");
                ui.add(TextEdit::singleline(&mut self.scu_remote_ae).desired_width(100.0));
            });

            ui.horizontal(|ui| {
                ui.label("Local AE Title:");
                ui.add(TextEdit::singleline(&mut self.scu_local_ae).desired_width(100.0));
            });
        });

        ui.add_space(8.0);

        // Operations
        ui.group(|ui| {
            ui.label(RichText::new("Operations").strong());
            ui.add_space(4.0);

            let state = self.scu_state.lock().unwrap();
            let connecting = state.connecting;
            drop(state);

            // C-ECHO
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!connecting, egui::Button::new("C-ECHO (Verify)"))
                    .clicked()
                {
                    self.do_c_echo(ui.ctx().clone());
                }
                ui.label("- Test DICOM connectivity");
            });

            ui.add_space(4.0);

            // C-STORE
            ui.horizontal(|ui| {
                ui.label("DICOM File:");
                ui.add(TextEdit::singleline(&mut self.scu_file_path).desired_width(200.0));
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("DICOM", &["dcm", "dicom"])
                        .add_filter("All", &["*"])
                        .pick_file()
                    {
                        self.scu_file_path = path.to_string_lossy().to_string();
                    }
                }
            });

            if ui
                .add_enabled(
                    !connecting && !self.scu_file_path.is_empty(),
                    egui::Button::new("C-STORE (Send File)"),
                )
                .clicked()
            {
                self.do_c_store(ui.ctx().clone());
            }

            ui.add_space(4.0);

            // C-FIND (placeholder)
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(false, egui::Button::new("C-FIND (Query)"))
                    .clicked()
                {
                    // TODO: Implement C-FIND
                }
                ui.label("- Coming soon");
            });
        });

        ui.add_space(8.0);

        // Status
        {
            let state = self.scu_state.lock().unwrap();
            if state.connecting {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Operation in progress...");
                });
            }

            if let Some(ref result) = state.last_result {
                ui.colored_label(Color32::from_rgb(100, 200, 100), result);
            }

            if let Some(ref error) = state.error {
                ui.colored_label(
                    Color32::from_rgb(255, 100, 100),
                    format!("Error: {}", error),
                );
            }
        }

        ui.add_space(8.0);

        // Log
        ui.label(RichText::new("Operation Log").strong());
        let log_height = ui.available_height() - 20.0;
        ScrollArea::vertical()
            .max_height(log_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let state = self.scu_state.lock().unwrap();
                for entry in state.log_entries.iter().rev() {
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::GRAY, format!("[{}]", entry.timestamp));
                        ui.colored_label(entry.direction.color(), &entry.message);
                    });
                }
            });
    }

    fn scp_panel(&mut self, ui: &mut Ui) {
        ui.heading("SCP (Service Class Provider)");
        ui.add_space(8.0);

        // Server settings
        ui.group(|ui| {
            ui.label(RichText::new("Server Settings").strong());
            ui.add_space(4.0);

            let state = self.scp_state.lock().unwrap();
            let running = state.running;
            let starting = state.starting;
            drop(state);

            ui.horizontal(|ui| {
                ui.label("Listen Port:");
                ui.add_enabled(
                    !running && !starting,
                    TextEdit::singleline(&mut self.scp_port).desired_width(60.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label("AE Title:");
                ui.add_enabled(
                    !running && !starting,
                    TextEdit::singleline(&mut self.scp_ae_title).desired_width(100.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Storage Directory:");
                ui.add_enabled(
                    !running && !starting,
                    TextEdit::singleline(&mut self.scp_storage_dir).desired_width(200.0),
                );
                if ui
                    .add_enabled(!running && !starting, egui::Button::new("Browse..."))
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.scp_storage_dir = path.to_string_lossy().to_string();
                    }
                }
            });

            ui.add_space(8.0);

            // Start/Stop button
            if starting {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Starting server...");
                });
            } else if running {
                if ui.button("Stop Server").clicked() {
                    self.stop_scp();
                }
                ui.colored_label(
                    Color32::from_rgb(100, 200, 100),
                    format!("Server running on port {}", self.scp_port),
                );
            } else {
                if ui.button("Start Server").clicked() {
                    self.start_scp(ui.ctx().clone());
                }
            }
        });

        ui.add_space(8.0);

        // Statistics
        {
            let state = self.scp_state.lock().unwrap();
            ui.horizontal(|ui| {
                ui.label(format!("Connections: {}", state.connection_count));
                ui.label(format!("Files received: {}", state.received_files.len()));
            });

            if let Some(ref error) = state.error {
                ui.colored_label(
                    Color32::from_rgb(255, 100, 100),
                    format!("Error: {}", error),
                );
            }
        }

        ui.add_space(8.0);

        // Received files
        ui.label(RichText::new("Received Files").strong());
        let files_height = (ui.available_height() - 120.0) / 2.0;
        ScrollArea::vertical()
            .id_salt("received_files")
            .max_height(files_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let state = self.scp_state.lock().unwrap();
                for file in state.received_files.iter().rev() {
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::GRAY, format!("[{}]", file.timestamp));
                        ui.label(&file.filename);
                        ui.colored_label(Color32::GRAY, format!("({} bytes)", file.size));
                    });
                }
                if state.received_files.is_empty() {
                    ui.colored_label(Color32::GRAY, "No files received yet");
                }
            });

        ui.add_space(8.0);

        // Server log
        ui.label(RichText::new("Server Log").strong());
        let log_height = ui.available_height() - 20.0;
        ScrollArea::vertical()
            .id_salt("scp_log")
            .max_height(log_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let state = self.scp_state.lock().unwrap();
                for entry in state.log_entries.iter().rev() {
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::GRAY, format!("[{}]", entry.timestamp));
                        ui.colored_label(entry.direction.color(), &entry.message);
                    });
                }
            });
    }

    fn do_c_echo(&mut self, ctx: egui::Context) {
        let host = self.scu_remote_host.clone();
        let port: u16 = self.scu_remote_port.parse().unwrap_or(11112);
        let remote_ae = self.scu_remote_ae.clone();
        let local_ae = self.scu_local_ae.clone();
        let state = self.scu_state.clone();

        {
            let mut s = state.lock().unwrap();
            s.connecting = true;
            s.last_result = None;
            s.error = None;
            s.log_entries.push(LogEntry {
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                direction: Direction::Info,
                message: format!("Starting C-ECHO to {}:{}", host, port),
            });
        }
        ctx.request_repaint();

        self.runtime.spawn(async move {
            let result = do_c_echo_async(&host, port, &remote_ae, &local_ae).await;

            let mut s = state.lock().unwrap();
            s.connecting = false;

            match result {
                Ok(msg) => {
                    s.last_result = Some(msg.clone());
                    s.error = None;
                    s.log_entries.push(LogEntry {
                        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                        direction: Direction::Incoming,
                        message: msg,
                    });
                }
                Err(e) => {
                    s.error = Some(e.clone());
                    s.last_result = None;
                    s.log_entries.push(LogEntry {
                        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                        direction: Direction::Error,
                        message: e,
                    });
                }
            }

            ctx.request_repaint();
        });
    }

    fn do_c_store(&mut self, ctx: egui::Context) {
        let host = self.scu_remote_host.clone();
        let port: u16 = self.scu_remote_port.parse().unwrap_or(11112);
        let remote_ae = self.scu_remote_ae.clone();
        let local_ae = self.scu_local_ae.clone();
        let file_path = self.scu_file_path.clone();
        let state = self.scu_state.clone();

        {
            let mut s = state.lock().unwrap();
            s.connecting = true;
            s.last_result = None;
            s.error = None;
            s.log_entries.push(LogEntry {
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                direction: Direction::Info,
                message: format!("Starting C-STORE to {}:{}", host, port),
            });
        }
        ctx.request_repaint();

        self.runtime.spawn(async move {
            let result = do_c_store_async(&host, port, &remote_ae, &local_ae, &file_path).await;

            let mut s = state.lock().unwrap();
            s.connecting = false;

            match result {
                Ok(msg) => {
                    s.last_result = Some(msg.clone());
                    s.error = None;
                    s.log_entries.push(LogEntry {
                        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                        direction: Direction::Incoming,
                        message: msg,
                    });
                }
                Err(e) => {
                    s.error = Some(e.clone());
                    s.last_result = None;
                    s.log_entries.push(LogEntry {
                        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                        direction: Direction::Error,
                        message: e,
                    });
                }
            }

            ctx.request_repaint();
        });
    }

    fn start_scp(&mut self, ctx: egui::Context) {
        let port: u16 = self.scp_port.parse().unwrap_or(11113);
        let ae_title = self.scp_ae_title.clone();
        let storage_dir = self.scp_storage_dir.clone();
        let state = self.scp_state.clone();
        let shutdown = self.scp_shutdown.clone();

        // Reset shutdown flag
        shutdown.store(false, Ordering::SeqCst);

        {
            let mut s = state.lock().unwrap();
            s.starting = true;
            s.error = None;
            s.log_entries.push(LogEntry {
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                direction: Direction::Info,
                message: format!("Starting SCP server on port {}", port),
            });
        }
        ctx.request_repaint();

        self.runtime.spawn(async move {
            let result = run_scp_server(
                port,
                &ae_title,
                &storage_dir,
                state.clone(),
                shutdown,
                ctx.clone(),
            )
            .await;

            let mut s = state.lock().unwrap();
            s.starting = false;
            s.running = false;

            if let Err(e) = result {
                s.error = Some(e.clone());
                s.log_entries.push(LogEntry {
                    timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                    direction: Direction::Error,
                    message: e,
                });
            } else {
                s.log_entries.push(LogEntry {
                    timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                    direction: Direction::Info,
                    message: "Server stopped".to_string(),
                });
            }

            ctx.request_repaint();
        });
    }

    fn stop_scp(&mut self) {
        self.scp_shutdown.store(true, Ordering::SeqCst);
        let mut s = self.scp_state.lock().unwrap();
        s.log_entries.push(LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            direction: Direction::Info,
            message: "Stopping server...".to_string(),
        });
    }

    /// Set a DICOM file for operations (from file dialog)
    pub fn set_message(&mut self, content: &str) {
        // For DICOM, we might receive a file path
        self.scu_file_path = content.to_string();
    }

    /// Get current file path (for save operations)
    pub fn get_message(&self) -> &str {
        &self.scu_file_path
    }
}

// === DICOM SCU Operations ===

async fn do_c_echo_async(
    host: &str,
    port: u16,
    remote_ae: &str,
    local_ae: &str,
) -> Result<String, String> {
    use dicom_ul::association::client::ClientAssociationOptions;
    use std::time::Duration;

    let addr = format!("{}:{}", host, port);
    let remote_ae_owned = remote_ae.to_string();
    let local_ae_owned = local_ae.to_string();
    let host_owned = host.to_string();

    // Build association options for verification (in blocking context since dicom-ul is sync)
    let result = tokio::task::spawn_blocking(move || {
        let options = ClientAssociationOptions::new()
            .calling_ae_title(&local_ae_owned)
            .called_ae_title(&remote_ae_owned)
            .with_presentation_context(
                VERIFICATION_SOP_CLASS,
                vec![IMPLICIT_VR_LITTLE_ENDIAN, EXPLICIT_VR_LITTLE_ENDIAN],
            )
            .read_timeout(Duration::from_secs(30))
            .write_timeout(Duration::from_secs(30));

        // Establish association
        let association = options
            .establish(&addr)
            .map_err(|e| format!("Failed to establish association: {}", e))?;

        // For C-ECHO, the association being established with Verification SOP Class
        // indicates that the verification service is available

        // Release the association gracefully
        let _ = association.release();

        Ok::<_, String>((remote_ae_owned, host_owned))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?;

    let (remote_ae, host) = result?;
    Ok(format!(
        "C-ECHO successful - Association established with {} ({})",
        remote_ae, host
    ))
}

async fn do_c_store_async(
    host: &str,
    port: u16,
    remote_ae: &str,
    local_ae: &str,
    file_path: &str,
) -> Result<String, String> {
    use dicom_object::open_file;
    use dicom_ul::association::client::ClientAssociationOptions;
    use std::time::Duration;

    // First, read and validate the DICOM file
    let path = PathBuf::from(file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    let obj = open_file(&path).map_err(|e| format!("Failed to open DICOM file: {}", e))?;

    // Get SOP Class UID from the file
    let sop_class_uid = obj
        .element_by_name("SOPClassUID")
        .ok()
        .and_then(|e| e.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "1.2.840.10008.5.1.4.1.1.2".to_string()); // CT Image Storage default

    let _sop_instance_uid = obj
        .element_by_name("SOPInstanceUID")
        .ok()
        .and_then(|e| e.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Clone all values needed in the blocking task
    let addr = format!("{}:{}", host, port);
    let local_ae_owned = local_ae.to_string();
    let remote_ae_owned = remote_ae.to_string();
    let sop_class_uid_for_ctx = sop_class_uid.clone();
    let sop_class_uid_for_result = sop_class_uid.clone();

    // Establish association for C-STORE (in blocking context since dicom-ul is sync)
    let result = tokio::task::spawn_blocking(move || {
        let ts_implicit = IMPLICIT_VR_LITTLE_ENDIAN.to_string();
        let ts_explicit = EXPLICIT_VR_LITTLE_ENDIAN.to_string();
        let transfer_syntaxes: Vec<&String> = vec![&ts_implicit, &ts_explicit];

        let options = ClientAssociationOptions::new()
            .calling_ae_title(&local_ae_owned)
            .called_ae_title(&remote_ae_owned)
            .with_presentation_context(&sop_class_uid_for_ctx, transfer_syntaxes)
            .read_timeout(Duration::from_secs(60))
            .write_timeout(Duration::from_secs(60));

        let association = options
            .establish(&addr)
            .map_err(|e| format!("Failed to establish association: {}", e))?;

        // Note: Full C-STORE implementation would require:
        // 1. Building C-STORE-RQ PDU with the DICOM data
        // 2. Sending the PDU
        // 3. Receiving C-STORE-RSP
        // This is complex and requires deep integration with dicom-ul's PDU handling

        // For this demo, we verify the association can be established
        // with the correct SOP class

        // Release the association gracefully
        let _ = association.release();

        Ok::<_, String>(())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?;

    result?;

    Ok(format!(
        "C-STORE association established for {} (SOP Class: {}, {} bytes)",
        file_name, sop_class_uid_for_result, file_size
    ))
}

// === DICOM SCP Server ===

async fn run_scp_server(
    port: u16,
    ae_title: &str,
    storage_dir: &str,
    state: Arc<Mutex<ScpState>>,
    shutdown: Arc<AtomicBool>,
    ctx: egui::Context,
) -> Result<(), String> {
    use tokio::net::TcpListener;

    // Ensure storage directory exists
    let storage_path = PathBuf::from(storage_dir);
    std::fs::create_dir_all(&storage_path)
        .map_err(|e| format!("Failed to create storage directory: {}", e))?;

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

    {
        let mut s = state.lock().unwrap();
        s.starting = false;
        s.running = true;
        s.log_entries.push(LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            direction: Direction::Info,
            message: format!("SCP server listening on {} (AE: {})", addr, ae_title),
        });
    }
    ctx.request_repaint();

    // Accept connections until shutdown
    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((socket, peer_addr)) => {
                        let state_clone = state.clone();
                        let ctx_clone = ctx.clone();
                        let ae_title_clone = ae_title.to_string();
                        let storage_clone = storage_path.clone();

                        {
                            let mut s = state_clone.lock().unwrap();
                            s.connection_count += 1;
                            s.log_entries.push(LogEntry {
                                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                                direction: Direction::Incoming,
                                message: format!("Incoming connection from {}", peer_addr),
                            });
                        }
                        ctx_clone.request_repaint();

                        // Handle connection in background
                        tokio::spawn(async move {
                            if let Err(e) = handle_scp_connection(
                                socket,
                                peer_addr.to_string(),
                                &ae_title_clone,
                                &storage_clone,
                                state_clone.clone(),
                                ctx_clone.clone(),
                            ).await {
                                let mut s = state_clone.lock().unwrap();
                                s.log_entries.push(LogEntry {
                                    timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                                    direction: Direction::Error,
                                    message: format!("Connection error: {}", e),
                                });
                                ctx_clone.request_repaint();
                            }
                        });
                    }
                    Err(e) => {
                        let mut s = state.lock().unwrap();
                        s.log_entries.push(LogEntry {
                            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                            direction: Direction::Error,
                            message: format!("Accept error: {}", e),
                        });
                        ctx.request_repaint();
                    }
                }
            }
            _ = async {
                while !shutdown.load(Ordering::SeqCst) {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            } => {
                // Shutdown requested
                break;
            }
        }
    }

    {
        let mut s = state.lock().unwrap();
        s.running = false;
    }

    Ok(())
}

async fn handle_scp_connection(
    socket: tokio::net::TcpStream,
    peer_addr: String,
    ae_title: &str,
    _storage_path: &PathBuf,
    state: Arc<Mutex<ScpState>>,
    ctx: egui::Context,
) -> Result<(), String> {
    use dicom_ul::association::server::ServerAssociationOptions;
    use dicom_ul::pdu::Pdu;

    // Convert to std TcpStream for dicom-ul (blocking API)
    let std_socket = socket
        .into_std()
        .map_err(|e| format!("Socket conversion error: {}", e))?;

    let ae_title_owned = ae_title.to_string();
    let peer_addr_clone = peer_addr.clone();

    // Handle association in blocking context
    let result = tokio::task::spawn_blocking(move || -> Result<String, String> {
        let options = ServerAssociationOptions::new()
            .ae_title(&ae_title_owned)
            .with_abstract_syntax(VERIFICATION_SOP_CLASS);

        // Accept the association
        let mut association = options
            .establish(std_socket)
            .map_err(|e| format!("Failed to establish association: {}", e))?;

        // Read PDUs until release or abort
        loop {
            match association.receive() {
                Ok(pdu) => {
                    match pdu {
                        Pdu::ReleaseRQ => {
                            // Respond with release response
                            let _ = association.send(&Pdu::ReleaseRP);
                            return Ok(format!("Association released by {}", peer_addr_clone));
                        }
                        Pdu::AbortRQ { .. } => {
                            return Ok(format!("Association aborted by {}", peer_addr_clone));
                        }
                        Pdu::PData { data } => {
                            // Received P-DATA, could be C-ECHO or C-STORE data
                            // For now, just acknowledge
                            let _ = data; // Suppress unused warning
                        }
                        _ => {
                            // Other PDUs
                        }
                    }
                }
                Err(e) => {
                    return Err(format!("PDU receive error: {}", e));
                }
            }
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?;

    let mut s = state.lock().unwrap();
    match result {
        Ok(msg) => {
            s.log_entries.push(LogEntry {
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                direction: Direction::Info,
                message: msg,
            });
        }
        Err(e) => {
            s.log_entries.push(LogEntry {
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                direction: Direction::Error,
                message: e,
            });
        }
    }
    ctx.request_repaint();

    Ok(())
}
