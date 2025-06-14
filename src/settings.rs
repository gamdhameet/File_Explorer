use serde::{Deserialize, Serialize};
use eframe::egui::{self, Context};
use crate::models::{Theme, ViewMode};
use std::path::PathBuf;
use std::fs;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppSettings {
    // Appearance
    pub theme: Theme,
    pub view_mode: ViewMode,
    pub show_hidden_files: bool,
    pub icon_size: f32,
    pub font_size: f32,
    
    // Behavior
    pub double_click_to_open: bool,
    pub confirm_deletions: bool,
    pub auto_refresh: bool,
    pub remember_window_size: bool,
    
    // Terminal
    pub terminal_font_family: String,
    pub terminal_font_size: f32,
    pub terminal_max_history: usize,
    pub terminal_max_output_lines: usize,
    pub terminal_shell_path: String,
    
    // File operations
    pub default_editor: String,
    pub default_terminal: String,
    pub show_file_extensions: bool,
    pub natural_sort: bool,
    
    // Advanced
    pub enable_thumbnails: bool,
    pub thumbnail_size: f32,
    pub cache_thumbnails: bool,
    pub follow_symlinks: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::Light,
            view_mode: ViewMode::List,
            show_hidden_files: false,
            icon_size: 16.0,
            font_size: 14.0,
            
            double_click_to_open: true,
            confirm_deletions: true,
            auto_refresh: false,
            remember_window_size: true,
            
            terminal_font_family: "JetBrains Mono".to_string(),
            terminal_font_size: 12.0,
            terminal_max_history: 1000,
            terminal_max_output_lines: 1000,
            terminal_shell_path: std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()),
            
            default_editor: std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string()),
            default_terminal: std::env::var("TERMINAL").unwrap_or_else(|_| "gnome-terminal".to_string()),
            show_file_extensions: true,
            natural_sort: true,
            
            enable_thumbnails: true,
            thumbnail_size: 64.0,
            cache_thumbnails: true,
            follow_symlinks: false,
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        if let Ok(content) = fs::read_to_string(&config_path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    fn get_config_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("fileexp").join("settings.json")
        } else {
            PathBuf::from(".").join("fileexp_settings.json")
        }
    }
}

pub struct SettingsWindow {
    pub show: bool,
    pub current_tab: SettingsTab,
}

#[derive(PartialEq)]
pub enum SettingsTab {
    Appearance,
    Behavior,
    Terminal,
    FileOperations,
    Advanced,
}

impl SettingsWindow {
    pub fn new() -> Self {
        Self {
            show: false,
            current_tab: SettingsTab::Appearance,
        }
    }

    pub fn show_window(&mut self, ctx: &Context, settings: &mut AppSettings) {
        if !self.show {
            return;
        }

        egui::Window::new("⚙ Settings")
            .collapsible(false)
            .resizable(true)
            .default_width(500.0)
            .default_height(400.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Tab buttons
                    ui.vertical(|ui| {
                        ui.set_min_width(120.0);
                        
                        ui.selectable_value(&mut self.current_tab, SettingsTab::Appearance, "🎨 Appearance");
                        ui.selectable_value(&mut self.current_tab, SettingsTab::Behavior, "⚙ Behavior");
                        ui.selectable_value(&mut self.current_tab, SettingsTab::Terminal, "⚡ Terminal");
                        ui.selectable_value(&mut self.current_tab, SettingsTab::FileOperations, "📁 File Ops");
                        ui.selectable_value(&mut self.current_tab, SettingsTab::Advanced, "🔧 Advanced");
                    });
                    
                    ui.separator();
                    
                    // Tab content
                    ui.vertical(|ui| {
                        match self.current_tab {
                            SettingsTab::Appearance => self.show_appearance_tab(ui, settings),
                            SettingsTab::Behavior => self.show_behavior_tab(ui, settings),
                            SettingsTab::Terminal => self.show_terminal_tab(ui, settings),
                            SettingsTab::FileOperations => self.show_file_operations_tab(ui, settings),
                            SettingsTab::Advanced => self.show_advanced_tab(ui, settings),
                        }
                    });
                });
                
                ui.separator();
                
