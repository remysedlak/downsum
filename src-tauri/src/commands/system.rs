use std::process::Command;

#[tauri::command]
pub fn show_in_folder(path: String) -> Result<(), String> {
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