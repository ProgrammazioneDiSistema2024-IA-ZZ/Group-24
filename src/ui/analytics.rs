use eframe::egui::{self, RichText, Ui, Color32};
use std::fs;

static mut SHOWN_LOGS_CPU: usize = 5;
static mut SHOWN_LOGS_BACKUP: usize = 5;

pub fn show_analytics_panel(ui: &mut Ui) {
    // Usa ScrollArea per tutto il contenuto
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.vertical(|ui| {
            // Sezione di panoramica
            ui.heading(RichText::new("Section 1: Overview").color(Color32::from_rgb(0x87, 0xCE, 0xFA)));

            // Calcola statistiche dalla CPU
            if let Some(cpu_data) = read_cpu_usage_data("cpu_usage_log.csv") {
                let cpu_values: Vec<f32> = cpu_data.iter().map(|(_, cpu)| *cpu).collect();
                let min_cpu = cpu_values.iter().cloned().fold(f32::INFINITY, f32::min);
                let max_cpu = cpu_values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                let avg_cpu = cpu_values.iter().cloned().sum::<f32>() / cpu_values.len() as f32;

                ui.horizontal(|ui| {
                    ui.label("CPU Min:");
                    ui.label(format!("{:.2}%", min_cpu));
                });
                ui.horizontal(|ui| {
                    ui.label("CPU Max:");
                    ui.label(format!("{:.2}%", max_cpu));
                });
                ui.horizontal(|ui| {
                    ui.label("CPU Avg:");
                    ui.label(format!("{:.2}%", avg_cpu));
                });
            } else {
                ui.label("No CPU statistics available.");
            }

            // Mostra il numero di backup logs
            if let Some(backup_entries) = read_backup_log("backup_log.csv") {
                ui.horizontal(|ui| {
                    ui.label("Total Backup Logs:");
                    ui.label(format!("{}", backup_entries.len()));
                });
            } else {
                ui.label("No backup logs available.");
            }
        });

        ui.separator();

        ui.vertical(|ui| {
            ui.heading(RichText::new("Section 2: CPU Usage over Time").color(Color32::from_rgb(0x87, 0xCE, 0xFA)));

            unsafe {
                if let Some(cpu_data) = read_cpu_usage_data("cpu_usage_log.csv") {
                    // Calcola gli ultimi 5 valori
                    let total_points = cpu_data.len();
                
                    // Creazione della tabella per visualizzare i log di utilizzo della CPU
                    egui::Grid::new("cpu_usage_table")
                        .striped(true)
                        .show(ui, |ui| {
                            // Intestazioni della tabella
                            ui.label("Timestamp");
                            ui.label("CPU Usage (%)");
                            ui.end_row();
                
                            // Popola la tabella con i dati (gli ultimi `SHOWN_LOGS` valori)
                            for (timestamp, cpu_usage) in cpu_data.iter().rev().take(SHOWN_LOGS_CPU) {
                                ui.label(timestamp);   // Timestamp
                                ui.label(format!("{:.2}%", cpu_usage)); // CPU Usage (%)
                                ui.end_row();                           // Fine della riga
                            }
                        });
                    ui.horizontal(|ui| {
                        // Aggiungi il pulsante per vedere più log (i 5 valori più vecchi)
                        if SHOWN_LOGS_CPU < total_points {
                            if ui.button("Show more").clicked() {
                                SHOWN_LOGS_CPU += 5; // Mostra 5 log in più
                            }
                        }
                        // Mostra il pulsante per ripristinare i log a 5
                        if SHOWN_LOGS_CPU > 5 {
                            if ui.button("Restore").clicked() {
                                SHOWN_LOGS_CPU = 5; // Ripristina a 5 log
                            }
                        }
                    });
                } else {
                    ui.label("No CPU usage data available.");
                }
            }
        });
                
        ui.separator();

        // Sezione Backup Log
        ui.vertical(|ui| {
            ui.heading(RichText::new("Section 3: Backup Log").color(Color32::from_rgb(0x87, 0xCE, 0xFA)));

            // Rendi sicura la variabile globale con unsafe
            unsafe {
                if let Some(backup_entries) = read_backup_log("backup_log.csv") {
                    // Inverti l'ordine degli entry per mostrare prima i più recenti
                    let total_entries = backup_entries.len();

                    // Creazione della tabella per visualizzare i log di backup
                    egui::Grid::new("backup_log_table")
                        .striped(true)
                        .show(ui, |ui| {
                            // Intestazioni della tabella
                            ui.label("Timestamp");
                            ui.label("Duration");
                            ui.label("Data");
                            ui.label("CPU Usage (%)");
                            ui.end_row();

                            // Popola la tabella con i log di backup (gli ultimi SHOWN_LOGS_BACKUP log)
                            for entry in backup_entries.iter().rev().take(SHOWN_LOGS_BACKUP) {
                                ui.label(&entry.timestamp);                // Timestamp
                                ui.label(format_duration(entry.duration));   // Durata formattata
                                ui.label(format_data_size(entry.data_transferred)); // Data (formattata)
                                ui.label(format!("{:.2}%", entry.cpu_usage));    // CPU Usage (%)
                                ui.end_row(); // Fine della riga
                            }
                        });
                    ui.horizontal(|ui| {
                        // Aggiungi il pulsante per mostrare più log se ce ne sono
                        if SHOWN_LOGS_BACKUP < total_entries {
                            if ui.button("Show more").clicked() {
                                SHOWN_LOGS_BACKUP += 5; // Mostra altri 5 log
                            }
                        } 
                        // Mostra il pulsante per ripristinare i log a 5
                        if SHOWN_LOGS_BACKUP > 5 {
                            if ui.button("Restore").clicked() {
                                SHOWN_LOGS_BACKUP = 5; // Ripristina a 5 log
                            }
                        }
                    });
                } else {
                    ui.label("No backup log data available.");
                }
            }
        });
    });
}

