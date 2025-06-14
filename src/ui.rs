use eframe::egui::{self, Color32, Context, RichText, ScrollArea, Ui};
use crate::app::FileExplorerApp;
use crate::models::ViewMode;
use crate::utils::{format_file_size, get_file_icon};

pub fn show_top_panel(app: &mut FileExplorerApp, ctx: &Context) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        // Navigation row
        ui.horizontal(|ui| {
            // Back/Forward buttons
            ui.add_enabled(app.history_index > 0, egui::Button::new("⬅")).clicked().then(|| app.go_back());
            ui.add_enabled(app.history_index < app.navigation_history.len() - 1, egui::Button::new("➡")).clicked().then(|| app.go_forward());
            
            if ui.button("⬆ Up").clicked() {
                if let Some(parent) = app.current_path.parent() {
                    app.navigate_to(parent.to_path_buf());
                }
            }
            
            ui.separator();
            
            // Breadcrumb navigation - collect paths first to avoid borrow issues
            let breadcrumbs = app.breadcrumbs.clone();
            for (i, (name, path)) in breadcrumbs.iter().enumerate() {
                if i > 0 {
                    ui.label("/");
                }
                if ui.link(name).clicked() {
                    app.navigate_to(path.clone());
                }
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("⚙ Settings").clicked() {
                    app.settings_window.show = true;
                }
                
                if ui.button("⭐ Bookmarks").clicked() {
                    app.show_bookmarks = !app.show_bookmarks;
                }
            });
        });
        
        // Action buttons row
        ui.horizontal(|ui| {
            if ui.button("📄 New File").clicked() {
                app.show_new_file_dialog = true;
                app.new_name_input.clear();
            }
            
            if ui.button("📁 New Folder").clicked() {
                app.show_new_folder_dialog = true;
                app.new_name_input.clear();
            }
            
            ui.separator();
            
            ui.label("View:");
            ui.selectable_value(&mut app.settings.view_mode, ViewMode::List, "📋 List");
            ui.selectable_value(&mut app.settings.view_mode, ViewMode::Grid, "⊞ Grid");
            
            ui.separator();
            
            if ui.button("🔄 Refresh").clicked() {
                app.read_directory();
            }
        });

        // Bookmarks panel
        if app.show_bookmarks {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Add bookmark:");
                ui.text_edit_singleline(&mut app.bookmark_name_input);
                if ui.button("Add").clicked() && !app.bookmark_name_input.is_empty() {
                    app.add_bookmark(app.bookmark_name_input.clone(), app.current_path.clone());
                    app.bookmark_name_input.clear();
                }
            });
            
            // Clone bookmarks to avoid borrow issues
            let bookmarks = app.bookmarks.clone();
            let mut bookmark_to_remove = None;
            ui.horizontal_wrapped(|ui| {
                for (i, bookmark) in bookmarks.iter().enumerate() {
                    if ui.button(&bookmark.name).clicked() {
                        app.navigate_to(bookmark.path.clone());
                    }
                    if ui.button("❌").clicked() {
                        bookmark_to_remove = Some(i);
                    }
                }
            });
            
            if let Some(index) = bookmark_to_remove {
                app.bookmarks.remove(index);
                app.save_bookmarks();
            }
        }

        ui.separator();

        // Status messages
        if let Some(error_message) = &app.error {
            ui.colored_label(Color32::RED, error_message);
        }

        if let Some(status_message) = &app.status_message {
            ui.colored_label(Color32::from_rgb(0, 150, 0), status_message);
        }
    });
}

pub fn show_file_list(app: &mut FileExplorerApp, ui: &mut Ui) {
    match app.settings.view_mode {
        ViewMode::List => show_list_view(app, ui),
        ViewMode::Grid => show_grid_view(app, ui),
    }
}

