//! Access the clipboard.

use crate::core::clipboard::{ClipboardData, Kind};
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use winit::window::{Window, WindowId};

/// MIME types used for text clipboard operations.
const TEXT_MIME_TYPES: &[&str] = &[
    "text/plain;charset=utf-8",
    "text/plain",
    "UTF8_STRING",
    "STRING",
    "TEXT",
];

/// A buffer for short-term storage and transfer within and between
/// applications.
pub struct Clipboard {
    state: State,
}

#[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
enum State {
    Wayland {
        clipboard: smithay_clipboard::Clipboard,
    },
    #[cfg(feature = "x11")]
    X11 {
        clipboard: clipboard_rs::ClipboardContext,
    },
    Unavailable,
}

#[cfg(all(not(feature = "wayland"), feature = "x11", unix, not(target_os = "macos")))]
enum State {
    X11 {
        clipboard: clipboard_rs::ClipboardContext,
    },
    Unavailable,
}

#[cfg(any(windows, target_os = "macos"))]
enum State {
    Connected {
        clipboard: clipboard_rs::ClipboardContext,
    },
    Unavailable,
}

#[cfg(not(any(
    windows,
    target_os = "macos",
    all(unix, any(feature = "wayland", feature = "x11"))
)))]
enum State {
    Unavailable,
}

