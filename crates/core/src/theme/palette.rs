//! Color palette for the theme.
//!
//! Compatible with libcosmic's CosmicPaletteInner.

use crate::Color;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The color palette containing all base colors for a theme.
///
/// This structure is compatible with libcosmic's palette format.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Palette {
    /// Theme name.
    pub name: String,

    // Semantic colors
    /// Bright red for errors/destructive actions.
    pub bright_red: Color,
    /// Bright green for success states.
    pub bright_green: Color,
    /// Bright orange for warnings.
    pub bright_orange: Color,

    // Surface grays
    /// Gray level 1 (darkest).
    pub gray_1: Color,
    /// Gray level 2.
    pub gray_2: Color,

    // Neutral colors (11 levels, 0 = darkest, 10 = lightest)
    /// Neutral 0 - Pure black or near-black.
    pub neutral_0: Color,
    /// Neutral 1.
    pub neutral_1: Color,
    /// Neutral 2.
    pub neutral_2: Color,
    /// Neutral 3.
    pub neutral_3: Color,
    /// Neutral 4.
    pub neutral_4: Color,
    /// Neutral 5 - Mid gray.
    pub neutral_5: Color,
    /// Neutral 6.
    pub neutral_6: Color,
    /// Neutral 7.
    pub neutral_7: Color,
    /// Neutral 8.
    pub neutral_8: Color,
    /// Neutral 9.
    pub neutral_9: Color,
    /// Neutral 10 - Pure white or near-white.
    pub neutral_10: Color,

    // Accent colors
    /// Blue accent.
    pub accent_blue: Color,
    /// Indigo accent.
    pub accent_indigo: Color,
    /// Purple accent.
    pub accent_purple: Color,
    /// Pink accent.
    pub accent_pink: Color,
    /// Red accent.
    pub accent_red: Color,
    /// Orange accent.
    pub accent_orange: Color,
    /// Yellow accent.
    pub accent_yellow: Color,
    /// Green accent.
    pub accent_green: Color,
    /// Warm gray accent.
    pub accent_warm_grey: Color,

    // Extended palette colors
    /// Extended warm gray.
    pub ext_warm_grey: Color,
    /// Extended orange.
    pub ext_orange: Color,
    /// Extended yellow.
    pub ext_yellow: Color,
    /// Extended blue.
    pub ext_blue: Color,
    /// Extended purple.
    pub ext_purple: Color,
    /// Extended pink.
    pub ext_pink: Color,
    /// Extended indigo.
    pub ext_indigo: Color,
}

impl Palette {
    /// The default light palette.
    pub fn light() -> Self {
        Self {
            name: "Light".into(),

            // Semantic colors
            bright_red: Color::from_rgb(0.92, 0.26, 0.21),
            bright_green: Color::from_rgb(0.30, 0.69, 0.31),
            bright_orange: Color::from_rgb(1.0, 0.60, 0.0),

            // Surface grays (lighter for light theme)
            gray_1: Color::from_rgb(0.96, 0.96, 0.96),
            gray_2: Color::from_rgb(0.93, 0.93, 0.93),

            // Neutrals (inverted for light theme)
            neutral_0: Color::WHITE,
            neutral_1: Color::from_rgb(0.98, 0.98, 0.98),
            neutral_2: Color::from_rgb(0.96, 0.96, 0.96),
            neutral_3: Color::from_rgb(0.93, 0.93, 0.93),
            neutral_4: Color::from_rgb(0.88, 0.88, 0.88),
            neutral_5: Color::from_rgb(0.74, 0.74, 0.74),
            neutral_6: Color::from_rgb(0.62, 0.62, 0.62),
            neutral_7: Color::from_rgb(0.46, 0.46, 0.46),
            neutral_8: Color::from_rgb(0.38, 0.38, 0.38),
            neutral_9: Color::from_rgb(0.26, 0.26, 0.26),
            neutral_10: Color::BLACK,

            // Accent colors
            accent_blue: Color::from_rgb(0.13, 0.59, 0.95),
            accent_indigo: Color::from_rgb(0.25, 0.32, 0.71),
            accent_purple: Color::from_rgb(0.61, 0.15, 0.69),
            accent_pink: Color::from_rgb(0.91, 0.12, 0.39),
            accent_red: Color::from_rgb(0.90, 0.22, 0.21),
            accent_orange: Color::from_rgb(1.0, 0.60, 0.0),
            accent_yellow: Color::from_rgb(1.0, 0.76, 0.03),
            accent_green: Color::from_rgb(0.30, 0.69, 0.31),
            accent_warm_grey: Color::from_rgb(0.47, 0.43, 0.38),

            // Extended colors
            ext_warm_grey: Color::from_rgb(0.47, 0.43, 0.38),
            ext_orange: Color::from_rgb(1.0, 0.60, 0.0),
            ext_yellow: Color::from_rgb(1.0, 0.76, 0.03),
            ext_blue: Color::from_rgb(0.13, 0.59, 0.95),
            ext_purple: Color::from_rgb(0.61, 0.15, 0.69),
            ext_pink: Color::from_rgb(0.91, 0.12, 0.39),
            ext_indigo: Color::from_rgb(0.25, 0.32, 0.71),
        }
    }

