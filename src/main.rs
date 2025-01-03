// Questo evita che il terminale si apra su Windows e non influisce su altri sistemi operativi.
// ------- ATTIVA QUANDO BUILD COMPLETA -------
//#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod analytics;
mod confirm_sign;
mod detector;
mod first_sign;
mod transfer;
mod ui;
mod utils;

use crate::ui::{AppState, MyApp};
use analytics::log_cpu_usage_to_csv;
use eframe::{egui, App, NativeOptions};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::{fs, process, thread};
use ui::BackupStatus;
use utils::load_image_as_icon;
use utils::manage_configuration_file;
use utils::Configuration;

//single-application
use signal_hook::consts::signal;
#[cfg(windows)]
use signal_hook::flag;
#[cfg(not(windows))]
use signal_hook::iterator::Signals; //da verificare

static LOCK_FILE_PATH: &str = ".app.lock"; // Il file di lock che indica se un'istanza è in esecuzione

// Funzione che rimuove il file di lock
fn remove_lock_file() {
    if let Err(e) = fs::remove_file(LOCK_FILE_PATH) {
        eprintln!("Failed to remove lock file: {}", e);
    }
    remove_cpu_log_file();
}

/// Rimuove il file `cpu_usage_log.csv` se esiste
fn remove_cpu_log_file() {
    let file_path = "cpu_usage_log.csv";
    if let Err(e) = fs::remove_file(file_path) {
        eprintln!("Failed to remove lock file: {}", e);
    }
}


fn main() -> Result<(), eframe::Error> {

    remove_cpu_log_file();
    
    // Imposta il panic hook per rimuovere il file di lock in caso di panico
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("Panic occurred: {:?}", panic_info);
        remove_lock_file(); // Rimuove il file di lock in caso di panico
    }));

    /* --- SINGOLA APPLICAZIONE --- */
    // Controlla se il file di lock esiste
    if Path::new(LOCK_FILE_PATH).exists() {
        println!("Another instance of this program is already running.");
        process::exit(1); // Esce se esiste già un'istanza in esecuzione
    }
    // Crea il file di lock per segnare che l'app è in esecuzione
    if let Err(e) = fs::write(LOCK_FILE_PATH, "locked") {
        eprintln!("Failed to create lock file: {}", e);
        process::exit(1); // Esci se non riesci a creare il file di lock
    }
    println!("Application is running...");

    // Gestione dei segnali
    #[cfg(not(windows))]
    {
        let mut signals = Signals::new(&[signal::SIGTERM, signal::SIGINT])
            .expect("Unable to set up signal handler");
        thread::spawn(move || {
            for signal in signals.forever() {
                match signal {
                    signal::SIGINT | signal::SIGTERM => {
                        println!("Received termination signal. Cleaning up...");
                        remove_lock_file();
                        process::exit(0);
                    }
                    _ => {}
                }
            }
        });
    }
    #[cfg(windows)]
    {
        let term_flag = Arc::new(AtomicBool::new(false));
        flag::register(signal::SIGINT, Arc::clone(&term_flag))
            .expect("Unable to set up signal handler");
        flag::register(signal::SIGTERM, Arc::clone(&term_flag))
            .expect("Unable to set up signal handler");

        thread::spawn(move || {
            while !term_flag.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            println!("Received termination signal. Cleaning up...");
            remove_lock_file();
            process::exit(0);
        });
    }

    // Avvia il logging della CPU in un thread separato
    thread::spawn(move || {
        log_cpu_usage_to_csv(); // Avvia la funzione di logging della CPU
    });

    let (tx, rx) = mpsc::channel::<String>(); // Canale per comunicazione
    let (tx1, rx1) = mpsc::channel::<String>(); // Canale per comunicazion
    let (tx_stop, rx_stop) = mpsc::channel::<String>(); // Canale per lo stop
    let rx_stop = Arc::new(Mutex::new(rx_stop)); // Incapsula il Receiver
                                                 // Ottieni la configurazione
    let progress = Arc::new(Mutex::new(0.0));
    let current_file: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
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

    let gui_tx = tx1.clone();
    let stop_tx = tx_stop.clone();
    let my_app = Arc::new(Mutex::new(MyApp::new(
        detector_state,
        gui_tx,
        stop_tx,
        progress.clone(),
        Arc::clone(&current_file),
    )));

    std::thread::spawn(move || {
        let rx_stop_clone = Arc::clone(&rx_stop);
        println!("Starting detector...");
        detector::run(
            my_app,
            detector_tx,
            rx1,           // Passa rx1 per la comunicazione normale
            rx_stop_clone, // Passa rx_stop incapsulato per il controllo dello stop
            Arc::new(AtomicBool::new(true)),
        );
    });
    let run_gui;
    //per rilasciare il lock
    {
        run_gui = shared_state.lock().unwrap().display.clone(); //viene presa dal file di configurazione
    }

    if run_gui {
        println!("GUI started for the first time.");
        let gui_tx = tx1.clone();
        let stop_tx = tx_stop.clone();
        let progress = progress.clone();
        let options_for_gui = options.clone();
        let my_app = MyApp::new(
            Arc::clone(&shared_state),
            gui_tx,
            stop_tx,
            progress,
            Arc::clone(&current_file),
        );
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
                        continue; //ritorna a rx.recv()
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                // Imposta un messaggio di errore nell'applicazione
                shared_state.lock().unwrap().exit_message =
                    Some("Failed to receive message. Invalid detector!".to_string());
            }
        }

        {
            //Print for debug
            //shared_state.lock().unwrap().pretty_print();
        }
        let gui_tx = tx1.clone();
        let stop_tx = tx_stop.clone();
        let progress = progress.clone();
        let options_for_gui = options.clone();
        let my_app = MyApp::new(
            Arc::clone(&shared_state),
            gui_tx,
            stop_tx,
            progress,
            current_file.clone(),
        );
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
        } else {
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
