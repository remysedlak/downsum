#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, fs, path::PathBuf};
use tauri::command;
use tauri_plugin_shell::ShellExt;

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

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_downloads_files,
            group_files_by_extension,
            group_files_by_modified_date
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
