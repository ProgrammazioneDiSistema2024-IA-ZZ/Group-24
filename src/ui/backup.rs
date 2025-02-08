use eframe::egui;
use serde::Serialize;
use crate::utils::manage_configuration_file;

use super::{AppState, ErrorSource, InfoSource};
use toml;

#[derive(Serialize)]
pub struct ConfigToSave {
    pub source_folder: String,
    pub destination_folder: String,
    pub backup_type: String,
    pub file_types: Vec<String>,
    pub display: bool
}

const MAX_FILE_TYPES: usize = 10;
const MAX_EXTENSION_LENGTH: usize = 6; // Including the dot

/// Display the backup panel and its related components
pub fn show_backup_panel(ui: &mut egui::Ui, state: &mut AppState) {
    // 1st row: Select source folder
    ui.horizontal(|ui| {
        ui.label("Select root source folder:");
        if ui.button("Choose").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                state.source_folder = path.to_string_lossy().to_string();
            }
        }
    });
    ui.horizontal_wrapped(|ui| {
        ui.label("Chosen:");
        ui.add(egui::Label::new(&state.source_folder).wrap(true));
    });

    ui.separator(); // Divider between rows

    // 2nd row: Select destination folder
    ui.horizontal(|ui| {
        ui.label("Select destination folder:");
        if ui.button("Choose").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                state.destination_folder = path.to_string_lossy().to_string();
            }
        }
    });
    ui.horizontal_wrapped(|ui| {
        ui.label("Chosen:");
        ui.add(egui::Label::new(&state.destination_folder).wrap(true));
    });

    ui.separator();

    // 3rd row: Select backup type
    ui.label("Which files do you want to backup?");
    ui.horizontal(|ui| {
        if ui.radio(state.backup_type == "total", "All").clicked() {
            state.backup_type = "total".to_string();
        }
        if ui.radio(state.backup_type == "custom", "Custom").clicked() {
            state.backup_type = "custom".to_string();
        }
    });

    // 4th row: File type customization (only for "custom")
    if state.backup_type == "custom" {
        ui.horizontal_wrapped(|ui| {
            ui.label("Insert type of file:");
            ui.text_edit_singleline(&mut state.new_file_type);

            if ui.button("+").clicked() {
                if state.new_file_type.is_empty() {
                    state.error_message = Some("File type cannot be empty.".to_string());
                    state.error_source = Some(ErrorSource::FileTypeValidation);
                    state.show_error_modal = true;
                } else if !state.new_file_type.starts_with('.') {
                    state.error_message = Some("File type must start with a dot (e.g., .txt, .png).".to_string());
                    state.error_source = Some(ErrorSource::FileTypeValidation);
                    state.show_error_modal = true;
                } else if state.new_file_type.len() > MAX_EXTENSION_LENGTH {
                    state.error_message = Some(format!(
                        "File type must be at most {} characters long, including the dot.",
                        MAX_EXTENSION_LENGTH
                    ));
                    state.error_source = Some(ErrorSource::FileTypeValidation);
                    state.show_error_modal = true;
                } else if !state.new_file_type[1..].chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                    state.error_message = Some("File type contains invalid characters. Only alphanumeric characters, hyphens (-), and underscores (_) are allowed.".to_string());
                    state.error_source = Some(ErrorSource::FileTypeValidation);
                    state.show_error_modal = true;
                } else if state.file_types.len() >= MAX_FILE_TYPES {
                    state.error_message = Some(format!(
                        "You can only add up to {} file types.",
                        MAX_FILE_TYPES
                    ));
                    state.error_source = Some(ErrorSource::FileTypeValidation);
                    state.show_error_modal = true;
                } else if state.file_types.contains(&state.new_file_type) {
                    state.error_message = Some("File type is already in the list.".to_string());
                    state.error_source = Some(ErrorSource::FileTypeValidation);
                    state.show_error_modal = true;
                } else {
                    // Add the file type and clear any errors
                    state.file_types.push(state.new_file_type.clone());
                    state.new_file_type.clear();
                    state.error_message = None;
                    state.error_source = None;
                    state.show_error_modal = false;
                }
            }
            
        });
        
        ui.horizontal(|ui| {
            ui.label("Selected:");
            ui.label(format!("{}/10", state.file_types.len()));
        });
        ui.horizontal_wrapped(|ui| {
            for (index, file_type) in state.file_types.clone().iter().enumerate() {

                let response = ui.add(egui::Button::new(file_type).sense(egui::Sense::click()))
                    .on_hover_ui(|ui| {
                        ui.label("Click to remove");
                    });
        
                if response.clicked() {
                    state.file_types.remove(index);
                }
            }
        });
    }

    ui.separator();

    // 5th row: Restore and Save buttons
    ui.horizontal(|ui| {
        if ui.button("Restore").clicked() {
            //siamo sicuri che se siamo qui, il file di configurazione è valido, i controlli importanti sono stati fattinel main
            let config = manage_configuration_file();
            *state = AppState::new_from_config(config); // Reload from config file
        }
        if ui.button("Save").clicked() {

            // Verifica che i percorsi a livello di stringa non siano vuoti
            if state.source_folder.is_empty() || state.destination_folder.is_empty() {
                state.error_message = Some("Source or destination folder path cannot be empty.".to_string());
                state.error_source = Some(ErrorSource::SaveOperation);
                state.show_error_modal = true;
                return;
            }

            // Verifica che le due cartelle siano diverse
            if state.source_folder == state.destination_folder {
                state.error_message = Some("Source and destination folders cannot be the same.".to_string());
                state.error_source = Some(ErrorSource::SaveOperation);
                state.show_error_modal = true;
                return;
            }

            // Controlla se la cartella `source_folder` è vuota
            if let Ok(entries) = std::fs::read_dir(&state.source_folder) {
                if entries.count() == 0 {
                    state.error_message = Some("Source folder is empty. Please ensure it contains files to back up.".to_string());
                    state.error_source = Some(ErrorSource::SaveOperation);
                    state.show_error_modal = true;
                    return;
                }
            } else {
                state.error_message = Some("Failed to read source folder. Ensure it is accessible.".to_string());
                state.error_source = Some(ErrorSource::SaveOperation);
                state.show_error_modal = true;
                return;
            }

            // Controlla se la cartella `destination_folder` contiene già dei file
            if let Ok(entries) = std::fs::read_dir(&state.destination_folder) {
                if entries.count() > 0 {
                    state.info_message = Some("Destination folder is not empty. Existing files may be overwritten.".to_string());
                    state.info_source = Some(InfoSource::Attention);
                    state.show_info_modal = true;
                    // Puoi decidere se interrompere qui o continuare con un messaggio informativo
                    return;
                }
            }

            save_folders(state);
        }
    });
    
}

