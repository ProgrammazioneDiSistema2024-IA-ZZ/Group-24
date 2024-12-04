use eframe::egui;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Conferma Backup",
        options,
        Box::new(|_cc| Box::new(ConfirmationApp::new())),
    );
}

struct ConfirmationApp;

impl ConfirmationApp {
    pub fn new() -> Self {
        ConfirmationApp
    }
}

impl eframe::App for ConfirmationApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Vuoi procedere con il backup?");
            ui.horizontal(|ui| {
                if ui.button("Conferma").clicked() {
                    println!("Backup confermato");
                    std::process::exit(0); // Conferma e chiude la finestra
                }
                if ui.button("Annulla").clicked() {
                    println!("Backup annullato");
                    std::process::exit(1); // Annulla e chiude la finestra
                }
            });
        });
    }
}
