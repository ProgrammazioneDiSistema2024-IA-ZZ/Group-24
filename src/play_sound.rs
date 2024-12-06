// sound.rs

use rodio::{Decoder, OutputStream, source::Source};
use std::fs::File;
use std::io::BufReader;

pub fn play_sound(file_path: &str) {
    if let Ok(file) = File::open(file_path) {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let source = Decoder::new(BufReader::new(file)).unwrap();
        stream_handle.play_raw(source.convert_samples()).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(3));
    } else {
        println!("Errore: file audio non trovato o non apribile.");
    }
}
