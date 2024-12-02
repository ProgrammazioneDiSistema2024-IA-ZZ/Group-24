use rdev::{listen, EventType, Button};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use rodio::{Decoder, OutputStream, source::Source};
use std::fs::File;
use std::io::BufReader;

// Struttura per tenere traccia dei bordi coperti
#[derive(Default)]
struct ScreenEdges {
    top: bool,
    bottom: bool,
    left: bool,
    right: bool,
}

impl ScreenEdges {
    fn all_edges_touched(&self) -> bool {   
        self.top && self.bottom && self.left && self.right
    }

    fn update_edges(&mut self, x: f64, y: f64, screen_width: f64, screen_height: f64) {
        if x <= 10.0 {
            self.left = true;
        }
        if x >= screen_width - 10.0 {
            self.right = true;
        }
        if y <= 10.0 {
            self.top = true;
        }
        if y >= screen_height - 10.0 {
            self.bottom = true;
        }
    }

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
    fn new() -> Self {
        HorizontalLineTracker::default()
    }

    fn reset(&mut self) {
        *self = HorizontalLineTracker::new();
    }

    fn update(&mut self, x: f64, y: f64) {
        if self.start_x.is_none() {
            self.start_x = Some(x);
            self.start_y = Some(y);
        }
        self.end_x = Some(x);
        self.end_y = Some(y);

        if let (Some(start_x), Some(start_y), Some(end_x), Some(end_y)) =
            (self.start_x, self.start_y, self.end_x, self.end_y)
        {
            let horizontal_distance = (end_x - start_x).abs();
            let vertical_distance = (end_y - start_y).abs();
            self.is_horizontal = horizontal_distance > 100.0 && vertical_distance < 50.0;
        }
    }

    fn is_valid_horizontal(&self) -> bool {
        self.is_horizontal
    }
}

fn play_sound(file_path: &str) {
    if let Ok(file) = File::open(file_path) {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let source = Decoder::new(BufReader::new(file)).unwrap();
        stream_handle.play_raw(source.convert_samples()).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(3));
    } else {
        eprintln!("Errore: file audio non trovato o non apribile.");
    }
}

fn avvia_backup() {
    play_sound("Sounds/bubblepop-254773.mp3");
    let src = Path::new("Esempio/to_save.txt");
    let dest = Path::new("Backup/to_save.txt");
    if let Err(e) = fs::copy(&src, &dest) {
        eprintln!("Errore durante il backup: {:?}", e);
    } else {
        play_sound("Sounds/bellding-254774.mp3");
        println!("Backup completato con successo!");
    }
}

pub fn run() {
    let screen_width = 1920.0;
    let screen_height = 1080.0;

    let mode = "x";

    let edges_tracker = Arc::new(Mutex::new(ScreenEdges::default()));
    let tracking_active = Arc::new(Mutex::new(false));
    let waiting_for_confirmation = Arc::new(Mutex::new(false));
    let horizontal_line_tracker = Arc::new(Mutex::new(HorizontalLineTracker::new()));

    let edges_tracker_clone = Arc::clone(&edges_tracker);
    let tracking_active_clone = Arc::clone(&tracking_active);
    let waiting_for_confirmation_clone = Arc::clone(&waiting_for_confirmation);
    let horizontal_line_tracker_clone = Arc::clone(&horizontal_line_tracker);

    if let Err(error) = listen(move |event| {
        match event.event_type {
            EventType::ButtonPress(Button::Left) => {
                let mut tracking = tracking_active_clone.lock().unwrap();
                let waiting = waiting_for_confirmation_clone.lock().unwrap();

                if !*waiting {
                    *tracking = true;
                    let mut edges = edges_tracker_clone.lock().unwrap();
                    edges.reset();
                    println!("Tracciamento iniziato.");
                } else {
                    let mut line_tracker = horizontal_line_tracker_clone.lock().unwrap();
                    line_tracker.reset();
                    println!("Inizio tracciamento della linea orizzontale.");
                }
            }
            EventType::ButtonRelease(Button::Left) => {
                let mut tracking = tracking_active_clone.lock().unwrap();
                let mut waiting = waiting_for_confirmation_clone.lock().unwrap();
                *tracking = false;

                if !*waiting {
                    let edges = edges_tracker_clone.lock().unwrap();
                    if edges.all_edges_touched() {
                        println!("Tutti i bordi coperti. Avvio del backup...");
                        *waiting = true;
                    } else {
                        println!("Non tutti i bordi sono stati coperti.");
                    }
                } else {
                    let line_tracker = horizontal_line_tracker_clone.lock().unwrap();
                    if line_tracker.is_valid_horizontal() {
                        println!("Linea orizzontale riconosciuta! Avvio del backup...");
                        avvia_backup();
                        *waiting = false;
                    } else {
                        println!("Movimento non riconosciuto come linea orizzontale.");
                    }
                }
            }
            EventType::MouseMove { x, y } => {
                let tracking = tracking_active.lock().unwrap();
                let waiting = waiting_for_confirmation_clone.lock().unwrap();

                if *tracking {
                    let mut edges = edges_tracker.lock().unwrap();
                    edges.update_edges(x, y, screen_width, screen_height);
                } else if *waiting {
                    let mut line_tracker = horizontal_line_tracker.lock().unwrap();
                    line_tracker.update(x, y);
                }
            }
            _ => {}
        }
    }) {
        eprintln!("Errore nell'ascolto degli eventi: {:?}", error);
    }
}

fn main() {
    run();
}
