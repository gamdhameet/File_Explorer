use eframe::{egui, NativeOptions};
use egui::{Color32, RichText, ScrollArea, Vec2, Context, Ui};
use std::fs::{self, File};
use std::path::PathBuf;
use std::process::Command;
use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};

#[derive(Clone, Debug)]
enum FileOperation {
    Copy(Vec<PathBuf>),
    Cut(Vec<PathBuf>),
}

#[derive(Clone, Debug)]
struct FileEntry {
    path: PathBuf,
    is_dir: bool,
    size: u64,
    modified: DateTime<Local>,
    name: String,
    extension: String,
}

#[derive(Debug, PartialEq)]
enum ViewMode {
    List,
    Grid,
}

#[derive(Debug, PartialEq)]
enum Theme {
    Light,
    Dark,
}

#[derive(Serialize, Deserialize, Clone)]
struct Bookmark {
    name: String,
    path: PathBuf,
}

struct FileExplorerApp {
    current_path: PathBuf,
    entries: Vec<FileEntry>,
    selected_entries: Vec<usize>,
    error: Option<String>,
    status_message: Option<String>,
    show_hidden: bool,
    show_settings: bool,
    
    // File operations
    clipboard_operation: Option<FileOperation>,
    clipboard: Result<Clipboard, arboard::Error>,
    
    // Navigation
    navigation_history: Vec<PathBuf>,
    history_index: usize,
    breadcrumbs: Vec<(String, PathBuf)>,
    
    // Bookmarks
    bookmarks: Vec<Bookmark>,
    show_bookmarks: bool,
    bookmark_name_input: String,
    
    // Terminal
    terminal_input: String,
    terminal_output: Vec<String>,
    terminal_history: Vec<String>,
    terminal_history_index: usize,
    
    // UI State
    view_mode: ViewMode,
    theme: Theme,
    show_properties_dialog: bool,
    properties_file: Option<FileEntry>,
    show_rename_dialog: bool,
    rename_text: String,
    rename_index: Option<usize>,
    context_menu_pos: Option<egui::Pos2>,
    context_menu_index: Option<usize>,
    
    // New file/folder dialogs
    show_new_file_dialog: bool,
    show_new_folder_dialog: bool,
    new_name_input: String,
}

