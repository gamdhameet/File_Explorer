use eframe::{egui, NativeOptions};
use egui::{Color32, RichText, ScrollArea, Vec2};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct FileExplorerApp {
    current_path: PathBuf,
    entries: Vec<PathBuf>,
    error: Option<String>,
    status_message: Option<String>,
    show_hidden: bool,
    show_settings: bool,
    terminal_input: String,
    terminal_output: Vec<String>,
    terminal_history: Vec<String>,
    history_index: usize,
}

impl FileExplorerApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let mut app = Self {
            current_path: path,
            entries: Vec::new(),
            error: None,
            status_message: None,
            show_hidden: false,
            show_settings: false,
            terminal_input: String::new(),
            terminal_output: Vec::new(),
            terminal_history: Vec::new(),
            history_index: 0,
        };
        app.read_directory();
        app.terminal_output.push(format!("Terminal ready. Current directory: {}", app.current_path.display()));
        app
    }

    fn read_directory(&mut self) {
        self.error = None;
        self.status_message = None;
        match fs::read_dir(&self.current_path) {
            Ok(entries) => {
                let mut collected_entries: Vec<PathBuf> = entries
                    .filter_map(Result::ok)
                    .map(|res| res.path())
                    .filter(|path| {
                        if self.show_hidden {
                            true
                        } else {
                            path.file_name()
                                .and_then(|name| name.to_str())
                                .map(|name| !name.starts_with('.'))
                                .unwrap_or(true)
                        }
                    })
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

    fn open_file(&mut self, path: &PathBuf) {
        match open::that(path) {
            Ok(()) => {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                self.status_message = Some(format!("Opened: {}", file_name));
            }
            Err(e) => {
                self.error = Some(format!("Failed to open file: {}", e));
            }
        }
    }

    fn execute_command(&mut self, command: &str) {
        let trimmed_command = command.trim();
        if trimmed_command.is_empty() {
            return;
        }

        self.terminal_history.push(trimmed_command.to_string());
        self.history_index = self.terminal_history.len();
        
        self.terminal_output.push(format!("$ {}", trimmed_command));

        if trimmed_command.starts_with("cd ") {
            let target_path = trimmed_command.strip_prefix("cd ").unwrap().trim();
            self.change_directory(target_path);
        } else if trimmed_command == "cd" {
            if let Some(home) = std::env::var_os("HOME") {
                self.change_directory(&home.to_string_lossy());
            } else {
                self.terminal_output.push("Error: HOME environment variable not set".to_string());
            }
        } else if trimmed_command == "pwd" {
            self.terminal_output.push(self.current_path.display().to_string());
        } else if trimmed_command == "ls" || trimmed_command == "ls -la" {
            let show_hidden = trimmed_command.contains("-a");
            self.list_directory(show_hidden);
        } else if trimmed_command == "clear" {
            self.terminal_output.clear();
            self.terminal_output.push("Terminal cleared".to_string());
        } else {
            self.execute_system_command(trimmed_command);
        }

        if self.terminal_output.len() > 1000 {
            self.terminal_output.drain(0..500);
        }
    }

    fn change_directory(&mut self, path: &str) {
        let new_path = if path.starts_with('/') {
            PathBuf::from(path)
        } else if path == ".." {
            if let Some(parent) = self.current_path.parent() {
                parent.to_path_buf()
            } else {
                self.terminal_output.push("Already at root directory".to_string());
                return;
            }
        } else {
            self.current_path.join(path)
        };

        if new_path.exists() && new_path.is_dir() {
            self.current_path = new_path;
            self.read_directory();
            self.terminal_output.push(format!("Changed to: {}", self.current_path.display()));
        } else {
            self.terminal_output.push(format!("cd: no such file or directory: {}", path));
        }
    }

    fn list_directory(&mut self, show_hidden: bool) {
        match fs::read_dir(&self.current_path) {
            Ok(entries) => {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                    
                    if !show_hidden && file_name.starts_with('.') {
                        continue;
                    }
                    
                    let prefix = if path.is_dir() { "d" } else { "-" };
                    self.terminal_output.push(format!("{} {}", prefix, file_name));
                }
            }
            Err(e) => {
                self.terminal_output.push(format!("ls: {}", e));
            }
        }
    }

    fn execute_system_command(&mut self, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        let mut cmd = Command::new(parts[0]);
        if parts.len() > 1 {
            cmd.args(&parts[1..]);
        }
        cmd.current_dir(&self.current_path);

        match cmd.output() {
            Ok(output) => {
                if !output.stdout.is_empty() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        self.terminal_output.push(line.to_string());
                    }
                }
                if !output.stderr.is_empty() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    for line in stderr.lines() {
                        self.terminal_output.push(format!("Error: {}", line));
                    }
                }
                if output.stdout.is_empty() && output.stderr.is_empty() {
                    self.terminal_output.push("Command executed successfully".to_string());
                }
            }
            Err(e) => {
                self.terminal_output.push(format!("Failed to execute '{}': {}", command, e));
            }
        }
    }
}

impl eframe::App for FileExplorerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
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
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⚙ Settings").clicked() {
                        self.show_settings = !self.show_settings;
                    }
                });
            });

            if self.show_settings {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Settings:");
                    if ui.checkbox(&mut self.show_hidden, "Show hidden files").changed() {
                        self.read_directory();
                    }
                });
            }

            ui.separator();

            if let Some(error_message) = &self.error {
                ui.colored_label(Color32::RED, error_message);
            }

            if let Some(status_message) = &self.status_message {
                ui.colored_label(Color32::from_rgb(0, 150, 0), status_message);
            }
        });

        egui::TopBottomPanel::bottom("terminal_panel").resizable(true).show(ctx, |ui| {
            ui.label(RichText::new("Terminal").strong());
            ui.separator();
            
            ScrollArea::vertical()
                .stick_to_bottom(true)
                .max_height(200.0)
                .show(ui, |ui| {
                    for line in &self.terminal_output {
                        ui.label(line);
                    }
                });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("$");
                let response = ui.text_edit_singleline(&mut self.terminal_input);
                
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.execute_command(&self.terminal_input.clone());
                    self.terminal_input.clear();
                    response.request_focus();
                }

                if response.has_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) && !self.terminal_history.is_empty() {
                        if self.history_index > 0 {
                            self.history_index -= 1;
                            self.terminal_input = self.terminal_history[self.history_index].clone();
                        }
                    } else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) && !self.terminal_history.is_empty() {
                        if self.history_index < self.terminal_history.len() - 1 {
                            self.history_index += 1;
                            self.terminal_input = self.terminal_history[self.history_index].clone();
                        } else {
                            self.history_index = self.terminal_history.len();
                            self.terminal_input.clear();
                        }
                    }
                }

                if ui.button("Execute").clicked() && !self.terminal_input.trim().is_empty() {
                    self.execute_command(&self.terminal_input.clone());
                    self.terminal_input.clear();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
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
                            self.terminal_output.push(format!("Navigated to: {}", self.current_path.display()));
                        } else {
                            self.open_file(entry);
                        }
                    }
                }
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(1000.0, 700.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Rust File Explorer",
        options,
        Box::new(|cc| Box::new(FileExplorerApp::new(cc))),
    )
}
