use rdev::{listen, EventType, Button};
use std::sync::{Arc, Mutex};

mod screen;
mod confirm_sign;  
mod backup; 
mod play_sound;
mod first_sign;

#[derive(Debug)]
struct ScreenResolution {
    width: f64,
    height: f64,
}
fn main() {
 
    let tolerance = 20.0; // La tolleranza, ad esempio 20px

    // Ottieni la risoluzione dello schermo
    let (screen_width, screen_height): (f64, f64) = match screen::get_screen_resolution() {
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


    // Crea un `ScreenEdges` protetto da Mutex per l’accesso sicuro
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
                        play_sound::play_sound("Sounds/system-notification-199277.mp3");
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
                        backup::avvia_backup();
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

                    // Debug: Mostra lo stato attuale dei bordi toccati
                    println!("Stato dei bordi (parziale): {:?}", *edges);
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