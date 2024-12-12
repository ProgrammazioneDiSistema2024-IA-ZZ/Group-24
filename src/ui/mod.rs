pub mod analytics;
pub mod backup;
pub mod info;

use backup::save_folders;
use eframe::egui::{self, Color32};
use serde::Serialize;

use std::{
    process,
    sync::{Arc, Mutex}, thread, time::Duration,
};

use crate::utils::Configuration;

/// Enum to define the types of panels in the UI
#[derive(Default, Serialize, PartialEq, Debug)]
pub enum PanelType {
    #[default]
    Backup,
    Analytics,
    Info,
}

#[derive(Serialize, Clone, Debug)]
pub enum ErrorSource {
    FileTypeValidation,
    SaveOperation,
}
#[derive(Serialize, Clone, Debug)]
pub enum InfoSource {
    Success,
    Attention,
}

#[derive(Serialize, PartialEq, Eq, Clone, Debug)]
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
    pub show_confirmation_modal: bool,      // utilizzato quando si vuole chiudere l'applicazione
    pub display: bool,        // permette di chiudere la GUI, senza terminare l'intero programma. Viene presa dal file di configurazione per una prima installazione
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
                    display: true,
                    backup_status: BackupStatus::NotStarted
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
                display: true,
                backup_status: BackupStatus::NotStarted
            },
        }
    }

    pub fn _pretty_print(&self) {
        println!("AppState {{");
        println!("  current_panel: {:?}", self.current_panel);
        println!("  source_folder: {}", self.source_folder);
        println!("  destination_folder: {}", self.destination_folder);
        println!("  backup_type: {}", self.backup_type);
        println!("  file_types: {:?}", self.file_types);
        println!("  new_file_type: {}", self.new_file_type);
        println!("  info_message: {:?}", self.info_message);
        println!("  info_source: {:?}", self.info_source);
        println!("  show_info_modal: {}", self.show_info_modal);
        println!("  error_message: {:?}", self.error_message);
        println!("  error_source: {:?}", self.error_source);
        println!("  exit_message: {:?}", self.exit_message);
        println!("  show_error_modal: {}", self.show_error_modal);
        println!("  show_confirmation_modal: {}", self.show_confirmation_modal);
        println!("  display: {}", self.display);
        println!("  backup_status: {:?}", self.backup_status);
        println!("}}");
    }
}

pub fn main_panel(ctx: &egui::Context, state: &mut MyApp) {
    let show_error_modal;
    let show_info_modal;
    //per rilasciare il lock
    {
        show_error_modal = state.state.lock().unwrap().show_error_modal.clone();
        show_info_modal = state.state.lock().unwrap().show_info_modal.clone();
    }

    let mut state = state.state.lock().unwrap(); // Accedi al Mutex

    render_sidebar(ctx, &mut *state);
    render_main_content(ctx, &mut *state);

    if show_error_modal {
        // Renderizza il modale di errore sopra l'overlay
        render_error_modal(ctx, &mut *state);
    }
    if show_info_modal {
        render_success_modal(ctx, &mut *state);
    }
}

fn render_sidebar(ctx: &egui::Context, state: &mut AppState) {
    egui::SidePanel::left("left_panel")
        .resizable(false)
        .min_width(150.0)
        .show(ctx, |ui| {
            // Disable interactions if modals are displayed
            if state.show_confirmation_modal || state.show_error_modal || state.show_info_modal {
                ui.set_enabled(false);
            }

            // Menu header
            ui.heading("Menu");
            
            // Render menu as a list of links
            ui.vertical(|ui| {
                if ui.selectable_label(state.current_panel == PanelType::Backup, "Backup Panel").clicked() {
                    state.current_panel = PanelType::Backup;
                }
                if ui.selectable_label(state.current_panel == PanelType::Analytics, "Analytics Panel").clicked() {
                    state.current_panel = PanelType::Analytics;
                }
                if ui.selectable_label(state.current_panel == PanelType::Info, "Info Panel").clicked() {
                    state.current_panel = PanelType::Info;
                }
            });

            // Spacer to push "Stop" to the bottom
            ui.add_space(ui.available_height() - 50.0);

            // Render the "Stop" button at the bottom, visually separated from the menu
            if ui.button("Stop").clicked() {
                println!("End of the program");
                process::exit(0); // Terminate the application: halts execution and bypasses Rust's usual stack unwinding mechanism.
            }
        });
}

// Render the main content area
fn render_main_content(
    ctx: &egui::Context,
    state: &mut AppState
) {

    egui::CentralPanel::default().show(ctx, |ui| {
        if state.show_confirmation_modal || state.show_error_modal || state.show_info_modal {
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

pub fn render_modal_exit(ctx: &egui::Context, state: &mut MyApp, frame: &mut eframe::Frame) {
    let show_confirmation_modal;
    //per rilasciare il lock
    {
        show_confirmation_modal = state.state.lock().unwrap().show_confirmation_modal.clone(); // Accedi al Mutex
    } 

    if show_confirmation_modal {
        // Mostra il modal di conferma
        egui::Window::new("Attention")
            .collapsible(false)
            .resizable(false)
            .fixed_size(egui::vec2(400.0, 200.0))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                // Messaggio di conferma
                ui.label("Are you sure you want to quit the user panel?");
                ui.label("Click 'No' if you wish to continue using it.");
                ui.label("Click 'Yes' if you want to close the user panel.");
                ui.colored_label(Color32::LIGHT_RED,"(Tip) If you want to stop the backup service, click 'Terminate'");

                // Pulsanti
                ui.horizontal(|ui| {
                    let mut app_state = state.state.lock().unwrap();
                    if ui.button("No").clicked() {
                        app_state.show_confirmation_modal = false; // Close the modal
                    }
                    if ui.button("Yes").clicked() {
                        app_state.show_confirmation_modal = false; // Close the modal */

                        app_state.display = false; // User chose to hide the GUI
                        //attendi qualche millisecondo prima di chiudere
                        thread::sleep(Duration::from_millis(100));
                        frame.close(); // Handle close event
                    }
                    
                });
            });
    }
}

// Function to display the error message panel
pub fn exit_panel(ctx: &egui::Context, state: &MyApp, error_message: &str) {
    let show_confirmation_modal;
    //per rilasciare il lock
    {
        show_confirmation_modal = state.state.lock().unwrap().show_confirmation_modal.clone();
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            if show_confirmation_modal {
                ui.set_enabled(false);
            }

            ui.label("An error occurred:");
            ui.label(error_message); // Display the error message
            if ui.button("Close").clicked() {
                // Exit the program after closing the error window
                process::exit(1); // Non-zero exit code indicates error
            }
        });
    });
}

pub fn show_backup_window(ctx: &egui::Context, state: &mut MyApp) {
    let mut state = state.state.lock().unwrap();
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
            if state.show_confirmation_modal  {
                ui.set_enabled(false);
            }

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
