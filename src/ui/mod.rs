pub mod backup;
pub mod analytics;
pub mod info;

use eframe::egui;
use serde::Serialize;

use std::{process, sync::{Arc, Mutex}};

use crate::utils::Configuration;

/// Enum to define the types of panels in the UI
#[derive(Default, Serialize)]
pub enum PanelType {
    #[default]
    Backup,
    Analytics,
    Info,
}

#[derive(Serialize, Clone)]
pub enum ErrorSource {
    FileTypeValidation,
    SaveOperation,
}

#[derive(Serialize, PartialEq, Eq, Clone)]
pub enum BackupStatus {
    NotStarted,
    InProgress,
    CompletedSuccess,
    CompletedError(String),
}

// Application state, including the selected panel and configuration
#[derive(Serialize)]
pub struct AppState {
    current_panel: PanelType,
    source_folder: String,
    destination_folder: String,
    backup_type: String,
    file_types: Vec<String>,
    new_file_type: String,       // for the backup panel
    error_message: Option<String>,           // for error messages
    error_source: Option<ErrorSource>, // Origine dell'errore
    pub exit_message: Option<String>,
    show_error_modal: bool,            // Controllo per il modale
    pub backup_status: BackupStatus
}

pub struct MyApp {
    pub state: Arc<Mutex<AppState>>,
}

impl MyApp {
    pub fn new(state: Arc<Mutex<AppState>>) -> Self {
        MyApp { state }
    }
}

impl AppState {
    /// Crea un nuovo stato applicativo basandosi su una configurazione o sui valori di default.
    pub fn new_from_config(config: Configuration) -> Self {
        match config {
            Configuration::Build(source_folder, destination_folder, backup_type, file_types) => {
                Self {
                    current_panel: PanelType::Backup, // Pannello di default
                    source_folder,
                    destination_folder,
                    backup_type: if backup_type.is_empty() { "total".to_string() } else { backup_type },
                    file_types,
                    new_file_type: "".to_string(),
                    error_message: None,
                    error_source: None,
                    exit_message: None,
                    show_error_modal: false,
                    backup_status: BackupStatus::NotStarted,
                }
            }
            _ => Self {
                current_panel: PanelType::Backup, // Pannello di default
                source_folder: "".to_string(),
                destination_folder: "".to_string(),
                backup_type: "total".to_string(),
                file_types: vec![],
                new_file_type: "".to_string(),
                error_message: None,
                error_source: None,
                exit_message: None,
                show_error_modal: false,
                backup_status: BackupStatus::NotStarted,
            },
        }
    }
}

pub fn main_panel(ctx: &egui::Context, state: &mut AppState) {
    render_sidebar(ctx, state, state.show_error_modal);
    render_main_content(ctx, state, state.show_error_modal);

    if state.show_error_modal {
        // Renderizza il modale di errore sopra l'overlay
        render_error_modal(ctx, state);
    }
}

// Render the left sidebar menu
fn render_sidebar(ctx: &egui::Context, state: &mut AppState, disable: bool) {
    egui::SidePanel::left("left_panel")
        .resizable(false)
        .min_width(150.0)
        .show(ctx, |ui| {
            if disable{
                ui.set_enabled(false);
            }

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
}

// Render the main content area
fn render_main_content(ctx: &egui::Context, state: &mut AppState, disable: bool) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if disable{
            ui.set_enabled(false);
        }
        match state.current_panel {
            PanelType::Backup => backup::show_backup_panel(ui, state),
            PanelType::Analytics => analytics::show_analytics_panel(ui),
            PanelType::Info => info::show_info_panel(ui),
        }
    });
}

// Render the error modal
fn render_error_modal(ctx: &egui::Context, state: &mut AppState) {

    let error_type = match state.error_source {
        Some(ErrorSource::FileTypeValidation) => "File Type Error",
        Some(ErrorSource::SaveOperation) => "Save Error",
        None => "Error",
    };
    
    egui::Window::new(error_type)
        .collapsible(false)
        .resizable(false)
        .fixed_size(egui::vec2(300.0, 150.0))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0)) // Center the modal
        .show(ctx, |ui| {
            // Display the error message and its source
            if let Some(error_message) = &state.error_message {
                ui.label(format!("{error_message}"));
            }

            // Button to close the modal
            if ui.button("Close").clicked() {
                state.show_error_modal = false; // Close the modal
            }
        });
}

// Function to display the error message panel
pub fn exit_panel(ctx: &egui::Context, error_message: &str) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.label("An error occurred:");
            ui.label(error_message); // Display the error message
            if ui.button("Close").clicked() {
                // Exit the program after closing the error window
                process::exit(1); // Non-zero exit code indicates error
            }
        });
    });
}

pub fn show_backup_window(ctx: &egui::Context, state: &mut AppState) {
    // Copia il valore di backup_status in una variabile separata
    let backup_status = state.backup_status.clone();

    // Determina il titolo e il messaggio in base allo stato del backup
    let (title, message, show_return_button) = match backup_status {
        BackupStatus::InProgress => ("Backup In Progress", "The backup is currently running...", false),
        BackupStatus::CompletedSuccess => ("Backup Completed", "Backup completed successfully!", true),
        BackupStatus::CompletedError(ref err) => ("Backup Failed", err.as_str(), true),
        BackupStatus::NotStarted => return, // Non mostrare la finestra se il backup non è iniziato
    };

    // Disegna il pannello centrale
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading(title); // Titolo
            ui.separator();    // Separatore
            ui.label(message); // Messaggio principale

            // Mostra il pulsante "Return back" se necessario
            if show_return_button {
                if ui.button("Return back").clicked() {
                    // Aggiorna lo stato solo quando il pulsante viene cliccato
                    state.backup_status = BackupStatus::NotStarted;
                }
            }
        });
    });
}
