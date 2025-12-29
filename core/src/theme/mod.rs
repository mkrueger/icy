//! Libcosmic-compatible theme system.
//!
//! This module provides a theming system compatible with Pop!_OS/libcosmic theme files.
//! On Pop!_OS, it can load system themes from the cosmic-config directories.
//! On other platforms, it provides light and dark defaults.
//!
//! # Theme Structure
//!
//! The theme is organized into:
//! - **Palette**: The raw color values (neutrals, accents, semantic colors)
//! - **Container**: Background layer colors (background, primary, secondary)
//! - **Component**: Widget state colors (base, hover, pressed, disabled, etc.)
//! - **Spacing**: Standardized spacing values
//! - **CornerRadii**: Standardized border radius values

mod component;
mod container;
mod loader;
pub mod palette;
mod spacing;

#[cfg(test)]
mod tests;

pub use component::Component;
pub use container::Container;
pub use loader::{LoadError, load_system_theme, load_theme_from_file};
pub use palette::Palette;
pub use spacing::{CornerRadii, Spacing};

use crate::Color;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::sync::Arc;

/// The base application style of a theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The background [`Color`] of the application.
    pub background_color: Color,

    /// The default text [`Color`] of the application.
    pub text_color: Color,
}

/// A base theme trait for theme polymorphism.
///
/// This trait allows widgets to work with different theme types
/// by providing a common interface for getting the mode and
/// creating default themes.
pub trait Base: Sized {
    /// Returns the mode (light or dark) of the theme.
    fn mode(&self) -> Mode;

    /// Returns the name of the theme.
    fn name(&self) -> &str;

    /// Returns the base application [`Style`] of the theme.
    fn base(&self) -> Style;

    /// Returns the color [`Palette`] of the theme (for debugging).
    fn palette(&self) -> Option<Palette>;

    /// Returns a default theme for the given mode.
    fn default(mode: Mode) -> Self;
}

impl Base for Theme {
    fn mode(&self) -> Mode {
        if self.is_dark {
            Mode::Dark
        } else {
            Mode::Light
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn base(&self) -> Style {
        Style {
            background_color: self.background.base,
            text_color: self.background.on,
        }
    }

    fn palette(&self) -> Option<Palette> {
        Some(self.palette.clone())
    }

    fn default(mode: Mode) -> Self {
        match mode {
            Mode::Light => Theme::light(),
            Mode::Dark | Mode::None => Theme::dark(),
        }
    }
}

/// A theme compatible with libcosmic/Pop!_OS theming.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Theme {
    /// The name of the theme.
    pub name: String,

    /// The background container (outermost layer).
    pub background: Container,

    /// The primary container (main content areas).
    pub primary: Container,

    /// The secondary container (dialogs, popovers).
    pub secondary: Container,

    /// Accent component colors.
    pub accent: Component,

    /// Success/positive component colors.
    pub success: Component,

    /// Destructive/danger component colors.
    pub destructive: Component,

    /// Warning component colors.
    pub warning: Component,

    /// Accent button colors.
    pub accent_button: Component,

    /// Success button colors.
    pub success_button: Component,

    /// Destructive button colors.
    pub destructive_button: Component,

    /// Warning button colors.
    pub warning_button: Component,

    /// Icon button colors (minimal style).
    pub icon_button: Component,

    /// Link button colors.
    pub link_button: Component,

    /// Text button colors.
    pub text_button: Component,

    /// Standard button colors.
    pub button: Component,

    /// The underlying color palette.
    pub palette: Palette,

    /// Spacing values.
    pub spacing: Spacing,

    /// Corner radii values.
    pub corner_radii: CornerRadii,

    /// Whether this is a dark theme.
    pub is_dark: bool,

    /// Whether this is a high contrast theme.
    pub is_high_contrast: bool,

    /// Window shade/overlay color.
    pub shade: Color,
}

impl Theme {
    /// The default light theme.
    pub fn light() -> Self {
        Self::from_palette(Palette::light(), false)
    }

    /// The default dark theme.
    pub fn dark() -> Self {
        Self::from_palette(Palette::dark(), true)
    }

    /// Creates a custom theme with the given name and palette.
    pub fn custom(name: impl Into<String>, palette: Palette) -> Self {
        let is_dark = palette.neutral_5.r < 0.5;
        let mut theme = Self::from_palette(palette, is_dark);
        theme.name = name.into();
        theme
    }

    /// Create a theme from a palette.
    pub fn from_palette(palette: Palette, is_dark: bool) -> Self {
        let (background, primary, secondary) = if is_dark {
            (
                Container::dark_background(&palette),
                Container::dark_primary(&palette),
                Container::dark_secondary(&palette),
            )
        } else {
            (
                Container::light_background(&palette),
                Container::light_primary(&palette),
                Container::light_secondary(&palette),
            )
        };

        let accent = Component::accent(&palette, is_dark);
        let success = Component::success(&palette, is_dark);
        let destructive = Component::destructive(&palette, is_dark);
        let warning = Component::warning(&palette, is_dark);

        Self {
            name: if is_dark {
                "Dark".into()
            } else {
                "Light".into()
            },
            background,
            primary,
            secondary,
            accent: accent.clone(),
            success: success.clone(),
            destructive: destructive.clone(),
            warning: warning.clone(),
            accent_button: accent.clone(),
            success_button: success.clone(),
            destructive_button: destructive.clone(),
            warning_button: warning.clone(),
            icon_button: Component::transparent(&palette, is_dark),
            link_button: Component::link(&palette, is_dark),
            text_button: Component::transparent(&palette, is_dark),
            button: Component::standard(&palette, is_dark),
            palette,
            spacing: Spacing::default(),
            corner_radii: CornerRadii::default(),
            is_dark,
            is_high_contrast: false,
            shade: if is_dark {
                Color::from_rgba(0.0, 0.0, 0.0, 0.5)
            } else {
                Color::from_rgba(0.0, 0.0, 0.0, 0.3)
            },
        }
    }

    /// Returns the text color for the background layer.
    pub fn on_background(&self) -> Color {
        self.background.on
    }

    /// Returns the text color for the primary layer.
    pub fn on_primary(&self) -> Color {
        self.primary.on
    }

    /// Returns the text color for the secondary layer.
    pub fn on_secondary(&self) -> Color {
        self.secondary.on
    }

    /// Returns a list of all built-in themes.
    ///
    /// Currently includes Light and Dark themes.
    pub fn all() -> Vec<Theme> {
        vec![Theme::light(), Theme::dark()]
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Default for Theme {
    fn default() -> Self {
        // Default to dark theme (like libcosmic)
        Self::dark()
    }
}

/// A shared reference to a theme.
pub type ThemeRef = Arc<Theme>;

/// The mode of a theme (light or dark).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// No specific mode.
    #[default]
    None,
    /// Light mode.
    Light,
    /// Dark mode.
    Dark,
}

impl Mode {
    /// Returns true if this is dark mode.
    pub fn is_dark(self) -> bool {
        matches!(self, Mode::Dark)
    }
}

/// The layer/depth of a container in the UI hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layer {
    /// The background layer (outermost).
    #[default]
    Background,
    /// The primary content layer.
    Primary,
    /// The secondary layer (dialogs, popovers).
    Secondary,
}

impl Theme {
    /// Get the container for a specific layer.
    pub fn container(&self, layer: Layer) -> &Container {
        match layer {
            Layer::Background => &self.background,
            Layer::Primary => &self.primary,
            Layer::Secondary => &self.secondary,
        }
    }
}
