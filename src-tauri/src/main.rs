#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, fs, path::PathBuf};
use tauri::command;
use tauri_plugin_shell::ShellExt;
use std::process::Command;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(serde::Serialize)]
struct FileInfo {
    name: String,
    path: String,
    size: u64,
}

#[derive(serde::Serialize)]
struct FileGroup {
    key: String, // extension or date
    files: Vec<FileInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DuplicateFile {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub duplicate_type: String, // "numbered", "exact", "similar"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub original_name: String,
    pub files: Vec<DuplicateFile>,
    pub total_size: u64,
}

fn get_downloads_path() -> PathBuf {
    dirs::download_dir().unwrap_or_default()
}

#[command]
fn get_downloads_files() -> Vec<FileInfo> {
    read_files_from_dir(&get_downloads_path())
}

#[command]
fn group_files_by_extension() -> Vec<FileGroup> {
    let files = read_files_from_dir(&get_downloads_path());
    let mut groups: HashMap<String, Vec<FileInfo>> = HashMap::new();

    for file in files {
        let ext = PathBuf::from(&file.path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        groups.entry(ext).or_default().push(file);
    }

    groups
        .into_iter()
        .map(|(key, files)| FileGroup { key, files })
        .collect()
}

#[command]
fn group_files_by_modified_date() -> Vec<FileGroup> {
    let mut groups: HashMap<String, Vec<FileInfo>> = HashMap::new();

    if let Ok(entries) = fs::read_dir(get_downloads_path()) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    let modified = metadata.modified().ok();
                    let datetime = modified
                        .and_then(|time| time.elapsed().ok())
                        .map(|elapsed| {
                            if elapsed.as_secs() < 86400 {
                                "Today".to_string()
                            } else if elapsed.as_secs() < 7 * 86400 {
                                "This Week".to_string()
                            } else {
                                "Older".to_string()
                            }
                        })
                        .unwrap_or("Unknown".to_string());

                    let file = FileInfo {
                        name: entry.file_name().to_string_lossy().to_string(),
                        path: entry.path().display().to_string(),
                        size: metadata.len(),
                    };

                    groups.entry(datetime).or_default().push(file);
                }
            }
        }
    }

    groups
        .into_iter()
        .map(|(key, files)| FileGroup { key, files })
        .collect()
}

fn read_files_from_dir(path: &PathBuf) -> Vec<FileInfo> {
    let mut files = vec![];

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    files.push(FileInfo {
                        name: entry.file_name().to_string_lossy().to_string(),
                        path: entry.path().display().to_string(),
                        size: metadata.len(),
                    });
                }
            }
        }
    }

    files
}

#[tauri::command]
fn find_duplicate_files(downloads_path: Option<String>) -> Result<Vec<DuplicateGroup>, String> {
    let downloads_dir = if let Some(path) = downloads_path {
        PathBuf::from(path)
    } else {
        // Default to user's Downloads folder
        dirs::download_dir().ok_or("Could not find Downloads directory")?
    };

    if !downloads_dir.exists() {
        return Err("Downloads directory does not exist".to_string());
    }

    let mut file_groups: HashMap<String, Vec<DuplicateFile>> = HashMap::new();
    let mut exact_matches: HashMap<String, Vec<DuplicateFile>> = HashMap::new();

    // Read all files in downloads directory
    let entries = fs::read_dir(&downloads_dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                let metadata = fs::metadata(&path)
                    .map_err(|e| format!("Failed to get metadata: {}", e))?;
                
                let duplicate_file = DuplicateFile {
                    name: file_name.to_string(),
                    path: path.to_string_lossy().to_string(),
                    size: metadata.len(),
                    duplicate_type: "unknown".to_string(),
                };

                // Check for exact duplicates
                exact_matches.entry(file_name.to_string())
                    .or_insert_with(Vec::new)
                    .push(duplicate_file.clone());

                // Check for numbered duplicates like "file (1).txt", "file (2).txt"
                let base_name = extract_base_name(file_name);
                file_groups.entry(base_name)
                    .or_insert_with(Vec::new)
                    .push(duplicate_file);
            }
        }
    }

    let mut duplicate_groups = Vec::new();

    // Process exact matches
    for (name, files) in exact_matches {
        if files.len() > 1 {
            let total_size = files.iter().map(|f| f.size).sum();
            let mut group_files = files;
            for file in &mut group_files {
                file.duplicate_type = "exact".to_string();
            }
            
            duplicate_groups.push(DuplicateGroup {
                original_name: name,
                files: group_files,
                total_size,
            });
        }
    }

    // Process numbered duplicates
    for (base_name, files) in file_groups {
        if files.len() > 1 {
            // Check if this group has numbered duplicates
            let has_numbered = files.iter().any(|f| is_numbered_duplicate(&f.name));
            
            if has_numbered {
                let total_size = files.iter().map(|f| f.size).sum();
                let mut group_files = files;
                for file in &mut group_files {
                    if is_numbered_duplicate(&file.name) {
                        file.duplicate_type = "numbered".to_string();
                    } else {
                        file.duplicate_type = "original".to_string();
                    }
                }
                
                // Sort by name to show original first, then numbered versions
                group_files.sort_by(|a, b| {
                    if a.duplicate_type == "original" { 
                        std::cmp::Ordering::Less 
                    } else if b.duplicate_type == "original" { 
                        std::cmp::Ordering::Greater 
                    } else { 
                        a.name.cmp(&b.name) 
                    }
                });

                duplicate_groups.push(DuplicateGroup {
                    original_name: base_name,
                    files: group_files,
                    total_size,
                });
            }
        }
    }

    Ok(duplicate_groups)
}

fn extract_base_name(filename: &str) -> String {
    // Simple regex-free approach to remove numbered suffixes like " (1)", " (2)", etc.
    if let Some(pos) = filename.rfind(" (") {
        let suffix = &filename[pos + 2..];
        if let Some(close_pos) = suffix.find(')') {
            let number_part = &suffix[..close_pos];
            if number_part.chars().all(|c| c.is_ascii_digit()) {
                return filename[..pos].to_string();
            }
        }
    }
    filename.to_string()
}

fn is_numbered_duplicate(filename: &str) -> bool {
    // Check if filename contains " (number)" pattern
    if let Some(pos) = filename.rfind(" (") {
        let suffix = &filename[pos + 2..];
        if let Some(close_pos) = suffix.find(')') {
            let number_part = &suffix[..close_pos];
            return number_part.chars().all(|c| c.is_ascii_digit()) && !number_part.is_empty();
        }
    }
    false
}

#[tauri::command]
fn delete_duplicate_file(file_path: String) -> Result<(), String> {
    let path = Path::new(&file_path);
    if path.exists() {
        fs::remove_file(path)
            .map_err(|e| format!("Failed to delete file: {}", e))?;
        Ok(())
    } else {
        Err("File does not exist".to_string())
    }
}

#[tauri::command]
fn show_in_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        // Try different file managers
        if Command::new("nautilus").arg(&path).spawn().is_ok() {
            // Nautilus (GNOME)
        } else if Command::new("dolphin").arg("--select").arg(&path).spawn().is_ok() {
            // Dolphin (KDE)
        } else if Command::new("thunar").arg(&path).spawn().is_ok() {
            // Thunar (XFCE)
        } else {
            // Fallback: open parent directory
            let parent = std::path::Path::new(&path).parent().unwrap_or(std::path::Path::new("/"));
            Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_downloads_files,
            group_files_by_extension,
            group_files_by_modified_date,
            show_in_folder,
            find_duplicate_files,
            delete_duplicate_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}