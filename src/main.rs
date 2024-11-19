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

    // Funzione per aggiornare i bordi in base alla posizione del mouse (Modalità rettangolo)
    //Ogni volta che il mouse si sposta, il codice verifica se il mouse è abbastanza vicino (entro una tolleranza)
    //a uno dei bordi dello schermo. Se il mouse è vicino a uno dei bordi, il programma segna quel bordo come "toccato"
    fn update_edges_rectangle(&mut self, x: f64, y: f64, screen_width: f64, screen_height: f64, tolerance: f64) {
        // Aggiorniamo i bordi in modo che coprano tutto il perimetro dello schermo
        if x <= tolerance || x >= screen_width - tolerance { // Bordo sinistro o destro
            self.left = true;
            self.right = true;
        }
        if y <= tolerance || y >= screen_height - tolerance { // Bordo superiore o inferiore
            self.top = true;
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
    let tolerance = 20.0; // La tolleranza, ad esempio 20px

    // Variabile per scegliere la modalità
    let mode = "rectangle"; // Puoi cambiarlo a "x" per la modalità X

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
                if mode == "rectangle" {
                    println!("Tracciamento dei bordi in modalità rettangolare iniziato");
                } else {
                    println!("Tracciamento dei bordi in modalità X iniziato");
                }
            }
            // Fine del tracciamento quando viene rilasciato il tasto sinistro
            EventType::ButtonRelease(Button::Left) => {
                let mut tracking = tracking_active_clone.lock().unwrap();
                *tracking = false; // Disattiva il tracciamento
                
                let edges = edges_tracker_clone.lock().unwrap();
                if edges.all_edges_touched() {
                    if mode == "rectangle" {
                        println!("Tutti i bordi coperti in modalità rettangolare, avvio del backup...");
                    } else {
                        println!("Tutti i bordi coperti in modalità X, avvio del backup...");
                    }
                    avvia_backup();
                } else {
                    if mode == "rectangle" {
                        println!("Tracciamento interrotto in modalità rettangolare, non tutti i bordi sono stati coperti.");
                    } else {
                        println!("Tracciamento interrotto in modalità X, non tutti i bordi sono stati coperti.");
                    }
                }
            }
            // Registra i movimenti solo se il tracciamento è attivo
            EventType::MouseMove { x, y } => {
                let tracking = tracking_active.lock().unwrap();
                if *tracking {
                    let mut edges = edges_tracker.lock().unwrap();
                    if mode == "rectangle" {
                        edges.update_edges_rectangle(x, y, screen_width, screen_height, tolerance);
                    } else {
                        edges.update_edges(x, y, screen_width, screen_height);
                    }
                }
            }
            _ => {}
        }
    }) {
        println!("Errore nell'ascolto degli eventi: {:?}", error);
    }
}