use iced::widget::Id;
use iced::widget::{button, column, container, operation::focus, row, text, text_input};
use iced::{
    keyboard::{self, key::Named},
    Alignment, Element, Event, Length, Task, Theme,
};
use rfd::FileDialog;
use std::path::PathBuf;

use crate::app_icon;
use crate::icons::{self, CHECK, ERROR, INFO, WARNING};
use crate::resolution::RES_DEFAULT;
use crate::theme;
use crate::{display, dynamic_patcher, game_detection, ini_handler, patcher, validation};
use iced::window::settings::PlatformSpecific;
use iced_fonts::BOOTSTRAP_FONT_BYTES;

const ID_GAME_PATH: &str = "game_path";
const ID_CUSTOM_WIDTH: &str = "custom_width";
const ID_CUSTOM_HEIGHT: &str = "custom_height";

#[derive(Debug, Clone)]
pub enum Message {
    GamePathChanged(String),
    BrowseGamePath,
    GamePathSelected(Option<PathBuf>),

    // Custom resolution
    CustomWidthChanged(String),
    CustomHeightChanged(String),

    ApplyPatch,
    RestoreDefault,
    OperationComplete(Result<(), String>),

    // Focus management
    TabPressed(i32),
}

#[derive(Debug, Clone, Copy)]
enum OperationKind {
    ApplyPatch,
    RestoreDefault,
}

#[derive(Debug, Clone, Copy)]
enum StatusKind {
    Success,
    Error,
}

#[derive(Debug, Clone)]
struct StatusMessage {
    text: String,
    kind: StatusKind,
}

impl StatusMessage {
    fn success<T: Into<String>>(text: T) -> Self {
        Self {
            text: text.into(),
            kind: StatusKind::Success,
        }
    }

    fn error<T: Into<String>>(text: T) -> Self {
        Self {
            text: text.into(),
            kind: StatusKind::Error,
        }
    }
}

pub struct SettlersPatcher {
    game_path: String,
    game_path_valid: bool,

    // Custom resolution (only mode)
    custom_width: String,
    custom_height: String,
    custom_width_parsed: Option<u32>,
    custom_height_parsed: Option<u32>,

    is_valid_gog_version: bool,
    is_processing: bool,
    status_message: Option<StatusMessage>,
    last_operation: Option<OperationKind>,

    // Focus management
    focusable_ids: Vec<String>,
    current_focus_index: Option<usize>,
}

