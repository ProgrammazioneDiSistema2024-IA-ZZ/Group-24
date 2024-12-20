use rdev::{listen, EventType, Button};
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
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

pub fn avvia_backup(shared_state: Arc<Mutex<AppState>>) {
    // Monitoraggio dello stato di avanzamento del backup...
    shared_state.lock().unwrap().backup_status = BackupStatus::InProgress;

    // // attesa fittizia
    // let total_steps = 10;
    // for i in 1..=total_steps {
    //     thread::sleep(Duration::from_secs(1)); // Pausa di 1 secondo per step
    //     if i % 3 == 0 {
    //         println!("Backup in corso... {}%", ((i as f64) / (total_steps as f64)) * 100.0);
    //     }
    // }

    match perform_backup() {
        Ok(_) => {
            shared_state.lock().unwrap().backup_status = BackupStatus::CompletedSuccess;
        }
        Err(err) => {
            shared_state.lock().unwrap().backup_status = BackupStatus::CompletedError(err);
        }
    }
}
pub fn run(shared_state: Arc<Mutex<AppState>>, tx: Sender<String>, rx: Receiver<String>) {
    let tolerance = 30.0; // Tolleranza

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

    // Clone `waiting_for_confirmation` per usarlo nel thread
    let waiting_for_confirmation_clone = Arc::clone(&waiting_for_confirmation);

    // Thread separato per ascoltare i messaggi su `rx`
    std::thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            if msg == "resetWaiting" {
                println!("Ricevuto messaggio: resetWaiting. Imposto waiting_for_confirmation a false.");
                let mut waiting = waiting_for_confirmation_clone.lock().unwrap();
                *waiting = false;
            }
        }
    });

    // Parametro per definire quanti segmenti per lato verificare
    let segment_count = 20; // ad esempio 20 segmenti per lato

    if let Err(error) = listen(move |event| {
        match event.event_type {
            EventType::ButtonPress(Button::Left) => {
                let mut tracking = tracking_active.lock().unwrap();
                let waiting = waiting_for_confirmation.lock().unwrap();

                if !*waiting {
                    *tracking = true;
                    let mut edges = edges_tracker.lock().unwrap();
                    edges.reset();
                    println!("Tracciamento del contorno iniziato (rettangolare)...");
                } else {
                    // Reset del tracciamento della linea orizzontale
                    let mut line_tracker = horizontal_line_tracker.lock().unwrap();
                    line_tracker.reset();
                    println!("Inizio tracciamento della linea orizzontale di conferma.");
                }
            }
            EventType::ButtonRelease(Button::Left) => {
                let mut tracking = tracking_active.lock().unwrap();
                let mut waiting = waiting_for_confirmation.lock().unwrap();
                *tracking = false;

                // Ottieni larghezza/altezza
                let resolution = screen_resolution.lock().unwrap();
                let screen_width = resolution.width;
                let screen_height = resolution.height;

                if !*waiting {
                    let edges = edges_tracker.lock().unwrap();

                    // Qui usiamo la nuova funzione anziché all_edges_touched()
                    if edges.is_contour_complete(screen_width, screen_height, segment_count) {
                        utils::play_sound("Sounds/system-notification-199277.mp3");
                        {
                            let mut state = shared_state.lock().unwrap();
                            state.backup_status = BackupStatus::ToConfirm; // Passa allo stato di conferma
                        }
                        println!("Contorno completo riconosciuto! Disegna una linea orizzontale per confermare e avviare il backup.");

                        {
                            let mut state = shared_state.lock().unwrap();
                            if !state.display {
                                // Se la GUI non è aperta, aggiorna lo stato e invia il messaggio
                                if let Err(err) = tx.send("showGUI".to_string()) {
                                    eprintln!("Failed to send message: {}", err);
                                    // Ripristina lo stato in caso di errore nell'invio del messaggio
                                    state.display = false;
                                } else {
                                    println!("Message sent successfully.");
                                }
                            } else {
                                println!("GUI already active. Skipping message.");
                            }
                        }

                        *waiting = true;
                    } else {
                        println!("Contorno non completo. Riprova disegnando il perimetro completo dello schermo.");
                    }
                } else {
                    // Controlla se è stata disegnata una linea orizzontale
                    let line_tracker = horizontal_line_tracker.lock().unwrap();
                    if line_tracker.is_valid_horizontal() {
                        println!("Linea orizzontale riconosciuta! Avvio del backup...");
                        /* --- GESTIONE COMUNICAZIONE CON THREAD PRINCIPALE ---- */
                        // fai apparire la GUI, per mostrare la schermata di backup in corso, o mostrare eventuali errori
                        // Prova fino a 3 volte, in caso di condizione temporanea (improbabile nel caso di mpsc)

                        {
                            let mut state = shared_state.lock().unwrap();
                            if !state.display {
                                // Se la GUI non è aperta, aggiorna lo stato e invia il messaggio
                                if let Err(err) = tx.send("showGUI".to_string()) {
                                    eprintln!("Failed to send message: {}", err);
                                    // Ripristina lo stato in caso di errore nell'invio del messaggio
                                    state.display = false;
                                } else {
                                    println!("Message sent successfully.");
                                }
                            } else {
                                println!("GUI already active. Skipping message.");
                            }
                        }
                    
                        avvia_backup(Arc::clone(&shared_state));
                        *waiting = false;
                    } else {
                        println!("Movimento non riconosciuto come linea orizzontale di conferma.");
                    }
                }
            }
            EventType::MouseMove { x, y } => {
                let tracking = tracking_active.lock().unwrap();
                let waiting = waiting_for_confirmation.lock().unwrap();
                let resolution = screen_resolution.lock().unwrap();
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
