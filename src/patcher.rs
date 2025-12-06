use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::game_detection::{get_config_path, get_dll_path, get_exe_path, validate_game_directory};
use crate::ini_handler;
#[cfg(test)]
use crate::resolution::Resolution;
#[cfg(test)]
use crate::validation;

const GAME_PROCESS_NAMES: [&str; 2] = ["S4.exe", "S4_Main.exe"];

fn is_matching_process_name<S: AsRef<[u8]>>(name: S) -> bool {
    let raw = name.as_ref();
    // Use lossy conversion to tolerate Wine/proton cmdlines that might contain non-UTF-8 bytes
    let trimmed = String::from_utf8_lossy(raw)
        .trim_matches(['\0', '\n', '\r', ' '])
        .trim_end_matches('.')
        .to_string();
    let candidate = std::path::Path::new(&trimmed)
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or(&trimmed);

    GAME_PROCESS_NAMES
        .iter()
        .any(|target| candidate.eq_ignore_ascii_case(target))
}

/// Check if S4.exe or S4_Main.exe process is running
pub fn is_game_running() -> bool {
    is_game_running_in(Path::new("/proc"))
}

fn is_game_running_in(proc_root: &Path) -> bool {
    let Ok(proc_entries) = fs::read_dir(proc_root) else {
        return false;
    };

    for entry in proc_entries.filter_map(|entry| entry.ok()) {
        let file_name = entry.file_name();
        if !file_name
            .to_string_lossy()
            .chars()
            .all(|c| c.is_ascii_digit())
        {
            continue;
        }

        let comm_path = entry.path().join("comm");
        if let Ok(name) = fs::read(&comm_path) {
            if is_matching_process_name(name) {
                return true;
            }
        }

        let cmdline_path = entry.path().join("cmdline");
        if let Ok(cmdline) = fs::read(cmdline_path) {
            // cmdline is null-terminated entries; first entry is the executable path/name
            if let Some(first_arg) = cmdline.split(|b| *b == 0).next() {
                if is_matching_process_name(first_arg) {
                    return true;
                }
            }
        }

        let status_path = entry.path().join("status");
        if let Ok(status) = fs::read_to_string(status_path) {
            if let Some(name_line) = status.lines().find(|line| line.starts_with("Name:")) {
                if let Some(name) = name_line.split_whitespace().nth(1) {
                    if is_matching_process_name(name.as_bytes()) {
                        return true;
                    }
                }
            }
        }

        let exe_link = entry.path().join("exe");
        if let Ok(exe_path) = fs::read_link(exe_link) {
            if let Some(name) = exe_path.file_name().and_then(|os_str| os_str.to_str()) {
                if is_matching_process_name(name) {
                    return true;
                }
            }
        }
    }

    false
}

/// Apply resolution patch to the game (tests only)
#[cfg(test)]
pub fn patch_game(game_path: &Path, resolution: &Resolution) -> Result<()> {
    // Step 1: Validate game directory
    validate_game_directory(game_path).context("Invalid game directory")?;

    let dll_path = get_dll_path(game_path);
    let config_path = get_config_path(game_path);

    // Step 2: Validate GOG version (skipped in tests due to fake files)
    #[allow(clippy::unnecessary_operation)]
    if cfg!(not(test)) {
        let exe_path = get_exe_path(game_path);
        let validation_result = validation::validate_gog_version(&exe_path)?;
        if !validation_result.is_valid {
            anyhow::bail!(
                "GOG version validation failed:\n{}",
                validation_result.message
            );
        }
    }

    // Step 3: Check if game is running
    if cfg!(not(test)) && is_game_running() {
        anyhow::bail!(
            "The Settlers 4 (S4.exe/S4_Main.exe) is currently running.\n\
             Please close the game before patching."
        );
    }

    // Step 4: Update GameSettings.cfg
    ini_handler::update_resolution(&config_path, resolution)
        .context("Failed to update GameSettings.cfg")?;

    // Step 5: Replace GfxEngine.dll
    fs::write(&dll_path, resolution.dll_data)
        .with_context(|| format!("Failed to write GfxEngine.dll to {}", dll_path.display()))?;

    Ok(())
}

