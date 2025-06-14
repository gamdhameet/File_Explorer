use eframe::egui::{self, Context, Pos2, RichText};
use crate::models::FileEntry;

#[derive(Clone, Debug)]
pub enum ContextMenuAction {
    Open,
    OpenWith,
    Cut,
    Copy,
    Paste,
    Delete,
    Rename,
    Properties,
    CreateNew(NewItemType),
    Compress,
    Extract,
    SetAsWallpaper,
    AddToBookmarks,
    CopyPath,
    OpenInTerminal,
    OpenInEditor,
    Share,
    SendTo,
}

#[derive(Clone, Debug)]
pub enum NewItemType {
    File,
    Folder,
    Shortcut,
    Document,
    Spreadsheet,
    Presentation,
}

pub struct ContextMenuState {
    pub position: Option<Pos2>,
    pub target_index: Option<usize>,
    pub show_new_submenu: bool,
    pub show_open_with_submenu: bool,
    pub show_send_to_submenu: bool,
    pub selected_action: Option<ContextMenuAction>,
}

impl ContextMenuState {
    pub fn new() -> Self {
        Self {
            position: None,
            target_index: None,
            show_new_submenu: false,
            show_open_with_submenu: false,
            show_send_to_submenu: false,
            selected_action: None,
        }
    }

    pub fn show_at(&mut self, pos: Pos2, index: Option<usize>) {
        self.position = Some(pos);
        self.target_index = index;
        self.show_new_submenu = false;
        self.show_open_with_submenu = false;
        self.show_send_to_submenu = false;
    }

    pub fn hide(&mut self) {
        self.position = None;
        self.target_index = None;
        self.show_new_submenu = false;
        self.show_open_with_submenu = false;
        self.show_send_to_submenu = false;
    }

    pub fn is_visible(&self) -> bool {
        self.position.is_some()
    }
}

pub fn show_context_menu(
    ctx: &Context,
    state: &mut ContextMenuState,
    entries: &[FileEntry],
    selected_entries: &[usize],
    clipboard_has_content: bool,
) -> Option<ContextMenuAction> {
    if let Some(pos) = state.position {
        let mut action = None;
        
        egui::Area::new("context_menu".into())
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_min_width(180.0);
                    
                    // Determine context based on selection
                    let has_selection = !selected_entries.is_empty();
                    let single_selection = selected_entries.len() == 1;
                    let is_directory = single_selection && 
                        state.target_index.map_or(false, |i| entries.get(i).map_or(false, |e| e.is_dir));
                    
                    if has_selection {
                        // Actions for selected items
                        if ui.button("🔗 Open").clicked() {
                            action = Some(ContextMenuAction::Open);
                        }
                        
                        if single_selection {
                            if ui.button("📂 Open with...").clicked() {
                                state.show_open_with_submenu = true;
                            }
                        }
                        
                        ui.separator();
                        
                        if ui.button("✂️ Cut").clicked() {
                            action = Some(ContextMenuAction::Cut);
                        }
                        
                        if ui.button("📋 Copy").clicked() {
                            action = Some(ContextMenuAction::Copy);
                        }
                        
                        if single_selection {
                            if ui.button("📄 Copy path").clicked() {
                                action = Some(ContextMenuAction::CopyPath);
                            }
                        }
                        
                        ui.separator();
                        
                        if ui.button("🗑️ Delete").clicked() {
                            action = Some(ContextMenuAction::Delete);
                        }
                        
                        if single_selection {
                            if ui.button("✏️ Rename").clicked() {
                                action = Some(ContextMenuAction::Rename);
                            }
                        }
                        
                        ui.separator();
                        
                        // Compression options
                        if ui.button("🗜️ Compress").clicked() {
                            action = Some(ContextMenuAction::Compress);
                        }
                        
                        // Extract if it's an archive
                        if single_selection {
                            if let Some(index) = state.target_index {
                                if let Some(entry) = entries.get(index) {
                                    let ext = entry.extension.to_lowercase();
                                    if matches!(ext.as_str(), "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar") {
                                        if ui.button("📦 Extract").clicked() {
                                            action = Some(ContextMenuAction::Extract);
                                        }
                                    }
                                }
                            }
                        }
                        
                        ui.separator();
                        
                        // Directory-specific actions
                        if is_directory {
                            if ui.button("⚡ Open in terminal").clicked() {
                                action = Some(ContextMenuAction::OpenInTerminal);
                            }
                            
                            if ui.button("⭐ Add to bookmarks").clicked() {
                                action = Some(ContextMenuAction::AddToBookmarks);
                            }
                        }
                        
                        // File-specific actions
                        if single_selection && !is_directory {
                            if let Some(index) = state.target_index {
                                if let Some(entry) = entries.get(index) {
                                    let ext = entry.extension.to_lowercase();
                                    
                                    // Text files
                                    if matches!(ext.as_str(), "txt" | "md" | "rs" | "py" | "js" | "html" | "css" | "json" | "xml" | "yaml" | "toml") {
                                        if ui.button("📝 Open in editor").clicked() {
                                            action = Some(ContextMenuAction::OpenInEditor);
                                        }
                                    }
                                    
                                    // Image files
                                    if matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg") {
                                        if ui.button("🖼️ Set as wallpaper").clicked() {
                                            action = Some(ContextMenuAction::SetAsWallpaper);
                                        }
                                    }
                                }
                            }
                        }
                        
                        ui.separator();
                        
                        // Send to submenu
                        if ui.button("📤 Send to...").clicked() {
                            state.show_send_to_submenu = true;
                        }
                        
                        if ui.button("🔗 Share").clicked() {
                            action = Some(ContextMenuAction::Share);
                        }
                        
                        ui.separator();
                        
                        if single_selection {
                            if ui.button("ℹ️ Properties").clicked() {
                                action = Some(ContextMenuAction::Properties);
                            }
                        }
                        
                    } else {
                        // Actions for empty space (no selection)
                        if ui.button("📄 New").clicked() {
                            state.show_new_submenu = true;
                        }
                        
                        ui.separator();
                        
                        if clipboard_has_content {
                            if ui.button("📁 Paste").clicked() {
                                action = Some(ContextMenuAction::Paste);
                            }
                            ui.separator();
                        }
                        
                        if ui.button("⚡ Open terminal here").clicked() {
                            action = Some(ContextMenuAction::OpenInTerminal);
                        }
                        
                        if ui.button("⭐ Add to bookmarks").clicked() {
                            action = Some(ContextMenuAction::AddToBookmarks);
                        }
                        
                        ui.separator();
                        
                        if ui.button("🔄 Refresh").clicked() {
                            // This will be handled in the main app
                            action = Some(ContextMenuAction::Open); // Reuse for refresh
                        }
                    }
                    
                    // Show submenus
                    if state.show_new_submenu {
                        show_new_submenu(ui, &mut action);
                    }
                    
                    if state.show_open_with_submenu {
                        show_open_with_submenu(ui, &mut action);
                    }
                    
                    if state.show_send_to_submenu {
                        show_send_to_submenu(ui, &mut action);
                    }
                });
            });
        
        // Hide menu if clicked outside
        if ctx.input(|i| i.pointer.any_click()) {
            let pointer_pos = ctx.input(|i| i.pointer.interact_pos());
            if let Some(pointer_pos) = pointer_pos {
                let menu_rect = egui::Rect::from_min_size(pos, egui::Vec2::new(180.0, 300.0));
                if !menu_rect.contains(pointer_pos) {
                    state.hide();
                }
            }
        }
        
        if action.is_some() {
            state.hide();
        }
        
        action
    } else {
        None
    }
}

