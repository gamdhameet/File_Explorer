use eframe::egui::{self, Context, Response};
use std::path::PathBuf;
use arboard::Clipboard;
use std::fs;

use crate::models::{Bookmark, FileEntry, FileOperation, Theme, ViewMode};
use crate::operations;
use crate::ui;
use crate::utils;

pub struct FileExplorerApp {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_entries: Vec<usize>,
    pub error: Option<String>,
    pub status_message: Option<String>,
    pub show_hidden: bool,
    pub show_settings: bool,
    
    // File operations
    pub clipboard_operation: Option<FileOperation>,
    pub clipboard: Result<Clipboard, arboard::Error>,
    
    // Navigation
    pub navigation_history: Vec<PathBuf>,
    pub history_index: usize,
    pub breadcrumbs: Vec<(String, PathBuf)>,
    
    // Bookmarks
    pub bookmarks: Vec<Bookmark>,
    pub show_bookmarks: bool,
    pub bookmark_name_input: String,
    
    // Terminal
    pub terminal_input: String,
    pub terminal_output: Vec<String>,
    pub terminal_history: Vec<String>,
    pub terminal_history_index: usize,
    
    // UI State
    pub view_mode: ViewMode,
    pub theme: Theme,
    pub show_properties_dialog: bool,
    pub properties_file: Option<FileEntry>,
    pub show_rename_dialog: bool,
    pub rename_text: String,
    pub rename_index: Option<usize>,
    pub context_menu_pos: Option<egui::Pos2>,
    pub context_menu_index: Option<usize>,
    
    // New file/folder dialogs
    pub show_new_file_dialog: bool,
    pub show_new_folder_dialog: bool,
    pub new_name_input: String,
}

impl FileExplorerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let mut app = Self {
            current_path: path.clone(),
            entries: Vec::new(),
            selected_entries: Vec::new(),
            error: None,
            status_message: None,
            show_hidden: false,
            show_settings: false,
            
            clipboard_operation: None,
            clipboard: Clipboard::new(),
            
            navigation_history: vec![path.clone()],
            history_index: 0,
            breadcrumbs: Vec::new(),
            
            bookmarks: Vec::new(),
            show_bookmarks: false,
            bookmark_name_input: String::new(),
            
            terminal_input: String::new(),
            terminal_output: Vec::new(),
            terminal_history: Vec::new(),
            terminal_history_index: 0,
            
            view_mode: ViewMode::List,
            theme: Theme::Light,
            show_properties_dialog: false,
            properties_file: None,
            show_rename_dialog: false,
            rename_text: String::new(),
            rename_index: None,
            context_menu_pos: None,
            context_menu_index: None,
            
