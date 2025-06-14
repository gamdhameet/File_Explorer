use crate::models::FileEntry;
use std::fs;
use std::path::PathBuf;
use serde_json;
use crate::models::Bookmark;

pub fn format_file_size(size: u64) -> String {
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

pub fn get_file_icon(entry: &FileEntry) -> &'static str {
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

pub fn save_bookmarks(bookmarks: &Vec<Bookmark>) -> Result<(), String> {
    match serde_json::to_string(bookmarks) {
        Ok(json) => {
            match fs::write("bookmarks.json", json) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to save bookmarks: {}", e)),
            }
        },
        Err(e) => Err(format!("Failed to serialize bookmarks: {}", e)),
    }
}

pub fn load_bookmarks() -> Vec<Bookmark> {
    match fs::read_to_string("bookmarks.json") {
        Ok(contents) => {
            match serde_json::from_str(&contents) {
                Ok(bookmarks) => bookmarks,
                Err(_) => Vec::new(),
            }
        },
        Err(_) => Vec::new(),
    }
}

pub fn generate_breadcrumbs(path: &PathBuf) -> Vec<(String, PathBuf)> {
    let mut breadcrumbs = Vec::new();
    let mut current = path.clone();
    
    while let Some(parent) = current.parent() {
        if let Some(name) = current.file_name() {
            breadcrumbs.insert(0, (name.to_string_lossy().to_string(), current.clone()));
        }
        current = parent.to_path_buf();
    }
    
    if current.to_string_lossy() == "/" {
        breadcrumbs.insert(0, ("/".to_string(), current));
    }
    
    breadcrumbs
} 