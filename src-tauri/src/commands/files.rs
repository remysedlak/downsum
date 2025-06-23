use std::{fs, path::{Path, PathBuf}, collections::HashMap};
use tauri::command;

use crate::models::file_info::{FileInfo, FileGroup};

pub fn get_downloads_path() -> PathBuf {
    dirs::download_dir().unwrap_or_default()
}

pub fn read_files_from_dir(path: &PathBuf) -> Vec<FileInfo> {
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

#[command]
pub fn get_downloads_files() -> Vec<FileInfo> {
    read_files_from_dir(&get_downloads_path())
}

#[command]
pub fn group_files_by_extension() -> Vec<FileGroup> {
    let files = read_files_from_dir(&get_downloads_path());
    let mut groups: HashMap<String, Vec<FileInfo>> = HashMap::new();

    for file in files {
        let ext = Path::new(&file.path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        groups.entry(ext).or_default().push(file);
    }

    groups.into_iter().map(|(key, files)| FileGroup { key, files }).collect()
}

#[command]
pub fn group_files_by_modified_date() -> Vec<FileGroup> {
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

    groups.into_iter().map(|(key, files)| FileGroup { key, files }).collect()
}

#[command]
pub fn delete_duplicate_file(file_path: String) -> Result<(), String> {
    let path = Path::new(&file_path);
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("Failed to delete file: {}", e))?;
        Ok(())
    } else {
        Err("File does not exist".to_string())
    }
}