            show_new_file_dialog: false,
            show_new_folder_dialog: false,
            new_name_input: String::new(),
        };
        
        app.load_bookmarks();
        app.read_directory();
        app.update_breadcrumbs();
        app.terminal_output.push(format!("Terminal ready. Current directory: {}", app.current_path.display()));
        app
    }

    pub fn read_directory(&mut self) {
        self.error = None;
        self.status_message = None;
        self.selected_entries.clear();
        
        match operations::read_directory(&self.current_path, self.show_hidden) {
            Ok(entries) => {
                self.entries = entries;
            },
            Err(e) => {
                self.error = Some(e);
            }
        }
    }

    pub fn update_breadcrumbs(&mut self) {
        self.breadcrumbs = utils::generate_breadcrumbs(&self.current_path);
    }

    pub fn navigate_to(&mut self, path: PathBuf) {
        if path.exists() && path.is_dir() {
            self.current_path = path.clone();
            
            // Update history
            if self.history_index < self.navigation_history.len() - 1 {
                self.navigation_history.truncate(self.history_index + 1);
            }
            self.navigation_history.push(path);
            self.history_index = self.navigation_history.len() - 1;
            
            self.read_directory();
            self.update_breadcrumbs();
        }
    }

    pub fn go_back(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.current_path = self.navigation_history[self.history_index].clone();
            self.read_directory();
            self.update_breadcrumbs();
        }
    }

    pub fn go_forward(&mut self) {
        if self.history_index < self.navigation_history.len() - 1 {
            self.history_index += 1;
            self.current_path = self.navigation_history[self.history_index].clone();
            self.read_directory();
            self.update_breadcrumbs();
        }
    }

    pub fn copy_selected(&mut self) {
        if !self.selected_entries.is_empty() {
            let paths: Vec<PathBuf> = self.selected_entries.iter()
                .map(|&i| self.entries[i].path.clone())
                .collect();
            self.clipboard_operation = Some(FileOperation::Copy(paths));
            self.status_message = Some(format!("Copied {} items", self.selected_entries.len()));
        }
    }

    pub fn cut_selected(&mut self) {
        if !self.selected_entries.is_empty() {
            let paths: Vec<PathBuf> = self.selected_entries.iter()
                .map(|&i| self.entries[i].path.clone())
                .collect();
            self.clipboard_operation = Some(FileOperation::Cut(paths));
            self.status_message = Some(format!("Cut {} items", self.selected_entries.len()));
        }
    }

    pub fn paste(&mut self) {
        if let Some(operation) = &self.clipboard_operation.clone() {
            match operation {
                FileOperation::Copy(paths) => {
                    for path in paths {
                        let file_name = path.file_name().unwrap().to_string_lossy();
                        let dest_path = self.current_path.join(&*file_name);
                        
                        if let Err(e) = operations::copy_item(path, &dest_path) {
                            self.error = Some(e);
                            return;
                        }
                    }
                    self.status_message = Some("Paste completed".to_string());
                }
                FileOperation::Cut(paths) => {
                    for path in paths {
                        let file_name = path.file_name().unwrap().to_string_lossy();
                        let dest_path = self.current_path.join(&*file_name);
                        
                        if let Err(e) = operations::move_item(path, &dest_path) {
                            self.error = Some(e);
                            return;
                        }
                    }
                    self.clipboard_operation = None;
                    self.status_message = Some("Move completed".to_string());
                }
            }
            self.read_directory();
        }
    }

    pub fn delete_selected(&mut self) {
        if !self.selected_entries.is_empty() {
            for &index in &self.selected_entries {
                let entry = &self.entries[index];
                if let Err(e) = operations::delete_item(&entry.path) {
                    self.error = Some(format!("Failed to delete {}: {}", entry.name, e));
                    return;
                }
            }
            self.status_message = Some(format!("Deleted {} items", self.selected_entries.len()));
            self.read_directory();
        }
    }

    pub fn create_new_file(&mut self, name: &str) {
        match operations::create_new_file(&self.current_path, name) {
            Ok(_) => {
                self.status_message = Some(format!("Created file: {}", name));
                self.read_directory();
            }
            Err(e) => {
                self.error = Some(e);
            }
        }
    }

    pub fn create_new_folder(&mut self, name: &str) {
        match operations::create_new_folder(&self.current_path, name) {
            Ok(_) => {
                self.status_message = Some(format!("Created folder: {}", name));
                self.read_directory();
            }
            Err(e) => {
                self.error = Some(e);
            }
        }
    }

    pub fn rename_file(&mut self, index: usize, new_name: &str) {
        if index < self.entries.len() {
            let old_path = &self.entries[index].path;
            match operations::rename_file(old_path, new_name) {
                Ok(_) => {
                    self.status_message = Some(format!("Renamed to: {}", new_name));
                    self.read_directory();
                }
                Err(e) => {
                    self.error = Some(e);
                }
            }
        }
    }

    pub fn add_bookmark(&mut self, name: String, path: PathBuf) {
        self.bookmarks.push(Bookmark { name, path });
        self.save_bookmarks();
    }

    pub fn save_bookmarks(&mut self) {
        if let Err(e) = utils::save_bookmarks(&self.bookmarks) {
            self.error = Some(e);
        }
    }

    pub fn load_bookmarks(&mut self) {
        self.bookmarks = utils::load_bookmarks();
    }

    pub fn open_file(&mut self, path: &PathBuf) {
        match operations::open_file(path) {
            Ok(_) => {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                self.status_message = Some(format!("Opened: {}", file_name));
            }
            Err(e) => {
                self.error = Some(e);
            }
        }
    }

    pub fn execute_command(&mut self, command: &str) {
        let trimmed_command = command.trim();
        if trimmed_command.is_empty() {
            return;
        }

        self.terminal_history.push(trimmed_command.to_string());
        self.terminal_history_index = self.terminal_history.len();
        
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

    pub fn change_directory(&mut self, path: &str) {
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
            self.navigate_to(new_path);
            self.terminal_output.push(format!("Changed to: {}", self.current_path.display()));
        } else {
            self.terminal_output.push(format!("cd: no such file or directory: {}", path));
        }
    }

    pub fn list_directory(&mut self, show_hidden: bool) {
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

    pub fn execute_system_command(&mut self, command: &str) {
        let (output_lines, error) = operations::execute_system_command(command, &self.current_path);
        
        for line in output_lines {
            self.terminal_output.push(line);
        }
        
        if let Some(err) = error {
            self.terminal_output.push(err);
        }
    }

    pub fn handle_keyboard_shortcuts(&mut self, ctx: &Context) {
        ctx.input(|i| {
            if i.modifiers.ctrl {
                if i.key_pressed(egui::Key::C) {
                    self.copy_selected();
                } else if i.key_pressed(egui::Key::X) {
                    self.cut_selected();
                } else if i.key_pressed(egui::Key::V) {
                    self.paste();
                }
            }
            
            if i.key_pressed(egui::Key::Delete) {
                self.delete_selected();
            }
            
            if i.key_pressed(egui::Key::F2) && self.selected_entries.len() == 1 {
                self.show_rename_dialog = true;
                self.rename_index = Some(self.selected_entries[0]);
                self.rename_text = self.entries[self.selected_entries[0]].name.clone();
            }
        });
    }

    pub fn apply_theme(&self, ctx: &Context) {
        match self.theme {
            Theme::Light => {
                ctx.set_visuals(egui::Visuals::light());
            }
            Theme::Dark => {
                ctx.set_visuals(egui::Visuals::dark());
            }
        }
    }

    pub fn handle_file_interaction(&mut self, response: Response, index: usize) {
        if response.clicked() {
            if response.ctx.input(|i| i.modifiers.ctrl) {
                // Multi-select with Ctrl
                if let Some(pos) = self.selected_entries.iter().position(|&x| x == index) {
                    self.selected_entries.remove(pos);
                } else {
                    self.selected_entries.push(index);
                }
            } else {
                // Single select or navigate
                if index < self.entries.len() {
                    self.selected_entries.clear();
                    self.selected_entries.push(index);
                }
            }
        }
        
        if response.double_clicked() && index < self.entries.len() {
            if self.entries[index].is_dir {
                let path = self.entries[index].path.clone();
                self.navigate_to(path);
            } else {
                let path = self.entries[index].path.clone();
                self.open_file(&path);
            }
        }
        
        if response.secondary_clicked() {
            self.context_menu_pos = Some(response.interact_pointer_pos().unwrap_or_default());
            self.context_menu_index = Some(index);
            if !self.selected_entries.contains(&index) {
                self.selected_entries.clear();
                self.selected_entries.push(index);
            }
        }
    }
}

impl eframe::App for FileExplorerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.apply_theme(ctx);
        self.handle_keyboard_shortcuts(ctx);
        
        ui::show_top_panel(self, ctx);
        ui::show_terminal(self, ctx);
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui::show_file_list(self, ui);
        });
        
        ui::show_context_menu(self, ctx);
        ui::show_dialogs(self, ctx);
    }
} 