impl Clipboard {
    /// Creates a new [`Clipboard`] for the given window.
    pub fn connect(window: Arc<Window>) -> Clipboard {
        #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
        {
            use winit::raw_window_handle::HasDisplayHandle;

            if let Ok(display_handle) = window.display_handle() {
                use winit::raw_window_handle::RawDisplayHandle;

                if let RawDisplayHandle::Wayland(wayland) = display_handle.as_raw() {
                    // SAFETY: The display pointer is valid for the lifetime of the window
                    #[allow(unsafe_code)]
                    let clipboard = unsafe {
                        smithay_clipboard::Clipboard::new(wayland.display.as_ptr().cast())
                    };

                    return Clipboard {
                        state: State::Wayland { clipboard },
                    };
                }
            }

            // Fall back to X11 if available
            #[cfg(feature = "x11")]
            {
                use clipboard_rs::ClipboardContext;

                if let Ok(clipboard) = ClipboardContext::new() {
                    return Clipboard {
                        state: State::X11 { clipboard },
                    };
                }
            }

            Clipboard {
                state: State::Unavailable,
            }
        }

        #[cfg(all(not(feature = "wayland"), feature = "x11", unix, not(target_os = "macos")))]
        {
            use clipboard_rs::ClipboardContext;

            let _ = window;

            let state = match ClipboardContext::new() {
                Ok(clipboard) => State::X11 { clipboard },
                Err(_) => State::Unavailable,
            };

            Clipboard { state }
        }

        #[cfg(any(windows, target_os = "macos"))]
        {
            use clipboard_rs::ClipboardContext;

            let _ = window; // Not needed on these platforms

            let state = match ClipboardContext::new() {
                Ok(clipboard) => State::Connected { clipboard },
                Err(_) => State::Unavailable,
            };

            Clipboard { state }
        }

        #[cfg(not(any(
            windows,
            target_os = "macos",
            all(unix, any(feature = "wayland", feature = "x11"))
        )))]
        {
            let _ = window;
            Clipboard {
                state: State::Unavailable,
            }
        }
    }

    /// Creates a new [`Clipboard`] that isn't associated with a window.
    /// This clipboard will never contain a copied value.
    pub fn unconnected() -> Clipboard {
        Clipboard {
            state: State::Unavailable,
        }
    }

    /// Reads the current content of the [`Clipboard`] as text.
    pub fn read_text(&self, kind: Kind) -> Option<String> {
        self.read(kind, TEXT_MIME_TYPES)
            .and_then(|data| data.into_text())
    }

    /// Writes the given text contents to the [`Clipboard`].
    pub fn write_text(&mut self, kind: Kind, contents: String) {
        self.write(kind, Cow::Owned(contents.into_bytes()), TEXT_MIME_TYPES);
    }

    /// Read data with preferred MIME types (first match wins).
    pub fn read(&self, kind: Kind, mime_types: &[&str]) -> Option<ClipboardData> {
        match &self.state {
            #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
            State::Wayland { clipboard } => {
                let result = match kind {
                    Kind::Standard => clipboard.load(mime_types),
                    Kind::Primary => clipboard.load_primary(mime_types),
                };
                result.ok().map(|data| ClipboardData {
                    mime_type: data.mime_type,
                    data: data.data,
                })
            }

            #[cfg(all(feature = "x11", unix, not(target_os = "macos")))]
            State::X11 { clipboard } => {
                use clipboard_rs::Clipboard as _;

                // clipboard-rs doesn't support MIME type selection, so we try common formats
                for mime in mime_types {
                    if let Ok(data) = clipboard.get_buffer(mime) {
                        return Some(ClipboardData::new(*mime, data));
                    }
                }

                // Fall back to text if text MIME types were requested
                if mime_types.iter().any(|m| m.contains("text")) {
                    if let Ok(text) = clipboard.get_text() {
                        return Some(ClipboardData::new("text/plain", text.into_bytes()));
                    }
                }

                None
            }

            #[cfg(any(windows, target_os = "macos"))]
            State::Connected { clipboard } => {
                use clipboard_rs::Clipboard as _;

                for mime in mime_types {
                    if let Ok(data) = clipboard.get_buffer(mime) {
                        return Some(ClipboardData::new(*mime, data));
                    }
                }

                // Fall back to text if text MIME types were requested
                if mime_types.iter().any(|m| m.contains("text")) {
                    if let Ok(text) = clipboard.get_text() {
                        return Some(ClipboardData::new("text/plain", text.into_bytes()));
                    }
                }

                // Primary clipboard not supported on Windows/macOS
                if kind == Kind::Primary {
                    return None;
                }

                None
            }

            State::Unavailable => None,
        }
    }

    /// Write data with the given MIME types.
    pub fn write(&mut self, kind: Kind, data: Cow<'_, [u8]>, mime_types: &[&str]) {
        match &mut self.state {
            #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
            State::Wayland { clipboard } => match kind {
                Kind::Standard => clipboard.store(&data, mime_types),
                Kind::Primary => clipboard.store_primary(&data, mime_types),
            },

            #[cfg(all(feature = "x11", unix, not(target_os = "macos")))]
            State::X11 { clipboard } => {
                use clipboard_rs::Clipboard as _;

                // clipboard-rs stores with the first MIME type
                if let Some(mime) = mime_types.first() {
                    if let Err(e) = clipboard.set_buffer(mime, data.into_owned()) {
                        log::warn!("Failed to write to clipboard: {e}");
                    }
                }
            }

            #[cfg(any(windows, target_os = "macos"))]
            State::Connected { clipboard } => {
                use clipboard_rs::Clipboard as _;

                if kind == Kind::Primary {
                    return; // Primary clipboard not supported
                }

                if let Some(mime) = mime_types.first() {
                    if let Err(e) = clipboard.set_buffer(mime, data.into_owned()) {
                        log::warn!("Failed to write to clipboard: {e}");
                    }
                }
            }

            State::Unavailable => {}
        }
    }

    /// Write multiple formats at once.
    pub fn write_multi(&mut self, kind: Kind, formats: &[(Cow<'_, [u8]>, &[&str])]) {
        match &mut self.state {
            #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
            State::Wayland { clipboard } => {
                let formats: Vec<(&[u8], &[&str])> =
                    formats.iter().map(|(d, m)| (d.as_ref(), *m)).collect();

                match kind {
                    Kind::Standard => clipboard.store_multi(&formats),
                    Kind::Primary => clipboard.store_primary_multi(&formats),
                }
            }

            #[cfg(all(feature = "x11", unix, not(target_os = "macos")))]
            State::X11 { clipboard } => {
                use clipboard_rs::Clipboard as _;
                use clipboard_rs::ClipboardContent;

                // clipboard-rs supports setting multiple contents
                let contents: Vec<ClipboardContent> = formats
                    .iter()
                    .filter_map(|(data, mimes)| {
                        mimes.first().map(|mime| {
                            ClipboardContent::Other(mime.to_string(), data.to_vec())
                        })
                    })
                    .collect();

                if let Err(e) = clipboard.set(contents) {
                    log::warn!("Failed to write multiple formats to clipboard: {e}");
                }
            }

            #[cfg(any(windows, target_os = "macos"))]
            State::Connected { clipboard } => {
                use clipboard_rs::Clipboard as _;
                use clipboard_rs::ClipboardContent;

                if kind == Kind::Primary {
                    return;
                }

                let contents: Vec<ClipboardContent> = formats
                    .iter()
                    .filter_map(|(data, mimes)| {
                        mimes.first().map(|mime| {
                            ClipboardContent::Other(mime.to_string(), data.to_vec())
                        })
                    })
                    .collect();

                if let Err(e) = clipboard.set(contents) {
                    log::warn!("Failed to write multiple formats to clipboard: {e}");
                }
            }

            State::Unavailable => {}
        }
    }

    /// Get all available MIME types in the clipboard.
    pub fn available_mime_types(&self, kind: Kind) -> Vec<String> {
        match &self.state {
            #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
            State::Wayland { clipboard } => {
                let result = match kind {
                    Kind::Standard => clipboard.available_mime_types(),
                    Kind::Primary => clipboard.available_mime_types_primary(),
                };
                result.unwrap_or_default()
            }

            #[cfg(all(feature = "x11", unix, not(target_os = "macos")))]
            State::X11 { clipboard } => {
                use clipboard_rs::Clipboard as _;

                clipboard.available_formats().unwrap_or_default()
            }

            #[cfg(any(windows, target_os = "macos"))]
            State::Connected { clipboard } => {
                use clipboard_rs::Clipboard as _;

                if kind == Kind::Primary {
                    return Vec::new();
                }

                clipboard.available_formats().unwrap_or_default()
            }

            State::Unavailable => Vec::new(),
        }
    }

    /// Read file URIs from clipboard.
    pub fn read_files(&self, kind: Kind) -> Option<Vec<PathBuf>> {
        match &self.state {
            #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
            State::Wayland { clipboard } => {
                // Try to read file URIs
                let mime_types = &["text/uri-list", "x-special/gnome-copied-files"];
                let result = match kind {
                    Kind::Standard => clipboard.load(mime_types),
                    Kind::Primary => clipboard.load_primary(mime_types),
                };

                result.ok().and_then(|data| {
                    data.as_text().map(|text| parse_file_uri_list(text))
                })
            }

            #[cfg(all(feature = "x11", unix, not(target_os = "macos")))]
            State::X11 { clipboard } => {
                use clipboard_rs::Clipboard as _;

                clipboard.get_files().ok().map(|files| {
                    files.into_iter().map(PathBuf::from).collect()
                })
            }

            #[cfg(any(windows, target_os = "macos"))]
            State::Connected { clipboard } => {
                use clipboard_rs::Clipboard as _;

                if kind == Kind::Primary {
                    return None;
                }

                clipboard.get_files().ok().map(|files| {
                    files.into_iter().map(PathBuf::from).collect()
                })
            }

            State::Unavailable => None,
        }
    }

    /// Write file URIs to clipboard.
    pub fn write_files(&mut self, kind: Kind, paths: &[PathBuf]) {
        match &mut self.state {
            #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
            State::Wayland { clipboard } => {
                let uri_list = paths_to_uri_list(paths);
                let mime_types = &["text/uri-list"];

                match kind {
                    Kind::Standard => clipboard.store(uri_list.as_bytes(), mime_types),
                    Kind::Primary => clipboard.store_primary(uri_list.as_bytes(), mime_types),
                }
            }

            #[cfg(all(feature = "x11", unix, not(target_os = "macos")))]
            State::X11 { clipboard } => {
                use clipboard_rs::Clipboard as _;

                let files: Vec<String> = paths
                    .iter()
                    .filter_map(|p| p.to_str().map(String::from))
                    .collect();

                if let Err(e) = clipboard.set_files(files) {
                    log::warn!("Failed to write files to clipboard: {e}");
                }
            }

            #[cfg(any(windows, target_os = "macos"))]
            State::Connected { clipboard } => {
                use clipboard_rs::Clipboard as _;

                if kind == Kind::Primary {
                    return;
                }

                let files: Vec<String> = paths
                    .iter()
                    .filter_map(|p| p.to_str().map(String::from))
                    .collect();

                if let Err(e) = clipboard.set_files(files) {
                    log::warn!("Failed to write files to clipboard: {e}");
                }
            }

            State::Unavailable => {}
        }
    }

    /// Clear the clipboard.
    pub fn clear(&mut self, kind: Kind) {
        match &mut self.state {
            #[cfg(all(feature = "wayland", unix, not(target_os = "macos")))]
            State::Wayland { clipboard: _ } => {
                // smithay-clipboard doesn't have a clear method,
                // so we write empty data
                self.write(kind, Cow::Borrowed(&[]), &["text/plain"]);
            }

            #[cfg(all(feature = "x11", unix, not(target_os = "macos")))]
            State::X11 { clipboard } => {
                use clipboard_rs::Clipboard as _;

                if let Err(e) = clipboard.clear() {
                    log::warn!("Failed to clear clipboard: {e}");
                }
            }

            #[cfg(any(windows, target_os = "macos"))]
            State::Connected { clipboard } => {
                use clipboard_rs::Clipboard as _;

                if kind == Kind::Primary {
                    return;
                }

                if let Err(e) = clipboard.clear() {
                    log::warn!("Failed to clear clipboard: {e}");
                }
            }

            State::Unavailable => {}
        }
    }

    /// Returns the identifier of the window used to create the [`Clipboard`], if any.
    pub fn window_id(&self) -> Option<WindowId> {
        // The new clipboard backends don't require a window reference
        None
    }
}

/// Parse a text/uri-list into PathBufs
fn parse_file_uri_list(text: &str) -> Vec<PathBuf> {
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with("file://") {
                let path = &line[7..];
                // URL decode the path
                let decoded = percent_decode(path);
                Some(PathBuf::from(decoded))
            } else {
                None
            }
        })
        .collect()
}

