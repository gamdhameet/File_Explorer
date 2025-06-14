use eframe::egui::{self, Context, Response};
use std::path::PathBuf;
use arboard::Clipboard;

use crate::models::{Bookmark, FileEntry, FileOperation, Theme};
use crate::operations;
use crate::ui;
use crate::utils;
use crate::terminal::TerminalState;
use crate::terminal_ui;
use crate::context_menu::{ContextMenuState, ContextMenuAction, NewItemType};
use crate::settings::{AppSettings, SettingsWindow};

pub struct FileExplorerApp {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_entries: Vec<usize>,
    pub error: Option<String>,
    pub status_message: Option<String>,
    
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
    
    // Terminal - new improved terminal
    pub terminal: TerminalState,
    
    // Context Menu - new comprehensive context menu
    pub context_menu: ContextMenuState,
    
    // Settings - new settings system
    pub settings: AppSettings,
    pub settings_window: SettingsWindow,
    
    // UI State
    pub show_properties_dialog: bool,
    pub properties_file: Option<FileEntry>,
    pub show_rename_dialog: bool,
    pub rename_text: String,
    pub rename_index: Option<usize>,
    
    // New file/folder dialogs
    pub show_new_file_dialog: bool,
    pub show_new_folder_dialog: bool,
    pub new_name_input: String,
}