pub fn save_folders(state: &mut AppState){
    // Se il backup scelto è di tipo "total" allora ripulisci file_types
    if state.backup_type == "total" {
        state.file_types.clear();
    }

    // Crea una versione semplificata con i campi che vogliamo serializzare
    let config_to_save = ConfigToSave {
        source_folder: state.source_folder.clone(),
        destination_folder: state.destination_folder.clone(),
        backup_type: state.backup_type.clone(),
        file_types: state.file_types.clone(),
        display: state.run_gui
    };

    // Prova a serializzare lo stato in formato TOML e a salvare il file
    match toml::to_string(&config_to_save) {
        Ok(config) => {
            if let Err(e) = std::fs::write("config_build.toml", config) {
                state.error_message = Some(format!("Failed to save configuration: {}", e));
                state.error_source = Some(ErrorSource::SaveOperation);
                state.show_error_modal = true;
            } else {
                state.info_message = Some("Configuration saved successfully.".to_string());
                state.info_source = Some(InfoSource::Success);
                state.show_info_modal=true;
                state.error_message = None; // Nessun errore, cancella eventuali messaggi precedenti
                state.error_source = None;
                state.show_error_modal = false;
            }
        }
        Err(e) => {
            state.error_message = Some(format!("Serialization error: {}", e));
            state.error_source = Some(ErrorSource::SaveOperation);
            state.show_error_modal = true;
        }
    }
}