/// Restore game to default resolution (1024×768)
pub fn restore_to_default(game_path: &Path) -> Result<()> {
    use crate::resolution::RES_DEFAULT;

    // Validate game directory
    validate_game_directory(game_path).context("Invalid game directory")?;

    // Check if game is running
    if cfg!(not(test)) && is_game_running() {
        anyhow::bail!(
            "The Settlers 4 (S4.exe/S4_Main.exe) is currently running.\n\
             Please close the game before restoring."
        );
    }

    let dll_path = get_dll_path(game_path);
    let config_path = get_config_path(game_path);

    // Update to default resolution
    ini_handler::update_resolution(&config_path, &RES_DEFAULT)
        .context("Failed to update GameSettings.cfg")?;

    // Replace with default DLL
    fs::write(&dll_path, RES_DEFAULT.dll_data).with_context(|| {
        format!(
            "Failed to write default GfxEngine.dll to {}",
            dll_path.display()
        )
    })?;

    Ok(())
}

/// Get current patched resolution by checking GameSettings.cfg
#[cfg(test)]
pub fn get_current_resolution(game_path: &Path) -> Result<(u32, u32)> {
    let config_path = get_config_path(game_path);
    ini_handler::read_resolution(&config_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolution::presets::RESOLUTIONS;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_game_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let game_path = temp_dir.path();

        // Create directory structure
        fs::create_dir(game_path.join("Config")).unwrap();
        fs::create_dir(game_path.join("Exe")).unwrap();

        // Create required files
        fs::write(game_path.join("S4.exe"), b"fake exe").unwrap();

        // Create a valid INI file
        fs::write(
            game_path.join("Config/GameSettings.cfg"),
            "[GAMESETTINGS]\n\
             {\n\
                 WindowWidth = 1024\n\
                 WindowHeight = 768\n\
                 Fullscreen = 0\n\
                 Screenmode = 0\n\
             }\n",
        )
        .unwrap();

        // Write default DLL (so SHA1 validation passes)
        use crate::resolution::RES_DEFAULT;
        fs::write(game_path.join("Exe/GfxEngine.dll"), RES_DEFAULT.dll_data).unwrap();

        temp_dir
    }

    #[test]
    fn test_patch_game() {
        let temp_dir = setup_test_game_dir();
        let resolution = &RESOLUTIONS[6]; // 1920×1080

        // Patch
        let result = patch_game(temp_dir.path(), resolution);
        assert!(result.is_ok(), "Patch failed: {:?}", result.err());

        // Verify DLL was replaced
        let dll_content = fs::read(temp_dir.path().join("Exe/GfxEngine.dll")).unwrap();
        assert_eq!(dll_content, resolution.dll_data);

        // Verify config was updated
        let (width, height) = get_current_resolution(temp_dir.path()).unwrap();
        assert_eq!(width, 1920);
        assert_eq!(height, 1080);
    }

    #[test]
    fn test_restore_to_default() {
        let temp_dir = setup_test_game_dir();
        let resolution = &RESOLUTIONS[6];

        // First patch to a widescreen resolution
        patch_game(temp_dir.path(), resolution).unwrap();

        // Then restore to default
        restore_to_default(temp_dir.path()).unwrap();

        // Verify restored to 1024×768
        let (width, height) = get_current_resolution(temp_dir.path()).unwrap();
        assert_eq!(width, 1024);
        assert_eq!(height, 768);
    }

    #[test]
    fn test_patch_invalid_directory() {
        let temp_dir = TempDir::new().unwrap();
        let resolution = &RESOLUTIONS[0];

        let result = patch_game(temp_dir.path(), resolution);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_game_running_in_detects_process() {
        let proc_root = TempDir::new().unwrap();
        let pid_dir = proc_root.path().join("12345");
        fs::create_dir(&pid_dir).unwrap();

        // Minimal proc-style files
        fs::write(pid_dir.join("comm"), b"S4_Main.exe\n").unwrap();
        fs::write(pid_dir.join("cmdline"), b"/fake/path/S4.exe\0--flag").unwrap();
        fs::write(pid_dir.join("status"), "Name:\tS4_Main.exe\n").unwrap();

        assert!(is_game_running_in(proc_root.path()));
    }
}