impl FileExplorerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let settings = AppSettings::load();
        
        let mut app = Self {
            current_path: path.clone(),
            entries: Vec::new(),
            selected_entries: Vec::new(),
            error: None,
            status_message: None,
            
            clipboard_operation: None,
            clipboard: Clipboard::new(),
            
            navigation_history: vec![path.clone()],
            history_index: 0,
            breadcrumbs: Vec::new(),
            
            bookmarks: Vec::new(),
            show_bookmarks: false,
            bookmark_name_input: String::new(),
            
            terminal: TerminalState::new(),
            context_menu: ContextMenuState::new(),
            settings,
            settings_window: SettingsWindow::new(),
            
            show_properties_dialog: false,
            properties_file: None,
            show_rename_dialog: false,
            rename_text: String::new(),
            rename_index: None,
            
            show_new_file_dialog: false,
            show_new_folder_dialog: false,
            new_name_input: String::new(),
        };
        
        app.load_bookmarks();
        app.read_directory();
        app.update_breadcrumbs();
        app
    }

    pub fn read_directory(&mut self) {
        self.error = None;
        self.status_message = None;
        self.selected_entries.clear();
        
        match operations::read_directory(&self.current_path, self.settings.show_hidden_files) {
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
            self.navigation_history.push(path.clone());
            self.history_index = self.navigation_history.len() - 1;
            
            // Update terminal directory
            self.terminal.current_dir = path;
            
            self.read_directory();
            self.update_breadcrumbs();
        }
    }

    pub fn go_back(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.current_path = self.navigation_history[self.history_index].clone();
            self.terminal.current_dir = self.current_path.clone();
            self.read_directory();
            self.update_breadcrumbs();
        }
    }

    pub fn go_forward(&mut self) {
        if self.history_index < self.navigation_history.len() - 1 {
            self.history_index += 1;
            self.current_path = self.navigation_history[self.history_index].clone();
            self.terminal.current_dir = self.current_path.clone();
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
        let should_confirm = self.settings.confirm_deletions;
        
        if should_confirm {
            // TODO: Show confirmation dialog
        }
        
        for &index in &self.selected_entries {
            if let Some(entry) = self.entries.get(index) {
                if let Err(e) = operations::delete_item(&entry.path) {
                    self.error = Some(e);
                    return;
                }
            }
        }
        self.status_message = Some(format!("Deleted {} items", self.selected_entries.len()));
        self.read_directory();
    }

    pub fn create_new_file(&mut self, name: &str) {
        let path = self.current_path.join(name);
        if let Err(e) = std::fs::File::create(&path) {
            self.error = Some(format!("Failed to create file: {}", e));
        } else {
            self.status_message = Some(format!("Created file: {}", name));
            self.read_directory();
        }
    }

    pub fn create_new_folder(&mut self, name: &str) {
        let path = self.current_path.join(name);
        if let Err(e) = std::fs::create_dir(&path) {
            self.error = Some(format!("Failed to create folder: {}", e));
        } else {
            self.status_message = Some(format!("Created folder: {}", name));
            self.read_directory();
        }
    }

    pub fn rename_file(&mut self, index: usize, new_name: &str) {
        if let Some(entry) = self.entries.get(index) {
            let new_path = self.current_path.join(new_name);
            if let Err(e) = std::fs::rename(&entry.path, &new_path) {
                self.error = Some(format!("Failed to rename: {}", e));
            } else {
                self.status_message = Some(format!("Renamed to: {}", new_name));
                self.read_directory();
            }
        }
    }

    pub fn add_bookmark(&mut self, name: String, path: PathBuf) {
        self.bookmarks.push(Bookmark { name, path });
        self.save_bookmarks();
    }

    pub fn save_bookmarks(&mut self) {
        // TODO: Implement bookmark saving
    }

    pub fn load_bookmarks(&mut self) {
        // TODO: Implement bookmark loading
    }

    pub fn open_file(&mut self, path: &PathBuf) {
        if let Err(e) = open::that(path) {
            self.error = Some(format!("Failed to open file: {}", e));
        }
    }

    pub fn handle_context_menu_action(&mut self, action: ContextMenuAction) {
        match action {
            ContextMenuAction::Open => {
                if let Some(&index) = self.selected_entries.first() {
                    let entry_path = self.entries[index].path.clone();
                    let is_dir = self.entries[index].is_dir;
                    
                    if is_dir {
                        self.navigate_to(entry_path);
                    } else {
                        self.open_file(&entry_path);
                    }
                }
            }
            ContextMenuAction::Cut => self.cut_selected(),
            ContextMenuAction::Copy => self.copy_selected(),
            ContextMenuAction::Paste => self.paste(),
            ContextMenuAction::Delete => self.delete_selected(),
            ContextMenuAction::Rename => {
                if let Some(&index) = self.selected_entries.first() {
                    self.show_rename_dialog = true;
                    self.rename_index = Some(index);
                    self.rename_text = self.entries[index].name.clone();
                }
            }
            ContextMenuAction::Properties => {
                if let Some(&index) = self.selected_entries.first() {
                    self.show_properties_dialog = true;
                    self.properties_file = Some(self.entries[index].clone());
                }
            }
            ContextMenuAction::CreateNew(item_type) => {
                match item_type {
                    NewItemType::File => {
                        self.show_new_file_dialog = true;
                        self.new_name_input.clear();
                    }
                    NewItemType::Folder => {
                        self.show_new_folder_dialog = true;
                        self.new_name_input.clear();
                    }
                    _ => {
                        self.status_message = Some("Feature not implemented yet".to_string());
                    }
                }
            }
            ContextMenuAction::CopyPath => {
                if let Some(&index) = self.selected_entries.first() {
                    let path = self.entries[index].path.to_string_lossy().to_string();
                    if let Ok(ref mut clipboard) = self.clipboard {
                        let _ = clipboard.set_text(path);
                        self.status_message = Some("Path copied to clipboard".to_string());
                    }
                }
            }
            ContextMenuAction::OpenInTerminal => {
                let path = if let Some(&index) = self.selected_entries.first() {
                    let entry = &self.entries[index];
                    if entry.is_dir {
                        entry.path.clone()
                    } else {
                        self.current_path.clone()
                    }
                } else {
                    self.current_path.clone()
                };
                
                // Change terminal directory
                self.terminal.change_directory(path.to_string_lossy().as_ref());
                self.navigate_to(path);
            }
            ContextMenuAction::AddToBookmarks => {
                let path = if let Some(&index) = self.selected_entries.first() {
                    self.entries[index].path.clone()
                } else {
                    self.current_path.clone()
                };
                let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                self.add_bookmark(name, path);
                self.status_message = Some("Added to bookmarks".to_string());
            }
            ContextMenuAction::OpenInEditor => {
                if let Some(&index) = self.selected_entries.first() {
                    let entry_path = self.entries[index].path.clone();
                    let editor = self.settings.default_editor.clone();
                    
                    if let Err(e) = std::process::Command::new(editor)
                        .arg(&entry_path)
                        .spawn() {
                        self.error = Some(format!("Failed to open editor: {}", e));
                    }
                }
            }
            _ => {
                self.status_message = Some("Feature not implemented yet".to_string());
            }
        }
    }

    pub fn handle_keyboard_shortcuts(&mut self, ctx: &Context) {
        ctx.input_mut(|i| {
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::C) {
                self.copy_selected();
            }
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::X) {
                self.cut_selected();
            }
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::V) {
                self.paste();
            }
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Delete) {
                self.delete_selected();
            }
            if i.consume_key(egui::Modifiers::NONE, egui::Key::F2) {
                if let Some(&index) = self.selected_entries.first() {
                    self.show_rename_dialog = true;
                    self.rename_index = Some(index);
                    self.rename_text = self.entries[index].name.clone();
                }
            }
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::N) {
                self.show_new_file_dialog = true;
                self.new_name_input.clear();
            }
            if i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::N) {
                self.show_new_folder_dialog = true;
                self.new_name_input.clear();
            }
            if i.consume_key(egui::Modifiers::NONE, egui::Key::F5) {
                self.read_directory();
            }
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::Comma) {
                self.settings_window.show = true;
            }
        });
    }

    pub fn apply_theme(&self, ctx: &Context) {
        match self.settings.theme {
            Theme::Light => ctx.set_visuals(egui::Visuals::light()),
            Theme::Dark => ctx.set_visuals(egui::Visuals::dark()),
        }
    }

    pub fn handle_file_interaction(&mut self, response: Response, index: usize, ctx: &Context) {
        if response.clicked() {
            if ctx.input(|i| i.modifiers.ctrl) {
                // Ctrl+click for multi-selection
                if let Some(pos) = self.selected_entries.iter().position(|&i| i == index) {
                    self.selected_entries.remove(pos);
                } else {
                    self.selected_entries.push(index);
                }
            } else {
                // Regular click
                self.selected_entries = vec![index];
            }
        }

        if response.double_clicked() && self.settings.double_click_to_open {
            let entry_path = self.entries[index].path.clone();
            let is_dir = self.entries[index].is_dir;
            
            if is_dir {
                self.navigate_to(entry_path);
            } else {
                self.open_file(&entry_path);
            }
        }

        if response.secondary_clicked() {
            // Right-click - show context menu
            if let Some(pos) = response.interact_pointer_pos() {
                if !self.selected_entries.contains(&index) {
                    self.selected_entries = vec![index];
                }
                self.context_menu.show_at(pos, Some(index));
            }
        }
    }
}

