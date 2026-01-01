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
//! drag_source.start_drag(data, mime_types)?;
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

#[cfg(feature = "url-handler")]
pub mod url_handler;

#[cfg(feature = "dnd")]
pub mod dnd;

#[cfg(feature = "url-handler")]
pub use url_handler::UrlHandler;

#[cfg(feature = "dnd")]
pub use dnd::{DragError, DragSource};

/// macOS platform-specific events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MacOsEvent {
    /// A URL was received via a custom URL scheme handler.
    ReceivedUrl(String),
}
