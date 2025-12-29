//! Spacing and corner radius values.
//!
//! These provide consistent sizing throughout the UI.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Standard spacing values for the theme.
///
/// Compatible with libcosmic's Spacing structure.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Spacing {
    /// No spacing (0).
    pub none: u16,
    /// Extra extra extra small spacing.
    pub xxxs: u16,
    /// Extra extra small spacing.
    pub xxs: u16,
    /// Extra small spacing.
    pub xs: u16,
    /// Small spacing.
    pub s: u16,
    /// Medium spacing (default).
    pub m: u16,
    /// Large spacing.
    pub l: u16,
    /// Extra large spacing.
    pub xl: u16,
    /// Extra extra large spacing.
    pub xxl: u16,
    /// Extra extra extra large spacing.
    pub xxxl: u16,
}

impl Default for Spacing {
    fn default() -> Self {
        // COSMIC default spacing values
        Self {
            none: 0,
            xxxs: 2,
            xxs: 4,
            xs: 8,
            s: 12,
            m: 16,
            l: 24,
            xl: 32,
            xxl: 48,
            xxxl: 64,
        }
    }
}

impl Spacing {
    /// Create compact spacing (smaller values).
    pub fn compact() -> Self {
        Self {
            none: 0,
            xxxs: 1,
            xxs: 2,
            xs: 4,
            s: 6,
            m: 8,
            l: 12,
            xl: 16,
            xxl: 24,
            xxxl: 32,
        }
    }

    /// Create comfortable spacing (larger values).
    pub fn comfortable() -> Self {
        Self {
            none: 0,
            xxxs: 4,
            xxs: 8,
            xs: 12,
            s: 16,
            m: 24,
            l: 32,
            xl: 48,
            xxl: 64,
            xxxl: 96,
        }
    }
}

/// Standard corner radius values for the theme.
///
/// Compatible with libcosmic's CornerRadii structure.
/// Each radius is specified as `[top_left, top_right, bottom_right, bottom_left]`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CornerRadii {
    /// No rounding (sharp corners).
    pub radius_0: [f32; 4],
    /// Extra small radius.
    pub radius_xs: [f32; 4],
    /// Small radius.
    pub radius_s: [f32; 4],
    /// Medium radius (default for buttons).
    pub radius_m: [f32; 4],
    /// Large radius (cards, containers).
    pub radius_l: [f32; 4],
    /// Extra large radius (dialogs, panels).
    pub radius_xl: [f32; 4],
}

impl Default for CornerRadii {
    fn default() -> Self {
        // COSMIC default corner radii
        Self {
            radius_0: [0.0; 4],
            radius_xs: [2.0; 4],
            radius_s: [4.0; 4],
            radius_m: [8.0; 4],
            radius_l: [12.0; 4],
            radius_xl: [16.0; 4],
        }
    }
}

impl CornerRadii {
    /// Create sharp corners (no rounding).
    pub fn sharp() -> Self {
        Self {
            radius_0: [0.0; 4],
            radius_xs: [0.0; 4],
            radius_s: [0.0; 4],
            radius_m: [0.0; 4],
            radius_l: [0.0; 4],
            radius_xl: [0.0; 4],
        }
    }

    /// Create rounded corners (more rounding).
    pub fn rounded() -> Self {
        Self {
            radius_0: [0.0; 4],
            radius_xs: [4.0; 4],
            radius_s: [8.0; 4],
            radius_m: [12.0; 4],
            radius_l: [16.0; 4],
            radius_xl: [24.0; 4],
        }
    }

    /// Create pill-shaped corners (maximum rounding).
    pub fn pill() -> Self {
        Self {
            radius_0: [0.0; 4],
            radius_xs: [100.0; 4],
            radius_s: [100.0; 4],
            radius_m: [100.0; 4],
            radius_l: [100.0; 4],
            radius_xl: [100.0; 4],
        }
    }
}
