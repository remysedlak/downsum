#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;

use tauri::Builder;

fn main() {
    Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_downloads_files,
            commands::group_files_by_extension,
            commands::group_files_by_modified_date,
            commands::show_in_folder,
            commands::find_duplicate_files,
            commands::delete_duplicate_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
