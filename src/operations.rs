use std::fs::{self, File};
use std::path::PathBuf;
use std::process::Command;
use chrono::Local;
use crate::models::FileEntry;

pub fn create_new_file(path: &PathBuf, name: &str) -> Result<(), String> {
    let file_path = path.join(name);
    match File::create(&file_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to create file: {}", e)),
    }
}

pub fn create_new_folder(path: &PathBuf, name: &str) -> Result<(), String> {
    let folder_path = path.join(name);
    match fs::create_dir(&folder_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to create folder: {}", e)),
    }
}

pub fn rename_file(old_path: &PathBuf, new_name: &str) -> Result<(), String> {
    let new_path = old_path.parent().unwrap().join(new_name);
    match fs::rename(old_path, &new_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to rename: {}", e)),
    }
}

pub fn delete_item(path: &PathBuf) -> Result<(), String> {
    let result = if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };
    
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to delete: {}", e)),
    }
}

pub fn copy_item(source: &PathBuf, destination: &PathBuf) -> Result<(), String> {
    if source.is_file() {
        match fs::copy(source, destination) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to copy file: {}", e)),
        }
    } else if source.is_dir() {
        match fs::create_dir_all(destination) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to create directory: {}", e)),
        }
    } else {
        Err("Unknown file type".to_string())
    }
}

pub fn move_item(source: &PathBuf, destination: &PathBuf) -> Result<(), String> {
    match fs::rename(source, destination) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to move: {}", e)),
    }
}

pub fn execute_system_command(command: &str, current_dir: &PathBuf) -> (Vec<String>, Option<String>) {
    let mut output_lines = Vec::new();
    let mut error = None;
    
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return (output_lines, error);
    }

    let mut cmd = Command::new(parts[0]);
    if parts.len() > 1 {
        cmd.args(&parts[1..]);
    }
    cmd.current_dir(current_dir);

    match cmd.output() {
        Ok(output) => {
            if !output.stdout.is_empty() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    output_lines.push(line.to_string());
                }
            }
            if !output.stderr.is_empty() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                for line in stderr.lines() {
                    output_lines.push(format!("Error: {}", line));
                }
            }
            if output.stdout.is_empty() && output.stderr.is_empty() {
                output_lines.push("Command executed successfully".to_string());
            }
        }
        Err(e) => {
            error = Some(format!("Failed to execute '{}': {}", command, e));
        }
    }
    
    (output_lines, error)
}

pub fn open_file(path: &PathBuf) -> Result<(), String> {
    match open::that(path) {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Failed to open file: {}", e)),
    }
}

pub fn read_directory(path: &PathBuf, show_hidden: bool) -> Result<Vec<FileEntry>, String> {
    match fs::read_dir(path) {
        Ok(entries) => {
            let mut file_entries: Vec<FileEntry> = entries
                .filter_map(Result::ok)
                .filter_map(|entry| {
                    let path = entry.path();
                    let file_name = path.file_name()?.to_str()?.to_string();
                    
                    if !show_hidden && file_name.starts_with('.') {
                        return None;
                    }
                    
                    let metadata = entry.metadata().ok()?;
                    let modified = metadata.modified().ok()?;
                    let modified = chrono::DateTime::<Local>::from(modified);
                    
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

            Ok(file_entries)
        }
        Err(e) => Err(format!("Error reading directory: {}", e)),
    }
} 