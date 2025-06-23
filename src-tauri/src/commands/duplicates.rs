use std::collections::HashMap;
use std::fs;
use std::path::{PathBuf};

use crate::models::duplicates::{DuplicateFile, DuplicateGroup};

#[tauri::command]
pub fn find_duplicate_files(downloads_path: Option<String>) -> Result<Vec<DuplicateGroup>, String> {
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

