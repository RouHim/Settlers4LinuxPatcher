use iced::widget::{row, text, Text};
use iced::{Color, Font};
use iced_fonts::Bootstrap;

/// Bootstrap icon font
pub const ICON_FONT: Font = Font::with_name("bootstrap-icons");

/// Create an icon with the Bootstrap font
pub fn icon(bootstrap_icon: Bootstrap) -> Text<'static> {
    text(char::from(bootstrap_icon)).font(ICON_FONT).size(16)
}

/// Create a colored icon
pub fn icon_colored(bootstrap_icon: Bootstrap, color: Color) -> Text<'static> {
    icon(bootstrap_icon).color(color)
}

/// Create a colored icon with text next to it
pub fn icon_text_colored<'a, Msg>(
    bootstrap_icon: Bootstrap,
    label: &'a str,
    color: Color,
) -> iced::widget::Row<'a, Msg> {
    row![
        icon(bootstrap_icon).color(color),
        text(" ").color(color),
        text(label).color(color),
    ]
    .spacing(5)
}

/// Success/OK icon (green checkmark)
pub const CHECK: Bootstrap = Bootstrap::CheckCircleFill;

/// Error icon (red X)
pub const ERROR: Bootstrap = Bootstrap::XCircleFill;

/// Warning icon (yellow/orange triangle)
pub const WARNING: Bootstrap = Bootstrap::ExclamationTriangleFill;

/// Info icon (blue info circle)
pub const INFO: Bootstrap = Bootstrap::InfoCircleFill;
