use eframe::IconData;
use image::RgbaImage;
use rodio::{source::Source, Decoder, OutputStream};
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use toml;
#[cfg(windows)]
use winapi::um::winuser::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
#[cfg(not(windows))]
extern crate x11;
#[cfg(not(windows))]
use x11::xlib;

//Questo approccio è specifico per Windows
pub fn get_screen_resolution() -> Option<(u32, u32)> {
    #[cfg(windows)]
    {
        unsafe {
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
#[derive(Deserialize, Debug)]
struct Config {
    source_folder: String,
    destination_folder: String,
    backup_type: String,
    file_types: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Configuration {
    Created,
    Build(String, String, String, Vec<String>),
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