                // Bottom buttons
                ui.horizontal(|ui| {
                    if ui.button("💾 Save").clicked() {
                        if let Err(e) = settings.save() {
                            eprintln!("Failed to save settings: {}", e);
                        }
                    }
                    
                    if ui.button("🔄 Reset to Defaults").clicked() {
                        *settings = AppSettings::default();
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✖ Close").clicked() {
                            self.show = false;
                        }
                    });
                });
            });
    }

    fn show_appearance_tab(&mut self, ui: &mut egui::Ui, settings: &mut AppSettings) {
        ui.heading("Appearance Settings");
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Theme:");
            ui.selectable_value(&mut settings.theme, Theme::Light, "☀ Light");
            ui.selectable_value(&mut settings.theme, Theme::Dark, "🌙 Dark");
        });
        
        ui.horizontal(|ui| {
            ui.label("View Mode:");
            ui.selectable_value(&mut settings.view_mode, ViewMode::List, "📋 List");
            ui.selectable_value(&mut settings.view_mode, ViewMode::Grid, "⊞ Grid");
        });
        
        ui.horizontal(|ui| {
            ui.label("Icon Size:");
            ui.add(egui::Slider::new(&mut settings.icon_size, 12.0..=32.0).suffix(" px"));
        });
        
        ui.horizontal(|ui| {
            ui.label("Font Size:");
            ui.add(egui::Slider::new(&mut settings.font_size, 10.0..=20.0).suffix(" pt"));
        });
        
        ui.checkbox(&mut settings.show_file_extensions, "Show file extensions");
        ui.checkbox(&mut settings.enable_thumbnails, "Enable thumbnails");
        
        if settings.enable_thumbnails {
            ui.horizontal(|ui| {
                ui.label("Thumbnail Size:");
                ui.add(egui::Slider::new(&mut settings.thumbnail_size, 32.0..=128.0).suffix(" px"));
            });
        }
    }

    fn show_behavior_tab(&mut self, ui: &mut egui::Ui, settings: &mut AppSettings) {
        ui.heading("Behavior Settings");
        ui.separator();
        
        ui.checkbox(&mut settings.show_hidden_files, "Show hidden files");
        ui.checkbox(&mut settings.double_click_to_open, "Double-click to open files");
        ui.checkbox(&mut settings.confirm_deletions, "Confirm file deletions");
        ui.checkbox(&mut settings.auto_refresh, "Auto-refresh directory");
        ui.checkbox(&mut settings.remember_window_size, "Remember window size");
        ui.checkbox(&mut settings.natural_sort, "Natural sorting (1, 2, 10 instead of 1, 10, 2)");
        ui.checkbox(&mut settings.follow_symlinks, "Follow symbolic links");
    }

    fn show_terminal_tab(&mut self, ui: &mut egui::Ui, settings: &mut AppSettings) {
        ui.heading("Terminal Settings");
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Shell Path:");
            ui.text_edit_singleline(&mut settings.terminal_shell_path);
        });
        
        ui.horizontal(|ui| {
            ui.label("Font Family:");
            ui.text_edit_singleline(&mut settings.terminal_font_family);
        });
        
        ui.horizontal(|ui| {
            ui.label("Font Size:");
            ui.add(egui::Slider::new(&mut settings.terminal_font_size, 8.0..=20.0).suffix(" pt"));
        });
        
        ui.horizontal(|ui| {
            ui.label("Max History:");
            ui.add(egui::Slider::new(&mut settings.terminal_max_history, 100..=10000));
        });
        
        ui.horizontal(|ui| {
            ui.label("Max Output Lines:");
            ui.add(egui::Slider::new(&mut settings.terminal_max_output_lines, 100..=10000));
        });
        
        ui.label("Terminal Features:");
        ui.label("• Command history (Up/Down arrows)");
        ui.label("• Tab completion for files and commands");
        ui.label("• Built-in cd command");
        ui.label("• Real-time command output");
    }

    fn show_file_operations_tab(&mut self, ui: &mut egui::Ui, settings: &mut AppSettings) {
        ui.heading("File Operations");
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Default Editor:");
            ui.text_edit_singleline(&mut settings.default_editor);
        });
        
        ui.horizontal(|ui| {
            ui.label("Default Terminal:");
            ui.text_edit_singleline(&mut settings.default_terminal);
        });
        
        ui.label("Supported Operations:");
        ui.label("• Copy, Cut, Paste");
        ui.label("• Create new files and folders");
        ui.label("• Rename and delete");
        ui.label("• Compress and extract archives");
        ui.label("• Open with specific applications");
        ui.label("• Set file permissions");
    }

    fn show_advanced_tab(&mut self, ui: &mut egui::Ui, settings: &mut AppSettings) {
        ui.heading("Advanced Settings");
        ui.separator();
        
        ui.checkbox(&mut settings.cache_thumbnails, "Cache thumbnails");
        
        ui.label("Performance:");
        ui.label("• Lazy loading for large directories");
        ui.label("• Background thumbnail generation");
        ui.label("• Efficient file watching");
        
        ui.separator();
        
        ui.label("Debug Information:");
        ui.label(format!("Config file: {}", AppSettings::get_config_path().display()));
        ui.label(format!("Shell: {}", settings.terminal_shell_path));
        ui.label(format!("Editor: {}", settings.default_editor));
        
        if ui.button("🗂 Open Config Directory").clicked() {
            if let Some(parent) = AppSettings::get_config_path().parent() {
                let _ = open::that(parent);
            }
        }
    }
} 