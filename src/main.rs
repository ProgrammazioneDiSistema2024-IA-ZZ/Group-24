mod ui;
mod utils;
mod detector;
mod transfer;

use eframe::{egui, App, NativeOptions};
use utils::manage_configuration_file;
use utils::Configuration;
use crate::ui::AppState;
use utils::load_image_as_icon;
use std::thread;
use std::time::Duration;

/* pub fn run_detector() {
    loop {
        println!("Detector is running...");
        thread::sleep(Duration::from_secs(5));
    }
} */

fn main() -> Result<(), eframe::Error> {
    // Ottieni la configurazione
    let config = manage_configuration_file();

    // Crea l'AppState basandoti sulla configurazione
    let app_state = match &config {
        Configuration::Error => {
            let mut state = AppState::new_from_config(Configuration::Error);
            state.exit_message = Some("Impossible to retrieve configuration file!".to_string());
            state
        }
        Configuration::Created | Configuration::Build(_, _, _, _) => {
            AppState::new_from_config(config.clone())
        }
    };

    // Load the application icon
    let icon_result = load_image_as_icon("images/icon.png");

    // Configure the application window
    let mut options = NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        resizable: false,
        ..Default::default()
    };

    //add the icon only if correctly loaded
    if let Ok(icon) = icon_result {
        options.icon_data = Some(icon);
    }

    // **** FOR TESTING, delete when you don't need anymore ****
    //app_state.exit_message = Some("Some error message".to_string()); // Set an error message for testing


    // Avvia il detector in un thread separato
    std::thread::spawn(move || {
        println!("Starting detector...");
        detector::run(); // Sostituisci con la tua logica per il detector
    });

    // Avvia la GUI come thread principale
    eframe::run_native(
        "Group 24 - Backup Application",
        options,
        Box::new(|_cc| Box::new(app_state)),
    )?; //propaga al main errori di run_native

    Ok(())
}

// The update method is the primary place where the UI is rendered and updated. It gets called continuously to refresh the UI.
impl App for AppState {
    /// Update function that draws the UI and handles events
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Show the error panel if there is an error message
        if let Some(ref error) = &self.exit_message {
            ui::exit_panel(ctx, error); // Show the error message panel
        } else {
            ui::main_panel(ctx, self); // Show the main panel if no error
        }
    }
}
