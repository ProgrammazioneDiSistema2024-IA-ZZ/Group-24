use eframe::IconData;
use image::RgbaImage;
use rodio::{source::Source, Decoder, OutputStream};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::{self, BufReader, Read, Write};
use std::path::Path;
use std::ptr;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{error::Error, thread};
use toml;
#[cfg(windows)]
use winapi::um::sysinfoapi::GetTickCount64;
#[cfg(windows)]
use winapi::um::winuser::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
#[cfg(windows)]
use winapi::um::winuser::SetProcessDPIAware;
#[cfg(not(windows))]
extern crate x11;
#[cfg(not(windows))]
use x11::xlib;

use crate::ui::AppState;
use crate::LockFileData;

// Funzione per ottenere il tempo di avvio del sistema
#[cfg(windows)]
pub fn get_system_boot_time() -> SystemTime {
    // Ottieni il tempo di attività del sistema in millisecondi
    let uptime_ms = unsafe { GetTickCount64() };
    let now = SystemTime::now(); // può generare discrepanza

    // Calcola il tempo di boot
    let boot_time = now - Duration::from_millis(uptime_ms);

    // Azzera i millisecondi --> per evitare discrepanze
    let duration_since_epoch = boot_time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::new(0, 0));
    let rounded_seconds = Duration::new(duration_since_epoch.as_secs(), 0);
    UNIX_EPOCH + rounded_seconds
}

#[cfg(not(windows))]
pub fn get_system_boot_time() -> std::time::SystemTime {
    use std::fs::File;
    use std::io::Read;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let mut file = File::open("/proc/uptime").expect("Failed to open /proc/uptime");
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let uptime_seconds: f64 = content
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .expect("Failed to parse uptime");

    // Ottieni il tempo corrente
    let now = SystemTime::now();

    // Calcola il tempo di boot
    let boot_time = now - Duration::from_secs_f64(uptime_seconds);

    // Arrotonda al secondo più vicino
    let duration_since_epoch = boot_time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::new(0, 0));
    let rounded_seconds = duration_since_epoch.as_secs(); // Arrotondamento verso il basso

    UNIX_EPOCH + Duration::from_secs(rounded_seconds)
}

#[cfg(windows)]
pub fn toggle_auto_start(enable: bool) {
    use std::env;
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            KEY_SET_VALUE,
        )
        .unwrap();
    let program_name = "BackupGroup24"; // Nome della tua applicazione

    // Ottieni il percorso dell'eseguibile dinamicamente
    let current_dir = env::current_exe().unwrap();
    let app_path = current_dir.to_str().unwrap(); // Converte il percorso in stringa

    if enable {
        key.set_value(program_name, &app_path).unwrap();
    } else {
        key.delete_value(program_name).unwrap();
    }
}

#[cfg(not(windows))]
pub fn toggle_auto_start(enable: bool) {
    use std::env;
    use std::fs::{self, OpenOptions};
    use std::io::Write;

    let autostart_dir = format!("{}/.config/autostart", env::var("HOME").unwrap());
    let autostart_file = format!("{}/BackupGroup24.desktop", autostart_dir);

    if enable {
        // Crea il file .desktop per l'avvio automatico
        let desktop_entry = format!(
            r#"[Desktop Entry]
            Type=Application
            Exec={}
            Hidden=false
            NoDisplay=false
            X-GNOME-Autostart-enabled=true
            Name=BackupGroup24
            Comment=Backup application"#,
            env::current_exe().unwrap().to_str().unwrap()
        );

        fs::create_dir_all(&autostart_dir).unwrap();
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&autostart_file)
            .unwrap();
        file.write_all(desktop_entry.as_bytes()).unwrap();
    } else {
        // Rimuovi il file .desktop
        let _ = fs::remove_file(&autostart_file);
    }
}

#[cfg(windows)]
pub fn check_auto_start_status() -> bool {
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
        .unwrap();
    let program_name = "BackupGroup24"; // Il nome del programma da verificare
    key.get_value::<String, _>(program_name).is_ok()
}

#[cfg(not(windows))]
pub fn check_auto_start_status() -> bool {
    use std::env;
    let autostart_file = format!(
        "{}/.config/autostart/BackupGroup24.desktop",
        env::var("HOME").unwrap()
    );
    std::path::Path::new(&autostart_file).exists()
}

