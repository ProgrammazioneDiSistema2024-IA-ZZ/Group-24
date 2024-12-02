use eframe::egui;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

pub fn run() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "GUI Mockup Detector",
        options,
        Box::new(|_cc| Box::new(MyApp::new())),
    )
}

struct MyApp {
    detector_running: Arc<Mutex<bool>>,
}

impl MyApp {
    pub fn new() -> Self {
        Self {
            detector_running: Arc::new(Mutex::new(false)),
        }
    }

    fn start_detector(&self) {
        let detector_running = Arc::clone(&self.detector_running);
        let already_running = {
            let running = detector_running.lock().unwrap();
            *running
        };

        if already_running {
            println!("Il detector è già in esecuzione.");
            return;
        }

        match Command::new("cargo")
            .arg("run")
            .arg("--bin")
            .arg("detector")
            .stdout(Stdio::inherit()) // Mostra l'output del detector nel terminale
            .stderr(Stdio::inherit())
            .spawn()
        {
            Ok(_child) => {
                let mut running = detector_running.lock().unwrap();
                *running = true;
                println!("Detector avviato!");
            }
            Err(e) => {
                let mut running = detector_running.lock().unwrap();
                *running = false;
                eprintln!("Errore nell'avvio del detector: {:?}", e);
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rilevatore di movimenti del mouse");

            let running = {
                let running = self.detector_running.lock().unwrap();
                *running
            };

            if running {
                ui.label("Detector in esecuzione...");
            } else {
                ui.label("Detector non in esecuzione.");
            }

            if ui.button("Avvia Detector").clicked() {
                self.start_detector();
            }

            if ui.button("Chiudi GUI").clicked() {
                std::process::exit(0);
            }
        });
    }
}
