use eframe::egui::{self, RichText};
use egui::Color32;
use crate::ui::AppState;
use crate::utils::{toggle_auto_start, update_config_file_display};

/// Show the info panel
pub fn show_info_panel(ui: &mut egui::Ui, state: &mut AppState) {

    //------------ AVVIO AUTOMATICO ---------------
    let mut auto_start_enabled = state.auto_start_enabled;
    // Checkbox per abilitare/disabilitare l'auto-start
    ui.checkbox(&mut auto_start_enabled, "Avvio automatico all'avvio del sistema");

    // Quando l'utente cambia lo stato della checkbox, aggiorniamo la configurazione
    if auto_start_enabled != state.auto_start_enabled {
        state.auto_start_enabled = auto_start_enabled;
        toggle_auto_start(auto_start_enabled); // Cambia il registro di Windows
    }
    ui.separator(); // Separatore tra le sezioni

    // ------------ ESEGUI GUI ALL'AVVIO ---------------
    let mut run_gui_enabled = state.run_gui;
    // Checkbox per abilitare/disabilitare l'esecuzione della GUI all'avvio
    ui.checkbox(&mut run_gui_enabled, "Esegui GUI all'avvio");

    // Quando l'utente cambia lo stato della checkbox, aggiorniamo la configurazione
    if run_gui_enabled != state.run_gui {
        state.run_gui = run_gui_enabled;
        if let Err(_err) = update_config_file_display(run_gui_enabled) {
            ui.colored_label(egui::Color32::RED, "Errore di aggiornamento.");
        }
    }
    
    ui.separator(); // Separatore tra le sezioni

    // ------------ MOSTRA INFORMAZIONI ----------------
    ui.label(
        RichText::new("Welcome to Backup")
            .color(Color32::from_rgb(0x87, 0xCE, 0xFA))
            .text_style(egui::TextStyle::Heading),
    );
    ui.strong("Thank you for choosing our Backup application. ");
    ui.label("This tool helps protect your files in a selected folder, making it especially useful in case of screen malfunctions. You can choose to save all the files or only specific types based on your preferences.");
    ui.separator(); // Separatore tra le sezioni

    // Sezione su come attivare il backup
    ui.label(
        RichText::new("How to Activate Backup in Case of Screen Malfunction")
            .color(Color32::from_rgb(0x87, 0xCE, 0xFA))
            .text_style(egui::TextStyle::Heading),
    );

    ui.strong("Preparing the Initial Command");
    ui.label("To begin, use your mouse to trace the outline of your screen as precisely as possible. This will trigger the backup process. Once detected, a confirmation sound will play.");

    ui.strong("Second Command to Start the Backup");
    ui.label("Next, draw a horizontal line with your mouse. Upon completion, you will hear another confirmation sound, indicating that the backup process has been activated.");

    ui.separator(); // Separatore tra le sezioni

    // Sezione di gestione della configurazione
    ui.label(
        RichText::new("Configuration Management")
            .color(Color32::from_rgb(0x87, 0xCE, 0xFA))
            .text_style(egui::TextStyle::Heading),
    );

    ui.strong("Source Folder");
    ui.label("In the Backup Panel, you can select the source folder, which contains the files to be backed up. This is where the files will be read from.");

    ui.strong("Destination Folder");
    ui.label("Choose the destination folder where the files will be stored. Ensure this folder is correctly set to avoid overwriting important data.");

    ui.strong("File Types to Save");
    ui.label("You can choose to back up all the files in the source folder or filter by specific file types. This allows you to back up only important files.");

    ui.label("Don't forget to click 'Save' to confirm your settings!");

    ui.separator(); // Separatore tra le sezioni

    // Sezione delle statistiche di monitoraggio
    ui.label(
        RichText::new("Monitoring Statistics")
            .color(Color32::from_rgb(0x87, 0xCE, 0xFA))
            .text_style(egui::TextStyle::Heading),
    );

    ui.label("The monitoring system provides comprehensive and user-friendly control over operations.");
    ui.label("It includes an analytics panel that displays CPU usage statistics, backup history, and detailed information about recent activities.");
    ui.label("During a backup process, the system offers real-time updates on the progress percentage, allowing users to track the operation's advancement.");
    ui.label("Additionally, it promptly alerts users to any errors or anomalies, ensuring timely and secure management of critical tasks.");

}
