// Example: How to integrate the data visualization pane into ChonkerApp
// This shows the changes needed to display extracted data in the right-hand pane

use eframe::egui;
use crate::data_visualization::DataVisualizationPane;
use crate::extraction_integration::ExtractionIntegrator;
use std::path::PathBuf;

/// Extended ChonkerApp with data visualization
pub struct ChonkerAppWithViz {
    // Existing app state
    pub status_message: String,
    pub current_file: Option<PathBuf>,
    
    // New visualization components
    pub data_viz_pane: DataVisualizationPane,
    pub extraction_integrator: ExtractionIntegrator,
    pub processing_in_progress: bool,
}

impl ChonkerAppWithViz {
    pub fn new() -> Self {
        Self {
            status_message: "CHONKER ready! Select a PDF to process".to_string(),
            current_file: None,
            data_viz_pane: DataVisualizationPane::new(),
            extraction_integrator: ExtractionIntegrator::new(),
            processing_in_progress: false,
        }
    }

    /// Process a PDF file and update the visualization
    pub async fn process_pdf(&mut self, pdf_path: PathBuf) {
        self.processing_in_progress = true;
        self.status_message = format!("Processing {}...", pdf_path.display());
        self.current_file = Some(pdf_path.clone());

        // Clear previous data
        self.data_viz_pane.clear();

        // Process the document (this would be called from the UI thread)
        match self.extraction_integrator.process_document(&pdf_path).await {
            Ok(extracted_data) => {
                self.data_viz_pane.load_data(extracted_data);
                self.status_message = format!("✅ Successfully processed {}", pdf_path.display());
            }
            Err(e) => {
                self.status_message = format!("❌ Error processing {}: {}", pdf_path.display(), e);
            }
        }

        self.processing_in_progress = false;
    }

    /// Load sample data for testing
    pub fn load_sample_data(&mut self) {
        use crate::data_visualization::ExtractedData;
        let sample_data = ExtractedData::create_sample();
        self.data_viz_pane.load_data(sample_data);
        self.status_message = "📊 Sample data loaded".to_string();
    }
}

impl eframe::App for ChonkerAppWithViz {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Main layout with three panes
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Left pane: File browser and controls (30% width)
                ui.allocate_ui_with_layout(
                    ui.available_size() * egui::vec2(0.3, 1.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        self.render_file_browser(ui);
                    },
                );

                ui.separator();

                // Middle pane: Document preview (40% width)
                ui.allocate_ui_with_layout(
                    ui.available_size() * egui::vec2(0.4 / 0.7, 1.0), // Adjust for remaining space
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        self.render_document_preview(ui);
                    },
                );

                ui.separator();

                // Right pane: Data visualization (remaining width, ~30%)
                ui.allocate_ui_with_layout(
                    ui.available_size(),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        // This is where the extracted data is displayed!
                        self.data_viz_pane.render(ui);
                    },
                );
            });

            // Status bar at the bottom
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(&self.status_message);
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.processing_in_progress {
                        ui.spinner();
                        ui.label("Processing...");
                    }
                });
            });
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Cleanup if needed
    }
}

impl ChonkerAppWithViz {
    fn render_file_browser(&mut self, ui: &mut egui::Ui) {
        ui.heading("📁 File Browser");
        ui.separator();

        // File selection button
        if ui.button("📂 Select PDF").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("PDF", &["pdf"])
                .pick_file()
            {
                // In a real app, you'd spawn this as a task
                // For this example, we'll just show how it would work
                self.status_message = format!("Selected: {}", path.display());
                self.current_file = Some(path);
            }
        }

        // Process button
        if let Some(ref file_path) = self.current_file {
            ui.separator();
            ui.label(format!("📄 {}", file_path.file_name().unwrap().to_string_lossy()));
            
            if ui.button("⚡ Process Document").clicked() && !self.processing_in_progress {
                // In a real app, spawn async task here
                // self.process_pdf(file_path.clone()).await;
                self.status_message = "Processing would start here (async)".to_string();
            }
        }

        ui.separator();

        // Quick actions
        ui.heading("🎯 Quick Actions");
        if ui.button("📊 Load Sample Data").clicked() {
            self.load_sample_data();
        }

        if ui.button("🗑️ Clear Data").clicked() {
            self.data_viz_pane.clear();
            self.status_message = "Data cleared".to_string();
        }

        ui.separator();

        // Processing options
        ui.heading("⚙️ Options");
        ui.checkbox(&mut self.data_viz_pane.show_metadata, "Show Metadata");
        ui.checkbox(&mut self.data_viz_pane.show_qualifiers, "Show Qualifiers");
    }

    fn render_document_preview(&mut self, ui: &mut egui::Ui) {
        ui.heading("📄 Document Preview");
        ui.separator();

        if let Some(ref file_path) = self.current_file {
            ui.label(format!("File: {}", file_path.display()));
            
            // In a real implementation, you'd show PDF preview here
            // using the MuPdfViewer or similar component
            ui.centered_and_justified(|ui| {
                ui.label("PDF preview would appear here");
                ui.label("(Use MuPdfViewer component)");
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No document selected");
                ui.label("Select a PDF file to preview");
            });
        }
    }
}

/// Example of how to integrate with existing ChonkerApp
/// Add this to your existing ChonkerApp implementation:
/*
use crate::data_visualization::DataVisualizationPane;
use crate::extraction_integration::ExtractionIntegrator;

pub struct ChonkerApp {
    // ... existing fields ...
    
    // Add these new fields:
    pub data_viz_pane: DataVisualizationPane,
    pub extraction_integrator: ExtractionIntegrator,
}

impl ChonkerApp {
    pub fn new(cc: &eframe::CreationContext, database: Option<ChonkerDatabase>) -> Self {
        Self {
            // ... existing initialization ...
            
            // Add these:
            data_viz_pane: DataVisualizationPane::new(),
            extraction_integrator: ExtractionIntegrator::new(),
        }
    }
    
    // In your render method, replace the right pane content with:
    fn render_right_pane(&mut self, ui: &mut egui::Ui) {
        self.data_viz_pane.render(ui);
    }
    
    // Add method to process documents
    pub async fn process_document(&mut self, pdf_path: PathBuf) -> Result<(), anyhow::Error> {
        let extracted_data = self.extraction_integrator.process_document(&pdf_path).await?;
        self.data_viz_pane.load_data(extracted_data);
        Ok(())
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = ChonkerAppWithViz::new();
        assert_eq!(app.status_message, "CHONKER ready! Select a PDF to process");
        assert!(!app.processing_in_progress);
    }

    #[test]
    fn test_sample_data_loading() {
        let mut app = ChonkerAppWithViz::new();
        app.load_sample_data();
        assert!(app.data_viz_pane.extracted_data.is_some());
        assert!(app.status_message.contains("Sample data loaded"));
    }
}
