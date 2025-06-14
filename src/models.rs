use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Local};

#[derive(Clone, Debug)]
pub enum FileOperation {
    Copy(Vec<PathBuf>),
    Cut(Vec<PathBuf>),
}

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: DateTime<Local>,
    pub name: String,
    pub extension: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum ViewMode {
    List,
    Grid,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Theme {
    Light,
    Dark,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Bookmark {
    pub name: String,
    pub path: PathBuf,
} 