// Funzione per leggere i dati del file CSV di CPU usage
fn read_cpu_usage_data(file_path: &str) -> Option<Vec<(String, f32)>> {
    if let Ok(data) = fs::read_to_string(file_path) {
        let mut cpu_data = vec![];

        // Salta la prima riga (intestazione)
        let mut lines = data.lines();
        lines.next(); // Salta la prima riga

        // Cicla sulle righe e raccoglie i dati
        for line in lines { // Usa lines già iterato
            let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if fields.len() == 2 {
                if let Ok(cpu_usage) = fields[1].parse::<f32>() {
                    cpu_data.push((fields[0].to_string(), cpu_usage)); // Aggiungi la tupla con il timestamp e cpu_usage
                }
            }
        }
        Some(cpu_data)
    } else {
        None
    }
}


// Funzione per leggere i dati dal file `backup_log.csv`
fn read_backup_log(file_path: &str) -> Option<Vec<BackupLogEntry>> {
    if let Ok(data) = fs::read_to_string(file_path) {
        let mut entries = vec![];
        for line in data.lines().skip(1) { // Salta l'intestazione
            let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if fields.len() == 4 {
                if let (Ok(duration), Ok(data_transferred), Ok(cpu_usage)) = (
                    fields[1].parse::<u64>(),
                    fields[2].parse::<u64>(),
                    fields[3].parse::<f32>(),
                ) {
                    entries.push(BackupLogEntry {
                        timestamp: fields[0].to_string(),
                        duration,
                        data_transferred,
                        cpu_usage,
                    });
                }
            }
        }
        Some(entries)
    } else {
        None
    }
}

// Struttura per i log di backup
#[derive(Debug)]
struct BackupLogEntry {
    timestamp: String,
    duration: u64,      // Durata in secondi
    data_transferred: u64, // Dati trasferiti in byte
    cpu_usage: f32,     // Utilizzo della CPU in percentuale
}

fn format_data_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn format_duration(duration: u64) -> String {
    const SECOND: u64 = 1;
    const MINUTE: u64 = 60 * SECOND;
    const HOUR: u64 = 60 * MINUTE;

    if duration >= HOUR {
        let hours = duration / HOUR;
        let minutes = (duration % HOUR) / MINUTE;
        let seconds = (duration % MINUTE) / SECOND;
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if duration >= MINUTE {
        let minutes = duration / MINUTE;
        let seconds = duration % MINUTE;
        format!("{}m {}s", minutes, seconds)
    } else if duration >= SECOND {
        format!("{}s", duration)
    } else {
        "~1s".to_string()  // Mostra "~1s" per durate inferiori a 1 secondo
    }
}