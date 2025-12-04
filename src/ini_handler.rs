use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::resolution::Resolution;
const KEY_WIDTH: &str = "WindowWidth";
const KEY_HEIGHT: &str = "WindowHeight";
const KEY_FULLSCREEN: &str = "Fullscreen";
const KEY_SCREENMODE: &str = "Screenmode";

/// Update GameSettings.cfg with new resolution settings
/// This handles the non-standard format with curly braces used by Settlers 4
pub fn update_resolution<P: AsRef<Path>>(config_path: P, resolution: &Resolution) -> Result<()> {
    update_resolution_values(config_path, resolution.width, resolution.height)
}

/// Update GameSettings.cfg with custom width/height values (for dynamic patching)
/// This handles the non-standard format with curly braces used by Settlers 4
pub fn update_resolution_values<P: AsRef<Path>>(
    config_path: P,
    width: u32,
    height: u32,
) -> Result<()> {
    let path = config_path.as_ref();

    // Read the entire file
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read INI file: {}", path.display()))?;

    // Update the content using string manipulation
    let updated_content = update_ini_values(
        &content,
        &[
            (KEY_WIDTH, width.to_string()),
            (KEY_HEIGHT, height.to_string()),
            (KEY_FULLSCREEN, "1".to_string()),
            (KEY_SCREENMODE, "2".to_string()),
        ],
    )?;

    // Write back to file
    fs::write(path, updated_content)
        .with_context(|| format!("Failed to write INI file: {}", path.display()))?;

    Ok(())
}

/// Update values in the non-standard INI format used by Settlers 4
/// Format: [SECTION]\n{\n    Key = Value\n}
fn update_ini_values(content: &str, updates: &[(&str, String)]) -> Result<String> {
    let mut result = content.to_string();

    for (key, new_value) in updates {
        // Pattern: "    Key = OldValue" or "    Key = OldValue\n" or "    Key = OldValue\r\n"
        // We need to handle both the presence and absence of the key

        let pattern = format!("    {} = ", key);

        if let Some(start_pos) = result.find(&pattern) {
            // Key exists, find the end of the line
            let value_start = start_pos + pattern.len();
            let rest = &result[value_start..];

            // Find the end of the line (either \n, \r\n, or end of string)
            let line_end = rest.find('\n').unwrap_or(rest.len());
            let old_value_end = value_start + line_end;

            // Replace the old value with new value
            let before = &result[..value_start];
            let after = &result[old_value_end..];
            result = format!("{}{}{}", before, new_value, after);
        } else {
            // Key doesn't exist, add it before the closing brace
            let closing_brace = result
                .rfind('}')
                .context("Could not find closing brace in config file")?;

            let before = &result[..closing_brace];
            let after = &result[closing_brace..];
            result = format!("{}    {} = {}\n{}", before, key, new_value, after);
        }
    }

    Ok(result)
}

/// Read current resolution from GameSettings.cfg (test-only)
#[cfg(test)]
pub fn read_resolution<P: AsRef<Path>>(config_path: P) -> Result<(u32, u32)> {
    let path = config_path.as_ref();

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read INI file: {}", path.display()))?;

    let width =
        extract_value(&content, KEY_WIDTH).context("WindowWidth not found in config file")?;

    let height =
        extract_value(&content, KEY_HEIGHT).context("WindowHeight not found in config file")?;

    let width = width
        .trim()
        .parse::<u32>()
        .context("Failed to parse WindowWidth")?;

    let height = height
        .trim()
        .parse::<u32>()
        .context("Failed to parse WindowHeight")?;

    Ok((width, height))
}

/// Extract a value from the non-standard INI format (test-only)
#[cfg(test)]
fn extract_value(content: &str, key: &str) -> Option<String> {
    let pattern = format!("    {} = ", key);

    let start_pos = content.find(&pattern)?;
    let value_start = start_pos + pattern.len();
    let rest = &content[value_start..];

    let line_end = rest.find('\n').unwrap_or(rest.len());
    let value = &rest[..line_end];

    // Remove any trailing whitespace and quotes
    Some(value.trim().trim_matches('"').to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolution::presets::RESOLUTIONS;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_update_and_read_resolution() {
        // Create a temporary INI file with Settlers 4 format
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "//\n\
             // Automatically generated file. Do not edit!\n\
             // \n\n\
             [GAMESETTINGS]\n\
             {{\n\
                 WindowWidth = 1024\n\
                 WindowHeight = 768\n\
                 Fullscreen = 0\n\
                 Screenmode = 0\n\
             }}"
        )
        .unwrap();
        temp_file.flush().unwrap();

        let resolution = &RESOLUTIONS[6]; // 1920×1080

        // Update resolution
        update_resolution(temp_file.path(), resolution).unwrap();

        // Read back
        let (width, height) = read_resolution(temp_file.path()).unwrap();
        assert_eq!(width, 1920);
        assert_eq!(height, 1080);

        // Verify the file still has the correct format
        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("[GAMESETTINGS]"));
        assert!(content.contains("{"));
        assert!(content.contains("}"));
    }

    #[test]
    fn test_read_resolution_missing_file() {
        let result = read_resolution("/nonexistent/file.cfg");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_value() {
        let content = "    WindowWidth = 1024\n    WindowHeight = 768\n";

        assert_eq!(
            extract_value(content, "WindowWidth"),
            Some("1024".to_string())
        );
        assert_eq!(
            extract_value(content, "WindowHeight"),
            Some("768".to_string())
        );
        assert_eq!(extract_value(content, "NonExistent"), None);
    }

    #[test]
    fn test_update_ini_values() {
        let content = "[GAMESETTINGS]\n{\n    WindowWidth = 800\n    WindowHeight = 600\n}\n";

        let updates = [
            ("WindowWidth", "1920".to_string()),
            ("WindowHeight", "1080".to_string()),
        ];

        let result = update_ini_values(content, &updates).unwrap();

        assert!(result.contains("WindowWidth = 1920"));
        assert!(result.contains("WindowHeight = 1080"));
        assert!(!result.contains("800"));
        assert!(!result.contains("600"));
    }
}
