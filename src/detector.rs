use rdev::{listen, EventType, Button};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use rodio::{Decoder, OutputStream, source::Source};
use std::fs::File;
use std::io::BufReader;
use std::process::Command;

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
        if self.start_x.is_none() || self.start_y.is_none() {
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

fn show_confirmation_window() {
    std::thread::spawn(|| {
        let status = Command::new("cargo")
            .arg("run")
            .arg("--bin")
            .arg("confirmation")
            .status();

        if let Ok(exit_status) = status {
            if exit_status.success() {
                println!("Conferma ricevuta, avvio backup...");
                avvia_backup();
            } else {
                println!("Backup annullato dall'utente.");
            }
        } else {
            eprintln!("Errore nell'avviare la finestra di conferma.");
        }
    });
}

pub fn run() {
    let screen_width = 1920.0;
    let screen_height = 1080.0;

    let edges_tracker = Arc::new(Mutex::new(ScreenEdges::default()));
    let tracking_active = Arc::new(Mutex::new(false));
    let line_tracker = Arc::new(Mutex::new(HorizontalLineTracker::new()));
    let waiting_for_confirmation = Arc::new(Mutex::new(false));

    let edges_tracker_clone = Arc::clone(&edges_tracker);
    let tracking_active_clone = Arc::clone(&tracking_active);
    let line_tracker_clone = Arc::clone(&line_tracker);
    let waiting_for_confirmation_clone = Arc::clone(&waiting_for_confirmation);

    if let Err(error) = listen(move |event| {
        match event.event_type {
            EventType::ButtonPress(Button::Left) => {
                let mut tracking = tracking_active_clone.lock().unwrap();
                *tracking = true;
                let mut edges = edges_tracker_clone.lock().unwrap();
                edges.reset();
                let mut line = line_tracker_clone.lock().unwrap();
                line.reset();
                println!("Tracciamento iniziato.");
            }
            EventType::ButtonRelease(Button::Left) => {
                let mut tracking = tracking_active_clone.lock().unwrap();
                *tracking = false;

                let edges = edges_tracker_clone.lock().unwrap();
                if edges.all_edges_touched() {
                    println!("Tutti i bordi coperti. Attendo conferma o linea orizzontale...");
                    *waiting_for_confirmation_clone.lock().unwrap() = true;
                    show_confirmation_window();
                } else {
                    println!("Non tutti i bordi sono stati coperti.");
                }
            }
            EventType::MouseMove { x, y } => {
                let tracking = tracking_active.lock().unwrap();
                let waiting = waiting_for_confirmation_clone.lock().unwrap();

                if *tracking {
                    let mut edges = edges_tracker.lock().unwrap();
                    edges.update_edges(x, y, screen_width, screen_height);
                }

                if *waiting {
                    let mut line = line_tracker.lock().unwrap();
                    line.update(x, y);

                    if line.is_valid_horizontal() {
                        println!("Linea orizzontale riconosciuta!");
                        *waiting_for_confirmation_clone.lock().unwrap() = false;
                        avvia_backup();
                    }
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
