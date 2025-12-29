//! Example demonstrating the new libcosmic-compatible theme system.
//!
//! This example shows:
//! - Using the default dark and light themes
//! - Accessing theme values for consistent styling
//! - Theme-aware widget styling

use iced::theme2::Theme;
use iced::widget::{button, column, container, row, text};
use iced::{Color, Element, Length};

pub fn main() -> iced::Result {
    iced::run(update, view)
}

#[derive(Default)]
struct State {
    is_dark: bool,
}

#[derive(Debug, Clone)]
enum Message {
    ToggleTheme,
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::ToggleTheme => {
            state.is_dark = !state.is_dark;
        }
    }
}

fn view(state: &State) -> Element<'_, Message> {
    // Get the theme based on state
    let theme = if state.is_dark {
        Theme::dark()
    } else {
        Theme::light()
    };

    // Access theme values (clone to avoid lifetime issues)
    let spacing_m = theme.spacing.m;
    let spacing_l = theme.spacing.l;
    let radius_s = theme.corner_radii.radius_s;
    let is_dark = theme.is_dark;
    let name = theme.name.clone();

    // Get colors
    let accent = theme.palette.accent();
    let accent_green = theme.palette.accent_green;
    let accent_red = theme.palette.accent_red;
    let accent_orange = theme.palette.accent_orange;
    let on_accent = if is_dark {
        theme.palette.neutral_0
    } else {
        theme.palette.neutral_10
    };
    let bg_color = theme.background.base;
    let on_bg = theme.on_background();

    // Create a themed layout
    let content = column![
        text("Theme2 Example").size(32),
        text(format!("Current theme: {}", name)).size(16),
        text(format!("Is dark: {}", is_dark)).size(16),
        text(format!("Spacing medium: {}px", spacing_m)).size(14),
        text(format!("Corner radius small: {:?}px", radius_s)).size(14),
        button(text(if is_dark {
            "Switch to Light"
        } else {
            "Switch to Dark"
        }))
        .on_press(Message::ToggleTheme),
        // Show some theme colors
        row![
            color_swatch("Accent", accent, on_accent),
            color_swatch("Success", accent_green, on_accent),
            color_swatch("Destructive", accent_red, on_accent),
            color_swatch("Warning", accent_orange, on_accent),
        ]
        .spacing(10),
    ]
    .spacing(spacing_m as u32)
    .padding(spacing_l);

    // Apply background styling
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(bg_color.into()),
            text_color: Some(on_bg),
            ..Default::default()
        })
        .into()
}

fn color_swatch<'a>(
    label: &'a str,
    bg: Color,
    text_color: Color,
) -> container::Container<'a, Message> {
    container(text(label))
        .padding(10)
        .style(move |_| container::Style {
            background: Some(bg.into()),
            text_color: Some(text_color),
            ..Default::default()
        })
}