impl eframe::App for FileExplorerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Apply theme
        self.apply_theme(ctx);
        
        // Handle keyboard shortcuts
        self.handle_keyboard_shortcuts(ctx);
        
        // Show main UI
        ui::show_top_panel(self, ctx);
        
        // Show settings window
        self.settings_window.show_window(ctx, &mut self.settings);
        
        // Show context menu
        if let Some(action) = crate::context_menu::show_context_menu(
            ctx,
            &mut self.context_menu,
            &self.entries,
            &self.selected_entries,
            self.clipboard_operation.is_some(),
        ) {
            self.handle_context_menu_action(action);
        }
        
        // Handle empty space right-click
        ctx.input(|i| {
            if i.pointer.secondary_clicked() && !self.context_menu.is_visible() {
                if let Some(pos) = i.pointer.interact_pos() {
                    self.selected_entries.clear();
                    self.context_menu.show_at(pos, None);
                }
            }
        });
        
        // Central panel for file list
        egui::CentralPanel::default().show(ctx, |ui| {
            ui::show_file_list(self, ui);
        });
        
        // Terminal panel
        terminal_ui::show_terminal_panel(ctx, &mut self.terminal, &self.settings);
        
        // Show dialogs
        ui::show_dialogs(self, ctx);
        
        // Update current directory from terminal if changed
        if self.terminal.current_dir != self.current_path {
            self.navigate_to(self.terminal.current_dir.clone());
        }
    }
} 