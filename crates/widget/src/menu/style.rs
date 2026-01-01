// From iced_aw, license MIT
// Ported from libcosmic

//! Styling for menu bars.

use crate::Theme;
use crate::button::{self, Status};
use crate::core::Color;

/// The appearance of a menu bar and its menus.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The background color of the menu bar and its menus.
    pub background: Color,
    /// The border width of the menu bar and its menus.
    pub border_width: f32,
    /// The border radius of the menu bar.
    pub bar_border_radius: [f32; 4],
    /// The border radius of the menus.
    pub menu_border_radius: [f32; 4],
    /// The border [`Color`] of the menu bar and its menus.
    pub border_color: Color,
    /// The expand value of the menus' background
    pub background_expand: [u16; 4],
    /// The highlighted path [`Color`] of popup menus (uses primary color).
    pub path: Color,
    /// The highlighted path [`Color`] of the menu bar (subtle highlight).
    pub bar_path: Color,
    /// The border radius of the path highlight (selected item).
    pub path_border_radius: [f32; 4],
    /// The padding inside the menu bar for item highlights [top, right, bottom, left].
    pub menu_content_padding: [f32; 4],
    /// The padding inside the popup menu for item highlights [top, right, bottom, left].
    pub menu_inner_content_padding: [f32; 4],
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            background: Color::from_rgb(0.95, 0.95, 0.95),
            border_width: 1.0,
            bar_border_radius: [8.0; 4],
            menu_border_radius: [8.0; 4],
            border_color: Color::from_rgb(0.8, 0.8, 0.8),
            background_expand: [1; 4],
            path: Color::from_rgba(0.5, 0.5, 0.5, 0.5),
            bar_path: Color::from_rgba(0.5, 0.5, 0.5, 0.3),
            path_border_radius: [4.0; 4],
            menu_content_padding: [2.0, 4.0, 1.0, 4.0],
            menu_inner_content_padding: [4.0, 4.0, 4.0, 4.0],
        }
    }
}

/// The style sheet of a menu bar and its menus.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default + Clone;

    /// Produces the [`Appearance`] of a menu bar and its menus.
    fn appearance(&self, style: &Self::Style) -> Appearance;
}

/// The default style of a menu bar.
#[derive(Default, Clone, Copy, Debug)]
pub enum Style {
    /// The default style.
    #[default]
    Default,
}

impl StyleSheet for crate::Theme {
    type Style = Style;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        match style {
            Style::Default => {
                let background = self.background.base;

                Appearance {
                    background,
                    border_width: 1.0,
                    bar_border_radius: [8.0; 4],
                    menu_border_radius: [8.0; 4],
                    border_color: Color {
                        a: 0.3,
                        ..self.background.on
                    },
                    background_expand: [1; 4],
                    // Use primary color for popup menu highlights
                    path: self.accent.hover,
                    // Use subtle color for menu bar highlights
                    bar_path: self.primary.base,
                    path_border_radius: [4.0; 4],
                    menu_content_padding: [2.0, 4.0, 1.0, 4.0],
                    menu_inner_content_padding: [4.0, 4.0, 4.0, 4.0],
                }
            }
        }
    }
}

/// A button style for menu items.
/// Note: The hover effect is handled by the menu's path highlight,
/// so we don't add a hover background here.
pub fn menu_item(theme: &Theme, status: Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: theme.background.on,
        ..button::Style::default()
    };

    match status {
        Status::Active | Status::Hovered | Status::Pressed | Status::Selected => base,
        Status::Disabled => button::Style {
            text_color: theme.background.on.scale_alpha(0.5),
            ..base
        },
    }
}

/// A button style for menu root items (top-level menu buttons).
/// Note: The hover/selected highlight is handled by the menu bar's path highlight,
/// so we don't add a hover background here.
pub fn menu_root_style(theme: &Theme, status: Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: theme.background.on,
        ..button::Style::default()
    };

    match status {
        Status::Active | Status::Hovered | Status::Pressed | Status::Selected => base,
        Status::Disabled => base, // Menu roots should not appear disabled
    }
}

/// A button style for menu folders (submenus).
pub fn menu_folder(theme: &Theme, status: Status) -> button::Style {
    // Menu folders use the same style as regular menu items
    menu_item(theme, status)
}
