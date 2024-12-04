pub mod analytics;
pub mod backup_panel;
pub mod info;

use crate::backup::backup;

use eframe::egui;
use serde::Serialize;
use std::fs;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use toml;

/// Enum to define the types of panels in the UI
#[derive(Default, Serialize)]
pub enum PanelType {
    #[default]
    Backup,
    Analytics,
    Info,
}

// use std::time::{Duration, Instant};

/// Struttura per gestire i messaggi di feedback con timestamp
#[derive(Serialize)]
pub struct FeedbackMessage {
    pub message: String,
    pub timestamp: Option<u64>,    // Quando il messaggio è stato creato
    pub message_type: MessageType, // Tipo di messaggio (es. errore o successo)
}

/// Tipi di messaggi (successo, errore, informazione, ecc.)
#[derive(Serialize)]
pub enum MessageType {
    Success,
    Error,
    Information,
}

// Stato del backup
#[derive(Serialize)]
pub enum BackupStatus {
    NotStarted,
    InProgress,
    CompletedSuccess,
    CompletedError,
}

// Application state, including the selected panel and configuration
#[derive(Serialize)]
pub struct AppState {
    pub current_panel: PanelType,
    pub source_folder: String,
    pub destination_folder: String,
    pub backup_type: String,
    pub file_types: Vec<String>,
    pub feedback_messages: Vec<FeedbackMessage>,
    pub backup_status: BackupStatus,
}

impl FeedbackMessage {
    pub fn new(message: String, message_type: MessageType) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::new(0, 0))
            .as_secs(); // Converte il tempo in secondi

        FeedbackMessage {
            message,
            timestamp: Some(timestamp),
            message_type,
        }
    }
}

impl Default for AppState {
    /// Initialize the application state with default or configuration file values.
    /// If the config file is not found or cannot be read, the program terminates.
    fn default() -> Self {
        // Attempt to read the configuration file.
        let config = fs::read_to_string("config_build.toml");

        // If the file cannot be read, terminate the program with an error message.
        let config = match config {
            Ok(content) => content,
            Err(_) => {
                eprintln!("Error: Configuration file 'config_build.toml' not found or unreadable.");
                std::process::exit(1); // Exit the program with a non-zero code to indicate an error.
            }
        };

        // Parse the TOML content from the file. If the parsing fails, terminate the program.
        let parsed: toml::Value = match toml::from_str(&config) {
            Ok(value) => value,
            Err(_) => {
                eprintln!("Error: Failed to parse the configuration file.");
                std::process::exit(1); // Exit the program with a non-zero code to indicate an error.
            }
        };

        // Initialize the AppState using the values from the configuration file.
        // If specific fields are missing, use default values.
        Self {
            current_panel: PanelType::Backup, // Default panel
            source_folder: parsed
                .get("source_folder")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            destination_folder: parsed
                .get("destination_folder")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            backup_type: parsed
                .get("backup_type")
                .and_then(|v| v.as_str())
                .unwrap_or("total")
                .to_string(),
            file_types: parsed
                .get("file_types")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            feedback_messages: vec![],
            backup_status: BackupStatus::NotStarted,
        }
    }
}
impl AppState {
    pub fn add_feedback_message(&mut self, message: String, message_type: MessageType) {
        // self.feedback_messages.push(FeedbackMessage {
        //     message,
        //     timestamp: Some(Instant::now().elapsed().as_secs()), // Salva il timestamp Unix
        //     message_type,
        // });
        let new_message = FeedbackMessage::new(message, message_type);
        self.feedback_messages.push(new_message);
    }

    pub fn remove_expired_messages(&mut self, timeout: std::time::Duration) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::new(0, 0))
            .as_secs(); // Tempo corrente in secondi

        self.feedback_messages.retain(|msg| {
            if let Some(timestamp) = msg.timestamp {
                // Se il timestamp è presente, controlla se è scaduto
                now - timestamp < timeout.as_secs()
            } else {
                // Se non c'è timestamp, mantieni il messaggio
                true
            }
        });
    }

    // Mostra il pannello con il messaggio del backup
    pub fn show_backup_window(&mut self, ui: &mut egui::Ui, message: &str) {
        ui.centered_and_justified(|ui| {
            ui.label(message);
            if ui.button("Close").clicked() {
                self.backup_status = BackupStatus::NotStarted; // Torna allo stato iniziale
            }
        });
    }
    // Funzione per eseguire il backup
    pub fn start_backup(&mut self) {
        self.backup_status = BackupStatus::InProgress; // Imposta il backup come in corso

        // Chiamata alla funzione di backup
        match backup::perform_backup(self) {
            Ok(_) => {
                self.backup_status = BackupStatus::CompletedSuccess;
                self.add_feedback_message(
                    "Backup completed successfully!".to_string(),
                    MessageType::Success,
                );
            }
            Err(err) => {
                self.backup_status = BackupStatus::CompletedError;
                self.add_feedback_message(format!("Backup failed: {}", err), MessageType::Error);
            }
        }
    }
}
/// Main panel logic
pub fn main_panel(ctx: &egui::Context, state: &mut AppState) {
    // Left sidebar menu
    egui::SidePanel::left("left_panel")
        .resizable(false)
        .min_width(150.0)
        .show(ctx, |ui| {
            ui.heading("Menu");

            if ui.button("Backup Panel").clicked() {
                state.current_panel = PanelType::Backup;
            }
            if ui.button("Analytics Panel").clicked() {
                state.current_panel = PanelType::Analytics;
            }
            if ui.button("Info Panel").clicked() {
                state.current_panel = PanelType::Info;
            }
        });

    // Right-side central panel
    egui::CentralPanel::default().show(ctx, |ui| match state.current_panel {
        PanelType::Backup => backup_panel::show_backup_panel(ui, state),
        PanelType::Analytics => analytics::show_analytics_panel(ui),
        PanelType::Info => info::show_info_panel(ui),
    });
}
