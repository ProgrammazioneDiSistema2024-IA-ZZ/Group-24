use crate::utils::manage_configuration_file;
use crate::utils::Configuration;
use crate::utils::play_sound;
use crate::analytics::log_backup_data_to_csv;
use systemstat::{System, Platform};
use std::fs;
use std::io;
use std::path::Path;
use std::time::Instant;


/// Esegue il backup dei file dalla sorgente alla destinazione
pub fn perform_backup() -> Result<(), String> {
    // recupera i dati dall aconfigurazione statica (e non da AppState, che è dinamico)
    let config = manage_configuration_file();

    // Verifica se config è di tipo Configuration::Build
    if let Configuration::Build(source_folder, destination_folder, backup_type, file_types) = config {
        // Salva i campi in variabili
        let source_folder = source_folder; // String
        let destination_folder = destination_folder; // String
        let backup_type = backup_type; // String
        let file_types = file_types; // Vec<String>

        let source_path = Path::new(&source_folder);
        let dest_path = Path::new(&destination_folder);

        // Verifica che le cartelle esistano
        if !source_path.is_dir() {
            return Err(format!("Source folder: `{}` does not exist.", source_folder));
        }
        if !dest_path.is_dir() {
            return Err(format!("Destination folder: `{}` does not exist.", destination_folder));
        }

        // Determina i tipi di file da includere
        let include_all = backup_type == "total" || (backup_type == "custom" && file_types.len() == 0);
        let file_types: Vec<&str> = file_types.iter().map(|s| s.as_str()).collect();

        // Calcola la durata del backup
        let start_time = Instant::now();

        //Riproduci suono inizio backup
        play_sound("Sounds/bubblepop-254773.mp3");

        // Esegui il backup
        if let Err(e) = backup_folder(source_path, dest_path, include_all, &file_types) {
            play_sound("Sounds/incorrect-buzzer-sound-147336.mp3");
            return Err(format!("Backup failed: {}", e));
            
        } else {
            play_sound("Sounds/bellding-254774.mp3");
            let duration = start_time.elapsed().as_secs(); // Durata del backup in secondi
            let total_size = get_total_size(source_path).map_err(|e| e.to_string())?; // Calcola i dati trasferiti in byte
            // Registra i dettagli del backup nelle analitiche
            let cpu_usage = get_cpu_usage();
            log_backup_data_to_csv(total_size, duration, cpu_usage);
        }

        Ok(())
    }
    else {
        return Err("Configurazione non valida. Imposta una configurazione valida dal pannello di Backup!".to_string())
    }
}

/// Funzione ricorsiva per copiare i file dalla sorgente alla destinazione.
fn backup_folder(
    source: &Path,
    destination: &Path,
    include_all: bool,
    file_types: &[&str],
) -> io::Result<()> {
    // Crea la directory di destinazione se non esiste
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
        } else if path.is_file() {
            // Copia il file se rientra nei criteri
            if include_all || matches_file_type(&path, file_types) {
                fs::copy(&path, &dest_path)?;
            }
        }
    }

    Ok(())
}

/// Controlla se un file corrisponde ai tipi specificati
fn matches_file_type(file: &Path, file_types: &[&str]) -> bool {
    // Estrazione dell'estensione:
    if let Some(ext) = file.extension().and_then(|ext| ext.to_str()) {
        // Aggiungi il punto all'estensione estratta se non c'è
        let ext_with_dot = if ext.starts_with('.') {
            ext.to_string() // L'estensione ha già il punto, la manteniamo invariata
        } else {
            format!(".{}", ext) // Aggiungiamo il punto se non presente
        };

        // Confronto delle estensioni:
        file_types.iter().any(|&ft| ft.eq_ignore_ascii_case(&ext_with_dot))
    } else {
        false
    }
}

fn get_total_size(path: &Path) -> io::Result<u64> {
    let mut total_size = 0;

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_file() {
                total_size += entry_path.metadata()?.len();
            } else if entry_path.is_dir() {
                total_size += get_total_size(&entry_path)?;
            }
        }
    }

    Ok(total_size)
}



fn get_cpu_usage() -> f32 {
    let sys = System::new();
    match sys.cpu_load_aggregate() {
        Ok(cpu) => {
            // Attendi un secondo per calcolare il carico
            std::thread::sleep(std::time::Duration::from_secs(1));
            match cpu.done() {
                Ok(cpu_load) => {
                    // Calcola il carico totale come somma dei consumi utente e di sistema
                    let usage = (cpu_load.user + cpu_load.system) * 100.0;
                    usage
                }
                Err(_) => 0.0, // Se fallisce, restituisci 0.0
            }
        }
        Err(_) => 0.0, // Se fallisce, restituisci 0.0
    }
}



