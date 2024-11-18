use rdev::{listen, Event, EventType, Button};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

// Struttura per tenere traccia dei bordi coperti
#[derive(Default)]
struct ScreenEdges {
    top: bool,
    bottom: bool,
    left: bool,
    right: bool,
}

impl ScreenEdges {
    // Funzione per controllare se tutti i bordi sono stati toccati
    fn all_edges_touched(&self) -> bool {
        self.top && self.bottom && self.left && self.right
    }

    // Funzione per aggiornare i bordi in base alla posizione del mouse
    fn update_edges(&mut self, x: f64, y: f64, screen_width: f64, screen_height: f64) {
        if x <= 10.0 { // Bordo sinistro
            self.left = true;
        }
        if x >= screen_width - 10.0 { // Bordo destro
            self.right = true;
        }
        if y <= 10.0 { // Bordo superiore
            self.top = true;
        }
        if y >= screen_height - 10.0 { // Bordo inferiore
            self.bottom = true;
        }
    }

    // Resetta i bordi
    fn reset(&mut self) {
        *self = ScreenEdges::default();
    }
}

// Funzione di backup per copiare il file
fn avvia_backup() {
    let src = Path::new("Esempio/to_save.txt"); // Cambia con il percorso del file di origine
    let dest = Path::new("Backup/to_save.txt"); // Cambia con il percorso di destinazione
    
    if let Err(e) = fs::copy(&src, &dest) {
        println!("Errore durante il backup: {:?}", e);
    } else {
        println!("Backup completato con successo!");
    }
}

fn main() {
    // Impostiamo le dimensioni dello schermo (esempio)
    let screen_width = 1920.0; // Larghezza dello schermo
    let screen_height = 1080.0; // Altezza dello schermo

    // Crea un `ScreenEdges` protetto da Mutex per l’accesso sicuro
    let edges_tracker = Arc::new(Mutex::new(ScreenEdges::default()));
    let tracking_active = Arc::new(Mutex::new(false));

    // Cloni per passare ai thread
    let edges_tracker_clone = Arc::clone(&edges_tracker);
    let tracking_active_clone = Arc::clone(&tracking_active);

    // Ascolta gli eventi
    if let Err(error) = listen(move |event| {
        match event.event_type {
            // Inizio del tracciamento quando viene premuto il tasto sinistro
            EventType::ButtonPress(Button::Left) => {
                let mut tracking = tracking_active_clone.lock().unwrap();
                *tracking = true; // Attiva il tracciamento
                let mut edges = edges_tracker_clone.lock().unwrap();
                edges.reset(); // Resetta i bordi all'inizio di una nuova sequenza
                println!("Tracciamento dei bordi iniziato");
            }
            // Fine del tracciamento quando viene rilasciato il tasto sinistro
            EventType::ButtonRelease(Button::Left) => {
                let mut tracking = tracking_active_clone.lock().unwrap();
                *tracking = false; // Disattiva il tracciamento
                
                let edges = edges_tracker_clone.lock().unwrap();
                if edges.all_edges_touched() {
                    println!("Tutti i bordi coperti, avvio del backup...");
                    avvia_backup();
                } else {
                    println!("Tracciamento interrotto, non tutti i bordi sono stati coperti.");
                }
            }
            // Registra i movimenti solo se il tracciamento è attivo
            EventType::MouseMove { x, y } => {
                let tracking = tracking_active.lock().unwrap();
                if *tracking {
                    let mut edges = edges_tracker.lock().unwrap();
                    edges.update_edges(x, y, screen_width, screen_height);
                }
            }
            _ => {}
        }
    }) {
        println!("Errore nell'ascolto degli eventi: {:?}", error);
    }
}