fn show_list_view(app: &mut FileExplorerApp, ui: &mut Ui) {
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
        let entries = app.entries.clone();
        let ctx = ui.ctx().clone();
        for (i, entry) in entries.iter().enumerate() {
            let response = ui.horizontal(|ui| {
                let icon = get_file_icon(entry);
                let selected = app.selected_entries.contains(&i);
                
                let response = ui.selectable_label(selected, format!("{} {}", icon, entry.name));
                ui.separator();
                
                if entry.is_dir {
                    ui.label("--");
                } else {
                    ui.label(format_file_size(entry.size));
                }
                ui.separator();
                
                ui.label(entry.modified.format("%Y-%m-%d %H:%M").to_string());
                
                response
            }).inner;
            
            app.handle_file_interaction(response, i, &ctx);
        }
    });
}

fn show_grid_view(app: &mut FileExplorerApp, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            // Clone entries to avoid borrow issues
            let entries = app.entries.clone();
            let ctx = ui.ctx().clone();
            for (i, entry) in entries.iter().enumerate() {
                let icon = get_file_icon(&entry);
                let selected = app.selected_entries.contains(&i);
                
                let response = ui.vertical(|ui| {
                    ui.set_max_width(80.0);
                    ui.set_min_height(80.0);
                    
                    let response = ui.selectable_label(selected, RichText::new(icon).size(32.0));
                    ui.label(&entry.name);
                    
                    response
                }).inner;
                
                app.handle_file_interaction(response, i, &ctx);
            }
        });
    });
}

pub fn show_dialogs(app: &mut FileExplorerApp, ctx: &Context) {
    // Properties dialog
    if app.show_properties_dialog {
        egui::Window::new("Properties")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                if let Some(ref file) = app.properties_file {
                    ui.label(format!("Name: {}", file.name));
                    ui.label(format!("Path: {}", file.path.display()));
                    ui.label(format!("Type: {}", if file.is_dir { "Directory" } else { "File" }));
                    if !file.is_dir {
                        ui.label(format!("Size: {}", format_file_size(file.size)));
                    }
                    ui.label(format!("Modified: {}", file.modified.format("%Y-%m-%d %H:%M:%S")));
                    
                    if ui.button("Close").clicked() {
                        app.show_properties_dialog = false;
                    }
                }
            });
    }
    
    // Rename dialog
    if app.show_rename_dialog {
        egui::Window::new("Rename")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("New name:");
                let response = ui.text_edit_singleline(&mut app.rename_text);
                
                ui.horizontal(|ui| {
                    if ui.button("Rename").clicked() && !app.rename_text.is_empty() {
                        if let Some(index) = app.rename_index {
                            let new_name = app.rename_text.clone();
                            app.rename_file(index, &new_name);
                        }
                        app.show_rename_dialog = false;
                    }
                    if ui.button("Cancel").clicked() {
                        app.show_rename_dialog = false;
                    }
                });
                
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !app.rename_text.is_empty() {
                    if let Some(index) = app.rename_index {
                        let new_name = app.rename_text.clone();
                        app.rename_file(index, &new_name);
                    }
                    app.show_rename_dialog = false;
                }
            });
    }
    
    // New file dialog
    if app.show_new_file_dialog {
        egui::Window::new("New File")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("File name:");
                let response = ui.text_edit_singleline(&mut app.new_name_input);
                
                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() && !app.new_name_input.is_empty() {
                        let name = app.new_name_input.clone();
                        app.create_new_file(&name);
                        app.show_new_file_dialog = false;
                    }
                    if ui.button("Cancel").clicked() {
                        app.show_new_file_dialog = false;
                    }
                });
                
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !app.new_name_input.is_empty() {
                    let name = app.new_name_input.clone();
                    app.create_new_file(&name);
                    app.show_new_file_dialog = false;
                }
            });
    }
    
    // New folder dialog
    if app.show_new_folder_dialog {
        egui::Window::new("New Folder")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Folder name:");
                let response = ui.text_edit_singleline(&mut app.new_name_input);
                
                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() && !app.new_name_input.is_empty() {
                        let name = app.new_name_input.clone();
                        app.create_new_folder(&name);
                        app.show_new_folder_dialog = false;
                    }
                    if ui.button("Cancel").clicked() {
                        app.show_new_folder_dialog = false;
                    }
                });
                
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !app.new_name_input.is_empty() {
                    let name = app.new_name_input.clone();
                    app.create_new_folder(&name);
                    app.show_new_folder_dialog = false;
                }
            });
    }
} 