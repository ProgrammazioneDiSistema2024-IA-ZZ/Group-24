mod gui;
mod detector;

fn main() {
    // Avvia solo la GUI
    if let Err(err) = gui::run() {
        eprintln!("Errore durante l'esecuzione della GUI: {}", err);
    }
}