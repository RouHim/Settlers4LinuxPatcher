use iced::widget::{row, text, Text};
use iced::Color;
use iced_fonts::bootstrap;

/// Create a colored icon with text next to it
pub fn icon_text_colored<'a, Msg>(
    bootstrap_icon: Text<'static>,
    label: &'a str,
    color: Color,
) -> iced::widget::Row<'a, Msg> {
    row![
        bootstrap_icon.color(color),
        text(" ").color(color),
        text(label).color(color),
    ]
    .spacing(5)
}

/// Success/OK icon (green checkmark)
pub fn check() -> Text<'static> {
    bootstrap::check_circle_fill()
}

/// Error icon (red X)
pub fn error() -> Text<'static> {
    bootstrap::x_circle_fill()
}

/// Warning icon (yellow/orange triangle)
pub fn warning() -> Text<'static> {
    bootstrap::exclamation_triangle_fill()
}

/// Info icon (blue info circle)
pub fn info() -> Text<'static> {
    bootstrap::info_circle_fill()
}
