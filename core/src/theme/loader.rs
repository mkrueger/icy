//! Theme loading from files and system configuration.
//!
//! Supports loading themes from:
//! - RON files (libcosmic format)
//! - System cosmic-config directories (on Pop!_OS/Linux)

use super::Theme;

#[cfg(feature = "serde")]
use super::Palette;

use std::path::Path;

/// Error type for theme loading.
#[derive(Debug)]
pub enum LoadError {
    /// File not found.
    NotFound(String),
    /// Failed to read file.
    ReadError(String),
    /// Failed to parse theme file.
    ParseError(String),
    /// Unsupported format.
    UnsupportedFormat(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::NotFound(path) => write!(f, "Theme file not found: {}", path),
            LoadError::ReadError(msg) => write!(f, "Failed to read theme: {}", msg),
            LoadError::ParseError(msg) => write!(f, "Failed to parse theme: {}", msg),
            LoadError::UnsupportedFormat(fmt) => write!(f, "Unsupported format: {}", fmt),
        }
    }
}

impl std::error::Error for LoadError {}

/// Load a theme from a file path.
///
/// Supports RON format (.ron extension).
///
/// # Example
///
/// ```ignore
/// use iced_core::theme2::load_theme_from_file;
///
/// let theme = load_theme_from_file("my-theme.ron")?;
/// ```
pub fn load_theme_from_file(path: impl AsRef<Path>) -> Result<Theme, LoadError> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(LoadError::NotFound(path.display().to_string()));
    }

    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match extension {
        "ron" => load_ron_theme(path),
        _ => Err(LoadError::UnsupportedFormat(extension.to_string())),
    }
}

/// Load a RON-format theme file.
#[cfg(feature = "serde")]
fn load_ron_theme(path: &Path) -> Result<Theme, LoadError> {
    let content = std::fs::read_to_string(path).map_err(|e| LoadError::ReadError(e.to_string()))?;

    // Try to parse as a full Theme first
    if let Ok(theme) = ron::from_str::<Theme>(&content) {
        return Ok(theme);
    }

    // Try to parse as just a Palette
    if let Ok(palette) = ron::from_str::<Palette>(&content) {
        // Determine if dark based on background luminance
        let is_dark = is_dark_color(palette.neutral_0);
        return Ok(Theme::from_palette(palette, is_dark));
    }

    Err(LoadError::ParseError(
        "Could not parse as Theme or Palette".to_string(),
    ))
}

#[cfg(not(feature = "serde"))]
fn load_ron_theme(_path: &Path) -> Result<Theme, LoadError> {
    Err(LoadError::UnsupportedFormat(
        "RON loading requires 'serde' feature".to_string(),
    ))
}

/// Try to load the system theme (on Pop!_OS/Linux with cosmic-config).
///
/// Falls back to light or dark theme based on preference.
///
/// # Arguments
/// * `prefer_dark` - Whether to prefer dark theme if system theme unavailable.
///
/// # Example
///
/// ```
/// use iced_core::theme2::load_system_theme;
///
/// let theme = load_system_theme(true); // Prefer dark if no system theme
/// ```
pub fn load_system_theme(prefer_dark: bool) -> Theme {
    // Try to load from cosmic-config directories
    if let Some(theme) = try_load_cosmic_theme(prefer_dark) {
        return theme;
    }

    // Fall back to defaults
    if prefer_dark {
        Theme::dark()
    } else {
        Theme::light()
    }
}

/// Try to load a theme from cosmic-config directories.
fn try_load_cosmic_theme(prefer_dark: bool) -> Option<Theme> {
    #[cfg(target_os = "linux")]
    {
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .ok()
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|home| format!("{}/.config", home))
            })
            .unwrap_or_else(|| "/etc".to_string());

        let theme_name = if prefer_dark {
            "com.system76.CosmicTheme.Dark"
        } else {
            "com.system76.CosmicTheme.Light"
        };

        let theme_path = format!("{}/cosmic/{}/v1/", config_dir, theme_name);

        // Check if the cosmic theme directory exists
        if Path::new(&theme_path).exists() {
            // Try to load the theme (implementation would parse cosmic config files)
            // For now, return None to fall back to defaults
            // TODO: Implement full cosmic-config parsing
            return None;
        }

        None
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = prefer_dark;
        None
    }
}

/// Determine if a color is dark based on relative luminance.
#[cfg(feature = "serde")]
fn is_dark_color(color: crate::Color) -> bool {
    // Calculate relative luminance using sRGB formula
    let luminance = 0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b;
    luminance < 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_themes() {
        let dark = Theme::dark();
        assert!(dark.is_dark);
        assert_eq!(dark.name, "Dark");

        let light = Theme::light();
        assert!(!light.is_dark);
        assert_eq!(light.name, "Light");
    }

    #[test]
    fn test_system_theme_fallback() {
        let theme = load_system_theme(true);
        assert!(theme.is_dark);

        let theme = load_system_theme(false);
        assert!(!theme.is_dark);
    }
}
