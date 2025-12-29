//! Component styling for widget states.
//!
//! A Component defines colors for all interactive states of a widget.

use crate::Color;

use super::Palette;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Colors for a component/widget across all interaction states.
///
/// Compatible with libcosmic's Component structure.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Component {
    /// Base/normal state background color.
    pub base: Color,

    /// Hovered state background color.
    pub hover: Color,

    /// Pressed/active state background color.
    pub pressed: Color,

    /// Selected state background color.
    pub selected: Color,

    /// Text color when selected.
    pub selected_text: Color,

    /// Focus indicator color.
    pub focus: Color,

    /// Divider/separator color within the component.
    pub divider: Color,

    /// Text/icon color on the component.
    pub on: Color,

    /// Disabled state background color.
    pub disabled: Color,

    /// Text color when disabled.
    pub on_disabled: Color,

    /// Border color.
    pub border: Color,

    /// Border color when disabled.
    pub disabled_border: Color,
}

impl Component {
    /// Create a new component with all colors set to the same value.
    pub fn new(base: Color, on: Color) -> Self {
        Self {
            base,
            hover: base,
            pressed: base,
            selected: base,
            selected_text: on,
            focus: base,
            divider: on.scale_alpha(0.2),
            on,
            disabled: base.scale_alpha(0.5),
            on_disabled: on.scale_alpha(0.5),
            border: Color::TRANSPARENT,
            disabled_border: Color::TRANSPARENT,
        }
    }

    /// Create an accent-colored component.
    pub fn accent(palette: &Palette, is_dark: bool) -> Self {
        let base = palette.accent();
        let on = if is_dark {
            palette.neutral_0
        } else {
            palette.neutral_10
        };

        Self {
            base,
            hover: lighten(base, 0.1),
            pressed: darken(base, 0.1),
            selected: base,
            selected_text: on,
            focus: lighten(base, 0.2),
            divider: on.scale_alpha(0.2),
            on,
            disabled: base.scale_alpha(0.5),
            on_disabled: on.scale_alpha(0.5),
            border: Color::TRANSPARENT,
            disabled_border: Color::TRANSPARENT,
        }
    }

    /// Create a success-colored component.
    pub fn success(palette: &Palette, is_dark: bool) -> Self {
        let base = palette.accent_green;
        let on = if is_dark {
            palette.neutral_0
        } else {
            palette.neutral_10
        };

        Self {
            base,
            hover: lighten(base, 0.1),
            pressed: darken(base, 0.1),
            selected: base,
            selected_text: on,
            focus: lighten(base, 0.2),
            divider: on.scale_alpha(0.2),
            on,
            disabled: base.scale_alpha(0.5),
            on_disabled: on.scale_alpha(0.5),
            border: Color::TRANSPARENT,
            disabled_border: Color::TRANSPARENT,
        }
    }

    /// Create a destructive/danger-colored component.
    pub fn destructive(palette: &Palette, is_dark: bool) -> Self {
        let base = palette.accent_red;
        let on = if is_dark {
            palette.neutral_0
        } else {
            palette.neutral_10
        };

        Self {
            base,
            hover: lighten(base, 0.1),
            pressed: darken(base, 0.1),
            selected: base,
            selected_text: on,
            focus: lighten(base, 0.2),
            divider: on.scale_alpha(0.2),
            on,
            disabled: base.scale_alpha(0.5),
            on_disabled: on.scale_alpha(0.5),
            border: Color::TRANSPARENT,
            disabled_border: Color::TRANSPARENT,
        }
    }

    /// Create a warning-colored component.
    pub fn warning(palette: &Palette, is_dark: bool) -> Self {
        let base = palette.accent_orange;
        let on = if is_dark {
            palette.neutral_0
        } else {
            palette.neutral_10
        };

        Self {
            base,
            hover: lighten(base, 0.1),
            pressed: darken(base, 0.1),
            selected: base,
            selected_text: on,
            focus: lighten(base, 0.2),
            divider: on.scale_alpha(0.2),
            on,
            disabled: base.scale_alpha(0.5),
            on_disabled: on.scale_alpha(0.5),
            border: Color::TRANSPARENT,
            disabled_border: Color::TRANSPARENT,
        }
    }

    /// Create a standard button component.
    pub fn standard(palette: &Palette, is_dark: bool) -> Self {
        let base = if is_dark {
            palette.neutral_3
        } else {
            palette.neutral_2
        };
        let on = if is_dark {
            palette.neutral_9
        } else {
            palette.neutral_9
        };

        Self {
            base,
            hover: if is_dark {
                lighten(base, 0.05)
            } else {
                darken(base, 0.05)
            },
            pressed: if is_dark {
                darken(base, 0.05)
            } else {
                darken(base, 0.1)
            },
            selected: palette.accent(),
            selected_text: if is_dark {
                palette.neutral_0
            } else {
                palette.neutral_10
            },
            focus: palette.accent().scale_alpha(0.5),
            divider: on.scale_alpha(0.2),
            on,
            disabled: base.scale_alpha(0.5),
            on_disabled: on.scale_alpha(0.5),
            border: Color::TRANSPARENT,
            disabled_border: Color::TRANSPARENT,
        }
    }

    /// Create a transparent/ghost button component.
    pub fn transparent(palette: &Palette, is_dark: bool) -> Self {
        let on = if is_dark {
            palette.neutral_9
        } else {
            palette.neutral_9
        };

        Self {
            base: Color::TRANSPARENT,
            hover: on.scale_alpha(0.1),
            pressed: on.scale_alpha(0.2),
            selected: palette.accent().scale_alpha(0.2),
            selected_text: palette.accent(),
            focus: palette.accent().scale_alpha(0.3),
            divider: on.scale_alpha(0.2),
            on,
            disabled: Color::TRANSPARENT,
            on_disabled: on.scale_alpha(0.5),
            border: Color::TRANSPARENT,
            disabled_border: Color::TRANSPARENT,
        }
    }

    /// Create a link-style component.
    pub fn link(palette: &Palette, _is_dark: bool) -> Self {
        let base = palette.accent();

        Self {
            base: Color::TRANSPARENT,
            hover: Color::TRANSPARENT,
            pressed: Color::TRANSPARENT,
            selected: Color::TRANSPARENT,
            selected_text: base,
            focus: base.scale_alpha(0.3),
            divider: Color::TRANSPARENT,
            on: base,
            disabled: Color::TRANSPARENT,
            on_disabled: base.scale_alpha(0.5),
            border: Color::TRANSPARENT,
            disabled_border: Color::TRANSPARENT,
        }
    }
}

impl Default for Component {
    fn default() -> Self {
        Self::new(Color::TRANSPARENT, Color::WHITE)
    }
}

/// Lighten a color by a factor (0.0 to 1.0).
fn lighten(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r + (1.0 - color.r) * amount).min(1.0),
        g: (color.g + (1.0 - color.g) * amount).min(1.0),
        b: (color.b + (1.0 - color.b) * amount).min(1.0),
        a: color.a,
    }
}

/// Darken a color by a factor (0.0 to 1.0).
fn darken(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r * (1.0 - amount)).max(0.0),
        g: (color.g * (1.0 - amount)).max(0.0),
        b: (color.b * (1.0 - amount)).max(0.0),
        a: color.a,
    }
}
