mod ui;

use eframe::{egui, App, NativeOptions};
use crate::ui::AppState;

fn main() -> Result<(), eframe::Error> {
    // Configure the application window
    let options = NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)), // Window size
        resizable: false, // Disable resizing
        ..Default::default()
    };

    // Launch the app with the default state
    eframe::run_native(
        "Backup Application", // Window title
        options,
        Box::new(|_cc| Box::new(AppState::default())), // App initialization
    )
}

impl App for AppState {
    /// Update function that draws the UI and handles events
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::main_panel(ctx, self); // Delegate to main panel rendering
    }
}
