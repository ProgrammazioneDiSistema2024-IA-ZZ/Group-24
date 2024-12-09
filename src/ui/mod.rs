pub mod analytics;
pub mod backup;
pub mod info;

use backup::save_folders;
use eframe::egui::{self, Color32};
use serde::Serialize;

use std::{
    process,
    sync::{Arc, Mutex},
};

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
#[derive(Serialize, Clone)]
pub enum InfoSource {
    Success,
    Attention,
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
    new_file_type: String,        // for the backup panel
    info_message: Option<String>, // for information
    info_source: Option<InfoSource>,
    show_info_modal: bool,
    pub error_message: Option<String>,     // for error messages
    pub error_source: Option<ErrorSource>, // Origine dell'errore
    pub exit_message: Option<String>,
    show_error_modal: bool, // Controllo per il modale
    pub show_confirmation_modal: bool,
    pub backup_status: BackupStatus,
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
                    backup_type: if backup_type.is_empty() {
                        "total".to_string()
                    } else {
                        backup_type
                    },
                    file_types,
                    new_file_type: "".to_string(),
                    info_message: None,
                    info_source: None,
                    show_error_modal: false,
                    error_message: None,
                    error_source: None,
                    exit_message: None,
                    show_info_modal: false,
                    show_confirmation_modal: false,
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
                info_message: None,
                info_source: None,
                show_info_modal: false,
                error_message: None,
                error_source: None,
                exit_message: None,
                show_error_modal: false,
                show_confirmation_modal: false,
                backup_status: BackupStatus::NotStarted,
            },
        }
    }
}

pub fn main_panel(ctx: &egui::Context, state: &mut AppState, frame: &mut eframe::Frame) {
    render_sidebar(ctx, state, state.show_error_modal);
    render_main_content(ctx, state, state.show_error_modal, frame);

    if state.show_error_modal {
        // Renderizza il modale di errore sopra l'overlay
        render_error_modal(ctx, state);
    }
    if state.show_info_modal {
        render_success_modal(ctx, state);
    }
}

// Render the left sidebar menu
fn render_sidebar(ctx: &egui::Context, state: &mut AppState, disable: bool) {
    egui::SidePanel::left("left_panel")
        .resizable(false)
        .min_width(150.0)
        .show(ctx, |ui| {
            if disable {
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
fn render_main_content(
    ctx: &egui::Context,
    state: &mut AppState,
    disable: bool,
    frame: &mut eframe::Frame,
) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if disable {
            ui.set_enabled(false);
        }
        match state.current_panel {
            PanelType::Backup => backup::show_backup_panel(ui, state, frame),
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

fn render_success_modal(ctx: &egui::Context, state: &mut AppState) {
    // Check if it's an attention message
    if let Some(InfoSource::Attention) = &state.info_source {
        // Create an attention window
        egui::Window::new("Attention")
            .collapsible(false)
            .resizable(false)
            .fixed_size(egui::vec2(400.0, 200.0))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0)) // Center the window
            .show(ctx, |ui| {
                // Display the attention message
                if let Some(info_message) = &state.info_message {
                    ui.label(info_message); // Show the informational message
                }

                // Add the buttons for the user's decision
                ui.horizontal(|ui| {
                    if ui.button("Return to configuration").clicked() {
                        // Set the flag to return to configuration
                        state.show_info_modal = false; // Close the modal
                        state.info_message = None; // Clear the message
                        state.info_source = None; // Reset the informational state
                                                  // You can add actions here to restore the configuration state if needed
                    }

                    if ui.button("Confirm and continue").clicked() {
                        // If the user confirms, proceed with the operation (e.g., saving)
                        state.show_info_modal = false; // Close the modal
                        save_folders(state);
                        // Start the save process or proceed with the action
                    }
                });
            });
    }

    // If it's a success message, just show the success message
    if let Some(InfoSource::Success) = &state.info_source {
        egui::Window::new("Success")
            .collapsible(false)
            .resizable(false)
            .fixed_size(egui::vec2(300.0, 150.0))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0)) // Center the window
            .show(ctx, |ui| {
                if let Some(success_message) = &state.info_message {
                    ui.label(success_message); // Show the success message
                }

                // Button to close the modal
                if ui.button("Close").clicked() {
                    state.show_info_modal = false; // Close the modal
                    state.info_message = None; // Clear the message
                    state.info_source = None; // Reset the informational state
                }
            });
    }
}

pub fn render_modal_exit(ctx: &egui::Context, state: &mut AppState) {
    if state.show_confirmation_modal {
        // Mostra il modal di conferma
        egui::Window::new("Attention")
            .collapsible(false)
            .resizable(false)
            .fixed_size(egui::vec2(400.0, 200.0))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                // Messaggio di conferma

                ui.label("Are you sure you want to close the program?");
                ui.label("Click 'Yes' if you no longer want to use the backup service.");
                ui.label("Click 'No' if you wish to continue using it.");
                ui.colored_label(Color32::YELLOW,"(Tip) If you just wanted to close the application, click 'Minimize to background.'");

                // Pulsanti "No" e "Sì"
                ui.horizontal(|ui| {
                    if ui.button("No").clicked() {
                        state.show_confirmation_modal = false; // Torna alla configurazione
                    }
                    if ui.button("Yes").clicked() {
                        std::process::exit(0); // Chiudi l'app
                    }
                });
            });
    }
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
        BackupStatus::InProgress => (
            "Backup In Progress",
            "The backup is currently running...",
            false,
        ),
        BackupStatus::CompletedSuccess => {
            ("Backup Completed", "Backup completed successfully!", true)
        }
        BackupStatus::CompletedError(ref err) => ("Backup Failed", err.as_str(), true),
        BackupStatus::NotStarted => return, // Non mostrare la finestra se il backup non è iniziato
    };

    // Disegna il pannello centrale
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading(title); // Titolo
            ui.separator(); // Separatore
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
