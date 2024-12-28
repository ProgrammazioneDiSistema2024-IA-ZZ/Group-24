use rdev::{listen, EventType, Button};
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::transfer::perform_backup;
use crate::transfer::perform_backup_with_stop;
use crate::ui::{AppState, BackupStatus};
use std::time::Duration;
use crate::utils;
use crate::first_sign;
use crate::confirm_sign;
use std::sync::atomic::{AtomicBool, Ordering};


#[derive(Debug)]
struct ScreenResolution {
    width: f64,
    height: f64,
}

pub fn avvia_backup(
    shared_state: Arc<Mutex<AppState>>,
    detector_running: Arc<AtomicBool>,
    tx: Sender<String>,
    stop_rx: Arc<Mutex<Receiver<String>>>, // Receiver incapsulato
){
    // Disattiva il detector
    detector_running.store(false, Ordering::Relaxed);
    println!("Detector disattivato.");

    // Avvia un nuovo thread per il backup
    std::thread::spawn(move || {
        {
            let mut state = shared_state.lock().unwrap();
            state.backup_status = BackupStatus::InProgress;
        }

        // Esegui il backup con controllo di stop
        // Sblocca il Mutex per ottenere il Receiver e passa la referenza
        let backup_result = {
            let stop_rx = stop_rx.lock().unwrap(); // Sblocca il Mutex
            perform_backup_with_stop(&*stop_rx)    // Passa una referenza al Receiver
        };
        match backup_result {
            Ok(_) => {
                let mut state = shared_state.lock().unwrap();
                state.backup_status = BackupStatus::CompletedSuccess;
                println!("Backup completato con successo.");
            }
            Err(err) => {
                let mut state = shared_state.lock().unwrap();
                if err == "stop" {
                    state.backup_status = BackupStatus::NotStarted;
                    println!("Backup interrotto dall'utente.");
                } else {
                    state.backup_status = BackupStatus::CompletedError(err);
                    println!("Backup fallito");
                }
            }
        }

        // Riattiva il detector al completamento del backup
        detector_running.store(true, Ordering::Relaxed);
        println!("Detector riattivato.");

        // Notifica la GUI del completamento del backup
        {
            let mut state = shared_state.lock().unwrap();
            if !state.display {
                if let Err(err) = tx.send("showGUI".to_string()) {
                    eprintln!("Failed to send showGUI message: {}", err);
                    state.display = false; // Assicurati che lo stato rifletta correttamente il fallimento
                } else {
                    println!("Message sent to show GUI.");
                }
            } else {
                println!("GUI already active. Skipping message.");
            }
        }
    });

    println!("Il backup è stato avviato in un thread separato.");
}

pub fn run(
    shared_state: Arc<Mutex<AppState>>,
    tx: Sender<String>,
    rx: Receiver<String>, // Canale per i messaggi generici
    rx_stop: Arc<Mutex<Receiver<String>>>,
    detector_running: Arc<AtomicBool>,
) {
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

    let segment_count = 20; // Numero di segmenti per lato

    if let Err(error) = listen(move |event| {
        // Controlla se il detector è attivo
        if !detector_running.load(Ordering::Relaxed) {
            return;
        }

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
                    let mut line_tracker = horizontal_line_tracker.lock().unwrap();
                    line_tracker.reset();
                    println!("Inizio tracciamento della linea orizzontale di conferma.");
                }
            }
            EventType::ButtonRelease(Button::Left) => {
                let mut tracking = tracking_active.lock().unwrap();
                let mut waiting = waiting_for_confirmation.lock().unwrap();
                *tracking = false;

                let resolution = screen_resolution.lock().unwrap();
                let screen_width = resolution.width;
                let screen_height = resolution.height;

                if !*waiting {
                    let edges = edges_tracker.lock().unwrap();

                    if edges.is_contour_complete(screen_width, screen_height, segment_count) {
                        utils::play_sound("Sounds/system-notification-199277.mp3");
                        {
                            let mut state = shared_state.lock().unwrap();
                            state.backup_status = BackupStatus::ToConfirm;
                        }
                        println!("Contorno completo riconosciuto! Disegna una linea orizzontale per confermare e avviare il backup.");

                        {
                            let mut state = shared_state.lock().unwrap();
                            if !state.display {
                                if let Err(err) = tx.send("showGUI".to_string()) {
                                    eprintln!("Failed to send message: {}", err);
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
                    let line_tracker = horizontal_line_tracker.lock().unwrap();
                    if line_tracker.is_valid_horizontal() {
                        println!("Linea orizzontale riconosciuta! Avvio del backup...");
                    
                        {
                            let mut state = shared_state.lock().unwrap();
                            if !state.display {
                                if let Err(err) = tx.send("showGUI".to_string()) {
                                    eprintln!("Failed to send message: {}", err);
                                    state.display = false;
                                } else {
                                    println!("Message sent successfully.");
                                }
                            } else {
                                println!("GUI already active. Skipping message.");
                            }
                        }
                    
                        // Disattiva il detector e avvia il backup
                        detector_running.store(false, Ordering::Relaxed);
                        

                        avvia_backup(
                            Arc::clone(&shared_state),
                            Arc::clone(&detector_running),
                            tx.clone(), // Aggiungi il trasmettitore
                            Arc::clone(&rx_stop), // Passa il canale di stop al backup
                        );
                    
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