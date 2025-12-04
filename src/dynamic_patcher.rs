/// Dynamic resolution patcher for GfxEngine.dll
///
/// This module allows patching the DLL to ANY resolution on-the-fly
/// without needing pre-patched DLL files.
use anyhow::Result;

// Offsets discovered through binary analysis
const WIDTH_OFFSET: usize = 67214; // 0x1068E - mov ebp, <width>
const HEIGHT_OFFSET: usize = 67219; // 0x10693 - mov ebx, <height>

/// Patch a base GfxEngine.dll with custom resolution
///
/// Takes the default/vanilla DLL and patches it with any width/height
///
/// # Arguments
/// * `base_dll` - The original/default GfxEngine.dll bytes
/// * `width` - Target width (e.g., 2560 for 2560×1440)
/// * `height` - Target height (e.g., 1440 for 2560×1440)
///
/// # Returns
/// A new Vec<u8> containing the patched DLL ready to be written
///
/// # Example
/// ```
/// let base_dll = include_bytes!("../dlls/GfxEngine_default.dll");
/// let patched = patch_dll_dynamic(base_dll, 2560, 1440)?;
/// std::fs::write("Exe/GfxEngine.dll", patched)?;
/// ```
pub fn patch_dll_dynamic(base_dll: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    // Validate input
    if !(800..=7680).contains(&width) {
        anyhow::bail!("Width must be between 800 and 7680 (got {})", width);
    }
    if !(600..=4320).contains(&height) {
        anyhow::bail!("Height must be between 600 and 4320 (got {})", height);
    }

    // Clone the base DLL
    let mut patched_dll = base_dll.to_vec();

    // Verify DLL is large enough
    if patched_dll.len() < HEIGHT_OFFSET + 4 {
        anyhow::bail!(
            "DLL file is too small (expected at least {} bytes)",
            HEIGHT_OFFSET + 4
        );
    }

    // Patch width (32-bit little-endian)
    let width_bytes = width.to_le_bytes();
    patched_dll[WIDTH_OFFSET..WIDTH_OFFSET + 4].copy_from_slice(&width_bytes);

    // Patch height (32-bit little-endian)
    let height_bytes = height.to_le_bytes();
    patched_dll[HEIGHT_OFFSET..HEIGHT_OFFSET + 4].copy_from_slice(&height_bytes);

    Ok(patched_dll)
}

