//! Drag and drop runtime support.
//!
//! This module provides tasks for initiating and managing drag and drop operations.
//!
//! # Example
//!
//! ```no_run
//! use iced::dnd::{self, DragData, DragIcon};
//!
//! // Start a drag with text data
//! let task = dnd::start_drag(
//!     DragData::from_text("Hello, world!"),
//!     None, // No custom icon
//! );
//!
//! // Start a drag with a custom icon
//! let task = dnd::start_drag(
//!     DragData::from_text("Dragged item"),
//!     Some(DragIcon::Buffer {
//!         width: 32,
//!         height: 32,
//!         data: vec![0; 32 * 32 * 4],
//!         hotspot_x: 0,
//!         hotspot_y: 0,
//!     }),
//! );
//! ```

use crate::core::dnd::{DndAction, DragData, DragIcon, DropResult, DropZone};
use crate::core::window::Id;
use crate::futures::futures::channel::oneshot;
use crate::task::{self, Task};

use std::borrow::Cow;

// ============================================================================
// Public API
// ============================================================================

/// Start a drag and drop operation.
///
/// This initiates a platform drag with the given data. The returned task
/// will complete when the drag finishes, either by dropping or being cancelled.
///
/// # Arguments
///
/// * `data` - The data to be dragged
/// * `icon` - Optional custom icon for the drag cursor
///
/// # Returns
///
/// A task that yields a [`DropResult`] when the drag completes.
pub fn start_drag(data: DragData, icon: Option<DragIcon>) -> Task<DropResult> {
    task::oneshot(|channel| {
        crate::Action::Dnd(Action::StartDrag {
            data,
            icon,
            allowed_actions: DndAction::Copy,
            channel,
        })
    })
}

/// Start a drag and drop operation with specific allowed actions.
///
/// This is like [`start_drag`] but allows specifying which actions
/// (Copy, Move, Link) the drag source supports.
pub fn start_drag_with_actions(
    data: DragData,
    icon: Option<DragIcon>,
    allowed_actions: DndAction,
) -> Task<DropResult> {
    task::oneshot(|channel| {
        crate::Action::Dnd(Action::StartDrag {
            data,
            icon,
            allowed_actions,
            channel,
        })
    })
}

/// Register drop zones for a window.
///
/// This tells the DnD system which areas of the window can accept drops.
/// Drop zones are used to provide feedback during drag operations.
pub fn set_drop_zones<T>(window: Id, zones: Vec<DropZone>) -> Task<T> {
    task::effect(crate::Action::Dnd(Action::SetDropZones { window, zones }))
}

/// Accept a drag with the given MIME types.
///
/// Call this in response to a `DragEntered` event to indicate which
/// MIME types your drop target can accept.
pub fn accept_drag<T>(
    window: Id,
    mime_types: Vec<Cow<'static, str>>,
    action: DndAction,
) -> Task<T> {
    task::effect(crate::Action::Dnd(Action::AcceptDrag {
        window,
        mime_types,
        action,
    }))
}

/// Reject the current drag.
///
/// Call this in response to a `DragEntered` or `DragMoved` event to indicate
/// the current position is not a valid drop target.
pub fn reject_drag<T>(window: Id) -> Task<T> {
    task::effect(crate::Action::Dnd(Action::RejectDrag { window }))
}

/// Request the drag data for a specific MIME type.
///
/// Call this when you want to peek at the data during drag (before drop).
/// Note: Some platforms may not support this.
pub fn request_data(window: Id, mime_type: String) -> Task<Option<Vec<u8>>> {
    task::oneshot(|channel| {
        crate::Action::Dnd(Action::RequestData {
            window,
            mime_type,
            channel,
        })
    })
}

// ============================================================================
// Action enum (used internally by the runtime)
// ============================================================================

/// A drag and drop action to be performed by some [`Task`].
#[derive(Debug)]
pub enum Action {
    /// Start a drag operation.
    StartDrag {
        /// The data to drag.
        data: DragData,
        /// Optional custom icon.
        icon: Option<DragIcon>,
        /// Allowed actions for this drag.
        allowed_actions: DndAction,
        /// Channel to receive the result.
        channel: oneshot::Sender<DropResult>,
    },

    /// Set drop zones for a window.
    SetDropZones {
        /// The window to set drop zones for.
        window: Id,
        /// The drop zones.
        zones: Vec<DropZone>,
    },

    /// Accept a drag at the current position.
    AcceptDrag {
        /// The window accepting the drag.
        window: Id,
        /// MIME types we accept.
        mime_types: Vec<Cow<'static, str>>,
        /// Action we prefer.
        action: DndAction,
    },

    /// Reject the current drag.
    RejectDrag {
        /// The window rejecting the drag.
        window: Id,
    },

    /// Request drag data for a MIME type.
    RequestData {
        /// The window requesting data.
        window: Id,
        /// The MIME type to request.
        mime_type: String,
        /// Channel to receive the data.
        channel: oneshot::Sender<Option<Vec<u8>>>,
    },
}
