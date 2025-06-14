use eframe::egui::{self, Color32, Context, RichText, ScrollArea, Ui};
use crate::app::FileExplorerApp;
use crate::models::{Theme, ViewMode};
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
                    app.show_settings = !app.show_settings;
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
            ui.selectable_value(&mut app.view_mode, ViewMode::List, "📋 List");
            ui.selectable_value(&mut app.view_mode, ViewMode::Grid, "⊞ Grid");
            
            ui.separator();
            
            ui.label("Theme:");
            ui.selectable_value(&mut app.theme, Theme::Light, "☀ Light");
            ui.selectable_value(&mut app.theme, Theme::Dark, "🌙 Dark");
        });

        // Settings panel
        if app.show_settings {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Settings:");
                if ui.checkbox(&mut app.show_hidden, "Show hidden files").changed() {
                    app.read_directory();
                }
            });
        }
        
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
    match app.view_mode {
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
            
            app.handle_file_interaction(response, i);
        }
    });
}

fn show_grid_view(app: &mut FileExplorerApp, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            // Clone entries to avoid borrow issues
            let entries = app.entries.clone();
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
                
                app.handle_file_interaction(response, i);
            }
        });
    });
}

pub fn show_context_menu(app: &mut FileExplorerApp, ctx: &Context) {
    if let (Some(pos), Some(_)) = (app.context_menu_pos, app.context_menu_index) {
        egui::Area::new("context_menu".into())
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    if ui.button("📋 Copy").clicked() {
                        app.copy_selected();
                        app.context_menu_pos = None;
                    }
                    if ui.button("✂️ Cut").clicked() {
                        app.cut_selected();
                        app.context_menu_pos = None;
                    }
                    if ui.button("📁 Paste").clicked() {
                        app.paste();
                        app.context_menu_pos = None;
                    }
                    ui.separator();
                    if ui.button("🗑️ Delete").clicked() {
                        app.delete_selected();
                        app.context_menu_pos = None;
                    }
                    if ui.button("✏️ Rename").clicked() && app.selected_entries.len() == 1 {
                        app.show_rename_dialog = true;
                        app.rename_index = Some(app.selected_entries[0]);
                        app.rename_text = app.entries[app.selected_entries[0]].name.clone();
                        app.context_menu_pos = None;
                    }
                    ui.separator();
                    if ui.button("ℹ️ Properties").clicked() && app.selected_entries.len() == 1 {
                        app.show_properties_dialog = true;
                        app.properties_file = Some(app.entries[app.selected_entries[0]].clone());
                        app.context_menu_pos = None;
                    }
                });
            });
        
        if ctx.input(|i| i.pointer.any_click()) {
            app.context_menu_pos = None;
            app.context_menu_index = None;
        }
    }
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

pub fn show_terminal(app: &mut FileExplorerApp, ctx: &Context) {
    egui::TopBottomPanel::bottom("terminal_panel").resizable(true).show(ctx, |ui| {
        ui.label(RichText::new("Terminal").strong());
        ui.separator();
        
        ScrollArea::vertical()
            .stick_to_bottom(true)
            .max_height(200.0)
            .show(ui, |ui| {
                for line in &app.terminal_output {
                    ui.label(line);
                }
            });

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("$");
            let response = ui.text_edit_singleline(&mut app.terminal_input);
            
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                app.execute_command(&app.terminal_input.clone());
                app.terminal_input.clear();
                response.request_focus();
            }

            if response.has_focus() {
                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) && !app.terminal_history.is_empty() {
                    if app.terminal_history_index > 0 {
                        app.terminal_history_index -= 1;
                        app.terminal_input = app.terminal_history[app.terminal_history_index].clone();
                    }
                } else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) && !app.terminal_history.is_empty() {
                    if app.terminal_history_index < app.terminal_history.len() - 1 {
                        app.terminal_history_index += 1;
                        app.terminal_input = app.terminal_history[app.terminal_history_index].clone();
                    } else {
                        app.terminal_history_index = app.terminal_history.len();
                        app.terminal_input.clear();
                    }
                }
            }

            if ui.button("Execute").clicked() && !app.terminal_input.trim().is_empty() {
                app.execute_command(&app.terminal_input.clone());
                app.terminal_input.clear();
            }
        });
    });
} 