impl SettlersPatcher {
    fn new() -> (Self, Task<Message>) {
        // Auto-detect game path
        let detected_path = game_detection::detect_game_path();
        let game_path = detected_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default();

        // Detect primary monitor resolution (works on Wayland and X11)
        let (default_width, default_height) =
            display::detect_primary_resolution().unwrap_or((1920, 1080));

        let focusable_ids = vec![
            ID_GAME_PATH.to_string(),
            ID_CUSTOM_WIDTH.to_string(),
            ID_CUSTOM_HEIGHT.to_string(),
        ];

        let mut app = Self {
            game_path: game_path.clone(),
            game_path_valid: false,

            // Initialize custom resolution with detected monitor resolution
            custom_width: default_width.to_string(),
            custom_height: default_height.to_string(),
            custom_width_parsed: Some(default_width),
            custom_height_parsed: Some(default_height),

            is_valid_gog_version: false,
            is_processing: false,
            status_message: None,
            last_operation: None,

            focusable_ids,
            current_focus_index: None,
        };

        // Validate initial game path if detected
        if !game_path.is_empty() {
            app.validate_game_path();
        }

        (app, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GamePathChanged(path) => {
                self.game_path = path;
                self.validate_game_path();
                Task::none()
            }
            Message::BrowseGamePath => Task::perform(
                async {
                    FileDialog::new()
                        .set_title("Select your Settlers 4 installation directory")
                        .pick_folder()
                },
                Message::GamePathSelected,
            ),
            Message::GamePathSelected(selected_path) => {
                if let Some(path) = selected_path {
                    self.game_path = path.display().to_string();
                    self.validate_game_path();
                }

                Task::none()
            }
            Message::CustomWidthChanged(value) => {
                self.custom_width = value;
                self.validate_custom_resolution();
                Task::none()
            }
            Message::CustomHeightChanged(value) => {
                self.custom_height = value;
                self.validate_custom_resolution();
                Task::none()
            }
            Message::ApplyPatch => {
                let (width, height) = match self.get_active_resolution() {
                    Some(res) => res,
                    None => return Task::none(),
                };

                self.is_processing = true;
                self.status_message = None;
                self.last_operation = Some(OperationKind::ApplyPatch);
                let game_path = self.resolved_game_path();

                Task::perform(
                    async move {
                        if cfg!(not(test)) && patcher::is_game_running() {
                            return Err(
                                "The Settlers 4 (S4.exe/S4_Main.exe) is currently running. Please close the game before patching."
                                    .to_string(),
                            );
                        }

                        // Use dynamic patching
                        let dll_path = game_detection::get_dll_path(&game_path);
                        let config_path = game_detection::get_config_path(&game_path);

                        // Patch DLL dynamically
                        let patched_dll = match dynamic_patcher::patch_dll_dynamic(
                            RES_DEFAULT.dll_data,
                            width,
                            height,
                        ) {
                            Ok(dll) => dll,
                            Err(e) => return Err(format!("DLL patching failed: {}", e)),
                        };

                        // Write DLL
                        if let Err(e) = std::fs::write(&dll_path, patched_dll) {
                            return Err(format!("Failed to write DLL: {}", e));
                        }

                        // Update INI
                        if let Err(e) =
                            ini_handler::update_resolution_values(&config_path, width, height)
                        {
                            return Err(format!("Failed to update config: {}", e));
                        }

                        Ok(())
                    },
                    Message::OperationComplete,
                )
            }
            Message::RestoreDefault => {
                self.is_processing = true;
                self.status_message = None;
                self.last_operation = Some(OperationKind::RestoreDefault);
                let game_path = self.resolved_game_path();

                Task::perform(
                    async move {
                        match patcher::restore_to_default(&game_path) {
                            Ok(_) => Ok(()),
                            Err(e) => Err(format!("Restore failed: {}", e)),
                        }
                    },
                    Message::OperationComplete,
                )
            }
            Message::OperationComplete(result) => {
                self.is_processing = false;
                let was_success = result.is_ok();
                self.status_message = Some(match result {
                    Ok(_) => {
                        let success_text = match self.last_operation {
                            Some(OperationKind::ApplyPatch) => {
                                "Patch applied successfully. Launch the game to verify."
                            }
                            Some(OperationKind::RestoreDefault) => {
                                "Restored to the default resolution."
                            }
                            None => "Operation completed.",
                        };
                        StatusMessage::success(success_text)
                    }
                    Err(e) => StatusMessage::error(format!("Operation failed: {}", e)),
                });
                if was_success {
                    // Revalidate after operation
                    self.validate_game_path();
                }
                Task::none()
            }
            Message::TabPressed(direction) => {
                if self.focusable_ids.is_empty() {
                    return Task::none();
                }

                let new_index = match self.current_focus_index {
                    Some(index) => {
                        let new_idx = index as i32 + direction;
                        let len = self.focusable_ids.len() as i32;
                        if new_idx < 0 {
                            Some(len - 1)
                        } else if new_idx >= len {
                            Some(0)
                        } else {
                            Some(new_idx)
                        }
                    }
                    None => Some(0),
                };

                if let Some(idx) = new_index {
                    if idx >= 0 && (idx as usize) < self.focusable_ids.len() {
                        let id = self.focusable_ids[idx as usize].clone();
                        self.current_focus_index = Some(idx as usize);
                        return focus(id);
                    }
                }

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Game path input
        let game_path_label = text("Game Installation Path:");
        let game_path_input = text_input("e.g., /home/user/Games/Settlers4", &self.game_path)
            .id(Id::new(ID_GAME_PATH))
            .on_input(Message::GamePathChanged)
            .padding(10);

        let browse_button = button(text("Browse..."))
            .on_press(Message::BrowseGamePath)
            .padding(10)
            .style(theme::gold_button);

        let game_path_row = row![game_path_input, browse_button]
            .spacing(10)
            .align_y(Alignment::Center);

        // Validation status
        let validation_status: Element<_> = if self.game_path_valid {
            if self.is_valid_gog_version {
                icons::icon_text_colored(CHECK, "Valid GOG version detected", theme::MOSS).into()
            } else {
                icons::icon_text_colored(
                    ERROR,
                    "Invalid version - not GOG Gold Edition v2.50.1508",
                    theme::RUST,
                )
                .into()
            }
        } else if !self.game_path.is_empty() {
            icons::icon_text_colored(ERROR, "Invalid game directory", theme::RUST).into()
        } else {
            icons::icon_text_colored(INFO, "Please select game directory", theme::GOLD).into()
        };

        // Resolution section
        let resolution_label = text("Target Resolution:").size(16);

        let width_input = text_input("Width", &self.custom_width)
            .id(Id::new(ID_CUSTOM_WIDTH))
            .on_input(Message::CustomWidthChanged)
            .padding(10)
            .width(Length::Fixed(120.0));

        let height_input = text_input("Height", &self.custom_height)
            .id(Id::new(ID_CUSTOM_HEIGHT))
            .on_input(Message::CustomHeightChanged)
            .padding(10)
            .width(Length::Fixed(120.0));

        let custom_inputs_row = row![
            text("Width:").width(Length::Fixed(60.0)),
            width_input,
            text("Height:").width(Length::Fixed(70.0)),
            height_input,
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // Aspect ratio info
        let aspect_info: Element<_> = if let Some(aspect) = self.get_aspect_ratio_info() {
            if self.custom_width_parsed.is_some() && self.custom_height_parsed.is_some() {
                row![
                    icons::icon_colored(INFO, theme::GOLD),
                    text(" ").color(theme::TEXT_CREAM).size(14),
                    text(format!("Aspect ratio: {}", aspect))
                        .color(theme::TEXT_CREAM)
                        .size(14),
                ]
                .align_y(Alignment::Center)
                .spacing(5)
                .into()
            } else {
                row![
                    icons::icon_colored(WARNING, theme::AMBER),
                    text(" ").color(theme::TEXT_CREAM).size(14),
                    text("Invalid resolution values (800-7680 × 600-4320)")
                        .color(theme::TEXT_CREAM)
                        .size(14),
                ]
                .align_y(Alignment::Center)
                .spacing(5)
                .into()
            }
        } else {
            row![
                icons::icon_colored(WARNING, theme::AMBER),
                text(" ").color(theme::TEXT_CREAM).size(14),
                text("Enter width and height")
                    .color(theme::TEXT_CREAM)
                    .size(14),
            ]
            .align_y(Alignment::Center)
            .spacing(5)
            .into()
        };

        let resolution_valid =
            self.custom_width_parsed.is_some() && self.custom_height_parsed.is_some();

        let resolution_content = column![custom_inputs_row, aspect_info].spacing(8);

        // Action buttons
        let button_width = Length::Fixed(180.0);
        let apply_button = button(text("Apply Patch").size(16))
            .padding(14)
            .style(theme::gold_button)
            .width(button_width);

        let apply_button = if self.is_valid_gog_version && resolution_valid && !self.is_processing {
            apply_button.on_press(Message::ApplyPatch)
        } else {
            apply_button
        };

        let restore_default_button = button(text("Restore to Default").size(16))
            .padding(14)
            .style(theme::gold_button)
            .width(button_width);

        let restore_default_button = if self.game_path_valid && !self.is_processing {
            restore_default_button.on_press(Message::RestoreDefault)
        } else {
            restore_default_button
        };

        let button_row = row![apply_button, restore_default_button].spacing(10);

        // Operation status banner
        let status_banner: Element<_> = if let Some(status) = &self.status_message {
            let (icon, color) = match status.kind {
                StatusKind::Success => (CHECK, theme::MOSS),
                StatusKind::Error => (ERROR, theme::RUST),
            };

            row![
                icons::icon_colored(icon, color),
                text(" ").color(color).size(14),
                text(&status.text).color(theme::TEXT_CREAM).size(14)
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .into()
        } else {
            text("").size(1).into()
        };

        // Main layout
        let content = column![
            game_path_label,
            game_path_row,
            validation_status,
            text("").size(10), // spacer
            resolution_label,
            resolution_content,
            text("").size(10), // spacer
            button_row,
            status_banner
        ]
        .padding(20)
        .spacing(10)
        .max_width(700);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(theme::panel_container)
            .into()
    }

    fn theme(&self) -> Theme {
        theme::settlers_theme()
    }

    fn resolved_game_path(&self) -> PathBuf {
        game_detection::resolve_game_path(&self.game_path)
    }

    /// Get currently active resolution
    fn get_active_resolution(&self) -> Option<(u32, u32)> {
        match (self.custom_width_parsed, self.custom_height_parsed) {
            (Some(w), Some(h)) => Some((w, h)),
            _ => None,
        }
    }

    /// Validate custom resolution inputs
    fn validate_custom_resolution(&mut self) {
        // Parse width
        self.custom_width_parsed = self
            .custom_width
            .trim()
            .parse::<u32>()
            .ok()
            .filter(|&w| (800..=7680).contains(&w));

        // Parse height
        self.custom_height_parsed = self
            .custom_height
            .trim()
            .parse::<u32>()
            .ok()
            .filter(|&h| (600..=4320).contains(&h));
    }

    /// Get aspect ratio string for display
    fn get_aspect_ratio_info(&self) -> Option<String> {
        if let (Some(w), Some(h)) = (self.custom_width_parsed, self.custom_height_parsed) {
            let ratio = w as f64 / h as f64;
            let aspect = if (ratio - 4.0 / 3.0).abs() < 0.01 {
                "4:3"
            } else if (ratio - 16.0 / 9.0).abs() < 0.01 {
                "16:9"
            } else if (ratio - 16.0 / 10.0).abs() < 0.01 {
                "16:10"
            } else if (ratio - 21.0 / 9.0).abs() < 0.01 {
                "21:9 (Ultrawide)"
            } else if (ratio - 32.0 / 9.0).abs() < 0.01 {
                "32:9 (Super Ultrawide)"
            } else {
                return Some(format!("Custom ({:.2}:1)", ratio));
            };
            Some(aspect.to_string())
        } else {
            None
        }
    }

    fn validate_game_path(&mut self) {
        let path = self.resolved_game_path();

        self.game_path_valid = game_detection::is_valid_game_directory(&path);

        if self.game_path_valid {
            // Validate GOG version using S4.exe
            let exe_path = game_detection::get_exe_path(&path);
            match validation::validate_gog_version(&exe_path) {
                Ok(result) => {
                    self.is_valid_gog_version = result.is_valid;
                }
                Err(_) => {
                    self.is_valid_gog_version = false;
                }
            }
        } else {
            self.is_valid_gog_version = false;
        }
    }
}

pub fn run_gui() -> iced::Result {
    let window_icon = app_icon::app_icon().ok();

    iced::application(
        SettlersPatcher::new,
        SettlersPatcher::update,
        SettlersPatcher::view,
    )
    .title("Settlers 4 Widescreen Tool")
    .font(BOOTSTRAP_FONT_BYTES)
    .theme(SettlersPatcher::theme)
    .subscription(|_app| {
        iced::event::listen().filter_map(|event| {
            if let Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::Tab),
                modifiers,
                ..
            }) = event
            {
                let direction = if modifiers.shift() { -1 } else { 1 };
                Some(Message::TabPressed(direction))
            } else {
                None
            }
        })
    })
    .window(iced::window::Settings {
        size: iced::Size::new(750.0, 450.0),
        resizable: true,
        icon: window_icon,
        platform_specific: PlatformSpecific {
            application_id: String::from("settlers4linuxpatcher"),
            ..Default::default()
        },
        ..Default::default()
    })
    .run()
}
