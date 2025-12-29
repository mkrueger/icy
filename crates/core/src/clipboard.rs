//! Access the clipboard.
use std::borrow::Cow;
use std::path::PathBuf;

// ============================================================================
// Format Constants
// ============================================================================

/// Common clipboard format identifiers.
///
/// These provide platform-appropriate format strings for common content types.
/// The actual format strings vary by platform:
///
/// - **Linux/X11/Wayland**: MIME types (e.g., `text/plain`, `text/html`, `image/png`)
/// - **macOS**: UTI strings (e.g., `public.utf8-plain-text`, `public.rtf`, `public.html`)
/// - **Windows**: Registered format names (e.g., `CF_UNICODETEXT`, `Rich Text Format`, `HTML Format`)
///
/// Using these constants abstracts away platform differences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Plain text (UTF-8).
    Text,
    /// Rich Text Format (RTF).
    Rtf,
    /// HTML content.
    Html,
    /// PNG image.
    Png,
    /// JPEG image.
    Jpeg,
    /// Any image format (PNG, JPEG, BMP, GIF, TIFF).
    Image,
    /// File paths/URIs.
    Files,
}

impl Format {
    /// Get the format strings for this format.
    ///
    /// Returns a slice of format identifiers that the clipboard backend
    /// will understand. The strings are platform-appropriate.
    #[cfg(target_os = "macos")]
    pub const fn formats(self) -> &'static [&'static str] {
        // macOS uses UTI (Uniform Type Identifier) strings
        match self {
            Format::Text => &["public.utf8-plain-text", "public.plain-text"],
            Format::Rtf => &["public.rtf"],
            Format::Html => &["public.html"],
            Format::Png => &["public.png"],
            Format::Jpeg => &["public.jpeg"],
            Format::Image => &["public.png", "public.tiff", "public.jpeg"],
            Format::Files => &["public.file-url"],
        }
    }

    /// Get the format strings for this format.
    ///
    /// Returns a slice of format identifiers that the clipboard backend
    /// will understand. The strings are platform-appropriate.
    #[cfg(target_os = "windows")]
    pub const fn formats(self) -> &'static [&'static str] {
        // Windows uses registered format names
        match self {
            Format::Text => &["CF_UNICODETEXT"],
            Format::Rtf => &["Rich Text Format"],
            Format::Html => &["HTML Format"],
            Format::Png => &["PNG"],
            Format::Jpeg => &["JFIF"],
            Format::Image => &["PNG", "CF_DIBV5"],
            Format::Files => &["CF_HDROP"],
        }
    }

    /// Get the format strings for this format.
    ///
    /// Returns a slice of format identifiers that the clipboard backend
    /// will understand. The strings are platform-appropriate.
    #[cfg(all(unix, not(target_os = "macos")))]
    pub const fn formats(self) -> &'static [&'static str] {
        // Linux/X11/Wayland use MIME types and X11 atom names
        match self {
            Format::Text => &[
                "text/plain;charset=utf-8",
                "text/plain",
                "UTF8_STRING",
                "STRING",
            ],
            Format::Rtf => &["text/rtf", "application/rtf"],
            Format::Html => &["text/html"],
            Format::Png => &["image/png"],
            Format::Jpeg => &["image/jpeg"],
            Format::Image => &["image/png", "image/jpeg", "image/bmp", "image/gif"],
            Format::Files => &["text/uri-list", "x-special/gnome-copied-files"],
        }
    }

    /// Get the format strings for this format.
    ///
    /// Fallback for other platforms.
    #[cfg(not(any(target_os = "macos", target_os = "windows", unix)))]
    pub const fn formats(self) -> &'static [&'static str] {
        match self {
            Format::Text => &["text/plain"],
            Format::Rtf => &["text/rtf"],
            Format::Html => &["text/html"],
            Format::Png => &["image/png"],
            Format::Jpeg => &["image/jpeg"],
            Format::Image => &["image/png", "image/jpeg"],
            Format::Files => &["text/uri-list"],
        }
    }

    /// Get the primary format string for this format (for writing).
    #[cfg(target_os = "macos")]
    pub const fn primary(self) -> &'static str {
        match self {
            Format::Text => "public.utf8-plain-text",
            Format::Rtf => "public.rtf",
            Format::Html => "public.html",
            Format::Png => "public.png",
            Format::Jpeg => "public.jpeg",
            Format::Image => "public.png",
            Format::Files => "public.file-url",
        }
    }

    /// Get the primary format string for this format (for writing).
    #[cfg(target_os = "windows")]
    pub const fn primary(self) -> &'static str {
        match self {
            Format::Text => "CF_UNICODETEXT",
            Format::Rtf => "Rich Text Format",
            Format::Html => "HTML Format",
            Format::Png => "PNG",
            Format::Jpeg => "JFIF",
            Format::Image => "PNG",
            Format::Files => "CF_HDROP",
        }
    }

    /// Get the primary format string for this format (for writing).
    #[cfg(all(unix, not(target_os = "macos")))]
    pub const fn primary(self) -> &'static str {
        match self {
            Format::Text => "text/plain;charset=utf-8",
            Format::Rtf => "text/rtf",
            Format::Html => "text/html",
            Format::Png => "image/png",
            Format::Jpeg => "image/jpeg",
            Format::Image => "image/png",
            Format::Files => "text/uri-list",
        }
    }

    /// Get the primary format string for this format (for writing).
    #[cfg(not(any(target_os = "macos", target_os = "windows", unix)))]
    pub const fn primary(self) -> &'static str {
        match self {
            Format::Text => "text/plain",
            Format::Rtf => "text/rtf",
            Format::Html => "text/html",
            Format::Png => "image/png",
            Format::Jpeg => "image/jpeg",
            Format::Image => "image/png",
            Format::Files => "text/uri-list",
        }
    }
}

