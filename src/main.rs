use eframe::{egui, NativeOptions};
use egui::{Color32, RichText, ScrollArea, Vec2};
use std::fs;
use std::path::{Path, PathBuf};

struct FileExplorerApp {
    current_path: PathBuf,
    entries: Vec<PathBuf>,
    error: Option<String>,
}

impl FileExplorerApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let mut app = Self {
            current_path: path,
            entries: Vec::new(),
            error: None,
        };
        app.read_directory();
        app
    }

    fn read_directory(&mut self) {
        self.error = None;
        match fs::read_dir(&self.current_path) {
            Ok(entries) => {
                let mut collected_entries: Vec<PathBuf> = entries
                    .filter_map(Result::ok)
                    .map(|res| res.path())
                    .collect();

                collected_entries.sort_by(|a, b| {
                    let a_is_dir = a.is_dir();
                    let b_is_dir = b.is_dir();
                    if a_is_dir && !b_is_dir {
                        std::cmp::Ordering::Less
                    } else if !a_is_dir && b_is_dir {
                        std::cmp::Ordering::Greater
                    } else {
                        a.file_name().unwrap_or_default()
                         .cmp(b.file_name().unwrap_or_default())
                    }
                });

                self.entries = collected_entries;
            }
            Err(e) => {
                self.error = Some(format!("Error reading directory: {}", e));
            }
        }
    }
}

impl eframe::App for FileExplorerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.current_path.parent().is_some() {
                    if ui.button("⬆ Up").clicked() {
                        if let Some(parent) = self.current_path.parent() {
                            self.current_path = parent.to_path_buf();
                            self.read_directory();
                        }
                    }
                }

                ui.label(RichText::new(self.current_path.to_string_lossy()).strong());
            });

            ui.separator();

            if let Some(error_message) = &self.error {
                ui.colored_label(Color32::RED, error_message);
            }

            ScrollArea::vertical().show(ui, |ui| {
                for entry in &self.entries.clone() {
                    let is_dir = entry.is_dir();
                    let icon = if is_dir { "📁" } else { "📄" };
                    let file_name = entry.file_name().unwrap_or_default().to_string_lossy();

                    let label = format!("{} {}", icon, file_name);
                    if ui.selectable_label(false, label).clicked() {
                        if is_dir {
                            self.current_path = entry.clone();
                            self.read_directory();
                        } else {
                            println!("Selected file: {:?}", entry);
                        }
                    }
                }
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Rust File Explorer",
        options,
        Box::new(|cc| Box::new(FileExplorerApp::new(cc))),
    )
}
