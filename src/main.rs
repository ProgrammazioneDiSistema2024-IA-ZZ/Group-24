mod ui;
mod utils;
mod detector;
mod transfer;

use eframe::{egui, App, NativeOptions};
use ui::BackupStatus;
use utils::manage_configuration_file;
use utils::Configuration;
use crate::ui::{AppState, MyApp};
use utils::load_image_as_icon;
use std::sync::{Arc, Mutex};


fn main() -> Result<(), eframe::Error> {
    // Ottieni la configurazione
    let config = manage_configuration_file();

    // Crea un `Arc<Mutex<AppState>>` condiviso
    let shared_state = Arc::new(Mutex::new(match &config {
        Configuration::Error => {
            let mut state = AppState::new_from_config(Configuration::Error);
            state.exit_message = Some("Impossible to retrieve configuration file!".to_string());
            state
        }
        Configuration::Created | Configuration::Build(_, _, _, _) => {
            AppState::new_from_config(config.clone())
        }
    }));

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

    // Clona lo stato condiviso per il detector
    let detector_state = Arc::clone(&shared_state);

    // Avvia il detector in un thread separato
    std::thread::spawn(move || {
        println!("Starting detector...");
        detector::run(detector_state); // Sostituisci con la tua logica per il detector
    });

    // Avvia la GUI come thread principale
    let my_app = MyApp::new(Arc::clone(&shared_state));
    eframe::run_native(
        "Group 24 - Backup Application",
        options,
        Box::new(|_cc| Box::new(my_app)),
    )?; //propaga al main errori di run_native

    Ok(())
}

// The update method is the primary place where the UI is rendered and updated. It gets called continuously to refresh the UI.
impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut state = self.state.lock().unwrap(); // Accedi al Mutex

        // Mostra il pannello principale o la finestra del backup
        if state.backup_status == BackupStatus::NotStarted {
            if let Some(ref error) = &state.exit_message {
                ui::exit_panel(ctx, error);
            } else {
                ui::main_panel(ctx, &mut *state);
            }
        } else {
            ui::show_backup_window(ctx, &mut *state);
        }
    }
}
