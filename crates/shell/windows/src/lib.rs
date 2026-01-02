//! Windows platform integration for icy_ui.
//!
//! This crate provides native Windows functionality that isn't available through winit,
//! including:
//!
//! - **Drag and Drop Initiation**: Starting drag operations from within the application
//!
//! # Usage
//!
//! ```rust,ignore
//! use icy_ui_windows::DragSource;
//!
//! // Create a drag source for a window
//! let drag_source = DragSource::new(hwnd)?;
//! drag_source.start_drag(data, mime_types, allowed_operations)?;
//! ```
//!
//! # Platform Support
//!
//! This crate only compiles on Windows. On other platforms, the types exist but
//! all operations are no-ops or return errors.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Only compile the actual implementations on Windows
#[cfg(all(target_os = "windows", feature = "dnd"))]
pub mod dnd;

#[cfg(all(target_os = "windows", feature = "dnd"))]
pub use dnd::{DragError, DragOperation, DragResult, DragSource};

// Provide stub implementations for non-Windows platforms
#[cfg(not(target_os = "windows"))]
mod stubs;

// Re-export stubs as the dnd module for non-Windows
#[cfg(all(not(target_os = "windows"), feature = "dnd"))]
pub mod dnd {
    //! Stub dnd module for non-Windows platforms.
    pub use super::stubs::{DragError, DragOperation, DragResult, DragSource};
}

#[cfg(all(not(target_os = "windows"), feature = "dnd"))]
pub use stubs::{DragError, DragOperation, DragResult, DragSource};
