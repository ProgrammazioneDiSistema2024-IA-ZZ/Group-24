use rdev::{listen, EventType, Button};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::transfer::perform_backup;
use crate::ui::{AppState, BackupStatus};
use std::time::Duration;
use crate::utils;
use crate::first_sign;
use crate::confirm_sign;


#[derive(Debug)]
struct ScreenResolution {
    width: f64,
    height: f64,
}

fn avvia_backup(shared_state: Arc<Mutex<AppState>>) {
    //viene monitorato lo stato di avanzamento del backup...
    shared_state.lock().unwrap().backup_status = BackupStatus::InProgress;

    //simula processo di backup di 60 secondi...
    let total_steps = 65;
    for i in 1..=total_steps {
        // Simula un passo del processo di backup
        thread::sleep(Duration::from_secs(1));  // Pausa di 1 secondo per ogni passo
        if i % 10 == 0 {
            println!("Backup in corso... {}%", i/total_steps * 100);
        }
    }

    match perform_backup() {
        Ok(_) => {
            shared_state.lock().unwrap().backup_status = BackupStatus::CompletedSuccess;
        }
        Err(err) => {
            shared_state.lock().unwrap().backup_status = BackupStatus::CompletedError(err);
        }
    }
}

pub fn run(shared_state: Arc<Mutex<AppState>>) {
    let tolerance = 20.0; // La tolleranza, ad esempio 20px


    // Ottieni la risoluzione dello schermo
    let (screen_width, screen_height): (f64, f64) = match utils::get_screen_resolution() {
        Some((w, h)) => {
            let w_f64 = w as f64;  // Converto w da u32 a f64
            let h_f64 = h as f64;  // Converto h da u32 a f64
            println!("Dimensioni dello schermo: {}x{}", w_f64, h_f64);
            (w_f64, h_f64) // Restituisco la tupla in f64
        }
        None => {
            println!("Errore: impossibile ottenere la risoluzione dello schermo.");
            return;  // Esci se non possiamo ottenere la risoluzione
        }
    };

    // Crea una struttura che contiene le dimensioni dello schermo
    let screen_resolution = Arc::new(Mutex::new(ScreenResolution {
        width: screen_width,
        height: screen_height,
    }));

    let edges_tracker = Arc::new(Mutex::new(first_sign::ScreenEdges::default()));
    let tracking_active = Arc::new(Mutex::new(false));
    let waiting_for_confirmation = Arc::new(Mutex::new(false));
    let horizontal_line_tracker = Arc::new(Mutex::new(confirm_sign::HorizontalLineTracker::new()));

    // Cloni per passare ai thread
    let edges_tracker_clone = Arc::clone(&edges_tracker);
    let tracking_active_clone = Arc::clone(&tracking_active);
    let waiting_for_confirmation_clone = Arc::clone(&waiting_for_confirmation);
    let horizontal_line_tracker_clone = Arc::clone(&horizontal_line_tracker);
    let screen_resolution_clone = Arc::clone(&screen_resolution);
    

     // Ascolta gli eventi
     if let Err(error) = listen(move |event| {
        match event.event_type {
            // Inizio del tracciamento quando viene premuto il tasto sinistro
            EventType::ButtonPress(Button::Left) => {
                let mut tracking = tracking_active_clone.lock().unwrap();
                let waiting = waiting_for_confirmation_clone.lock().unwrap();


                if !*waiting {
                    *tracking = true;
                    let mut edges = edges_tracker_clone.lock().unwrap();
                    edges.reset();
                    println!("Tracciamento dei bordi in modalità rettangolare iniziato");
                } else {
                    // Resetta il tracciamento per la linea orizzontale
                    let mut line_tracker = horizontal_line_tracker_clone.lock().unwrap();
                    line_tracker.reset();
                    println!("Inizio tracciamento della linea orizzontale.");
                }
                
                
            }
            // Fine del tracciamento quando viene rilasciato il tasto sinistro
            EventType::ButtonRelease(Button::Left) => {
                let mut tracking = tracking_active_clone.lock().unwrap();
                let mut waiting = waiting_for_confirmation_clone.lock().unwrap();
                *tracking = false; // Disattiva il tracciamento
                
                if !*waiting {
                    *tracking = false;
                    let edges = edges_tracker_clone.lock().unwrap();
                    if edges.all_edges_touched() {
                        utils::play_sound("Sounds/system-notification-199277.mp3");
                        println!("Tutti i bordi coperti. Conferma con il segno '-' per avviare il backup.");
                        *waiting = true;
                    } else {
                        println!("Non tutti i bordi sono stati coperti.");
                    }
                } else {
                    // Controlla se è stata disegnata una linea orizzontale
                    let line_tracker = horizontal_line_tracker_clone.lock().unwrap();
                    if line_tracker.is_valid_horizontal() {
                        println!("Linea orizzontale riconosciuta! Avvio del backup...");
                         avvia_backup(Arc::clone(&shared_state));
                        *waiting = false; // Resetta la modalità di conferma
                    } else {
                        println!("Movimento non riconosciuto come linea orizzontale.");
                    }
                }
            }
            // Registra i movimenti solo se il tracciamento è attivo
            EventType::MouseMove { x, y } => {
                let tracking = tracking_active.lock().unwrap();
                let waiting = waiting_for_confirmation_clone.lock().unwrap();
                // Ottieni la risoluzione dello schermo dalla struttura condivisa
                let resolution = screen_resolution_clone.lock().unwrap();
                let screen_width = resolution.width;
                let screen_height = resolution.height;
                if *tracking {
                    let mut edges = edges_tracker.lock().unwrap();
                    edges.update_edges_rectangle(x, y, screen_width, screen_height, tolerance);
                } else if *waiting {
                    // Aggiorna il tracciamento della linea orizzontale
                    let mut line_tracker = horizontal_line_tracker.lock().unwrap();
                    line_tracker.update(x, y);
                }

            }
            _ => {}
        }
    }) {
        println!("Errore nell'ascolto degli eventi: {:?}", error);
    }
}