//Questo approccio è specifico per Windows
pub fn get_screen_resolution() -> Option<(u32, u32)> {
    #[cfg(windows)]
    {
        unsafe {
            SetProcessDPIAware();

            // Usa le funzioni di WinAPI per ottenere la risoluzione dello schermo
            let width = GetSystemMetrics(SM_CXSCREEN);
            let height = GetSystemMetrics(SM_CYSCREEN);

            if width > 0 && height > 0 {
                Some((width as u32, height as u32))
            } else {
                None
            }
        }
    }
    #[cfg(not(windows))]
    {
        unsafe {
            // Usa le X11 API per ottenere la risoluzione dello schermo su Linux
            let display = xlib::XOpenDisplay(ptr::null());
            if display.is_null() {
                return None; // Se non riesce a ottenere il display, ritorna None
            }

            let screen = xlib::XDefaultScreen(display);
            let width = xlib::XDisplayWidth(display, screen) as u32;
            let height = xlib::XDisplayHeight(display, screen) as u32;

            xlib::XCloseDisplay(display); // Chiudi il display dopo l'uso

            Some((width, height))
        }
    }
}

pub fn play_sound(file_path: &str) {
    let file_path = file_path.to_string(); // Copia file_path in una String
    std::thread::spawn(move || {
        if let Ok(file) = File::open(&file_path) {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let source = Decoder::new(BufReader::new(file)).unwrap();
            let duration = source
                .total_duration()
                .unwrap_or_else(|| std::time::Duration::from_secs(3));
            stream_handle.play_raw(source.convert_samples()).unwrap();
            std::thread::sleep(duration); // Mantieni il thread attivo
        } else {
            println!("Errore: file audio non trovato o non apribile.");
        }
    });
}
#[derive(Deserialize, Debug, Serialize)]
struct Config {
    source_folder: String,
    destination_folder: String,
    backup_type: String,
    file_types: Vec<String>,
    display: bool
}

#[derive(Debug, Clone)]
pub enum Configuration {
    Created,
    Build(String, String, String, Vec<String>, bool),
    Error,
}

pub fn manage_configuration_file() -> Configuration {
    let config_path = "config_build.toml";

    // Controlla se il file esiste
    if !Path::new(config_path).exists() {
        // Crea il file con una configurazione di default
        let default_config = r#"
source_folder = ''
destination_folder = ''
backup_type = 'total'
file_types = []
display = true
"#;
        if fs::write(config_path, default_config).is_err() {
            return Configuration::Error;
        }
        return Configuration::Created;
    }

    // Prova a leggere il file
    let config_content = match fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(_) => return Configuration::Error,
    };

    // Prova a fare il parsing del contenuto
    let parsed: Config = match toml::from_str(&config_content) {
        Ok(config) => config,
        Err(_) => {
            // Se il parsing fallisce, ricrea il file con la configurazione di default
            let default_config = r#"
source_folder = ''
destination_folder = ''
backup_type = 'total'
file_types = []
display = true
"#;
            if fs::write(config_path, default_config).is_err() {
                return Configuration::Error;
            }
            return Configuration::Created;
        }
    };

    // Verifica che tutti i campi siano rispettati
    if parsed.source_folder.is_empty()
        || parsed.destination_folder.is_empty()
        || parsed.backup_type.is_empty()
    {
        // vuol dire che la configurazione non è completa, quindi il detector non può partire
        return Configuration::Created;
    }

    // Tutti i campi sono validi, ritorna Configuration::Build
    Configuration::Build(
        parsed.source_folder,
        parsed.destination_folder,
        parsed.backup_type,
        parsed.file_types,
        parsed.display,
    )
}

/// Loads an image from a given path and converts it to RGBA format.
/// Returns an `IconData` structure containing the image data and dimensions,
/// or an error if the image cannot be loaded.
pub fn load_image_as_icon(path: &str) -> Result<IconData, Box<dyn Error>> {
    // Load the image using the image crate
    let img = image::open(path)?;

    // Convert to RGBA8 (RGBA format)
    let rgba_img: RgbaImage = img.to_rgba8();

    // Get image dimensions
    let (width, height) = rgba_img.dimensions();

    // Convert the RGBA image into raw byte data
    let rgba_data = rgba_img.into_raw();

    // Return the IconData structure
    Ok(IconData {
        rgba: rgba_data,
        width,
        height,
    })
}

