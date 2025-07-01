use eframe::{egui, NativeOptions};

// Import from the parent crate modules
use chonker_tui::app::ChonkerApp;
use chonker_tui::database::ChonkerDatabase;

fn main() -> Result<(), eframe::Error> {
    // Initialize logging

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 800.0])
            .with_resizable(true)
            .with_title("CHONKER_SNYFTER - Document Processing Workbench"),
        ..Default::default()
    };

    eframe::run_native(
        "CHONKER_SNYFTER",
        options,
        Box::new(|cc| {
            // Initialize database for the GUI (simplified for now)
            let app = ChonkerApp::new(cc, None);
            Ok(Box::new(app))
        }),
    )
}