    /// The default dark palette.
    pub fn dark() -> Self {
        Self {
            name: "Dark".into(),

            // Semantic colors
            bright_red: Color::from_rgb(1.0, 0.63, 0.60),
            bright_green: Color::from_rgb(0.37, 0.86, 0.55),
            bright_orange: Color::from_rgb(1.0, 0.64, 0.49),

            // Surface grays
            gray_1: Color::from_rgb(0.11, 0.11, 0.11),
            gray_2: Color::from_rgb(0.15, 0.15, 0.15),

            // Neutrals (0 = black, 10 = white)
            neutral_0: Color::BLACK,
            neutral_1: Color::from_rgb(0.07, 0.07, 0.07),
            neutral_2: Color::from_rgb(0.13, 0.13, 0.13),
            neutral_3: Color::from_rgb(0.18, 0.18, 0.18),
            neutral_4: Color::from_rgb(0.24, 0.24, 0.24),
            neutral_5: Color::from_rgb(0.32, 0.32, 0.32),
            neutral_6: Color::from_rgb(0.44, 0.44, 0.44),
            neutral_7: Color::from_rgb(0.62, 0.62, 0.62),
            neutral_8: Color::from_rgb(0.74, 0.74, 0.74),
            neutral_9: Color::from_rgb(0.87, 0.87, 0.87),
            neutral_10: Color::WHITE,

            // Accent colors (COSMIC defaults)
            accent_blue: Color::from_rgb(0.38, 0.68, 0.94),
            accent_indigo: Color::from_rgb(0.51, 0.58, 0.93),
            accent_purple: Color::from_rgb(0.74, 0.58, 0.98),
            accent_pink: Color::from_rgb(0.96, 0.56, 0.75),
            accent_red: Color::from_rgb(1.0, 0.63, 0.60),
            accent_orange: Color::from_rgb(1.0, 0.64, 0.49),
            accent_yellow: Color::from_rgb(0.99, 0.83, 0.46),
            accent_green: Color::from_rgb(0.37, 0.86, 0.55),
            accent_warm_grey: Color::from_rgb(0.55, 0.52, 0.48),

            // Extended colors
            ext_warm_grey: Color::from_rgb(0.55, 0.52, 0.48),
            ext_orange: Color::from_rgb(1.0, 0.64, 0.49),
            ext_yellow: Color::from_rgb(0.99, 0.83, 0.46),
            ext_blue: Color::from_rgb(0.38, 0.68, 0.94),
            ext_purple: Color::from_rgb(0.74, 0.58, 0.98),
            ext_pink: Color::from_rgb(0.96, 0.56, 0.75),
            ext_indigo: Color::from_rgb(0.51, 0.58, 0.93),
        }
    }

    /// Get a neutral color by index (0-10).
    pub fn neutral(&self, index: u8) -> Color {
        match index {
            0 => self.neutral_0,
            1 => self.neutral_1,
            2 => self.neutral_2,
            3 => self.neutral_3,
            4 => self.neutral_4,
            5 => self.neutral_5,
            6 => self.neutral_6,
            7 => self.neutral_7,
            8 => self.neutral_8,
            9 => self.neutral_9,
            _ => self.neutral_10,
        }
    }

    /// Get the current accent color (defaults to blue).
    pub fn accent(&self) -> Color {
        self.accent_blue
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::dark()
    }
}
