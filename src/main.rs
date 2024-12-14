mod confirm_sign;
mod detector;
mod first_sign;
mod transfer;
mod ui;
mod utils;
mod analytics;

use crate::ui::{AppState, MyApp};
use eframe::{egui, App, NativeOptions};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use ui::BackupStatus;
use utils::load_image_as_icon;
use utils::manage_configuration_file;
use utils::Configuration;
//use single_instance::SingleInstance;

fn main() -> Result<(), eframe::Error> {
    /* SINGOLA APPLICAZIONE - NON FUNZIONA*/
    /* let instance = SingleInstance::new("unique_program_identifier");
    match instance {
        Ok(instance) => {
            if !instance.is_single() {
                println!("Another instance of this program is already running.");
                return Ok(());
            }
        }
        Err(e) => {
            eprintln!("Failed to create single instance: {}", e);
            std::process::exit(1); // Esce con un codice di errore
        }
    } */

    let (tx, rx) = mpsc::channel::<String>(); // Canale per comunicazione

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
    //shared_state.lock().unwrap().exit_message = Some("Some error message".to_string()); // Set an error message for testing

    // Clona lo stato condiviso per il detector
    let detector_state = Arc::clone(&shared_state);
    // Cloniamo il trasmettitore per il detector
    let detector_tx = tx.clone(); 

    // Avvia il detector in un thread separato
    std::thread::spawn(move || {
        println!("Starting detector...");
        detector::run(detector_state, detector_tx);
    });

    let run_gui;
    //per rilasciare il lock
    {
        run_gui = shared_state.lock().unwrap().display.clone();     //viene presa dal file di configurazione
    }

    if run_gui {
        println!("GUI started for the first time.");

        let options_for_gui = options.clone();
        let my_app = MyApp::new(Arc::clone(&shared_state));
        eframe::run_native(
            "Group 24 - Backup Application",
            options_for_gui,
            Box::new(|_cc| Box::new(my_app)),
        )?;
    }

    // Se arrivi qui è perché hai chiuso la GUI
    loop {
        // rimani qui finché non arriva il comando di lanciare la GUI e la GUI non è attiva
        // recv() è un metodo che blocca il thread fino a quando un messaggio non viene ricevuto.
        let result = rx.recv();
        match result {
            Ok(msg) => {
                match msg.as_str() {
                    "showGUI" => {
                        println!("Restarting GUI...");
                        //la GUI ritorna visibile (si tratta di una nuova GUI)
                        shared_state.lock().unwrap().display = true;
                    }
                    _ => {
                        println!("Unknown message: {}", msg);
                        continue;       //ritorna a rx.recv()
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                // Imposta un messaggio di errore nell'applicazione
                shared_state.lock().unwrap().exit_message = Some("Failed to receive message. Invalid detector!".to_string());
            }
        }

        {
            //Print for debug
            //shared_state.lock().unwrap().pretty_print();
        }

        let options_for_gui = options.clone();  // Clone options here
        let my_app = MyApp::new(Arc::clone(&shared_state));
        eframe::run_native(
            "Group 24 - Backup Application",
            options_for_gui,
            Box::new(|_cc| Box::new(my_app)),
        )?;
    }
}

// The update method is the primary place where the UI is rendered and updated. It gets called continuously to refresh the UI.
impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        //ottieni backup status e show_confirmation_modal
        let backup_status;
        let show_confirmation_modal;
        let exit_message;
        //per rilasciare il lock
        {
            backup_status = self.state.lock().unwrap().backup_status.clone();
            show_confirmation_modal = self.state.lock().unwrap().show_confirmation_modal.clone();
            exit_message = self.state.lock().unwrap().exit_message.clone();
        }

        if let Some(ref error) = exit_message {
            ui::exit_panel(ctx, self, error);
            return;
        }
        // Mostra il pannello principale o la finestra del backup
        if backup_status == BackupStatus::NotStarted {
            ui::main_panel(ctx, self)
        } else {
            ui::show_backup_window(ctx, self);
        }

        // Renderizza il modale di errore sopra l'overlay: viene messo qui perché rappresenta la conferma di chiusura dell'app
        if show_confirmation_modal {
            ui::render_modal_exit(ctx, self, frame); // Renderizza il modal
        }

    }
    fn on_close_event(&mut self) -> bool {
        let mut state = self.state.lock().unwrap(); // Accedi allo stato protetto dal Mutex

        if !state.display {
            return true;
        }
        else{
            // Se c'è un errore e non è stato già mostrato il modal di conferma
            if state.show_confirmation_modal == false {
                // Imposta il flag per mostrare la conferma di chiusura
                state.show_confirmation_modal = true;
            }
            // Indica che non bisogna chiudere la finestra finché l'utente non conferma
            return false;
        } 
    }
}