/// The kind of [`Clipboard`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The standard clipboard.
    Standard,
    /// The primary clipboard.
    ///
    /// Normally only present in X11 and Wayland.
    Primary,
}

/// Data retrieved from the clipboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardData {
    /// The MIME type of the data.
    pub mime_type: String,
    /// The raw bytes.
    pub data: Vec<u8>,
}

impl ClipboardData {
    /// Create new clipboard data.
    pub fn new(mime_type: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            mime_type: mime_type.into(),
            data,
        }
    }

    /// Try to interpret the data as UTF-8 text.
    pub fn as_text(&self) -> Option<&str> {
        std::str::from_utf8(&self.data).ok()
    }

    /// Convert the data to a String if it's valid UTF-8.
    pub fn into_text(self) -> Option<String> {
        String::from_utf8(self.data).ok()
    }
}

/// A buffer for short-term storage and transfer within and between
/// applications.
pub trait Clipboard {
    // ========== Text (convenience) ==========

    /// Reads the current content of the [`Clipboard`] as text.
    fn read_text(&self, kind: Kind) -> Option<String>;

    /// Writes the given text contents to the [`Clipboard`].
    fn write_text(&mut self, kind: Kind, contents: String);

    // ========== Generic MIME-type API ==========

    /// Read data with preferred MIME types (first match wins).
    ///
    /// The first available MIME type from `mime_types` will be used.
    /// Returns the data along with the actual MIME type used.
    fn read(&self, kind: Kind, mime_types: &[&str]) -> Option<ClipboardData>;

    /// Write data with the given MIME types.
    ///
    /// The data will be offered to other applications with all the specified
    /// MIME types.
    fn write(&mut self, kind: Kind, data: Cow<'_, [u8]>, mime_types: &[&str]);

    /// Write multiple formats at once (e.g., text + HTML + image).
    ///
    /// Each entry contains the data and a list of MIME types for that data.
    fn write_multi(&mut self, kind: Kind, formats: &[(Cow<'_, [u8]>, &[&str])]);

    /// Get all available MIME types in the clipboard.
    fn available_mime_types(&self, kind: Kind) -> Vec<String>;

    /// Check if a specific MIME type is available.
    fn has_mime_type(&self, kind: Kind, mime_type: &str) -> bool {
        self.available_mime_types(kind)
            .iter()
            .any(|m| m == mime_type)
    }

    // ========== Image (convenience) ==========

    /// Read image data (PNG preferred).
    fn read_image(&self, kind: Kind) -> Option<ClipboardData> {
        self.read(kind, Format::Image.formats())
    }

    /// Write PNG image data.
    fn write_image(&mut self, kind: Kind, png_data: Cow<'_, [u8]>) {
        self.write(kind, png_data, Format::Png.formats());
    }

    // ========== HTML (convenience) ==========

    /// Read HTML from clipboard.
    fn read_html(&self, kind: Kind) -> Option<String> {
        self.read(kind, Format::Html.formats())
            .and_then(|d| d.into_text())
    }

    /// Write HTML with optional plain text fallback.
    fn write_html(&mut self, kind: Kind, html: &str, alt_text: Option<&str>) {
        if let Some(alt) = alt_text {
            self.write_multi(
                kind,
                &[
                    (Cow::Borrowed(html.as_bytes()), Format::Html.formats()),
                    (Cow::Borrowed(alt.as_bytes()), Format::Text.formats()),
                ],
            );
        } else {
            self.write(kind, Cow::Borrowed(html.as_bytes()), Format::Html.formats());
        }
    }

    // ========== Files (convenience) ==========

    /// Read file URIs from clipboard.
    fn read_files(&self, kind: Kind) -> Option<Vec<PathBuf>>;

    /// Write file URIs to clipboard.
    fn write_files(&mut self, kind: Kind, paths: &[PathBuf]);

    // ========== Clear ==========

    /// Clear the clipboard.
    fn clear(&mut self, kind: Kind);
}

/// A null implementation of the [`Clipboard`] trait.
#[derive(Debug, Clone, Copy)]
pub struct Null;

impl Clipboard for Null {
    fn read_text(&self, _kind: Kind) -> Option<String> {
        None
    }

    fn write_text(&mut self, _kind: Kind, _contents: String) {}

    fn read(&self, _kind: Kind, _mime_types: &[&str]) -> Option<ClipboardData> {
        None
    }

    fn write(&mut self, _kind: Kind, _data: Cow<'_, [u8]>, _mime_types: &[&str]) {}

    fn write_multi(&mut self, _kind: Kind, _formats: &[(Cow<'_, [u8]>, &[&str])]) {}

    fn available_mime_types(&self, _kind: Kind) -> Vec<String> {
        Vec::new()
    }

    fn read_files(&self, _kind: Kind) -> Option<Vec<PathBuf>> {
        None
    }

    fn write_files(&mut self, _kind: Kind, _paths: &[PathBuf]) {}

    fn clear(&mut self, _kind: Kind) {}
}
