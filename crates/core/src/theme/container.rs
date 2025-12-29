//! Container styling for layered backgrounds.
//!
//! Containers represent the different depth layers in the UI.

use crate::Color;

use super::{Component, Palette};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A container represents a background layer in the UI hierarchy.
///
/// Compatible with libcosmic's Container structure.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Container {
    /// The base background color of the container.
    pub base: Color,

    /// Component colors for widgets within this container.
    pub component: Component,

    /// Divider/separator color.
    pub divider: Color,

    /// Text color on this container.
    pub on: Color,

    /// Background for small widgets (checkboxes, toggles, etc.).
    pub small_widget: Color,
}

impl Container {
    /// Create a new container with the given colors.
    pub fn new(base: Color, on: Color) -> Self {
        Self {
            base,
            component: Component::new(base, on),
            divider: on.scale_alpha(0.15),
            on,
            small_widget: on.scale_alpha(0.1),
        }
    }

    /// Create the background container for dark theme.
    pub fn dark_background(palette: &Palette) -> Self {
        let base = palette.neutral_2;
        let on = palette.neutral_9;

        Self {
            base,
            component: Component::standard(palette, true),
            divider: on.scale_alpha(0.15),
            on,
            small_widget: palette.neutral_4,
        }
    }

    /// Create the primary container for dark theme.
    pub fn dark_primary(palette: &Palette) -> Self {
        let base = palette.neutral_3;
        let on = palette.neutral_9;

        Self {
            base,
            component: Component::standard(palette, true),
            divider: on.scale_alpha(0.15),
            on,
            small_widget: palette.neutral_5,
        }
    }

    /// Create the secondary container for dark theme.
    pub fn dark_secondary(palette: &Palette) -> Self {
        let base = palette.neutral_4;
        let on = palette.neutral_9;

        Self {
            base,
            component: Component::standard(palette, true),
            divider: on.scale_alpha(0.15),
            on,
            small_widget: palette.neutral_5,
        }
    }

    /// Create the background container for light theme.
    pub fn light_background(palette: &Palette) -> Self {
        let base = palette.neutral_1;
        let on = palette.neutral_9;

        Self {
            base,
            component: Component::standard(palette, false),
            divider: on.scale_alpha(0.15),
            on,
            small_widget: palette.neutral_3,
        }
    }

    /// Create the primary container for light theme.
    pub fn light_primary(palette: &Palette) -> Self {
        let base = palette.neutral_0;
        let on = palette.neutral_9;

        Self {
            base,
            component: Component::standard(palette, false),
            divider: on.scale_alpha(0.15),
            on,
            small_widget: palette.neutral_3,
        }
    }

    /// Create the secondary container for light theme.
    pub fn light_secondary(palette: &Palette) -> Self {
        let base = palette.neutral_2;
        let on = palette.neutral_9;

        Self {
            base,
            component: Component::standard(palette, false),
            divider: on.scale_alpha(0.15),
            on,
            small_widget: palette.neutral_4,
        }
    }
}

impl Default for Container {
    fn default() -> Self {
        let palette = Palette::dark();
        Self::dark_background(&palette)
    }
}
