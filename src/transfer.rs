use crate::analytics::log_backup_data_to_csv;
use crate::ui::MyApp;
use crate::utils::manage_configuration_file;
use crate::utils::play_sound;
use crate::utils::Configuration;
use sha2::{Digest, Sha256};
use std::fs;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::time::Instant;
use systemstat::{Platform, System};

pub fn perform_backup_with_stop(
    stop_rx: &Receiver<String>,
    state: &mut MyApp,
) -> Result<(), String> {
    // Recupera i dati dalla configurazione statica
    let config = manage_configuration_file();

    // Verifica se config è di tipo Configuration::Build
    if let Configuration::Build(source_folder, destination_folder, backup_type, file_types, _) = config
    {
        let source_path = Path::new(&source_folder);
        let dest_path = Path::new(&destination_folder);

        // Verifica che le cartelle esistano
        if !source_path.is_dir() {
            return Err(format!(
                "Source folder: `{}` does not exist.",
                source_folder
            ));
        }
        if !dest_path.is_dir() {
            return Err(format!(
                "Destination folder: `{}` does not exist.",
                destination_folder
            ));
        }

        // Determina i tipi di file da includere
        let include_all =
            backup_type == "total" || (backup_type == "custom" && file_types.is_empty());
        let file_types: Vec<&str> = file_types.iter().map(|s| s.as_str()).collect();

        // Calcola la durata del backup
        let start_time = Instant::now();

        // Riproduci suono di inizio backup
        play_sound("Sounds/bubblepop-254773.mp3");

        // Conta i file da copiare
        let total_files = count_files_in_directory(source_path).unwrap();

        let mut files_copied = 0;
        let mut total_copied_size = 0;
        // Esegui il backup
        if let Err(e) = backup_folder_with_stop(
            source_path,
            dest_path,
            include_all,
            &file_types,
            stop_rx,
            total_files,
            &mut files_copied,
            state,
            &mut total_copied_size,
        ) {
            play_sound("Sounds/incorrect-buzzer-sound-147336.mp3");
            return Err(format!("Backup failed: {}", e));
        } else {
            play_sound("Sounds/bellding-254774.mp3");
            let duration = start_time.elapsed().as_secs(); // Durata del backup in secondi

            // let total_size = get_total_size(source_path).map_err(|e| e.to_string())?; // Calcola i dati trasferiti in byte
            // Registra i dettagli del backup nelle analitiche
            let cpu_usage = get_cpu_usage();
            log_backup_data_to_csv(total_copied_size, duration, cpu_usage);
        }

        Ok(())
    } else {
        Err(
            "Configurazione non valida. Imposta una configurazione valida dal pannello di Backup!"
                .to_string(),
        )
    }
}

fn backup_folder_with_stop(
    source: &Path,
    destination: &Path,
    include_all: bool,
    file_types: &[&str],
    stop_rx: &Receiver<String>,
    total_files: u64,
    files_copied: &mut i32,
    state: &mut MyApp,
    total_copied_size: &mut u64,
) -> io::Result<()> {
    // Crea la directory di destinazione se non esiste
    if !destination.exists() {
        fs::create_dir_all(destination)?;
    }

    // Itera sui file e sottocartelle nella sorgente
    for entry in fs::read_dir(source)? {
        // Controlla se è stato ricevuto il comando di stop
        if let Ok(msg) = stop_rx.try_recv() {
            if msg == "stop" {
                play_sound("Sounds/incorrect-buzzer-sound-147336.mp3");
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "Backup interrotto dall'utente.",
                ));
            }
        }

        let entry = entry?;
        let path = entry.path();
        let dest_path = destination.join(entry.file_name());

        println!("Processing: {:?}", path);
        if path.is_dir() {
            println!("Entering directory: {:?}", path);
            // Esegui il backup ricorsivamente per le sottocartelle
            backup_folder_with_stop(
                &path,
                &dest_path,
                include_all,
                file_types,
                stop_rx,
                total_files,
                files_copied,
                state,
                total_copied_size,
            )?;
        } else if path.is_file() {
            // Copia il file se rientra nei criteri
            if include_all || matches_file_type(&path, file_types) {
                println!("Copying file: {:?} -> {:?}", path, dest_path);
                {
                    let mut current_file = state.current_file.lock().unwrap();
                    *current_file = Some(path.to_string_lossy().to_string()); // Salva il percorso completo
                }
                fs::copy(&path, &dest_path)?;

                // Aggiorna la dimensione totale dei dati copiati
                let file_size = path.metadata()?.len();
                *total_copied_size += file_size;

                // Aggiorna il progresso
                *files_copied += 1;
                println!("File copiati: {:?}", files_copied);
                let progress_value = *files_copied as f32 / total_files as f32;
                println!("Progresso: {:?}", progress_value);

                let mut progress_lock = state.progress.lock().unwrap();
                *progress_lock = progress_value;
            } else {
                println!("Skipping file: {:?}", path);
            }
        }
    }

    Ok(())
}

fn count_files_in_directory(path: &Path) -> io::Result<u64> {
    let mut file_count = 0;

    if path.is_dir() {
        // Itera attraverso tutte le voci nella directory
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            // Se è una directory, chiama ricorsivamente count_files_in_directory
            if entry_path.is_dir() {
                file_count += count_files_in_directory(&entry_path)?; // Aggiungi i file dalla sottocartella
            } else {
                file_count += 1; // Aggiungi 1 per ogni file
            }
        }
    }

    Ok(file_count)
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
        file_types
            .iter()
            .any(|&ft| ft.eq_ignore_ascii_case(&ext_with_dot))
    } else {
        false
    }
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
