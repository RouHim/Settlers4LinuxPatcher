use anyhow::Result;
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_DEPTH: usize = 5;
const REQUIRED_FILES: [&str; 3] = ["S4.exe", "Config/GameSettings.cfg", "Exe/GfxEngine.dll"];

/// Check if a directory is a valid Settlers 4 installation
pub fn is_valid_game_directory(path: &Path) -> bool {
    REQUIRED_FILES.iter().all(|file| path.join(file).exists())
}

/// Expand user input into a usable path (supports '~' for the home directory)
pub fn resolve_game_path(input: &str) -> PathBuf {
    let trimmed = input.trim();

    if trimmed == "~" {
        if let Some(home) = user_home_dir() {
            return home;
        }
    }

    if let Some(stripped) = trimmed.strip_prefix("~/") {
        if let Some(home) = user_home_dir() {
            return home.join(stripped);
        }
    }

    PathBuf::from(trimmed)
}

/// Auto-detect Settlers 4 installation by scanning ~/Games directory
pub fn detect_game_path() -> Option<PathBuf> {
    // Get user home directory
    let games_dir = user_home_dir()?.join("Games");

    if !games_dir.exists() || !games_dir.is_dir() {
        return None;
    }

    // Search for S4.exe recursively (max depth 5)
    let mut queue = VecDeque::new();
    queue.push_back((games_dir, 0));

    while let Some((dir, depth)) = queue.pop_front() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();

            if path.file_name().is_some_and(|name| name == "S4.exe") && path.is_file() {
                if let Some(game_dir) = path.parent() {
                    if is_valid_game_directory(game_dir) {
                        return Some(game_dir.to_path_buf());
                    }
                }
            }

            if depth < MAX_DEPTH {
                let file_type = match entry.file_type() {
                    Ok(file_type) => file_type,
                    Err(_) => continue,
                };

                if file_type.is_dir() {
                    queue.push_back((path, depth + 1));
                } else if file_type.is_symlink() {
                    if let Ok(metadata) = fs::metadata(&path) {
                        if metadata.is_dir() {
                            queue.push_back((path, depth + 1));
                        }
                    }
                }
            }
        }
    }

    None
}

/// Get path to GfxEngine.dll for a given game directory
pub fn get_dll_path(game_path: &Path) -> PathBuf {
    game_path.join("Exe/GfxEngine.dll")
}

/// Get path to GameSettings.cfg for a given game directory
pub fn get_config_path(game_path: &Path) -> PathBuf {
    game_path.join("Config/GameSettings.cfg")
}

/// Get path to S4.exe for a given game directory
pub fn get_exe_path(game_path: &Path) -> PathBuf {
    game_path.join("S4.exe")
}

/// Validate that all required files exist in the game directory
pub fn validate_game_directory(game_path: &Path) -> Result<()> {
    if !game_path.exists() {
        anyhow::bail!("Game directory does not exist: {}", game_path.display());
    }

    if !game_path.is_dir() {
        anyhow::bail!("Path is not a directory: {}", game_path.display());
    }

    for file in &REQUIRED_FILES {
        let file_path = game_path.join(file);
        if !file_path.exists() {
            anyhow::bail!("Required file not found: {}", file_path.display());
        }
    }

    Ok(())
}

fn user_home_dir() -> Option<PathBuf> {
    if let Some(home) = env::var_os("HOME") {
        return Some(PathBuf::from(home));
    }

    #[cfg(windows)]
    {
        if let Some(user_profile) = env::var_os("USERPROFILE") {
            return Some(PathBuf::from(user_profile));
        }

        let home_drive = env::var_os("HOMEDRIVE");
        let home_path = env::var_os("HOMEPATH");
        match (home_drive, home_path) {
            (Some(drive), Some(path)) => {
                let mut home = PathBuf::from(drive);
                home.push(path);
                return Some(home);
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_fake_game_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let game_path = temp_dir.path();

        // Create directory structure
        fs::create_dir(game_path.join("Config")).unwrap();
        fs::create_dir(game_path.join("Exe")).unwrap();

        // Create required files
        fs::write(game_path.join("S4.exe"), b"fake exe").unwrap();
        fs::write(game_path.join("Config/GameSettings.cfg"), b"fake config").unwrap();
        fs::write(game_path.join("Exe/GfxEngine.dll"), b"fake dll").unwrap();

        temp_dir
    }

    #[test]
    fn test_is_valid_game_directory() {
        let temp_dir = setup_fake_game_dir();
        assert!(is_valid_game_directory(temp_dir.path()));
    }

    #[test]
    fn test_is_valid_game_directory_missing_files() {
        let temp_dir = TempDir::new().unwrap();
        assert!(!is_valid_game_directory(temp_dir.path()));
    }

    #[test]
    fn test_get_paths() {
        let game_path = Path::new("/fake/game");

        assert_eq!(
            get_dll_path(game_path),
            PathBuf::from("/fake/game/Exe/GfxEngine.dll")
        );
        assert_eq!(
            get_config_path(game_path),
            PathBuf::from("/fake/game/Config/GameSettings.cfg")
        );
        assert_eq!(get_exe_path(game_path), PathBuf::from("/fake/game/S4.exe"));
    }

    #[test]
    fn test_validate_game_directory() {
        let temp_dir = setup_fake_game_dir();
        assert!(validate_game_directory(temp_dir.path()).is_ok());

        let invalid_dir = TempDir::new().unwrap();
        assert!(validate_game_directory(invalid_dir.path()).is_err());
    }

    #[test]
    fn test_resolve_game_path_expands_tilde() {
        let home = user_home_dir().expect("HOME must be set for tests");
        let resolved = resolve_game_path("~/Games/Settlers4");
        assert_eq!(resolved, home.join("Games/Settlers4"));
    }

    #[test]
    fn test_resolve_game_path_passthrough() {
        let resolved = resolve_game_path("/opt/settlers4");
        assert_eq!(resolved, PathBuf::from("/opt/settlers4"));
    }
}
