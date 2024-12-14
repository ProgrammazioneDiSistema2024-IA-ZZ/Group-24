use std::fs::OpenOptions;
use std::io::{Write, BufWriter};
use chrono::Local;
use std::fs;

/// Funzione per inizializzare il file di log e scrivere l'intestazione se necessario
fn initialize_log_file(log_file: &str) -> std::io::Result<BufWriter<std::fs::File>> {
    // Apre o crea il file CSV
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)?;

    let mut writer = BufWriter::new(file);

    // Verifica se il file esiste e se è vuoto
    if let Ok(metadata) = fs::metadata(log_file) {
        if metadata.len() == 0 {
            // Scrive l'intestazione solo se il file è vuoto
            writeln!(
                writer,
                "Timestamp, Durata Trasferimento (s), Dati Trasferiti (byte), CPU Occupata (%)"
            )?;
        }
    } else {
        // Se il file non esiste, scrive l'intestazione
        writeln!(
            writer,
            "Timestamp, Durata Trasferimento (s), Dati Trasferiti (byte), CPU Occupata (%)"
        )?;
    }

    Ok(writer)
}

/// Registra i dettagli del backup nel file CSV
pub fn log_backup_data_to_csv(total_size: u64, duration: u64, cpu_usage: f32) {
    let log_file = "backup_log.csv"; // Percorso del file CSV

    // Inizializza il file di log
    let mut writer = match initialize_log_file(log_file) {
        Ok(writer) => writer,
        Err(e) => {
            eprintln!("Errore durante l'apertura o la creazione del file di log: {}", e);
            return;
        }
    };

    // Scrive i dettagli del backup
    let result = writeln!(
        writer,
        "{}, {}, {}, {}",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        duration,
        total_size,
        cpu_usage
    );

    // Gestisci eventuali errori nella scrittura
    if let Err(e) = result {
        eprintln!("Errore durante la scrittura dei dati nel file di log: {}", e);
    }

    // Forza il flush per scrivere effettivamente i dati nel file
    if let Err(e) = writer.flush() {
        eprintln!("Errore durante il flush del file di log: {}", e);
    }
}
