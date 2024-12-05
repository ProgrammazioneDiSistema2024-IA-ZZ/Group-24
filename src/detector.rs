use rdev::{listen, EventType, Button};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::transfer::perform_backup;
use crate::ui::{AppState, BackupStatus};
use std::time::Duration;

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
    let screen_width = 1920.0;
    let screen_height = 1080.0;

    let edges_tracker = Arc::new(Mutex::new(ScreenEdges::default()));
    let tracking_active = Arc::new(Mutex::new(false));

    let edges_tracker_clone = Arc::clone(&edges_tracker);
    let tracking_active_clone = Arc::clone(&tracking_active);


    if let Err(error) = listen(move |event| {
        match event.event_type {
            EventType::ButtonPress(Button::Left) => {
                let mut tracking = tracking_active_clone.lock().unwrap();
                *tracking = true;
                let mut edges = edges_tracker_clone.lock().unwrap();
                edges.reset();
                println!("Tracciamento iniziato.");
            }
            EventType::ButtonRelease(Button::Left) => {
                let mut tracking = tracking_active_clone.lock().unwrap();
                *tracking = false;

                let edges = edges_tracker_clone.lock().unwrap();
                if edges.all_edges_touched() {
                    println!("Tutti i bordi coperti. Avvio del backup...");

                    // start backup + clona lo stato condiviso
                    avvia_backup(Arc::clone(&shared_state));
                } else {
                    println!("Non tutti i bordi sono stati coperti.");
                }
            }
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
        eprintln!("Errore nell'ascolto degli eventi: {:?}", error);
        //dovrebbe terminare l'applicazione? se detector fallisce ############
    }
}