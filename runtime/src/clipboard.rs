//! Access the clipboard.
//!
//! # Example
//!
//! ```no_run
//! use iced::clipboard::{self, STANDARD, PRIMARY, Format};
//!
//! // Read text from the standard clipboard
//! let task = STANDARD.read_text();
//!
//! // Write text to the primary clipboard (X11/Wayland selection)
//! let task = PRIMARY.write_text("Hello".to_string());
//!
//! // Read an image
//! let task = STANDARD.read_image();
//!
//! // Use format constants
//! let task = STANDARD.read_format(Format::Png.formats());
//!
//! // Use the builder pattern for complex writes
//! let task = STANDARD.write()
//!     .html("<b>Hello</b>".to_string())
//!     .text("Hello".to_string())
//!     .finish();
//!
//! // Clear the clipboard
//! let task = STANDARD.clear();
//! ```

use crate::core::clipboard::{ClipboardData, Kind};
use crate::futures::futures::channel::oneshot;
use crate::task::{self, Task};
use std::path::PathBuf;

// Re-export Format from core
pub use crate::core::clipboard::Format;

// ============================================================================
// Error Types
// ============================================================================

/// An error that can occur during clipboard operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The clipboard is empty or doesn't contain the requested format.
    Empty,
    /// The requested format is not available.
    /// Contains the list of formats that are available.
    FormatNotAvailable {
        /// The formats that are available in the clipboard.
        available: Vec<String>,
    },
    /// Clipboard access was denied (e.g., window not focused on Wayland).
    AccessDenied,
    /// The clipboard is not available on this platform/configuration.
    Unavailable,
    /// Failed to decode the clipboard data.
    DecodingFailed,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Empty => write!(f, "clipboard is empty"),
            Error::FormatNotAvailable { available } => {
                write!(f, "format not available (available: {:?})", available)
            }
            Error::AccessDenied => write!(f, "clipboard access denied"),
            Error::Unavailable => write!(f, "clipboard unavailable"),
            Error::DecodingFailed => write!(f, "failed to decode clipboard data"),
        }
    }
}

impl std::error::Error for Error {}

// ============================================================================
// Clipboard Targets
// ============================================================================

/// The standard system clipboard.
pub const STANDARD: Target = Target(Kind::Standard);

/// The primary selection clipboard (X11/Wayland middle-click paste).
pub const PRIMARY: Target = Target(Kind::Primary);

/// A clipboard target.
///
/// Use [`STANDARD`] for the regular system clipboard, or [`PRIMARY`] for the
/// X11/Wayland primary selection (middle-click paste).
#[derive(Clone, Copy, Debug)]
pub struct Target(Kind);

impl Target {
    // ---- Text ----

    /// Read text from this clipboard.
    pub fn read_text(self) -> Task<Option<String>> {
        task::oneshot(|channel| {
            crate::Action::Clipboard(Action::ReadText {
                target: self.0,
                channel,
            })
        })
    }

    /// Write text to this clipboard.
    pub fn write_text<T>(self, contents: String) -> Task<T> {
        task::effect(crate::Action::Clipboard(Action::WriteText {
            target: self.0,
            contents,
        }))
    }

    // ---- Format ----

    /// Read data with the specified formats (first available wins).
    pub fn read_format(self, formats: &[&str]) -> Task<Option<ClipboardData>> {
        let formats: Vec<String> = formats.iter().map(|s| s.to_string()).collect();
        task::oneshot(|channel| {
            crate::Action::Clipboard(Action::Read {
                target: self.0,
                formats,
                channel,
            })
        })
    }

    /// Write data with the specified formats.
    pub fn write_format<T>(self, data: Vec<u8>, formats: &[&str]) -> Task<T> {
        let formats: Vec<String> = formats.iter().map(|s| s.to_string()).collect();
        task::effect(crate::Action::Clipboard(Action::Write {
            target: self.0,
            data,
            formats,
        }))
    }

    /// Write multiple formats at once.
    pub fn write_multi<T>(self, formats: Vec<(Vec<u8>, Vec<String>)>) -> Task<T> {
        task::effect(crate::Action::Clipboard(Action::WriteMulti {
            target: self.0,
            formats,
        }))
    }

    /// Get the available formats in this clipboard.
    pub fn available_formats(self) -> Task<Vec<String>> {
        task::oneshot(|channel| {
            crate::Action::Clipboard(Action::AvailableFormats {
                target: self.0,
                channel,
            })
        })
    }

    // ---- Image ----

