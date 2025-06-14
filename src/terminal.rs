use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct TerminalState {
    pub output_lines: Arc<Mutex<VecDeque<String>>>,
    pub input_buffer: String,
    pub history: Vec<String>,
    pub history_index: usize,
    pub current_dir: std::path::PathBuf,
    pub is_running_command: bool,
    pub shell_path: String,
    pub autocomplete_suggestions: Vec<String>,
    pub show_autocomplete: bool,
}

impl TerminalState {
    pub fn new() -> Self {
        let shell_path = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));
        
        Self {
            output_lines: Arc::new(Mutex::new(VecDeque::new())),
            input_buffer: String::new(),
            history: Vec::new(),
            history_index: 0,
            current_dir,
            is_running_command: false,
            shell_path,
            autocomplete_suggestions: Vec::new(),
            show_autocomplete: false,
        }
    }

    pub fn execute_command(&mut self, command: &str) {
        if command.trim().is_empty() {
            return;
        }

        // Add to history
        if self.history.is_empty() || self.history.last() != Some(&command.to_string()) {
            self.history.push(command.to_string());
        }
        self.history_index = self.history.len();

        // Add command to output
        {
            let mut output = self.output_lines.lock().unwrap();
            output.push_back(format!("{}$ {}", self.current_dir.display(), command));
            
            // Keep only last 1000 lines
            while output.len() > 1000 {
                output.pop_front();
            }
        }

        // Handle built-in commands
        if command.starts_with("cd ") {
            let path = command.strip_prefix("cd ").unwrap().trim();
            self.cd_internal(path);
            return;
        }

        // Execute external command
        self.execute_external_command(command);
    }

    fn cd_internal(&mut self, path: &str) {
        let new_path = if path.starts_with('/') {
            std::path::PathBuf::from(path)
        } else if path == "~" {
            dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"))
        } else if path.starts_with("~/") {
            dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("/"))
                .join(path.strip_prefix("~/").unwrap())
        } else {
            self.current_dir.join(path)
        };

        match std::env::set_current_dir(&new_path) {
            Ok(_) => {
                self.current_dir = new_path.canonicalize().unwrap_or(new_path);
                let mut output = self.output_lines.lock().unwrap();
                output.push_back(format!("Changed directory to: {}", self.current_dir.display()));
            }
            Err(e) => {
                let mut output = self.output_lines.lock().unwrap();
                output.push_back(format!("cd: {}: {}", path, e));
            }
        }
    }

    pub fn change_directory(&mut self, path: &str) {
        self.cd_internal(path);
    }

    fn execute_external_command(&mut self, command: &str) {
        let output_lines = Arc::clone(&self.output_lines);
        let current_dir = self.current_dir.clone();
        let command_string = command.to_string();

        self.is_running_command = true;

        thread::spawn(move || {
            let result = Command::new("sh")
                .arg("-c")
                .arg(&command_string)
                .current_dir(&current_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match result {
                Ok(mut child) => {
                    // Handle stdout
                    if let Some(stdout) = child.stdout.take() {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines() {
                            match line {
                                Ok(line) => {
                                    let mut output = output_lines.lock().unwrap();
                                    output.push_back(line);
                                    while output.len() > 1000 {
                                        output.pop_front();
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    }

                    // Handle stderr
                    if let Some(stderr) = child.stderr.take() {
                        let reader = BufReader::new(stderr);
                        for line in reader.lines() {
                            match line {
                                Ok(line) => {
                                    let mut output = output_lines.lock().unwrap();
                                    output.push_back(format!("ERROR: {}", line));
                                    while output.len() > 1000 {
                                        output.pop_front();
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    }

                    match child.wait() {
                        Ok(status) => {
                            if !status.success() {
                                let mut output = output_lines.lock().unwrap();
                                output.push_back(format!("Command exited with status: {}", status));
                            }
                        }
                        Err(e) => {
                            let mut output = output_lines.lock().unwrap();
                            output.push_back(format!("Failed to wait for command: {}", e));
                        }
                    }
                }
                Err(e) => {
                    let mut output = output_lines.lock().unwrap();
                    output.push_back(format!("Failed to execute command: {}", e));
                }
            }
        });
    }

    pub fn get_autocomplete_suggestions(&mut self, input: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Get command suggestions for empty input or command position
        if input.trim().is_empty() || !input.contains(' ') {
            let common_commands = vec![
                "ls", "cd", "pwd", "mkdir", "rmdir", "rm", "cp", "mv", "find", "grep",
                "cat", "less", "head", "tail", "touch", "chmod", "chown", "ps", "kill",
                "top", "df", "du", "tar", "zip", "unzip", "wget", "curl", "git", "nano",
                "vim", "emacs", "code", "python", "node", "npm", "cargo", "rustc"
            ];

            for cmd in common_commands {
                if cmd.starts_with(input) {
                    suggestions.push(cmd.to_string());
                }
            }
        }

        // Get file/directory suggestions
        let (path_part, file_part) = if let Some(space_pos) = input.rfind(' ') {
            let args = &input[space_pos + 1..];
            if let Some(slash_pos) = args.rfind('/') {
                (&args[..slash_pos + 1], &args[slash_pos + 1..])
            } else {
                ("", args)
            }
        } else {
            ("", input)
        };

        let search_dir = if path_part.is_empty() {
            self.current_dir.clone()
        } else if path_part.starts_with('/') {
            std::path::PathBuf::from(path_part)
        } else {
            self.current_dir.join(path_part)
        };

        if let Ok(entries) = std::fs::read_dir(&search_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with(file_part) {
                        let suggestion = if path_part.is_empty() {
                            name.to_string()
                        } else {
                            format!("{}{}", path_part, name)
                        };
                        
                        if entry.file_type().map_or(false, |ft| ft.is_dir()) {
                            suggestions.push(format!("{}/", suggestion));
                        } else {
                            suggestions.push(suggestion);
                        }
                    }
                }
            }
        }

        suggestions.sort();
        suggestions.truncate(10); // Limit to 10 suggestions
        suggestions
    }

    pub fn navigate_history(&mut self, direction: i32) {
        if self.history.is_empty() {
            return;
        }

        if direction < 0 && self.history_index > 0 {
            self.history_index -= 1;
            self.input_buffer = self.history[self.history_index].clone();
        } else if direction > 0 && self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            self.input_buffer = self.history[self.history_index].clone();
        } else if direction > 0 && self.history_index == self.history.len() - 1 {
            self.history_index = self.history.len();
            self.input_buffer.clear();
        }
    }

    pub fn get_output_lines(&self) -> Vec<String> {
        let output = self.output_lines.lock().unwrap();
        output.iter().cloned().collect()
    }

    pub fn clear_output(&mut self) {
        let mut output = self.output_lines.lock().unwrap();
        output.clear();
        output.push_back(format!("Terminal cleared. Current directory: {}", self.current_dir.display()));
    }
} 