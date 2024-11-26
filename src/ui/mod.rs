pub mod backup;
pub mod analytics;
pub mod info;

use eframe::egui;
use std::fs;
use toml;
use serde::Serialize;

/// Enum to define the types of panels in the UI
#[derive(Default, Serialize)]
pub enum PanelType {
    #[default]
    Backup,
    Analytics,
    Info,
}

// Application state, including the selected panel and configuration
#[derive(Serialize)]
pub struct AppState {
    current_panel: PanelType,
    source_folder: String,
    destination_folder: String,
    backup_type: String,
    file_types: Vec<String>,
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
            source_folder: parsed.get("source_folder").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            destination_folder: parsed.get("destination_folder").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            backup_type: parsed.get("backup_type").and_then(|v| v.as_str()).unwrap_or("total").to_string(),
            file_types: parsed.get("file_types").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
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
    egui::CentralPanel::default().show(ctx, |ui| {
        match state.current_panel {
            PanelType::Backup => backup::show_backup_panel(ui, state),
            PanelType::Analytics => analytics::show_analytics_panel(ui),
            PanelType::Info => info::show_info_panel(ui),
        }
    });
}
