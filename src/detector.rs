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
    // Monitoraggio dello stato di avanzamento del backup...
    shared_state.lock().unwrap().backup_status = BackupStatus::InProgress;

    let total_steps = 65;
    for i in 1..=total_steps {
        thread::sleep(Duration::from_secs(1)); // Pausa di 1 secondo per step
        if i % 10 == 0 {
            println!("Backup in corso... {}%", i as f64 / total_steps as f64 * 100.0);
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
    let tolerance = 20.0; // Tolleranza

    let (screen_width, screen_height): (f64, f64) = match utils::get_screen_resolution() {
        Some((w, h)) => {
            let w_f64 = w as f64;
            let h_f64 = h as f64;
            println!("Dimensioni dello schermo: {}x{}", w_f64, h_f64);
            (w_f64, h_f64)
        }
        None => {
            println!("Errore: impossibile ottenere la risoluzione dello schermo.");
            return;
        }
    };

    let screen_resolution = Arc::new(Mutex::new(ScreenResolution {
        width: screen_width,
        height: screen_height,
    }));

    let edges_tracker = Arc::new(Mutex::new(first_sign::ScreenEdges::default()));
    let tracking_active = Arc::new(Mutex::new(false));
    let waiting_for_confirmation = Arc::new(Mutex::new(false));
    let horizontal_line_tracker = Arc::new(Mutex::new(confirm_sign::HorizontalLineTracker::new()));

    // Cloni per i thread
    let edges_tracker_clone = Arc::clone(&edges_tracker);
    let tracking_active_clone = Arc::clone(&tracking_active);
    let waiting_for_confirmation_clone = Arc::clone(&waiting_for_confirmation);
    let horizontal_line_tracker_clone = Arc::clone(&horizontal_line_tracker);
    let screen_resolution_clone = Arc::clone(&screen_resolution);

    // Parametro per definire quanti segmenti per lato verificare
    let segment_count = 20; // ad esempio 20 segmenti per lato

    if let Err(error) = listen(move |event| {
        match event.event_type {
            EventType::ButtonPress(Button::Left) => {
                let mut tracking = tracking_active_clone.lock().unwrap();
                let waiting = waiting_for_confirmation_clone.lock().unwrap();

                if !*waiting {
                    *tracking = true;
                    let mut edges = edges_tracker_clone.lock().unwrap();
                    edges.reset();
                    println!("Tracciamento del contorno iniziato (rettangolare)...");
                } else {
                    // Reset del tracciamento della linea orizzontale
                    let mut line_tracker = horizontal_line_tracker_clone.lock().unwrap();
                    line_tracker.reset();
                    println!("Inizio tracciamento della linea orizzontale di conferma.");
                }
            }
            EventType::ButtonRelease(Button::Left) => {
                let mut tracking = tracking_active_clone.lock().unwrap();
                let mut waiting = waiting_for_confirmation_clone.lock().unwrap();
                *tracking = false;

                // Ottieni larghezza/altezza
                let resolution = screen_resolution_clone.lock().unwrap();
                let screen_width = resolution.width;
                let screen_height = resolution.height;

                if !*waiting {
                    let edges = edges_tracker_clone.lock().unwrap();
                    
                    // Qui usiamo la nuova funzione anziché all_edges_touched()
                    if edges.is_contour_complete(screen_width, screen_height, segment_count) {
                        utils::play_sound("Sounds/system-notification-199277.mp3");
                        println!("Contorno completo riconosciuto! Disegna una linea orizzontale per confermare e avviare il backup.");
                        *waiting = true;
                    } else {
                        println!("Contorno non completo. Riprova disegnando il perimetro completo dello schermo.");
                    }
                } else {
                    // Controlla se è stata disegnata una linea orizzontale
                    let line_tracker = horizontal_line_tracker_clone.lock().unwrap();
                    if line_tracker.is_valid_horizontal() {
                        println!("Linea orizzontale riconosciuta! Avvio del backup...");
                        avvia_backup(Arc::clone(&shared_state));
                        *waiting = false;
                    } else {
                        println!("Movimento non riconosciuto come linea orizzontale di conferma.");
                    }
                }
            }
            EventType::MouseMove { x, y } => {
                let tracking = tracking_active.lock().unwrap();
                let waiting = waiting_for_confirmation_clone.lock().unwrap();
                let resolution = screen_resolution_clone.lock().unwrap();
                let screen_width = resolution.width;
                let screen_height = resolution.height;

                if *tracking && !*waiting {
                    let mut edges = edges_tracker.lock().unwrap();
                    edges.update_edges_rectangle(x, y, screen_width, screen_height, tolerance);
                } else if *waiting {
                    // Aggiornamento per la linea di conferma
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
