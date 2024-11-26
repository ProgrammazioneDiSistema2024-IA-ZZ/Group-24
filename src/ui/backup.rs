use eframe::egui;
use super::AppState;
use toml;

const MAX_FILE_TYPES: usize = 10;

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
    ui.horizontal(|ui| {
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
    ui.horizontal(|ui| {
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
        ui.horizontal(|ui| {
            ui.label("Insert type of file:");
            let mut new_file_type = String::new();
            ui.text_edit_singleline(&mut new_file_type);
            // Define a mutable variable to track the validation error message
            let mut error_message = String::new();

            if ui.button("+").clicked() {
                // Check if the file type is not empty and contains a valid extension
                if new_file_type.is_empty() {
                    error_message = "File type cannot be empty.".to_string();
                } else if !new_file_type.starts_with('.') {
                    // Check if the entered text starts with a dot (valid file type)
                    error_message = "File type must start with a dot (e.g., .txt, .png).".to_string();
                } else if state.file_types.len() >= MAX_FILE_TYPES {
                    // Check if the maximum number of file types has been reached
                    error_message = format!("You can only add up to {} file types.", MAX_FILE_TYPES);
                } else {
                    // If no validation errors, add the file type to the list
                    state.file_types.push(new_file_type.clone());
                }
            }

            // Show error message in red if there is an error
            if !error_message.is_empty() {
                ui.label(format!("{}", error_message));
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
            }
        });
    }

    ui.separator();

    // 5th row: Restore and Save buttons
    ui.horizontal(|ui| {
        if ui.button("Restore").clicked() {
            *state = AppState::default(); // Reload from config file
        }
        if ui.button("Save").clicked() {
            let config = toml::to_string(&state).unwrap_or_default();
            std::fs::write("config_build.toml", config).expect("Failed to save configuration");
        }
    });
}
