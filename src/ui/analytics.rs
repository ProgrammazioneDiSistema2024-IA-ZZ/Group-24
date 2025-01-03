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
            ui.separator();

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
            ui.separator();

            if let Some(cpu_data) = read_cpu_usage_data("cpu_usage_log.csv") {
                // Calcola gli ultimi 5 valori
                let total_points = cpu_data.len();
                let start_index = total_points.saturating_sub(unsafe { SHOWN_LOGS_CPU });
            
                // Creazione della tabella per visualizzare i log di utilizzo della CPU
                egui::Grid::new("cpu_usage_table")
                    .striped(true)
                    .show(ui, |ui| {
                        // Intestazioni della tabella
                        ui.label("Timestamp");
                        ui.label("CPU Usage (%)");
                        ui.end_row();
            
                        // Popola la tabella con i dati (gli ultimi `SHOWN_LOGS` valori)
                        for (timestamp, cpu_usage) in cpu_data.iter().skip(start_index) {
                            ui.label(timestamp);   // Timestamp
                            ui.label(format!("{:.2}%", cpu_usage)); // CPU Usage (%)
                            ui.end_row();                           // Fine della riga
                        }
                    });
            
                // Aggiungi il pulsante per vedere più log (i 5 valori più vecchi)
                unsafe {
                    if SHOWN_LOGS_CPU < total_points {
                        if ui.button("Show older logs").clicked() {
                            SHOWN_LOGS_CPU += 5; // Mostra 5 log in più
                        }
                    }
                }
            } else {
                ui.label("No CPU usage data available.");
            }
        });

        
                
        ui.separator();

        // Sezione Backup Log
        ui.vertical(|ui| {
            ui.label("Backup Log:");

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
                            ui.label("Duration (s)");
                            ui.label("Data (bytes)");
                            ui.label("CPU Usage (%)");
                            ui.end_row();

                            // Popola la tabella con i log di backup (gli ultimi SHOWN_LOGS_BACKUP log)
                            for entry in backup_entries.iter().rev().skip(total_entries.saturating_sub(SHOWN_LOGS_BACKUP)) {
                                ui.label(&entry.timestamp);                // Timestamp
                                ui.label(format!("{}", entry.duration));   // Duration (s)
                                ui.label(format!("{}", entry.data_transferred)); // Data (bytes)
                                ui.label(format!("{:.2}%", entry.cpu_usage));    // CPU Usage (%)
                                ui.end_row(); // Fine della riga
                            }
                        });

                    // Aggiungi il pulsante per mostrare più log se ce ne sono
                    if SHOWN_LOGS_BACKUP < total_entries {
                        if ui.button("Show more logs").clicked() {
                            SHOWN_LOGS_BACKUP += 5; // Mostra altri 5 log
                        }
                    } else {
                        ui.label("No more logs to display.");
                    }
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
