use rdev::{listen, EventType, Button};
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

#[derive(Default)]
struct HorizontalLineTracker {
    start_x: Option<f64>,
    start_y: Option<f64>,
    end_x: Option<f64>,
    end_y: Option<f64>,
    is_horizontal: bool,
}

impl HorizontalLineTracker {
    // Crea una nuova istanza
    fn new() -> Self {
        HorizontalLineTracker {
            start_x: None,
            start_y: None,
            end_x: None,
            end_y: None,
            is_horizontal: false,
        }
    }

    // Resetta il tracker per iniziare un nuovo tracciamento
    fn reset(&mut self) {
        *self = HorizontalLineTracker::new();
    }

    // Aggiorna il tracker con una nuova posizione del mouse
    fn update(&mut self, x: f64, y: f64) {
        if self.start_x.is_none() || self.start_y.is_none() {
            // Imposta il punto iniziale
            self.start_x = Some(x);
            self.start_y = Some(y);
        }

        // Aggiorna il punto finale
        self.end_x = Some(x);
        self.end_y = Some(y);

        // Verifica se il movimento è principalmente orizzontale
        if let (Some(start_x), Some(start_y), Some(end_x), Some(end_y)) = 
            (self.start_x, self.start_y, self.end_x, self.end_y) {
            let horizontal_distance = (end_x - start_x).abs();
            let vertical_distance = (end_y - start_y).abs();
            
            self.is_horizontal = horizontal_distance > 100.0 && vertical_distance < 50.0;
        }
    }

    // Controlla se il movimento soddisfa i criteri per essere una linea orizzontale
    fn is_valid_horizontal(&self) -> bool {
        self.is_horizontal
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
    let mode = "x"; // x per tutti i lati, rectangle per il rett lungo i bordi 

    // Crea un `ScreenEdges` protetto da Mutex per l’accesso sicuro
    let edges_tracker = Arc::new(Mutex::new(ScreenEdges::default()));
    let tracking_active = Arc::new(Mutex::new(false));
    let waiting_for_confirmation = Arc::new(Mutex::new(false));

    let horizontal_line_tracker = Arc::new(Mutex::new(HorizontalLineTracker::new()));

    // Cloni per passare ai thread
    let edges_tracker_clone = Arc::clone(&edges_tracker);
    let tracking_active_clone = Arc::clone(&tracking_active);
    let waiting_for_confirmation_clone = Arc::clone(&waiting_for_confirmation);
    let horizontal_line_tracker_clone = Arc::clone(&horizontal_line_tracker);

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
                    if mode == "rectangle" {
                    println!("Tracciamento dei bordi in modalità rettangolare iniziato");
                    } else {
                        println!("Tracciamento dei bordi in modalità X iniziato");
                    }
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
                        avvia_backup();
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

                if *tracking {
                    let mut edges = edges_tracker.lock().unwrap();
                    if mode == "rectangle" {
                        edges.update_edges_rectangle(x, y, screen_width, screen_height, tolerance);
                    } else {
                        edges.update_edges(x, y, screen_width, screen_height);
                    }
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