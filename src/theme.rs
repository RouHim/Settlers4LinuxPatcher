use iced::theme::Palette;
use iced::widget::{button, container};
use iced::{border, Background, Color, Shadow, Theme, Vector};

pub const STONE: Color = Color {
    r: 58.0 / 255.0,
    g: 54.0 / 255.0,
    b: 40.0 / 255.0,
    a: 1.0,
};
pub const STONE_LIGHT: Color = Color {
    r: 72.0 / 255.0,
    g: 68.0 / 255.0,
    b: 50.0 / 255.0,
    a: 1.0,
};
pub const GOLD: Color = Color {
    r: 199.0 / 255.0,
    g: 160.0 / 255.0,
    b: 74.0 / 255.0,
    a: 1.0,
};
pub const GOLD_DARK: Color = Color {
    r: 138.0 / 255.0,
    g: 102.0 / 255.0,
    b: 37.0 / 255.0,
    a: 1.0,
};
pub const TEXT_CREAM: Color = Color {
    r: 242.0 / 255.0,
    g: 232.0 / 255.0,
    b: 200.0 / 255.0,
    a: 1.0,
};
pub const MOSS: Color = Color {
    r: 109.0 / 255.0,
    g: 122.0 / 255.0,
    b: 59.0 / 255.0,
    a: 1.0,
};
pub const AMBER: Color = Color {
    r: 193.0 / 255.0,
    g: 147.0 / 255.0,
    b: 61.0 / 255.0,
    a: 1.0,
};
pub const RUST: Color = Color {
    r: 164.0 / 255.0,
    g: 72.0 / 255.0,
    b: 50.0 / 255.0,
    a: 1.0,
};

pub fn settlers_palette() -> Palette {
    Palette {
        background: STONE,
        text: TEXT_CREAM,
        primary: GOLD,
        success: MOSS,
        danger: RUST,
    }
}

pub fn settlers_theme() -> Theme {
    Theme::custom(String::from("Settlers 4 Stone & Gold"), settlers_palette())
}

pub fn panel_container(_theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(TEXT_CREAM),
        background: Some(Background::Color(STONE_LIGHT)),
        border: border::rounded(10).width(2.0).color(GOLD_DARK),
        shadow: Shadow {
            color: Color {
                a: 0.45,
                ..Color::BLACK
            },
            offset: Vector::new(0.0, 3.0),
            blur_radius: 6.0,
        },
    }
}

pub fn gold_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let base = button::Style {
        background: Some(Background::Color(GOLD)),
        text_color: STONE,
        border: border::rounded(6).width(1.5).color(GOLD_DARK),
        shadow: Shadow {
            color: Color {
                a: 0.35,
                ..Color::BLACK
            },
            offset: Vector::new(0.0, 2.0),
            blur_radius: 5.0,
        },
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(palette.primary.strong.color)),
            ..base
        },
        button::Status::Pressed => button::Style {
            shadow: Shadow {
                color: Color {
                    a: 0.20,
                    ..Color::BLACK
                },
                offset: Vector::new(0.0, 1.0),
                blur_radius: 3.0,
            },
            ..base
        },
        button::Status::Disabled => button::Style {
            background: base.background.map(|bg| match bg {
                Background::Color(color) => Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
                other => other,
            }),
            text_color: Color {
                a: base.text_color.a * 0.5,
                ..base.text_color
            },
            shadow: Shadow::default(),
            ..base
        },
    }
}
