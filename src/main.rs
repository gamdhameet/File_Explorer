mod app;
mod models;
mod ui;
mod operations;
mod utils;
mod terminal;
mod terminal_ui;
mod context_menu;
mod settings;

use eframe::{egui, NativeOptions};

fn main() -> Result<(), eframe::Error> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(egui::Vec2::new(1200.0, 800.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Rust File Explorer Pro",
        options,
        Box::new(|cc| Box::new(app::FileExplorerApp::new(cc))),
    )
}