    /// Read image data (PNG, JPEG, BMP, or GIF).
    pub fn read_image(self) -> Task<Option<ClipboardData>> {
        self.read_format(Format::Image.formats())
    }

    /// Write PNG image data.
    pub fn write_image<T>(self, png_data: Vec<u8>) -> Task<T> {
        self.write_format(png_data, Format::Png.formats())
    }

    // ---- HTML ----

    /// Read HTML content.
    pub fn read_html(self) -> Task<Option<String>> {
        self.read_format(Format::Html.formats())
            .map(|data| data.and_then(|d| d.into_text()))
    }

    /// Write HTML with optional plain text fallback.
    pub fn write_html<T>(self, html: String, alt_text: Option<String>) -> Task<T> {
        if let Some(alt) = alt_text {
            self.write_multi(vec![
                (
                    html.into_bytes(),
                    Format::Html.formats().iter().map(|s| s.to_string()).collect(),
                ),
                (
                    alt.into_bytes(),
                    Format::Text.formats().iter().map(|s| s.to_string()).collect(),
                ),
            ])
        } else {
            self.write_format(html.into_bytes(), Format::Html.formats())
        }
    }

    // ---- Rich Text ----

    /// Read rich text (RTF format).
    pub fn read_rich_text(self) -> Task<Option<String>> {
        self.read_format(Format::Rtf.formats())
            .map(|data| data.and_then(|d| d.into_text()))
    }

    /// Write rich text (RTF format) with optional plain text fallback.
    pub fn write_rich_text<T>(self, rtf: String, alt_text: Option<String>) -> Task<T> {
        if let Some(alt) = alt_text {
            self.write_multi(vec![
                (
                    rtf.into_bytes(),
                    Format::Rtf.formats().iter().map(|s| s.to_string()).collect(),
                ),
                (
                    alt.into_bytes(),
                    Format::Text.formats().iter().map(|s| s.to_string()).collect(),
                ),
            ])
        } else {
            self.write_format(rtf.into_bytes(), Format::Rtf.formats())
        }
    }

    // ---- Files ----

    /// Read file paths.
    pub fn read_files(self) -> Task<Option<Vec<PathBuf>>> {
        task::oneshot(|channel| {
            crate::Action::Clipboard(Action::ReadFiles {
                target: self.0,
                channel,
            })
        })
    }

    /// Write file paths.
    pub fn write_files<T>(self, paths: Vec<PathBuf>) -> Task<T> {
        task::effect(crate::Action::Clipboard(Action::WriteFiles {
            target: self.0,
            paths,
        }))
    }

    // ---- Clear ----

    /// Clear this clipboard.
    pub fn clear<T>(self) -> Task<T> {
        task::effect(crate::Action::Clipboard(Action::Clear {
            target: self.0,
        }))
    }

    // ---- Availability checks ----

    /// Check if the clipboard has any content.
    pub fn has_content(self) -> Task<bool> {
        self.available_formats().map(|formats| !formats.is_empty())
    }

    /// Check if any of the specified formats are available.
    ///
    /// Returns `true` if at least one of the requested formats is present.
    pub fn has_format(self, requested: Vec<String>) -> Task<bool> {
        self.available_formats().map(move |available| {
            requested.iter().any(|f| available.contains(f))
        })
    }

    // ---- Bulk read ----

    /// Read all available data for the specified formats.
    ///
    /// Unlike [`read_format`] which returns the first match, this returns
    /// data for all requested formats that are available.
    ///
    /// [`read_format`]: Self::read_format
    pub fn read_all_formats(self, formats: Vec<String>) -> Task<Vec<ClipboardData>> {
        task::oneshot(|channel| {
            crate::Action::Clipboard(Action::ReadAll {
                target: self.0,
                formats,
                channel,
            })
        })
    }

    // ---- Builder ----

    /// Create a write builder for this clipboard.
    ///
    /// The builder allows composing multiple formats in a fluent API:
    ///
    /// ```no_run
    /// use iced::clipboard::STANDARD;
    ///
    /// let task = STANDARD.write()
    ///     .html("<b>Hello</b>".to_string())
    ///     .text("Hello".to_string())
    ///     .finish();
    /// ```
    pub fn write(self) -> WriteBuilder {
        WriteBuilder {
            target: self.0,
            formats: Vec::new(),
        }
    }
}

// ============================================================================
// Write Builder
// ============================================================================

/// A builder for writing multiple formats to the clipboard.
///
/// Created by [`Target::write`].
#[derive(Debug, Clone)]
pub struct WriteBuilder {
    target: Kind,
    formats: Vec<(Vec<u8>, Vec<String>)>,
}

