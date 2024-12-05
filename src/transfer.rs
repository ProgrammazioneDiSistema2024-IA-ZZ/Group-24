/* use crate::ui::AppState;
use std::fs;
use std::io;
use std::path::Path;

/// Esegue il backup dei file dalla sorgente alla destinazione
pub fn perform_backup() -> Result<(), String> {
    // Verifica che le cartelle di origine e destinazione siano definite
    if state.source_folder.is_empty() {
        return Err("Source folder is not selected.".to_string());
    }
    if state.destination_folder.is_empty() {
        return Err("Destination folder is not selected.".to_string());
    }

    let source_path = Path::new(&state.source_folder);
    let dest_path = Path::new(&state.destination_folder);

    // Verifica che le cartelle esistano
    if !source_path.is_dir() {
        return Err("Source folder does not exist.".to_string());
    }
    if !dest_path.is_dir() {
        return Err("Destination folder does not exist.".to_string());
    }

    // Determina i tipi di file da includere
    let include_all = state.backup_type == "total";
    let file_types: Vec<&str> = state.file_types.iter().map(|s| s.as_str()).collect();

    // Esegui il backup
    if let Err(e) = backup_folder(source_path, dest_path, include_all, &file_types) {
        return Err(format!("Backup failed: {}", e));
    }

    Ok(())
}

/// Funzione ricorsiva per copiare i file dalla sorgente alla destinazione
fn backup_folder(
    source: &Path,
    destination: &Path,
    include_all: bool,
    file_types: &[&str],
) -> io::Result<()> {
    // Crea la cartella di destinazione, se non esiste
    if !destination.exists() {
        fs::create_dir_all(destination)?;
    }

    // Itera sui file e sottocartelle nella sorgente
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = destination.join(entry.file_name());

        if path.is_dir() {
            // Esegui il backup ricorsivamente per le sottocartelle
            backup_folder(&path, &dest_path, include_all, file_types)?;
        } else if include_all || matches_file_type(&path, file_types) {
            // Copia il file se rientra nei criteri
            fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}

/// Controlla se un file corrisponde ai tipi specificati
fn matches_file_type(file: &Path, file_types: &[&str]) -> bool {
    if let Some(ext) = file.extension() {
        let ext = ext.to_string_lossy();
        file_types.iter().any(|&ft| ft == format!(".{}", ext))
    } else {
        false
    }
}
 */