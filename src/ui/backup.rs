use eframe::egui;
use crate::utils::manage_configuration_file;

use super::{AppState, ErrorSource};
use toml;

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
                if ui.button(file_type).clicked() {
                    state.file_types.remove(index);
                }


                /* // Carica l'immagine del cestino come texture
                let delete_icon = load_image_as_egui_texture(ui.ctx(), "images/delete.png");

                // Configura il bottone con testo e icona
                let button_content = ui.horizontal(|ui| {
                    ui.label(file_type); // Testo dell'estensione
                    if let Some(icon) = &delete_icon {
                        ui.image(icon.id(), [16.0, 16.0]); // Aggiunge l'icona accanto al testo
                    }
                });

                // Disegna il pulsante e gestisce il click
                if ui
                    .add(egui::Button::new(button_content.response))
                    .on_hover_text("Click to remove") // Tooltip
                    .clicked()
                {
                    state.file_types.remove(index); // Rimuove l'estensione
                } */
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
            // Prova a serializzare lo stato in formato TOML e a salvare il file
            match toml::to_string(&state) {
                Ok(config) => {
                    if let Err(e) = std::fs::write("config_build.toml", config) {
                        state.error_message = Some(format!("Failed to save configuration: {}", e));
                        state.error_source = Some(ErrorSource::SaveOperation);
                        state.show_error_modal = true;
                    } else {
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
    });

    //Stato dell'applicazione
    
}