impl WriteBuilder {
    /// Add plain text to the clipboard.
    pub fn text(mut self, text: String) -> Self {
        self.formats.push((
            text.into_bytes(),
            Format::Text.formats().iter().map(|s| s.to_string()).collect(),
        ));
        self
    }

    /// Add HTML content to the clipboard.
    pub fn html(mut self, html: String) -> Self {
        self.formats.push((
            html.into_bytes(),
            Format::Html.formats().iter().map(|s| s.to_string()).collect(),
        ));
        self
    }

    /// Add RTF content to the clipboard.
    pub fn rtf(mut self, rtf: String) -> Self {
        self.formats.push((
            rtf.into_bytes(),
            Format::Rtf.formats().iter().map(|s| s.to_string()).collect(),
        ));
        self
    }

    /// Add PNG image data to the clipboard.
    pub fn png(mut self, data: Vec<u8>) -> Self {
        self.formats.push((
            data,
            Format::Png.formats().iter().map(|s| s.to_string()).collect(),
        ));
        self
    }

    /// Add JPEG image data to the clipboard.
    pub fn jpeg(mut self, data: Vec<u8>) -> Self {
        self.formats.push((
            data,
            Format::Jpeg.formats().iter().map(|s| s.to_string()).collect(),
        ));
        self
    }

    /// Add custom format data to the clipboard.
    pub fn custom(mut self, mime_type: impl Into<String>, data: Vec<u8>) -> Self {
        self.formats.push((data, vec![mime_type.into()]));
        self
    }

    /// Add data with multiple MIME types.
    pub fn with_formats(mut self, data: Vec<u8>, formats: Vec<String>) -> Self {
        self.formats.push((data, formats));
        self
    }

    /// Finish building and create the write task.
    pub fn finish<T>(self) -> Task<T> {
        if self.formats.is_empty() {
            return Task::none();
        }

        if self.formats.len() == 1 {
            let (data, formats) = self.formats.into_iter().next().unwrap();
            return task::effect(crate::Action::Clipboard(Action::Write {
                target: self.target,
                data,
                formats,
            }));
        }

        task::effect(crate::Action::Clipboard(Action::WriteMulti {
            target: self.target,
            formats: self.formats,
        }))
    }
}

// ============================================================================
// Action enum (used internally by the runtime)
// ============================================================================

/// A clipboard action to be performed by some [`Task`].
///
/// [`Task`]: crate::Task
#[derive(Debug)]
pub enum Action {
    /// Read text from the clipboard.
    ReadText {
        /// The clipboard target.
        target: Kind,
        /// The channel to send the read contents.
        channel: oneshot::Sender<Option<String>>,
    },

    /// Write text to the clipboard.
    WriteText {
        /// The clipboard target.
        target: Kind,
        /// The text to be written.
        contents: String,
    },

    /// Read data with specific formats from the clipboard.
    Read {
        /// The clipboard target.
        target: Kind,
        /// The formats to try, in order of preference.
        formats: Vec<String>,
        /// The channel to send the read data.
        channel: oneshot::Sender<Option<ClipboardData>>,
    },

    /// Write data with specific formats to the clipboard.
    Write {
        /// The clipboard target.
        target: Kind,
        /// The data to write.
        data: Vec<u8>,
        /// The formats to offer.
        formats: Vec<String>,
    },

    /// Write multiple formats at once.
    WriteMulti {
        /// The clipboard target.
        target: Kind,
        /// The formats to write (data, formats).
        formats: Vec<(Vec<u8>, Vec<String>)>,
    },

    /// Get available formats in the clipboard.
    AvailableFormats {
        /// The clipboard target.
        target: Kind,
        /// The channel to send the available formats.
        channel: oneshot::Sender<Vec<String>>,
    },

    /// Read files from the clipboard.
    ReadFiles {
        /// The clipboard target.
        target: Kind,
        /// The channel to send the read files.
        channel: oneshot::Sender<Option<Vec<PathBuf>>>,
    },

    /// Write files to the clipboard.
    WriteFiles {
        /// The clipboard target.
        target: Kind,
        /// The file paths to write.
        paths: Vec<PathBuf>,
    },

    /// Clear the clipboard.
    Clear {
        /// The clipboard target.
        target: Kind,
    },

    /// Read all available data for multiple formats.
    ReadAll {
        /// The clipboard target.
        target: Kind,
        /// The formats to read.
        formats: Vec<String>,
        /// The channel to send all read data.
        channel: oneshot::Sender<Vec<ClipboardData>>,
    },
}
