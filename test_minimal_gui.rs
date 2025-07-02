use eframe::{egui, NativeOptions};

struct MinimalApp;

impl eframe::App for MinimalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Minimal GUI Test");
            if ui.button("Exit").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    println!("Starting minimal egui app...");
    
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_title("Minimal Test"),
        ..Default::default()
    };

    eframe::run_native(
        "MinimalTest",
        options,
        Box::new(|_cc| Ok(Box::new(MinimalApp))),
    )
}