/// Extract current resolution from a GfxEngine.dll
///
/// Reads the patched width/height values from the DLL
#[cfg(test)]
pub fn read_dll_resolution(dll_data: &[u8]) -> Result<(u32, u32)> {
    if dll_data.len() < HEIGHT_OFFSET + 4 {
        anyhow::bail!("DLL file is too small");
    }

    // Read width (32-bit little-endian)
    let width = u32::from_le_bytes([
        dll_data[WIDTH_OFFSET],
        dll_data[WIDTH_OFFSET + 1],
        dll_data[WIDTH_OFFSET + 2],
        dll_data[WIDTH_OFFSET + 3],
    ]);

    // Read height (32-bit little-endian)
    let height = u32::from_le_bytes([
        dll_data[HEIGHT_OFFSET],
        dll_data[HEIGHT_OFFSET + 1],
        dll_data[HEIGHT_OFFSET + 2],
        dll_data[HEIGHT_OFFSET + 3],
    ]);

    Ok((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolution::presets::RESOLUTIONS;
    use crate::resolution::RES_DEFAULT;

    #[test]
    fn test_patch_dll_dynamic() {
        // Start with default DLL (1024×768)
        let base_dll = RES_DEFAULT.dll_data;

        // Patch to 2560×1440 (ultrawide QHD)
        let patched = patch_dll_dynamic(base_dll, 2560, 1440).unwrap();

        // Verify the patched values
        let (width, height) = read_dll_resolution(&patched).unwrap();
        assert_eq!(width, 2560);
        assert_eq!(height, 1440);
    }

    #[test]
    fn test_patch_existing_resolutions() {
        let base_dll = RES_DEFAULT.dll_data;

        // Test all existing resolutions
        let resolutions = [
            (1024, 600),
            (1280, 720),
            (1280, 800),
            (1366, 768),
            (1440, 900),
            (1680, 1050),
            (1920, 1080),
            (1920, 1200),
        ];

        for (width, height) in resolutions {
            let patched = patch_dll_dynamic(base_dll, width, height).unwrap();
            let (read_w, read_h) = read_dll_resolution(&patched).unwrap();
            assert_eq!(read_w, width, "Width mismatch for {}×{}", width, height);
            assert_eq!(read_h, height, "Height mismatch for {}×{}", width, height);
        }
    }

    #[test]
    fn test_patch_8k_resolution() {
        let base_dll = RES_DEFAULT.dll_data;

        // Test 8K resolution (7680×4320)
        let patched = patch_dll_dynamic(base_dll, 7680, 4320).unwrap();
        let (width, height) = read_dll_resolution(&patched).unwrap();
        assert_eq!(width, 7680);
        assert_eq!(height, 4320);
    }

    #[test]
    fn test_invalid_resolutions() {
        let base_dll = RES_DEFAULT.dll_data;

        // Too small
        assert!(patch_dll_dynamic(base_dll, 640, 480).is_err());

        // Too large
        assert!(patch_dll_dynamic(base_dll, 10000, 10000).is_err());
    }

    #[test]
    fn test_read_existing_dll_resolutions() {
        // Verify we can read existing pre-patched DLLs (only those with embedded data)
        let mut tested = 0;
        for res in &RESOLUTIONS {
            // Skip resolutions without embedded DLL data
            if res.dll_data.is_empty() {
                continue;
            }

            let (width, height) = read_dll_resolution(res.dll_data).unwrap();
            assert_eq!(width, res.width, "Width mismatch for {}", res.name());
            assert_eq!(height, res.height, "Height mismatch for {}", res.name());
            tested += 1;
        }

        assert!(tested >= 3, "Should test at least 3 resolutions");
    }

    #[test]
    fn test_dynamic_patch_produces_identical_dlls() {
        // CRITICAL TEST: Verify dynamic patching produces byte-identical DLLs
        // We test 3 representative resolutions (low/mid/high) to verify correctness
        println!("\n=== Byte-Identical DLL Verification ===");

        let mut tested_count = 0;
        let mut skipped_count = 0;

        for res in &RESOLUTIONS {
            // Skip resolutions that don't have embedded DLL data (empty slices)
            if res.dll_data.is_empty() {
                println!("\nSkipping {} (no test data embedded)", res.name());
                skipped_count += 1;
                continue;
            }

            println!("\nTesting {}...", res.name());
            tested_count += 1;

            // Dynamically patch from default DLL
            let dynamically_patched =
                patch_dll_dynamic(RES_DEFAULT.dll_data, res.width, res.height).unwrap();

            // Compare with pre-patched DLL
            let pre_patched = res.dll_data;

            // They should be EXACTLY identical
            assert_eq!(
                dynamically_patched.len(),
                pre_patched.len(),
                "DLL size mismatch for {}: dynamic={}, pre-patched={}",
                res.name(),
                dynamically_patched.len(),
                pre_patched.len()
            );

            // Byte-by-byte comparison
            let mut differences = Vec::new();
            for (offset, (dynamic_byte, prepatched_byte)) in dynamically_patched
                .iter()
                .zip(pre_patched.iter())
                .enumerate()
            {
                if dynamic_byte != prepatched_byte {
                    differences.push((offset, *dynamic_byte, *prepatched_byte));
                }
            }

            if !differences.is_empty() {
                eprintln!(
                    "\n❌ FAILED: {} has {} byte differences:",
                    res.name(),
                    differences.len()
                );
                for (offset, dynamic, prepatched) in differences.iter().take(10) {
                    eprintln!(
                        "  Offset 0x{:08X} ({:7}): dynamic=0x{:02X}, pre-patched=0x{:02X}",
                        offset, offset, dynamic, prepatched
                    );
                }
                if differences.len() > 10 {
                    eprintln!("  ... and {} more differences", differences.len() - 10);
                }
                panic!(
                    "Dynamic patching did NOT produce identical DLL for {}",
                    res.name()
                );
            }

            println!(
                "  ✅ IDENTICAL: {} bytes match perfectly",
                dynamically_patched.len()
            );
        }

        println!(
            "\n=== Tested: {} resolutions (skipped: {}) ===",
            tested_count, skipped_count
        );
        println!("=== All tested resolutions produce byte-identical DLLs! ===");

        // Ensure we tested at least some resolutions
        assert!(
            tested_count >= 3,
            "Should test at least 3 resolutions, but only tested {}",
            tested_count
        );
    }

    #[test]
    fn test_dynamic_patch_only_changes_expected_bytes() {
        // Verify that dynamic patching ONLY changes bytes in the expected region
        use crate::resolution::RES_DEFAULT;

        let base_dll = RES_DEFAULT.dll_data;
        let patched = patch_dll_dynamic(base_dll, 2560, 1440).unwrap();

        // Count differences
        let mut differences = Vec::new();
        for (offset, (base_byte, patched_byte)) in base_dll.iter().zip(patched.iter()).enumerate() {
            if base_byte != patched_byte {
                differences.push(offset);
            }
        }

        // Note: Not all 8 bytes necessarily change - only the bytes that differ
        // For example, 1024 (0x00 0x04 0x00 0x00) → 2560 (0x00 0x0A 0x00 0x00)
        // only changes byte at offset 67215 (0x04 → 0x0A)

        println!("Bytes that changed: {:?}", differences);

        // Verify all changed bytes are within the expected ranges
        for offset in &differences {
            let in_width_range = *offset >= 67214 && *offset <= 67217;
            let in_height_range = *offset >= 67219 && *offset <= 67222;

            assert!(
                in_width_range || in_height_range,
                "Byte changed at unexpected offset 0x{:X} ({})",
                offset,
                offset
            );
        }

        // Verify we changed at least some bytes (otherwise patching didn't work)
        assert!(
            !differences.is_empty(),
            "No bytes changed - patching failed!"
        );

        // Verify we didn't change more than 8 bytes (4 width + 4 height)
        assert!(
            differences.len() <= 8,
            "Too many bytes changed: {} (max 8)",
            differences.len()
        );

        println!(
            "✅ Dynamic patching changed {} bytes, all within expected offsets",
            differences.len()
        );
    }
}
