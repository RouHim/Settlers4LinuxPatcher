use anyhow::{Context, Result};
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::resolution::KNOWN_GOG_S4_EXE_HASHES;

/// Calculate SHA1 hash of a file
pub fn calculate_sha1<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(path.as_ref())
        .with_context(|| format!("Failed to open file: {}", path.as_ref().display()))?;

    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .context("Failed to read file for hashing")?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(hex::encode_upper(result))
}

/// Get detailed validation result with error message
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    #[allow(dead_code)]
    pub hash: String,
    #[allow(dead_code)]
    pub message: String,
}

pub fn validate_gog_version<P: AsRef<Path>>(exe_path: P) -> Result<ValidationResult> {
    let hash = calculate_sha1(&exe_path).context("Failed to calculate SHA1 hash of S4.exe")?;

    let is_valid = KNOWN_GOG_S4_EXE_HASHES.contains(&hash.as_str());

    let message = if is_valid {
        "Valid GOG Gold Edition v2.50.1508 detected".to_string()
    } else {
        format!(
            "Invalid S4.exe hash: {}\n\
             This is not a recognized GOG Gold Edition v2.50.1508 version.\n\
             Patching is not allowed for safety reasons.",
            hash
        )
    };

    Ok(ValidationResult {
        is_valid,
        hash,
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_calculate_sha1() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();

        let hash = calculate_sha1(temp_file.path()).unwrap();
        // SHA1 of "test content"
        assert_eq!(hash, "1EEBDF4FDC9FC7BF283031B93F9AEF3338DE9052");
    }

    #[test]
    fn test_known_exe_hashes_count() {
        // Should have 1 hash: GOG Gold Edition S4.exe
        assert_eq!(KNOWN_GOG_S4_EXE_HASHES.len(), 1);
    }

    #[test]
    fn test_validation_result() {
        // Test with a file that won't match
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"fake dll content").unwrap();

        let result = validate_gog_version(temp_file.path()).unwrap();
        assert!(!result.is_valid);
        assert!(result.message.contains("Invalid"));
    }
}
