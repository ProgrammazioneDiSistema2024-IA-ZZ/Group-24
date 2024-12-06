use std::{fs, path::Path};
use crate::play_sound::play_sound; // Importiamo la funzione play_sound

pub fn avvia_backup() {
    play_sound("Sounds/bubblepop-254773.mp3");
    let src = Path::new("Esempio/to_save.txt"); // Cambia con il percorso del file di origine
    let dest = Path::new("Backup/to_save.txt"); // Cambia con il percorso di destinazione
    
    if let Err(e) = fs::copy(&src, &dest) {
        println!("Errore durante il backup: {:?}", e);
    } else {
        play_sound("Sounds/bellding-254774.mp3");
        println!("Backup completato con successo!");
    }
}
