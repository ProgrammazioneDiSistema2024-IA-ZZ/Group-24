use eframe::egui::{self, Ui};
use eframe::egui::plot::{Line, Plot, PlotPoints};
use std::fs;

pub fn show_analytics_panel(ui: &mut Ui) {
    // Usa ScrollArea per tutto il contenuto
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.vertical(|ui| {
            // Sezione di panoramica
            ui.heading("Section 1: Overview");
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

        // Sezione grafico CPU Usage con Scroll Area
        ui.vertical(|ui| {
            ui.heading("Section 2: Detailed Analytics");
            ui.separator();
            ui.label("CPU Usage over Time:");
        
            if let Some(cpu_data) = read_cpu_usage_data("cpu_usage_log.csv") {
                // Crea i punti per il grafico: ogni punto è composto da (tempo in minuti, uso CPU)
                let plot_points = cpu_data
                    .iter()
                    .filter(|&(_, value)| *value > 0.0) // Filtra solo i valori di CPU maggiori di 0
                    .map(|(time, value)| {
                        // La variabile 'time' è già in minuti e 'value' è l'uso della CPU
                        [*time, *value as f64 ]
                    })
                    .collect::<Vec<_>>();
        
                if !plot_points.is_empty() {
                    // Linea principale del grafico
                    let line = Line::new(PlotPoints::from(plot_points))
                        .color(egui::Color32::from_rgb(100, 200, 100)) // Colore verde chiaro
                        .width(2.0); // Imposta lo spessore della linea
        
                    // Creazione e visualizzazione del grafico
                    Plot::new("cpu_usage_plot")
                        .view_aspect(2.0) // Imposta l'aspetto orizzontale
                        .include_x(0.0) // Assicura che l'asse X parta da 0
                        .include_y(0.0) // Assicura che l'asse Y parta da 0
                        .show(ui, |plot_ui| {
                            plot_ui.line(line);
                        });
                } else {
                    ui.label("No valid CPU usage data available.");
                }
            } else {
                ui.label("No CPU usage data available.");
            }
        });
        
        ui.separator();

        // Sezione Backup Log
        ui.vertical(|ui| {
            ui.label("Backup Log:");
            if let Some(backup_entries) = read_backup_log("backup_log.csv") {
                for entry in backup_entries.iter().take(5) { // Mostra solo i primi 5 log
                    ui.horizontal(|ui| {
                        ui.label(format!("Timestamp: {}", entry.timestamp));
                        ui.label(format!("Duration: {} s", entry.duration));
                        ui.label(format!("Data: {} bytes", entry.data_transferred));
                        ui.label(format!("CPU: {:.2} %", entry.cpu_usage));
                    });
                }
            } else {
                ui.label("No backup log data available.");
            }
        });
    });
}

// Funzione per leggere i dati del file CSV di CPU usage
fn read_cpu_usage_data(file_path: &str) -> Option<Vec<(f64, f32)>> {
    if let Ok(data) = fs::read_to_string(file_path) {
        let mut cpu_data = vec![];

        // Salta la prima riga (intestazione)
        let mut lines = data.lines();
        lines.next(); // Salta la prima riga

        for line in lines {
            let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if fields.len() == 2 {
                if let (Ok(cpu_usage), timestamp) = (fields[1].parse::<f32>(), fields[0].to_string()) {
                    let time_in_minutes = timestamp_to_minutes(&timestamp); // Converti timestamp in minuti
                    cpu_data.push((time_in_minutes, cpu_usage));
                }
            }
        }
        Some(cpu_data)
    } else {
        None
    }
}


fn timestamp_to_minutes(timestamp: &str) -> f64 {
    let parts: Vec<&str> = timestamp.split_whitespace().collect();
    if parts.len() == 2 {
        let time = parts[1].split(':').collect::<Vec<&str>>();
        if time.len() == 3 {
            let hours = time[0].parse::<u64>().unwrap_or(0);
            let minutes = time[1].parse::<u64>().unwrap_or(0);
            let seconds = time[2].parse::<u64>().unwrap_or(0);
            return (hours * 60 + minutes) as f64 + (seconds as f64 / 60.0); // Aggiungi la parte dei secondi
        }
    }
    0.0
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
