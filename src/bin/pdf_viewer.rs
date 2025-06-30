use eframe::{egui, App, NativeOptions};
use std::process::Command;
use std::path::Path;

struct PDFApp {
    pdf_images: Vec<egui::ColorImage>, // Store rendered page images
    markdown_content: String,
    texture_handles: Vec<Option<egui::TextureHandle>>,
}

impl PDFApp {
    pub fn new(pdf_path: &str, markdown_file: &str) -> Self {
        let mut pdf_images = Vec::new();
        
        // Convert PDF to PNG images using pdftoppm
        println!("Converting PDF to images: {}", pdf_path);
        
        let temp_dir = std::env::temp_dir();
        let temp_prefix = temp_dir.join("pdf_page");
        
        // Use pdftoppm to convert PDF to PNG images
        let output = Command::new("pdftoppm")
            .args([
                "-png",
                "-r", "150", // 150 DPI for good quality
                pdf_path,
                temp_prefix.to_str().unwrap()
            ])
            .output();
            
        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("PDF conversion successful");
                    
                    // Look for generated PNG files
                    let mut page_num = 1;
                    loop {
                        let png_path = format!("{}-{:02}.png", temp_prefix.to_str().unwrap(), page_num);
                        if Path::new(&png_path).exists() {
                            println!("Loading page {}: {}", page_num, png_path);
                            
                            if let Ok(img_bytes) = std::fs::read(&png_path) {
                                if let Ok(dynamic_img) = image::load_from_memory(&img_bytes) {
                                    let rgba_img = dynamic_img.to_rgba8();
                                    let size = [rgba_img.width() as usize, rgba_img.height() as usize];
                                    let color_img = egui::ColorImage::from_rgba_unmultiplied(size, &rgba_img);
                                    pdf_images.push(color_img);
                                    println!("Page {} loaded: {}x{}", page_num, size[0], size[1]);
                                } else {
                                    println!("Failed to decode image for page {}", page_num);
                                }
                            } else {
                                println!("Failed to read image file for page {}", page_num);
                            }
                            
                            // Clean up the temp file
                            let _ = std::fs::remove_file(&png_path);
                            page_num += 1;
                        } else {
                            break;
                        }
                    }
                } else {
                    println!("PDF conversion failed: {}", String::from_utf8_lossy(&result.stderr));
                }
            }
            Err(e) => {
                println!("Failed to run pdftoppm: {:?}", e);
                println!("Make sure poppler-utils is installed (brew install poppler)");
            }
        }

        // Load markdown
        let markdown_content = std::fs::read_to_string(markdown_file).unwrap_or_default();

        let page_count = pdf_images.len();
        Self {
            pdf_images,
            markdown_content,
            texture_handles: vec![None; page_count],
        }
    }
}

impl App for PDFApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Add CHONKER title at the top
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("🐹 CHONKER")
                        .size(32.0)
                        .strong()
                        .color(egui::Color32::from_rgb(255, 140, 0)) // Orange color
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("PDF & Markdown Preview")
                            .size(14.0)
                            .color(egui::Color32::GRAY)
                    );
                });
            });
            
            ui.separator();
            ui.add_space(5.0);
            
            // Remove any spacing that might constrain height
            ui.spacing_mut().item_spacing.y = 0.0;
            
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                // Left pane: Rendered PDF (50% width, full height)
                ui.allocate_ui_with_layout(
                    ui.available_size() * egui::vec2(0.5, 1.0), // 50% width, 100% height
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.heading("📄 Original PDF");
                        ui.separator();
                        
                        // Use remaining space for scroll area
                        let remaining_height = ui.available_height();
                        egui::ScrollArea::vertical()
                            .max_height(remaining_height)
                            .show(ui, |ui| {
                                if self.pdf_images.is_empty() {
                                    ui.label("No PDF pages loaded. Make sure 'pdftoppm' is installed (brew install poppler)");
                                } else {
                                    for (i, color_image) in self.pdf_images.iter().enumerate() {
                                        // Create texture if not already created
                                        if self.texture_handles[i].is_none() {
                                            let texture = ctx.load_texture(
                                                format!("page_{}", i),
                                                color_image.clone(),
                                                egui::TextureOptions::default()
                                            );
                                            self.texture_handles[i] = Some(texture);
                                        }
                                        
                                        // Display the texture
                                        if let Some(texture) = &self.texture_handles[i] {
                                            let img_width = color_image.size[0] as f32;
                                            let scale = ui.available_width() / img_width;
                                            let scaled_height = color_image.size[1] as f32 * scale;
                                            
                                            let img = egui::Image::from_texture(texture)
                                                .fit_to_exact_size([ui.available_width(), scaled_height].into());
                                            ui.add(img);
                                            ui.add_space(10.0); // Space between pages
                                        }
                                    }
                                }
                            });
                    },
                );

                ui.separator();

                // Right pane: Markdown (remaining width, full height)
                ui.allocate_ui_with_layout(
                    ui.available_size(), // Use all remaining space
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.heading("📝 Proposed Markdown");
                        ui.separator();
                        
                        // Use remaining space for scroll area
                        let remaining_height = ui.available_height();
                        egui::ScrollArea::vertical()
                            .max_height(remaining_height)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.markdown_content.as_str())
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(50) // Give it plenty of rows
                                );
                            });
                    },
                );
            });
        });
    }
}

fn main() {
    let options = NativeOptions::default();
    eframe::run_native(
        "PDF & Markdown Preview",
        options,
        Box::new(|_cc| Ok(Box::new(PDFApp::new("input.pdf", "proposed_markdown.md")))),
    ).unwrap();
}
