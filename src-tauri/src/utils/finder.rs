use log::info;
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
use winreg::{enums::*, RegKey};

const AMONG_US_EXE: &str = "Among Us.exe";

/// Checks if the directory exists and contains the Among Us executable.
fn verify_among_us_directory(path: &Path) -> bool {
    path.is_dir() && path.join(AMONG_US_EXE).is_file()
}

#[cfg(target_os = "windows")]
fn parse_registry_icon_value(raw_value: &str) -> Option<PathBuf> {
    let path = raw_value
        .split(',')
        .next()?
        .trim()
        .trim_matches(|c| c == '"' || c == '\'');

    if path.is_empty() {
        return None;
    }

    PathBuf::from(path).parent().map(|p| p.to_path_buf())
}

#[cfg(target_os = "windows")]
fn find_among_us_from_registry() -> Option<PathBuf> {
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

    for key_name in ["AmongUs", "amongus"] {
        if let Ok(key) = hkcr.open_subkey(key_name) {
            if let Ok(icon_key) = key.open_subkey("DefaultIcon") {
                if let Ok(raw_value) = icon_key.get_value::<String, _>("") {
                    if let Some(directory) = parse_registry_icon_value(&raw_value) {
                        if verify_among_us_directory(&directory) {
                            info!("Found Among Us via registry: {}", directory.display());
                            return Some(directory);
                        }
                    }
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn find_among_us_linux_paths() -> Vec<PathBuf> {
    let mut detected_paths = Vec::new();
    if let Some(home) = home::home_dir() {
        // Common Steam installation paths on Linux
        let steam_apps = [
            ".local/share/Steam/steamapps/common/Among Us",
            ".steam/steam/steamapps/common/Among Us",
            ".var/app/com.valvesoftware.Steam/data/Steam/steamapps/common/Among Us", // Flatpak
        ];

        for sub_path in steam_apps {
            let full_path = home.join(sub_path);
            if verify_among_us_directory(&full_path) {
                info!("Found Among Us at: {}", full_path.display());
                detected_paths.push(full_path);
            }
        }
    }
    detected_paths
}

pub fn get_among_us_paths() -> Vec<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Some(path) = find_among_us_from_registry() {
            return vec![path];
        }
    }

    #[cfg(target_os = "linux")]
    {
        let paths = find_among_us_linux_paths();
        if !paths.is_empty() {
            return paths;
        }
    }

    info!("Among Us installation not detected");
    Vec::new()
}
