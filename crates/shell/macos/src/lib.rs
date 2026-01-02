//! macOS platform integration for icy_ui.
//!
//! This crate provides native macOS functionality that isn't available through winit,
//! including:
//!
//! - **URL Handler**: Handling custom URL schemes (`myapp://...`) via Apple Events
//! - **Drag and Drop Initiation**: Starting drag operations from within the application
//!
//! # Usage
//!
//! ```rust,ignore
//! use icy_ui_macos::{UrlHandler, DragSource};
//!
//! // Set up URL handler (call once at app startup)
//! let url_receiver = UrlHandler::install();
//!
//! // Create a drag source for a window
//! let drag_source = DragSource::new(ns_view_ptr);
//! drag_source.start_drag(data, formats)?;
//! ```
//!
//! # Platform Support
//!
//! This crate only compiles on macOS. On other platforms, the types exist but
//! all operations are no-ops or return errors.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Only compile the actual implementations on macOS
#[cfg(all(target_os = "macos", feature = "url-handler"))]
pub mod url_handler;

#[cfg(all(target_os = "macos", feature = "dnd"))]
pub mod dnd;

#[cfg(all(target_os = "macos", feature = "url-handler"))]
pub use url_handler::UrlHandler;

#[cfg(all(target_os = "macos", feature = "dnd"))]
pub use dnd::{DragError, DragOperation, DragResult, DragSource};

// Provide stub implementations for non-macOS platforms
#[cfg(not(target_os = "macos"))]
mod stubs;

// Re-export stubs as the dnd module for non-macOS
#[cfg(all(not(target_os = "macos"), feature = "dnd"))]
pub mod dnd {
    //! Stub dnd module for non-macOS platforms.
    pub use super::stubs::{DragError, DragOperation, DragResult, DragSource};
}

#[cfg(all(not(target_os = "macos"), feature = "url-handler"))]
pub use stubs::UrlHandler;

#[cfg(all(not(target_os = "macos"), feature = "dnd"))]
pub use stubs::{DragError, DragOperation, DragResult, DragSource};

/// macOS platform-specific events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MacOsEvent {
    /// A URL was received via a custom URL scheme handler.
    ReceivedUrl(String),
}
