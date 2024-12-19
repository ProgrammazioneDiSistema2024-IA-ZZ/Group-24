use std::fs::OpenOptions;
use std::io::{Write, BufWriter};
use chrono::Local;
use std::fs;
use std::{ thread, time::Duration};
use systemstat::{System, Platform};

//// Funzione per inizializzare il file di log e scrivere l'intestazione se necessario
fn initialize_log_file(log_file: &str, header: &str) -> std::io::Result<BufWriter<std::fs::File>> {
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
            writeln!(writer, "{}", header)?; // Scrive l'intestazione
        }
    } else {
        // Se il file non esiste, scrive l'intestazione
        writeln!(writer, "{}", header)?; // Scrive l'intestazione
    }

    Ok(writer)
}

/// Funzione per registrare i dettagli del backup nel file CSV
pub fn log_backup_data_to_csv(total_size: u64, duration: u64, cpu_usage: f32) {
    let log_file = "backup_log.csv"; // Percorso del file CSV
    let header = "Timestamp, Durata Trasferimento (s), Dati Trasferiti (byte), CPU Occupata (%)";

    // Inizializza il file di log
    let mut writer = match initialize_log_file(log_file, header) {
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

/// Funzione per registrare il consumo di CPU in un file CSV ogni minuto
pub fn log_cpu_usage_to_csv() {
    let log_file = "cpu_usage_log.csv"; // Percorso del file CSV
    let header = "Timestamp, CPU Occupata (%)";

    // Inizializza il file di log
    let mut writer = match initialize_log_file(log_file, header) {
        Ok(writer) => writer,
        Err(e) => {
            eprintln!("Errore durante l'apertura o la creazione del file di log: {}", e);
            return;
        }
    };

    let sys = System::new(); // Crea una nuova istanza di System

    // Avvia il ciclo per raccogliere l'uso della CPU ogni minuto
    loop {
        // Ottieni il carico aggregato della CPU
        match sys.cpu_load_aggregate() {
            Ok(load) => {
                // Chiama done() dopo che è passato abbastanza tempo
                thread::sleep(Duration::from_secs(1)); // Attendi 1 secondo per avere un campione sufficiente

                match load.done() {
                    Ok(load) => {
                        // Scrive i dettagli nel file CSV ogni minuto
                        writeln!(
                            writer,
                            "{}, {}",
                            Local::now().format("%Y-%m-%d %H:%M:%S"),
                            load.user * 100.0 // CPU utilizzata in percentuale
                        ).unwrap_or_else(|e| eprintln!("Errore durante la scrittura dei dati nel file di log: {}", e));

                        // Forza il flush per scrivere effettivamente i dati nel file
                        if let Err(e) = writer.flush() {
                            eprintln!("Errore durante il flush del file di log: {}", e);
                        }
                    },
                    Err(e) => eprintln!("Errore nel recupero del carico della CPU: {}", e),
                }
            },
            Err(e) => eprintln!("Errore nel recupero del carico della CPU: {}", e),
        }

        // Aspetta un minuto prima di registrare di nuovo
        thread::sleep(Duration::from_secs(60)); // Aspetta 60 secondi prima di ripetere
    }
}