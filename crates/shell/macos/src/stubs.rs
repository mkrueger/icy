//! Stub implementations for non-macOS platforms.
//!
//! These provide the same API as the macOS implementations but return errors
//! or no-ops, allowing the crate to compile on all platforms.

#![allow(unsafe_code)]

use std::ptr::NonNull;
use std::sync::mpsc::Receiver;

use icy_ui_core::menu::{AppMenu, ContextMenuItem, MenuId, MenuNode};

/// Errors that can occur during drag operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DragError {
    /// Not running on the main thread (required for AppKit operations).
    NotMainThread,
    /// No current event available (drag must be initiated during mouse event processing).
    NoCurrentEvent,
    /// The provided view pointer is invalid.
    InvalidView,
    /// Failed to create pasteboard item.
    PasteboardError,
    /// The drag operation is not supported on this platform.
    NotSupported,
}

impl std::fmt::Display for DragError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DragError::NotMainThread => write!(f, "Not on main thread"),
            DragError::NoCurrentEvent => {
                write!(f, "No current event (call during mouse event processing)")
            }
            DragError::InvalidView => write!(f, "Invalid NSView pointer"),
            DragError::PasteboardError => write!(f, "Failed to create pasteboard item"),
            DragError::NotSupported => write!(f, "Drag operation not supported on this platform"),
        }
    }
}

impl std::error::Error for DragError {}

/// Allowed drag operations (stub for non-macOS).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragOperation {
    bits: usize,
}

impl DragOperation {
    /// No operation allowed.
    pub const NONE: Self = Self { bits: 0 };
    /// Copy the dragged data.
    pub const COPY: Self = Self { bits: 1 };
    /// Link to the dragged data.
    pub const LINK: Self = Self { bits: 2 };
    /// Move the dragged data.
    pub const MOVE: Self = Self { bits: 16 };
    /// All operations allowed.
    pub const ALL: Self = Self {
        bits: 1 | 2 | 16, // Copy | Link | Move
    };
}

impl std::ops::BitOr for DragOperation {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits | rhs.bits,
        }
    }
}

/// Result of a completed drag operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragResult {
    /// The drag was cancelled.
    Cancelled,
    /// The data was copied.
    Copied,
    /// The data was linked.
    Linked,
    /// The data was moved.
    Moved,
}

/// Stub drag source for non-macOS platforms.
///
/// All operations return `DragError::NotSupported`.
pub struct DragSource {
    _private: (),
}

impl DragSource {
    /// Create a new drag source (stub - always returns NotSupported).
    ///
    /// # Errors
    ///
    /// Always returns `DragError::NotSupported` on non-macOS platforms.
    pub fn new(_ns_view: NonNull<std::ffi::c_void>) -> Result<Self, DragError> {
        Err(DragError::NotSupported)
    }

    /// Start a drag operation (stub - always returns NotSupported).
    ///
    /// # Errors
    ///
    /// Always returns `DragError::NotSupported` on non-macOS platforms.
    pub fn start_drag(
        &self,
        _data: &[u8],
        _format: &str,
        _allowed_operations: DragOperation,
    ) -> Result<&Receiver<DragResult>, DragError> {
        Err(DragError::NotSupported)
    }

    /// Try to receive the drag result (stub - always returns None).
    pub fn try_recv_result(&self) -> Option<DragResult> {
        None
    }
}

/// Stub URL handler for non-macOS platforms.
///
/// Returns an empty receiver that will never receive any URLs.
pub struct UrlHandler {
    receiver: Receiver<String>,
}

impl UrlHandler {
    /// Install the URL handler (stub - returns handler with empty receiver).
    #[must_use]
    pub fn install() -> Self {
        let (_sender, receiver) = std::sync::mpsc::channel();
        UrlHandler { receiver }
    }

    /// Try to receive a URL without blocking (stub - always returns None).
    pub fn try_recv(&self) -> Option<String> {
        None
    }

    /// Get a reference to the internal receiver for custom polling.
    pub fn receiver(&self) -> &Receiver<String> {
        &self.receiver
    }

    /// Consume the handler and return the receiver.
    pub fn into_receiver(self) -> Receiver<String> {
        self.receiver
    }
}

/// Errors that can occur while installing or updating menus (stub for non-macOS).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuError {
    /// Menu operations are not supported on this platform.
    NotSupported,
}

impl std::fmt::Display for MenuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuError::NotSupported => write!(f, "Menu operations not supported on this platform"),
        }
    }
}

impl std::error::Error for MenuError {}

/// Stub macOS main menu for non-macOS platforms.
pub struct MacMenu {
    _private: (),
}

impl MacMenu {
    /// Create a new menu (stub - always returns NotSupported).
    pub fn new() -> Result<Self, MenuError> {
        Err(MenuError::NotSupported)
    }

    /// Sync the menu (stub - always returns NotSupported).
    pub fn sync<Message>(&mut self, _menu: &AppMenu<Message>) -> Result<(), MenuError> {
        Err(MenuError::NotSupported)
    }

    /// Try to receive an activated menu id (stub - always returns None).
    pub fn try_recv(&self) -> Option<MenuId> {
        None
    }
}

/// Stub macOS context menu for non-macOS platforms.
pub struct MacContextMenu {
    _private: (),
}

impl MacContextMenu {
    /// Create a new context menu (stub - always returns NotSupported).
    pub fn new() -> Result<Self, MenuError> {
        Err(MenuError::NotSupported)
    }

    /// Show a context menu (stub - always returns NotSupported).
    ///
    /// # Safety
    /// Matches the macOS API; does nothing on non-macOS.
    pub unsafe fn show<Message>(
        &self,
        _nodes: &[MenuNode<Message>],
        _view_ptr: *mut std::ffi::c_void,
        _x: f64,
        _y: f64,
    ) -> Result<(), MenuError> {
        Err(MenuError::NotSupported)
    }

    /// Try to receive an activated menu id (stub - always returns None).
    pub fn try_recv(&self) -> Option<MenuId> {
        None
    }

    /// Show a context menu from items (stub - always returns NotSupported).
    ///
    /// # Safety
    /// Matches the macOS API; does nothing on non-macOS.
    pub unsafe fn show_items(
        &self,
        _items: &[ContextMenuItem],
        _view_ptr: *mut std::ffi::c_void,
        _x: f64,
        _y: f64,
    ) -> Result<(), MenuError> {
        Err(MenuError::NotSupported)
    }
}