impl FileExplorerApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
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

    fn get_file_icon(&self, entry: &FileEntry) -> &'static str {
        if entry.is_dir {
            "📁"
        } else {
            match entry.extension.to_lowercase().as_str() {
                "txt" | "md" | "readme" => "📄",
                "mp3" | "wav" | "flac" | "m4a" => "🎵",
                "mp4" | "avi" | "mkv" | "mov" => "🎬",
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" => "🖼️",
                "pdf" => "📕",
                "doc" | "docx" => "📘",
                "xls" | "xlsx" => "📗",
                "ppt" | "pptx" => "📙",
                "zip" | "rar" | "7z" | "tar" | "gz" => "🗜️",
                "exe" | "msi" => "⚙️",
                "rs" | "py" | "js" | "html" | "css" | "cpp" | "c" | "java" => "💻",
                _ => "📄",
            }
        }
    }

    fn format_file_size(&self, size: u64) -> String {
        if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else if size < 1024 * 1024 * 1024 {
            format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    fn read_directory(&mut self) {
        self.error = None;
        self.status_message = None;
        self.selected_entries.clear();
        
        match fs::read_dir(&self.current_path) {
            Ok(entries) => {
                let mut file_entries: Vec<FileEntry> = entries
                    .filter_map(Result::ok)
                    .filter_map(|entry| {
                        let path = entry.path();
                        let file_name = path.file_name()?.to_str()?.to_string();
                        
                        if !self.show_hidden && file_name.starts_with('.') {
                            return None;
                        }
                        
                        let metadata = entry.metadata().ok()?;
                        let modified = metadata.modified().ok()?;
                        let modified = DateTime::<Local>::from(modified);
                        
                        let extension = path.extension()
                            .and_then(|ext| ext.to_str())
                            .unwrap_or("")
                            .to_string();
                        
                        Some(FileEntry {
                            path: path.clone(),
                            is_dir: path.is_dir(),
                            size: if metadata.is_file() { metadata.len() } else { 0 },
                            modified,
                            name: file_name,
                            extension,
                        })
                    })
                    .collect();

                file_entries.sort_by(|a, b| {
                    if a.is_dir && !b.is_dir {
                        std::cmp::Ordering::Less
                    } else if !a.is_dir && b.is_dir {
                        std::cmp::Ordering::Greater
                    } else {
                        a.name.cmp(&b.name)
                    }
                });

                self.entries = file_entries;
            }
            Err(e) => {
                self.error = Some(format!("Error reading directory: {}", e));
            }
        }
    }

    fn update_breadcrumbs(&mut self) {
        self.breadcrumbs.clear();
        let mut current = self.current_path.clone();
        
        while let Some(parent) = current.parent() {
            if let Some(name) = current.file_name() {
                self.breadcrumbs.insert(0, (name.to_string_lossy().to_string(), current.clone()));
            }
            current = parent.to_path_buf();
        }
        
        if current.to_string_lossy() == "/" {
            self.breadcrumbs.insert(0, ("/".to_string(), current));
        }
    }

    fn navigate_to(&mut self, path: PathBuf) {
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

    fn go_back(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.current_path = self.navigation_history[self.history_index].clone();
            self.read_directory();
            self.update_breadcrumbs();
        }
    }

    fn go_forward(&mut self) {
        if self.history_index < self.navigation_history.len() - 1 {
            self.history_index += 1;
            self.current_path = self.navigation_history[self.history_index].clone();
            self.read_directory();
            self.update_breadcrumbs();
        }
    }

    fn copy_selected(&mut self) {
        if !self.selected_entries.is_empty() {
            let paths: Vec<PathBuf> = self.selected_entries.iter()
                .map(|&i| self.entries[i].path.clone())
                .collect();
            self.clipboard_operation = Some(FileOperation::Copy(paths));
            self.status_message = Some(format!("Copied {} items", self.selected_entries.len()));
        }
    }

    fn cut_selected(&mut self) {
        if !self.selected_entries.is_empty() {
            let paths: Vec<PathBuf> = self.selected_entries.iter()
                .map(|&i| self.entries[i].path.clone())
                .collect();
            self.clipboard_operation = Some(FileOperation::Cut(paths));
            self.status_message = Some(format!("Cut {} items", self.selected_entries.len()));
        }
    }

    fn paste(&mut self) {
        if let Some(operation) = &self.clipboard_operation.clone() {
            match operation {
                FileOperation::Copy(paths) => {
                    for path in paths {
                        let file_name = path.file_name().unwrap().to_string_lossy();
                        let dest_path = self.current_path.join(&*file_name);
                        
                        if path.is_file() {
                            if let Err(e) = fs::copy(path, &dest_path) {
                                self.error = Some(format!("Failed to copy {}: {}", file_name, e));
                                return;
                            }
                        } else if path.is_dir() {
                            // Simple directory copy (non-recursive for now)
                            if let Err(e) = fs::create_dir_all(&dest_path) {
                                self.error = Some(format!("Failed to create directory {}: {}", file_name, e));
                                return;
                            }
                        }
                    }
                    self.status_message = Some("Paste completed".to_string());
                }
                FileOperation::Cut(paths) => {
                    for path in paths {
                        let file_name = path.file_name().unwrap().to_string_lossy();
                        let dest_path = self.current_path.join(&*file_name);
                        
                        if let Err(e) = fs::rename(path, &dest_path) {
                            self.error = Some(format!("Failed to move {}: {}", file_name, e));
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

    fn delete_selected(&mut self) {
        if !self.selected_entries.is_empty() {
            for &index in &self.selected_entries {
                let entry = &self.entries[index];
                let result = if entry.is_dir {
                    fs::remove_dir_all(&entry.path)
                } else {
                    fs::remove_file(&entry.path)
                };
                
                if let Err(e) = result {
                    self.error = Some(format!("Failed to delete {}: {}", entry.name, e));
                    return;
                }
            }
            self.status_message = Some(format!("Deleted {} items", self.selected_entries.len()));
            self.read_directory();
        }
    }

    fn create_new_file(&mut self, name: &str) {
        let path = self.current_path.join(name);
        match File::create(&path) {
            Ok(_) => {
                self.status_message = Some(format!("Created file: {}", name));
                self.read_directory();
            }
            Err(e) => {
                self.error = Some(format!("Failed to create file: {}", e));
            }
        }
    }

    fn create_new_folder(&mut self, name: &str) {
        let path = self.current_path.join(name);
        match fs::create_dir(&path) {
            Ok(_) => {
                self.status_message = Some(format!("Created folder: {}", name));
                self.read_directory();
            }
            Err(e) => {
                self.error = Some(format!("Failed to create folder: {}", e));
            }
        }
    }

    fn rename_file(&mut self, index: usize, new_name: &str) {
        if index < self.entries.len() {
            let old_path = &self.entries[index].path;
            let new_path = old_path.parent().unwrap().join(new_name);
            
            match fs::rename(old_path, &new_path) {
                Ok(_) => {
                    self.status_message = Some(format!("Renamed to: {}", new_name));
                    self.read_directory();
                }
                Err(e) => {
                    self.error = Some(format!("Failed to rename: {}", e));
                }
            }
        }
    }

    fn add_bookmark(&mut self, name: String, path: PathBuf) {
        self.bookmarks.push(Bookmark { name, path });
        self.save_bookmarks();
    }

    fn save_bookmarks(&self) {
        if let Ok(json) = serde_json::to_string(&self.bookmarks) {
            let _ = fs::write("bookmarks.json", json);
        }
    }

    fn load_bookmarks(&mut self) {
        if let Ok(contents) = fs::read_to_string("bookmarks.json") {
            if let Ok(bookmarks) = serde_json::from_str(&contents) {
                self.bookmarks = bookmarks;
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
            self.navigate_to(new_path);
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

    fn handle_keyboard_shortcuts(&mut self, ctx: &Context) {
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

    fn apply_theme(&self, ctx: &Context) {
        match self.theme {
            Theme::Light => {
                ctx.set_visuals(egui::Visuals::light());
            }
            Theme::Dark => {
                ctx.set_visuals(egui::Visuals::dark());
            }
        }
    }

    fn show_top_panel(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // Navigation row
            ui.horizontal(|ui| {
                // Back/Forward buttons
                ui.add_enabled(self.history_index > 0, egui::Button::new("⬅")).clicked().then(|| self.go_back());
                ui.add_enabled(self.history_index < self.navigation_history.len() - 1, egui::Button::new("➡")).clicked().then(|| self.go_forward());
                
                if ui.button("⬆ Up").clicked() {
                    if let Some(parent) = self.current_path.parent() {
                        self.navigate_to(parent.to_path_buf());
                    }
                }
                
                ui.separator();
                
                // Breadcrumb navigation - collect paths first to avoid borrow issues
                let breadcrumbs: Vec<(String, PathBuf)> = self.breadcrumbs.clone();
                for (i, (name, path)) in breadcrumbs.iter().enumerate() {
                    if i > 0 {
                        ui.label("/");
                    }
                    if ui.link(name).clicked() {
                        self.navigate_to(path.clone());
                    }
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⚙ Settings").clicked() {
                        self.show_settings = !self.show_settings;
                    }
                    
                    if ui.button("⭐ Bookmarks").clicked() {
                        self.show_bookmarks = !self.show_bookmarks;
                    }
                });
            });
            
            // Action buttons row
            ui.horizontal(|ui| {
                if ui.button("📄 New File").clicked() {
                    self.show_new_file_dialog = true;
                    self.new_name_input.clear();
                }
                
                if ui.button("📁 New Folder").clicked() {
                    self.show_new_folder_dialog = true;
                    self.new_name_input.clear();
                }
                
                ui.separator();
                
                ui.label("View:");
                ui.selectable_value(&mut self.view_mode, ViewMode::List, "📋 List");
                ui.selectable_value(&mut self.view_mode, ViewMode::Grid, "⊞ Grid");
                
                ui.separator();
                
                ui.label("Theme:");
                ui.selectable_value(&mut self.theme, Theme::Light, "☀ Light");
                ui.selectable_value(&mut self.theme, Theme::Dark, "🌙 Dark");
            });

            // Settings panel
            if self.show_settings {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Settings:");
                    if ui.checkbox(&mut self.show_hidden, "Show hidden files").changed() {
                        self.read_directory();
                    }
                });
            }
            
            // Bookmarks panel
            if self.show_bookmarks {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Add bookmark:");
                    ui.text_edit_singleline(&mut self.bookmark_name_input);
                    if ui.button("Add").clicked() && !self.bookmark_name_input.is_empty() {
                        self.add_bookmark(self.bookmark_name_input.clone(), self.current_path.clone());
                        self.bookmark_name_input.clear();
                    }
                });
                
                // Clone bookmarks to avoid borrow issues
                let bookmarks = self.bookmarks.clone();
                let mut bookmark_to_remove = None;
                ui.horizontal_wrapped(|ui| {
                    for (i, bookmark) in bookmarks.iter().enumerate() {
                        if ui.button(&bookmark.name).clicked() {
                            self.navigate_to(bookmark.path.clone());
                        }
                        if ui.button("❌").clicked() {
                            bookmark_to_remove = Some(i);
                        }
                    }
                });
                
                if let Some(index) = bookmark_to_remove {
                    self.bookmarks.remove(index);
                    self.save_bookmarks();
                }
            }

            ui.separator();

            // Status messages
            if let Some(error_message) = &self.error {
                ui.colored_label(Color32::RED, error_message);
            }

            if let Some(status_message) = &self.status_message {
                ui.colored_label(Color32::from_rgb(0, 150, 0), status_message);
            }
        });
    }

    fn show_file_list(&mut self, ui: &mut Ui) {
        match self.view_mode {
            ViewMode::List => self.show_list_view(ui),
            ViewMode::Grid => self.show_grid_view(ui),
        }
    }

    fn show_list_view(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(RichText::new("Name").strong());
                ui.separator();
                ui.label(RichText::new("Size").strong());
                ui.separator();
                ui.label(RichText::new("Modified").strong());
            });
            ui.separator();
            
            // Clone entries to avoid borrow issues
            let entries = self.entries.clone();
            for (i, entry) in entries.iter().enumerate() {
                let response = ui.horizontal(|ui| {
                    let icon = self.get_file_icon(entry);
                    let selected = self.selected_entries.contains(&i);
                    
                    let response = ui.selectable_label(selected, format!("{} {}", icon, entry.name));
                    ui.separator();
                    
                    if entry.is_dir {
                        ui.label("--");
                    } else {
                        ui.label(self.format_file_size(entry.size));
                    }
                    ui.separator();
                    
                    ui.label(entry.modified.format("%Y-%m-%d %H:%M").to_string());
                    
                    response
                }).inner;
                
                self.handle_file_interaction(response, i);
            }
        });
    }

    fn show_grid_view(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                // Clone entries to avoid borrow issues
                let entries = self.entries.clone();
                for (i, entry) in entries.iter().enumerate() {
                    let icon = self.get_file_icon(&entry);
                    let selected = self.selected_entries.contains(&i);
                    
                    let response = ui.vertical(|ui| {
                        ui.set_max_width(80.0);
                        ui.set_min_height(80.0);
                        
                        let response = ui.selectable_label(selected, RichText::new(icon).size(32.0));
                        ui.label(&entry.name);
                        
                        response
                    }).inner;
                    
                    self.handle_file_interaction(response, i);
                }
            });
        });
    }

    fn handle_file_interaction(&mut self, response: egui::Response, index: usize) {
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
                    if self.entries[index].is_dir {
                        let path = self.entries[index].path.clone();
                        self.navigate_to(path);
                    } else {
                        let path = self.entries[index].path.clone();
                        self.open_file(&path);
                    }
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

    fn show_context_menu(&mut self, ctx: &Context) {
        if let (Some(pos), Some(_)) = (self.context_menu_pos, self.context_menu_index) {
            egui::Area::new("context_menu".into())
                .fixed_pos(pos)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        if ui.button("📋 Copy").clicked() {
                            self.copy_selected();
                            self.context_menu_pos = None;
                        }
                        if ui.button("✂️ Cut").clicked() {
                            self.cut_selected();
                            self.context_menu_pos = None;
                        }
                        if ui.button("📁 Paste").clicked() {
                            self.paste();
                            self.context_menu_pos = None;
                        }
                        ui.separator();
                        if ui.button("🗑️ Delete").clicked() {
                            self.delete_selected();
                            self.context_menu_pos = None;
                        }
                        if ui.button("✏️ Rename").clicked() && self.selected_entries.len() == 1 {
                            self.show_rename_dialog = true;
                            self.rename_index = Some(self.selected_entries[0]);
                            self.rename_text = self.entries[self.selected_entries[0]].name.clone();
                            self.context_menu_pos = None;
                        }
                        ui.separator();
                        if ui.button("ℹ️ Properties").clicked() && self.selected_entries.len() == 1 {
                            self.show_properties_dialog = true;
                            self.properties_file = Some(self.entries[self.selected_entries[0]].clone());
                            self.context_menu_pos = None;
                        }
                    });
                });
            
            if ctx.input(|i| i.pointer.any_click()) {
                self.context_menu_pos = None;
                self.context_menu_index = None;
            }
        }
    }

    fn show_dialogs(&mut self, ctx: &Context) {
        // Properties dialog
        if self.show_properties_dialog {
            egui::Window::new("Properties")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    if let Some(ref file) = self.properties_file {
                        ui.label(format!("Name: {}", file.name));
                        ui.label(format!("Path: {}", file.path.display()));
                        ui.label(format!("Type: {}", if file.is_dir { "Directory" } else { "File" }));
                        if !file.is_dir {
                            ui.label(format!("Size: {}", self.format_file_size(file.size)));
                        }
                        ui.label(format!("Modified: {}", file.modified.format("%Y-%m-%d %H:%M:%S")));
                        
                        if ui.button("Close").clicked() {
                            self.show_properties_dialog = false;
                        }
                    }
                });
        }
        
        // Rename dialog
        if self.show_rename_dialog {
            egui::Window::new("Rename")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("New name:");
                    let response = ui.text_edit_singleline(&mut self.rename_text);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Rename").clicked() && !self.rename_text.is_empty() {
                            if let Some(index) = self.rename_index {
                                let new_name = self.rename_text.clone();
                                self.rename_file(index, &new_name);
                            }
                            self.show_rename_dialog = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_rename_dialog = false;
                        }
                    });
                    
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.rename_text.is_empty() {
                        if let Some(index) = self.rename_index {
                            let new_name = self.rename_text.clone();
                            self.rename_file(index, &new_name);
                        }
                        self.show_rename_dialog = false;
                    }
                });
        }
        
        // New file dialog
        if self.show_new_file_dialog {
            egui::Window::new("New File")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("File name:");
                    let response = ui.text_edit_singleline(&mut self.new_name_input);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() && !self.new_name_input.is_empty() {
                            let name = self.new_name_input.clone();
                            self.create_new_file(&name);
                            self.show_new_file_dialog = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_new_file_dialog = false;
                        }
                    });
                    
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.new_name_input.is_empty() {
                        let name = self.new_name_input.clone();
                        self.create_new_file(&name);
                        self.show_new_file_dialog = false;
                    }
                });
        }
        
        // New folder dialog
        if self.show_new_folder_dialog {
            egui::Window::new("New Folder")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Folder name:");
                    let response = ui.text_edit_singleline(&mut self.new_name_input);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() && !self.new_name_input.is_empty() {
                            let name = self.new_name_input.clone();
                            self.create_new_folder(&name);
                            self.show_new_folder_dialog = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_new_folder_dialog = false;
                        }
                    });
                    
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.new_name_input.is_empty() {
                        let name = self.new_name_input.clone();
                        self.create_new_folder(&name);
                        self.show_new_folder_dialog = false;
                    }
                });
        }
    }

    fn show_terminal(&mut self, ctx: &Context) {
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
                        if self.terminal_history_index > 0 {
                            self.terminal_history_index -= 1;
                            self.terminal_input = self.terminal_history[self.terminal_history_index].clone();
                        }
                    } else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) && !self.terminal_history.is_empty() {
                        if self.terminal_history_index < self.terminal_history.len() - 1 {
                            self.terminal_history_index += 1;
                            self.terminal_input = self.terminal_history[self.terminal_history_index].clone();
                        } else {
                            self.terminal_history_index = self.terminal_history.len();
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
    }
}

impl eframe::App for FileExplorerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.apply_theme(ctx);
        self.handle_keyboard_shortcuts(ctx);
        
        self.show_top_panel(ctx);
        self.show_terminal(ctx);
        
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_file_list(ui);
        });
        
        self.show_context_menu(ctx);
        self.show_dialogs(ctx);
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(1200.0, 800.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Rust File Explorer Pro",
        options,
        Box::new(|cc| Box::new(FileExplorerApp::new(cc))),
    )
}