fn show_new_submenu(ui: &mut egui::Ui, action: &mut Option<ContextMenuAction>) {
    ui.separator();
    ui.label(RichText::new("New:").strong());
    
    if ui.button("📄 File").clicked() {
        *action = Some(ContextMenuAction::CreateNew(NewItemType::File));
    }
    
    if ui.button("📁 Folder").clicked() {
        *action = Some(ContextMenuAction::CreateNew(NewItemType::Folder));
    }
    
    if ui.button("🔗 Shortcut").clicked() {
        *action = Some(ContextMenuAction::CreateNew(NewItemType::Shortcut));
    }
    
    ui.separator();
    ui.label(RichText::new("Documents:").strong());
    
    if ui.button("📝 Text Document").clicked() {
        *action = Some(ContextMenuAction::CreateNew(NewItemType::Document));
    }
    
    if ui.button("📊 Spreadsheet").clicked() {
        *action = Some(ContextMenuAction::CreateNew(NewItemType::Spreadsheet));
    }
    
    if ui.button("📈 Presentation").clicked() {
        *action = Some(ContextMenuAction::CreateNew(NewItemType::Presentation));
    }
}

fn show_open_with_submenu(ui: &mut egui::Ui, action: &mut Option<ContextMenuAction>) {
    ui.separator();
    ui.label(RichText::new("Open with:").strong());
    
    if ui.button("📝 Text Editor").clicked() {
        *action = Some(ContextMenuAction::OpenInEditor);
    }
    
    if ui.button("🌐 Web Browser").clicked() {
        *action = Some(ContextMenuAction::OpenWith);
    }
    
    if ui.button("🖼️ Image Viewer").clicked() {
        *action = Some(ContextMenuAction::OpenWith);
    }
    
    if ui.button("📺 Video Player").clicked() {
        *action = Some(ContextMenuAction::OpenWith);
    }
    
    if ui.button("🎵 Audio Player").clicked() {
        *action = Some(ContextMenuAction::OpenWith);
    }
    
    if ui.button("📄 Document Viewer").clicked() {
        *action = Some(ContextMenuAction::OpenWith);
    }
}

fn show_send_to_submenu(ui: &mut egui::Ui, action: &mut Option<ContextMenuAction>) {
    ui.separator();
    ui.label(RichText::new("Send to:").strong());
    
    if ui.button("💾 Desktop").clicked() {
        *action = Some(ContextMenuAction::SendTo);
    }
    
    if ui.button("📁 Documents").clicked() {
        *action = Some(ContextMenuAction::SendTo);
    }
    
    if ui.button("📧 Email").clicked() {
        *action = Some(ContextMenuAction::SendTo);
    }
    
    if ui.button("📱 Mobile Device").clicked() {
        *action = Some(ContextMenuAction::SendTo);
    }
    
    if ui.button("☁️ Cloud Storage").clicked() {
        *action = Some(ContextMenuAction::SendTo);
    }
} 