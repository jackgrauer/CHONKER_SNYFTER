use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Simple Text Edit Test",
        options,
        Box::new(|_cc| Box::new(SimpleApp::default())),
    )
}

#[derive(Default)]
struct SimpleApp {
    matrix: Vec<Vec<char>>,
    selected: Option<(usize, usize)>,
}

impl SimpleApp {
    fn new() -> Self {
        let mut matrix = vec![vec![' '; 10]; 5];
        // Put some test data
        matrix[0] = "Hello     ".chars().collect();
        matrix[1] = "World     ".chars().collect();
        Self {
            matrix,
            selected: None,
        }
    }
}

impl Default for SimpleApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for SimpleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Click a cell and type to edit");
            
            // Show current selection
            if let Some((x, y)) = self.selected {
                ui.label(format!("Selected: ({}, {})", x, y));
            }
            
            // Handle keyboard input FIRST, before rendering
            if let Some((sel_x, sel_y)) = self.selected {
                ctx.input(|i| {
                    for event in &i.events {
                        if let egui::Event::Text(text) = event {
                            println!("Got text event: '{}'", text);
                            if let Some(ch) = text.chars().next() {
                                if ch.is_ascii_graphic() || ch == ' ' {
                                    self.matrix[sel_y][sel_x] = ch;
                                    println!("Updated cell ({}, {}) to '{}'", sel_x, sel_y, ch);
                                }
                            }
                        }
                    }
                });
            }
            
            // Render the matrix
            ui.group(|ui| {
                for (y, row) in self.matrix.iter().enumerate() {
                    ui.horizontal(|ui| {
                        for (x, &ch) in row.iter().enumerate() {
                            let selected = self.selected == Some((x, y));
                            let text = if selected {
                                format!("[{}]", ch)
                            } else {
                                format!(" {} ", ch)
                            };
                            
                            if ui.button(text).clicked() {
                                self.selected = Some((x, y));
                                println!("Selected cell ({}, {})", x, y);
                            }
                        }
                    });
                }
            });
            
            if ui.button("Clear Selection").clicked() {
                self.selected = None;
            }
        });
    }
}