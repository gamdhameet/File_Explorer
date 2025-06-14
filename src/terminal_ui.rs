use eframe::egui::{self, Color32, Context, RichText, ScrollArea, TextStyle};
use crate::terminal::TerminalState;
use crate::settings::AppSettings;

pub fn show_terminal_panel(ctx: &Context, terminal: &mut TerminalState, settings: &AppSettings) {
    egui::TopBottomPanel::bottom("terminal_panel")
        .resizable(true)
        .min_height(100.0)
        .default_height(250.0)
        .show(ctx, |ui| {
            // Terminal header
            ui.horizontal(|ui| {
                ui.label(RichText::new("⚡ Terminal").strong().size(16.0));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🗑 Clear").clicked() {
                        terminal.clear_output();
                    }
                    
                    ui.label(format!("📁 {}", terminal.current_dir.display()));
                });
            });
            
            ui.separator();
            
            // Terminal output area
            let output_height = ui.available_height() - 60.0; // Reserve space for input
            
            ScrollArea::vertical()
                .stick_to_bottom(true)
                .max_height(output_height)
                .show(ui, |ui| {
                    ui.style_mut().override_text_style = Some(TextStyle::Monospace);
                    
                    let output_lines = terminal.get_output_lines();
                    for line in &output_lines {
                        if line.starts_with("ERROR:") {
                            ui.colored_label(Color32::from_rgb(255, 100, 100), line);
                        } else if line.contains("$") && !line.starts_with(" ") {
                            // Command line
                            ui.colored_label(Color32::from_rgb(100, 200, 100), line);
                        } else {
                            ui.label(line);
                        }
                    }
                    
                    // Show running indicator
                    if terminal.is_running_command {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Running command...");
                        });
                    }
                });
            
            ui.separator();
            
            // Terminal input area
            show_terminal_input(ui, terminal, settings);
        });
}

fn show_terminal_input(ui: &mut egui::Ui, terminal: &mut TerminalState, _settings: &AppSettings) {
    ui.horizontal(|ui| {
        // Prompt
        ui.label(RichText::new("$").color(Color32::from_rgb(100, 200, 100)).strong());
        
        // Input field
        let response = ui.add(
            egui::TextEdit::singleline(&mut terminal.input_buffer)
                .desired_width(ui.available_width() - 80.0)
                .font(TextStyle::Monospace)
        );
        
        // Execute button
        if ui.button("⏎ Run").clicked() && !terminal.input_buffer.trim().is_empty() {
            execute_command(terminal);
        }
        
        // Handle input events
        if response.has_focus() {
            handle_terminal_input_events(ui, terminal, &response);
        }
        
        // Auto-focus the input
        if !response.has_focus() && !terminal.is_running_command {
            response.request_focus();
        }
    });
    
    // Show autocomplete suggestions
    if terminal.show_autocomplete && !terminal.autocomplete_suggestions.is_empty() {
        show_autocomplete_popup(ui, terminal);
    }
}

fn handle_terminal_input_events(
    ui: &mut egui::Ui, 
    terminal: &mut TerminalState, 
    _response: &egui::Response
) {
    let events = ui.input(|i| i.events.clone());
    
    for event in events {
        match event {
            egui::Event::Key { key, pressed: true, modifiers, .. } => {
                match key {
                    egui::Key::Enter => {
                        if !terminal.input_buffer.trim().is_empty() {
                            execute_command(terminal);
                        }
                    }
                    egui::Key::Tab => {
                        handle_tab_completion(terminal);
                    }
                    egui::Key::ArrowUp => {
                        terminal.navigate_history(-1);
                    }
                    egui::Key::ArrowDown => {
                        terminal.navigate_history(1);
                    }
                    egui::Key::C if modifiers.ctrl => {
                        // Ctrl+C - interrupt current command (if running)
                        if terminal.is_running_command {
                            terminal.is_running_command = false;
                            let mut output = terminal.output_lines.lock().unwrap();
                            output.push_back("^C".to_string());
                        }
                    }
                    egui::Key::L if modifiers.ctrl => {
                        // Ctrl+L - clear terminal
                        terminal.clear_output();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn execute_command(terminal: &mut TerminalState) {
    let command = terminal.input_buffer.trim().to_string();
    terminal.execute_command(&command);
    terminal.input_buffer.clear();
    terminal.show_autocomplete = false;
}

fn handle_tab_completion(terminal: &mut TerminalState) {
    let input_buffer = terminal.input_buffer.clone();
    let suggestions = terminal.get_autocomplete_suggestions(&input_buffer);
    
    if suggestions.len() == 1 {
        // Single suggestion - auto-complete
        let suggestion = &suggestions[0];
        if let Some(space_pos) = terminal.input_buffer.rfind(' ') {
            let prefix = &terminal.input_buffer[..space_pos + 1];
            terminal.input_buffer = format!("{}{}", prefix, suggestion);
        } else {
            terminal.input_buffer = suggestion.clone();
        }
        terminal.show_autocomplete = false;
    } else if suggestions.len() > 1 {
        // Multiple suggestions - show popup
        terminal.autocomplete_suggestions = suggestions;
        terminal.show_autocomplete = true;
    }
}

fn show_autocomplete_popup(ui: &mut egui::Ui, terminal: &mut TerminalState) {
    let popup_id = egui::Id::new("autocomplete_popup");
    let dummy_response = ui.allocate_response(
        egui::Vec2::new(200.0, 20.0), 
        egui::Sense::hover()
    );
    
    egui::popup_below_widget(ui, popup_id, &dummy_response, |ui| {
        ui.set_max_width(300.0);
        ui.set_max_height(150.0);
        
        ui.label(RichText::new("Suggestions:").strong());
        ui.separator();
        
        ScrollArea::vertical().show(ui, |ui| {
            let suggestions = terminal.autocomplete_suggestions.clone();
            for (_i, suggestion) in suggestions.iter().enumerate() {
                let is_dir = suggestion.ends_with('/');
                let icon = if is_dir { "📁" } else { "📄" };
                
                if ui.button(format!("{} {}", icon, suggestion)).clicked() {
                    // Apply the selected suggestion
                    if let Some(space_pos) = terminal.input_buffer.rfind(' ') {
                        let prefix = &terminal.input_buffer[..space_pos + 1];
                        terminal.input_buffer = format!("{}{}", prefix, suggestion);
                    } else {
                        terminal.input_buffer = suggestion.clone();
                    }
                    terminal.show_autocomplete = false;
                }
            }
        });
        
        ui.separator();
        ui.label(RichText::new("Press Tab to cycle, Esc to cancel").small());
    });
}

pub fn show_terminal_shortcuts_help(ui: &mut egui::Ui) {
    ui.collapsing("Terminal Shortcuts", |ui| {
        ui.label("• Enter: Execute command");
        ui.label("• Tab: Auto-complete");
        ui.label("• ↑/↓: Navigate history");
        ui.label("• Ctrl+C: Interrupt command");
        ui.label("• Ctrl+L: Clear terminal");
        ui.label("• cd <path>: Change directory");
        ui.label("• Built-in commands: cd, pwd, ls, clear");
    });
} 