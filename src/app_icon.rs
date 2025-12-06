use iced::window;

const APP_ICON_BYTES: &[u8] = include_bytes!("../assets/app-icon-64.rgba");
const APP_ICON_WIDTH: u32 = 64;
const APP_ICON_HEIGHT: u32 = 64;

pub fn app_icon() -> Result<window::Icon, Box<dyn std::error::Error>> {
    let icon = window::icon::from_rgba(APP_ICON_BYTES.to_vec(), APP_ICON_WIDTH, APP_ICON_HEIGHT)?;

    Ok(icon)
}