/// Convert paths to a text/uri-list format
fn paths_to_uri_list(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .filter_map(|p| p.to_str())
        .map(|s| format!("file://{s}"))
        .collect::<Vec<_>>()
        .join("\r\n")
}

/// Simple percent decoding for file URIs
fn percent_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else {
            result.push(c);
        }
    }

    result
}

impl crate::core::Clipboard for Clipboard {
    fn read_text(&self, kind: Kind) -> Option<String> {
        self.read_text(kind)
    }

    fn write_text(&mut self, kind: Kind, contents: String) {
        self.write_text(kind, contents);
    }

    fn read(&self, kind: Kind, mime_types: &[&str]) -> Option<ClipboardData> {
        self.read(kind, mime_types)
    }

    fn write(&mut self, kind: Kind, data: Cow<'_, [u8]>, mime_types: &[&str]) {
        self.write(kind, data, mime_types);
    }

    fn write_multi(&mut self, kind: Kind, formats: &[(Cow<'_, [u8]>, &[&str])]) {
        self.write_multi(kind, formats);
    }

    fn available_mime_types(&self, kind: Kind) -> Vec<String> {
        self.available_mime_types(kind)
    }

    fn read_files(&self, kind: Kind) -> Option<Vec<PathBuf>> {
        self.read_files(kind)
    }

    fn write_files(&mut self, kind: Kind, paths: &[PathBuf]) {
        self.write_files(kind, paths);
    }

    fn clear(&mut self, kind: Kind) {
        self.clear(kind);
    }
}
