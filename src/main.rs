mod gui;
mod detector;

fn main() {
    // Avvia solo la GUI posso far si che si possa scegliere cosa far partire se gui o detector
    if let Err(err) = gui::run() {
        eprintln!("Errore durante l'esecuzione della GUI: {}", err);
    }
}