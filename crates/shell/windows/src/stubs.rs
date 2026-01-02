//! Stub implementations for non-Windows platforms.
//!
//! These provide the same API as the Windows implementations but return errors
//! or no-ops, allowing the crate to compile on all platforms.

use std::ptr::NonNull;

/// Errors that can occur during drag operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DragError {
    /// Failed to initialize COM.
    ComInitFailed,
    /// The provided window handle is invalid.
    InvalidWindow,
    /// Failed to create data object.
    DataObjectError,
    /// The drag operation is not supported on this platform.
    NotSupported,
    /// Windows API error.
    WindowsError(String),
}

impl std::fmt::Display for DragError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DragError::ComInitFailed => write!(f, "Failed to initialize COM"),
            DragError::InvalidWindow => write!(f, "Invalid window handle"),
            DragError::DataObjectError => write!(f, "Failed to create data object"),
            DragError::NotSupported => write!(f, "Drag operation not supported on this platform"),
            DragError::WindowsError(msg) => write!(f, "Windows error: {}", msg),
        }
    }
}

impl std::error::Error for DragError {}

/// Allowed drag operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragOperation {
    bits: u32,
}

impl DragOperation {
    /// No operation allowed.
    pub const NONE: Self = Self { bits: 0 };
    /// Copy the dragged data.
    pub const COPY: Self = Self { bits: 1 };
    /// Move the dragged data.
    pub const MOVE: Self = Self { bits: 2 };
    /// Link to the dragged data.
    pub const LINK: Self = Self { bits: 4 };
    /// All operations allowed.
    pub const ALL: Self = Self { bits: 1 | 2 | 4 };
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
    /// The data was moved.
    Moved,
    /// The data was linked.
    Linked,
}

/// Stub drag source for non-Windows platforms.
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
    /// Always returns `DragError::NotSupported` on non-Windows platforms.
    pub fn new(_hwnd: NonNull<std::ffi::c_void>) -> Result<Self, DragError> {
        Err(DragError::NotSupported)
    }

    /// Start a drag operation (stub - always returns NotSupported).
    ///
    /// # Errors
    ///
    /// Always returns `DragError::NotSupported` on non-Windows platforms.
    pub fn start_drag(
        &self,
        _data: &[u8],
        _mime_type: &str,
        _allowed_operations: DragOperation,
    ) -> Result<DragResult, DragError> {
        Err(DragError::NotSupported)
    }
}
