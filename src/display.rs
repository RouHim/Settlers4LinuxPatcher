use display_info::DisplayInfo;

/// Detect the primary display resolution
/// Works on both Wayland and X11 via display-info crate
pub fn detect_primary_resolution() -> Option<(u32, u32)> {
    match DisplayInfo::all() {
        Ok(displays) => {
            // Find primary display, or use first one, or fallback to 1920×1080
            displays
                .iter()
                .find(|d| d.is_primary)
                .or_else(|| displays.first())
                .map(|d| (d.width, d.height))
                .or(Some((1920, 1080)))
        }
        Err(_) => {
            // Fallback if display detection fails (rare on modern systems)
            Some((1920, 1080))
        }
    }
}
