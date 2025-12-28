//! Access the clipboard.
use crate::core::clipboard::{ClipboardData, Kind};
use crate::futures::futures::channel::oneshot;
use crate::task::{self, Task};
use std::path::PathBuf;

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

    /// Read data with specific MIME types from the clipboard.
    Read {
        /// The clipboard target.
        target: Kind,
        /// The MIME types to try, in order of preference.
        mime_types: Vec<String>,
        /// The channel to send the read data.
        channel: oneshot::Sender<Option<ClipboardData>>,
    },

    /// Write data with specific MIME types to the clipboard.
    Write {
        /// The clipboard target.
        target: Kind,
        /// The data to write.
        data: Vec<u8>,
        /// The MIME types to offer.
        mime_types: Vec<String>,
    },

    /// Write multiple formats at once.
    WriteMulti {
        /// The clipboard target.
        target: Kind,
        /// The formats to write (data, mime_types).
        formats: Vec<(Vec<u8>, Vec<String>)>,
    },

    /// Get available MIME types in the clipboard.
    AvailableMimeTypes {
        /// The clipboard target.
        target: Kind,
        /// The channel to send the available MIME types.
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
}

// ========== Text API ==========

/// Read the current text contents of the clipboard.
pub fn read() -> Task<Option<String>> {
    task::oneshot(|channel| {
        crate::Action::Clipboard(Action::ReadText {
            target: Kind::Standard,
            channel,
        })
    })
}

/// Read the current text contents of the primary clipboard.
pub fn read_primary() -> Task<Option<String>> {
    task::oneshot(|channel| {
        crate::Action::Clipboard(Action::ReadText {
            target: Kind::Primary,
            channel,
        })
    })
}

/// Write the given text contents to the clipboard.
pub fn write<T>(contents: String) -> Task<T> {
    task::effect(crate::Action::Clipboard(Action::WriteText {
        target: Kind::Standard,
        contents,
    }))
}

/// Write the given text contents to the primary clipboard.
pub fn write_primary<Message>(contents: String) -> Task<Message> {
    task::effect(crate::Action::Clipboard(Action::WriteText {
        target: Kind::Primary,
        contents,
    }))
}

// ========== Generic MIME Type API ==========

/// Read data from the clipboard with the specified MIME types.
///
/// The first available MIME type will be used.
pub fn read_mime(mime_types: Vec<String>) -> Task<Option<ClipboardData>> {
    task::oneshot(|channel| {
        crate::Action::Clipboard(Action::Read {
            target: Kind::Standard,
            mime_types,
            channel,
        })
    })
}

/// Read data from the primary clipboard with the specified MIME types.
pub fn read_mime_primary(mime_types: Vec<String>) -> Task<Option<ClipboardData>> {
    task::oneshot(|channel| {
        crate::Action::Clipboard(Action::Read {
            target: Kind::Primary,
            mime_types,
            channel,
        })
    })
}

/// Write data to the clipboard with the specified MIME types.
pub fn write_mime<T>(data: Vec<u8>, mime_types: Vec<String>) -> Task<T> {
    task::effect(crate::Action::Clipboard(Action::Write {
        target: Kind::Standard,
        data,
        mime_types,
    }))
}

/// Write data to the primary clipboard with the specified MIME types.
pub fn write_mime_primary<T>(data: Vec<u8>, mime_types: Vec<String>) -> Task<T> {
    task::effect(crate::Action::Clipboard(Action::Write {
        target: Kind::Primary,
        data,
        mime_types,
    }))
}

/// Write multiple formats to the clipboard at once.
pub fn write_multi<T>(formats: Vec<(Vec<u8>, Vec<String>)>) -> Task<T> {
    task::effect(crate::Action::Clipboard(Action::WriteMulti {
        target: Kind::Standard,
        formats,
    }))
}

/// Get the available MIME types in the clipboard.
pub fn available_formats() -> Task<Vec<String>> {
    task::oneshot(|channel| {
        crate::Action::Clipboard(Action::AvailableMimeTypes {
            target: Kind::Standard,
            channel,
        })
    })
}

/// Get the available MIME types in the primary clipboard.
pub fn available_formats_primary() -> Task<Vec<String>> {
    task::oneshot(|channel| {
        crate::Action::Clipboard(Action::AvailableMimeTypes {
            target: Kind::Primary,
            channel,
        })
    })
}

// ========== Image API ==========

/// Read image data from the clipboard (PNG preferred).
pub fn read_image() -> Task<Option<ClipboardData>> {
    read_mime(vec![
        "image/png".to_string(),
        "image/jpeg".to_string(),
        "image/bmp".to_string(),
        "image/gif".to_string(),
    ])
}

/// Write PNG image data to the clipboard.
pub fn write_image<T>(png_data: Vec<u8>) -> Task<T> {
    write_mime(png_data, vec!["image/png".to_string()])
}

// ========== HTML API ==========

/// Read HTML from the clipboard.
pub fn read_html() -> Task<Option<String>> {
    task::oneshot(|channel| {
        crate::Action::Clipboard(Action::Read {
            target: Kind::Standard,
            mime_types: vec!["text/html".to_string()],
            channel,
        })
    })
    .map(|data| data.and_then(|d| d.into_text()))
}

/// Write HTML to the clipboard with optional plain text fallback.
pub fn write_html<T>(html: String, alt_text: Option<String>) -> Task<T> {
    if let Some(alt) = alt_text {
        task::effect(crate::Action::Clipboard(Action::WriteMulti {
            target: Kind::Standard,
            formats: vec![
                (html.into_bytes(), vec!["text/html".to_string()]),
                (
                    alt.into_bytes(),
                    vec![
                        "text/plain;charset=utf-8".to_string(),
                        "text/plain".to_string(),
                    ],
                ),
            ],
        }))
    } else {
        write_mime(html.into_bytes(), vec!["text/html".to_string()])
    }
}

// ========== Files API ==========

/// Read file paths from the clipboard.
pub fn read_files() -> Task<Option<Vec<PathBuf>>> {
    task::oneshot(|channel| {
        crate::Action::Clipboard(Action::ReadFiles {
            target: Kind::Standard,
            channel,
        })
    })
}

/// Write file paths to the clipboard.
pub fn write_files<T>(paths: Vec<PathBuf>) -> Task<T> {
    task::effect(crate::Action::Clipboard(Action::WriteFiles {
        target: Kind::Standard,
        paths,
    }))
}

// ========== Clear API ==========

/// Clear the clipboard.
pub fn clear<T>() -> Task<T> {
    task::effect(crate::Action::Clipboard(Action::Clear {
        target: Kind::Standard,
    }))
}

/// Clear the primary clipboard.
pub fn clear_primary<T>() -> Task<T> {
    task::effect(crate::Action::Clipboard(Action::Clear {
        target: Kind::Primary,
    }))
}