pub fn monitor_lock_file(
    lock_file_path: &'static str,
    shared_state: Arc<Mutex<AppState>>,
    tx: mpsc::Sender<String>,
) {
    thread::spawn(move || {
        // Ottieni l'ultimo tempo di modifica del file di lock
        let mut last_modified = fs::metadata(lock_file_path)
            .and_then(|metadata| Ok(metadata.modified().unwrap()))
            .unwrap_or_else(|_| SystemTime::now());

        loop {
            // Controlla se il file è stato modificato
            if let Ok(metadata) = fs::metadata(lock_file_path) {
                let modified = metadata.modified().unwrap();
                if modified > last_modified {
                    last_modified = modified;

                    // Leggi il contenuto del file TOML
                    if let Ok(mut file) = OpenOptions::new().read(true).open(lock_file_path) {
                        let mut content = String::new();
                        if let Ok(_) = file.read_to_string(&mut content) {
                            // Analizza il file TOML
                            if let Ok(mut parsed_data) = toml::from_str::<LockFileData>(&content) {
                                // Verifica se il campo show_gui è true
                                if parsed_data.show_gui {
                                    // Controlla lo stato della GUI prima di inviare il messaggio
                                    let mut state = shared_state.lock().unwrap();
                                    if !state.display {
                                        if let Err(err) = tx.send("showGUI".to_string()) {
                                            eprintln!("Failed to send showGUI message: {}", err);
                                            state.display = false; // Assicurati che lo stato rifletta correttamente il fallimento
                                        } else {
                                            println!("Message sent to show GUI.");
                                        }
                                    } else {
                                        println!(
                                            "GUI already active. Skipping message from lock file."
                                        );
                                    }

                                    // Imposta show_gui a false nel file
                                    parsed_data.show_gui = false;

                                    // Scrivi il file aggiornato
                                    if let Ok(updated_content) = toml::to_string(&parsed_data) {
                                        if let Err(e) = fs::write(lock_file_path, updated_content) {
                                            eprintln!("Failed to update lock file: {}", e);
                                        } else {
                                            println!("Lock file updated successfully.");
                                        }
                                    }
                                } else {
                                    println!("showGUI is false. No action taken.");
                                }
                            } else {
                                eprintln!("Failed to parse lock file content.");
                            }
                        } else {
                            eprintln!("Failed to read lock file.");
                        }
                    }
                }
            }
            thread::sleep(Duration::from_secs(1)); // Controlla ogni secondo
        }
    });
}


pub fn update_config_file_display(display: bool) -> io::Result<()> {
    // Carichiamo il contenuto del file config_build.toml
    let path = "config_build.toml"; // Modifica il percorso se necessario
    let content = fs::read_to_string(path)?;

    // Deserializziamo il contenuto in un'istanza di Config
    let mut config_file_data: Config = toml::from_str(&content)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;

    // Aggiorniamo il valore del campo `display`
    config_file_data.display = display;

    // Serializziamo di nuovo i dati aggiornati
    let updated_content = toml::to_string(&config_file_data)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;

    // Scriviamo i dati aggiornati nel file config_build.toml
    let mut file = fs::File::create(path)?;
    file.write_all(updated_content.as_bytes())?;

    Ok(())
}

pub fn read_config_file_display() -> bool {
    // Percorso del file config_build.toml
    let path = "config_build.toml"; // Modifica il percorso se necessario

    // Carichiamo il contenuto del file config_build.toml
    let content = fs::read_to_string(path);
    if content.is_err() {
        return false; // Fallimento nella lettura del file
    }

    // Deserializziamo il contenuto in un'istanza di Config
    let config_file_data: Result<Config, _> = toml::from_str(&content.unwrap());
    if let Ok(data) = config_file_data {
        return data.display; // Restituisce il valore del campo `display`
    }
    
    false // Fallimento nella deserializzazione o assenza del campo `display`
}

pub fn set_display_true() -> io::Result<()> {
    // Carichiamo il contenuto del file config_build.toml
    let path = "config_build.toml"; // Modifica il percorso se necessario
    let content = fs::read_to_string(path)?;

    // Deserializziamo il contenuto in un'istanza di Config
    let mut config_file_data: Config = toml::from_str(&content)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;

    // Aggiorniamo il valore del campo `display`
    config_file_data.display = true;

    // Serializziamo di nuovo i dati aggiornati
    let updated_content = toml::to_string(&config_file_data)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;

    // Scriviamo i dati aggiornati nel file config_build.toml
    let mut file = fs::File::create(path)?;
    file.write_all(updated_content.as_bytes())?;

    Ok(())
}
