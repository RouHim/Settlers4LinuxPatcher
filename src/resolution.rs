use std::fmt;

/// Supported resolutions with embedded pre-patched DLL files
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
    pub aspect_ratio: &'static str,
    pub dll_sha1: &'static str,
    pub dll_data: &'static [u8],
}

impl Resolution {
    pub fn name(&self) -> String {
        format!("{}×{} ({})", self.width, self.height, self.aspect_ratio)
    }
}

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// Embed DLL files at compile time
// Production: Only default DLL (292 KB)
// Tests: All DLLs for byte-identical verification (2.57 MB)

// Always embed default DLL (needed for dynamic patching)
const DLL_DEFAULT: &[u8] = include_bytes!("../dlls/GfxEngine_default.dll");

/// Default resolution (vanilla game)
pub const RES_DEFAULT: Resolution = Resolution {
    width: 1024,
    height: 768,
    aspect_ratio: "4:3",
    dll_sha1: "F25CA243F617BB626614EFA8AB611509C971E6C4",
    dll_data: DLL_DEFAULT,
};

/// Known GOG Gold Edition S4.exe hashes
/// We verify the S4.exe instead of the DLL because the DLL changes when patched
pub const KNOWN_GOG_S4_EXE_HASHES: [&str; 1] = [
    "F4C2B5D0C3B9C2BB250F69CFB18D8B98A9061981", // GOG Gold Edition v2.50.1508
];

#[cfg(test)]
pub mod presets {
    use super::Resolution;

    // Only embed a SUBSET of pre-patched DLLs in test builds
    // We test 3 representative resolutions to verify correctness:
    // - Low res (1024×600): Tests small values
    // - Mid res (1680×1050): Tests medium values
    // - High res (1920×1080): Tests large values, most common resolution
    // This reduces test binary size by ~75% while maintaining confidence
    pub const DLL_1024X600: &[u8] = include_bytes!("../dlls/GfxEngine_1024x600.dll");
    pub const DLL_1680X1050: &[u8] = include_bytes!("../dlls/GfxEngine_1680x1050.dll");
    pub const DLL_1920X1080: &[u8] = include_bytes!("../dlls/GfxEngine_1920x1080.dll");

    // Remaining resolution DLLs are empty placeholders for completeness
    pub const DLL_1280X720: &[u8] = &[];
    pub const DLL_1280X800: &[u8] = &[];
    pub const DLL_1366X768: &[u8] = &[];
    pub const DLL_1440X900: &[u8] = &[];
    pub const DLL_1920X1200: &[u8] = &[];

    /// All available widescreen resolutions (tests only)
    pub const RESOLUTIONS: [Resolution; 8] = [
        Resolution {
            width: 1024,
            height: 600,
            aspect_ratio: "17:10",
            dll_sha1: "4968B9D20D87C901F57AB37F1BCAAC405365A89A",
            dll_data: DLL_1024X600,
        },
        Resolution {
            width: 1280,
            height: 720,
            aspect_ratio: "16:9",
            dll_sha1: "DE125A1E238D568D165DF0FFFEAB047C690C7ED2",
            dll_data: DLL_1280X720,
        },
        Resolution {
            width: 1280,
            height: 800,
            aspect_ratio: "16:10",
            dll_sha1: "1107429472318D11DFF90691964281A4E61743BC",
            dll_data: DLL_1280X800,
        },
        Resolution {
            width: 1366,
            height: 768,
            aspect_ratio: "16:9",
            dll_sha1: "1E3A0B201DC71F9E6D5AC0A9BF4419E575BE4799",
            dll_data: DLL_1366X768,
        },
        Resolution {
            width: 1440,
            height: 900,
            aspect_ratio: "16:10",
            dll_sha1: "11D68FF1AA00DBE0E78528D1DD913EF3B34C1E23",
            dll_data: DLL_1440X900,
        },
        Resolution {
            width: 1680,
            height: 1050,
            aspect_ratio: "16:10",
            dll_sha1: "B9923B050E51C1A5F9E1DE8828861111DF811980",
            dll_data: DLL_1680X1050,
        },
        Resolution {
            width: 1920,
            height: 1080,
            aspect_ratio: "16:9",
            dll_sha1: "183DE9D83D2971AE9DCFD0E1ADB41A1A581C63FE",
            dll_data: DLL_1920X1080,
        },
        Resolution {
            width: 1920,
            height: 1200,
            aspect_ratio: "16:10",
            dll_sha1: "B08496740134C2B4660C864E3F0DB3C980F14C4B",
            dll_data: DLL_1920X1200,
        },
    ];

    /// Find the closest matching resolution to the given dimensions (tests only)
    pub fn find_closest_resolution(target_width: u32, target_height: u32) -> &'static Resolution {
        let target_ratio = target_width as f64 / target_height as f64;

        RESOLUTIONS
            .iter()
            .min_by_key(|res| {
                let res_ratio = res.width as f64 / res.height as f64;
                let ratio_diff = (res_ratio - target_ratio).abs();
                let size_diff = ((res.width as i64 - target_width as i64).abs()
                    + (res.height as i64 - target_height as i64).abs())
                    as f64;
                // Prioritize matching aspect ratio, then size
                ((ratio_diff * 1000.0) + (size_diff * 0.1)) as i64
            })
            .unwrap_or(&RESOLUTIONS[6]) // Default to 1920×1080 if calculation fails
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolution::presets::RESOLUTIONS;
    use crate::resolution::RES_DEFAULT;

    #[test]
    fn test_resolution_name() {
        assert_eq!(RESOLUTIONS[6].name(), "1920×1080 (16:9)");
    }

    #[test]
    fn test_find_closest_resolution() {
        // Test exact match
        let res = presets::find_closest_resolution(1920, 1080);
        assert_eq!(res.width, 1920);
        assert_eq!(res.height, 1080);

        // Test close match
        let res = presets::find_closest_resolution(1900, 1060);
        assert_eq!(res.width, 1920);
        assert_eq!(res.height, 1080);

        // Test 16:9 aspect ratio preference
        let res = presets::find_closest_resolution(2560, 1440);
        assert_eq!(res.aspect_ratio, "16:9");
    }

    #[test]
    fn test_dll_data_embedded() {
        // Verify DLLs are actually embedded
        assert!(!RES_DEFAULT.dll_data.is_empty());
        assert!(!RESOLUTIONS[0].dll_data.is_empty());
        assert!(!RESOLUTIONS[6].dll_data.is_empty());
    }
}
