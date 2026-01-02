//! Drag and Drop support.
//!
//! This module provides types and functionality for drag and drop operations,
//! supporting both internal (widget-to-widget) and external (cross-application)
//! drag and drop via OS-level DnD protocols.

use std::borrow::Cow;

use crate::clipboard::Format;

/// The action to perform when a drag is dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DndAction {
    /// No action.
    #[default]
    None,
    /// Copy the data.
    Copy,
    /// Move the data.
    Move,
    /// Create a link to the data.
    Link,
    /// Ask the user what action to perform.
    Ask,
}

impl DndAction {
    /// Returns `true` if this is `None`.
    pub fn is_none(&self) -> bool {
        matches!(self, DndAction::None)
    }
}

/// Data for a drag and drop operation.
///
/// Contains the serialized payload and the formats it's available in.
#[derive(Debug, Clone)]
pub struct DragData {
    /// The serialized data bytes.
    pub data: Vec<u8>,
    /// The formats this data is available in (e.g., "text/plain", "text/uri-list").
    pub formats: Vec<Cow<'static, str>>,
}

impl DragData {
    /// Create new drag data with the given bytes and formats.
    pub fn new(data: impl Into<Vec<u8>>, formats: Vec<Cow<'static, str>>) -> Self {
        Self {
            data: data.into(),
            formats,
        }
    }

    /// Create drag data from a text string.
    ///
    /// Automatically sets appropriate text formats using platform-appropriate
    /// format strings from [`clipboard::Format`].
    pub fn from_text(text: impl AsRef<str>) -> Self {
        Self {
            data: text.as_ref().as_bytes().to_vec(),
            formats: Format::Text
                .formats()
                .iter()
                .map(|s| Cow::Borrowed(*s))
                .collect(),
        }
    }

    /// Create drag data from file paths.
    ///
    /// Paths are encoded as a newline-separated URI list using platform-appropriate
    /// format strings from [`clipboard::Format`].
    pub fn from_paths(paths: &[std::path::PathBuf]) -> Self {
        let uri_list: String = paths
            .iter()
            .filter_map(|p| p.to_str())
            .map(|s| format!("file://{}", s))
            .collect::<Vec<_>>()
            .join("\r\n");

        Self {
            data: uri_list.into_bytes(),
            formats: Format::Files
                .formats()
                .iter()
                .map(|s| Cow::Borrowed(*s))
                .collect(),
        }
    }
}

/// The result of a completed drag operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropResult {
    /// The drag was completed with the given action.
    Dropped(DndAction),
    /// The drag was cancelled by the user or system.
    Cancelled,
}

impl DropResult {
    /// Returns `true` if the drag was successfully dropped.
    pub fn is_dropped(&self) -> bool {
        matches!(self, DropResult::Dropped(_))
    }

    /// Returns `true` if the drag was cancelled.
    pub fn is_cancelled(&self) -> bool {
        matches!(self, DropResult::Cancelled)
    }

    /// Returns the action if the drag was dropped, or `None` if cancelled.
    pub fn action(&self) -> Option<DndAction> {
        match self {
            DropResult::Dropped(action) => Some(*action),
            DropResult::Cancelled => None,
        }
    }
}

/// An icon to display during a drag operation.
#[derive(Debug, Clone)]
pub enum DragIcon {
    /// Use a widget's rendered content as the icon.
    ///
    /// The widget will be rendered to a buffer and used as the drag icon.
    /// This is handled by the shell layer.
    Widget,

    /// Use pixel data as the icon.
    ///
    /// The data should be in ARGB8888 format (pre-multiplied alpha).
    Buffer {
        /// Width of the icon in pixels.
        width: u32,
        /// Height of the icon in pixels.
        height: u32,
        /// The pixel data (ARGB8888, pre-multiplied).
        data: Vec<u8>,
        /// The hotspot X offset (where the cursor points within the icon).
        hotspot_x: i32,
        /// The hotspot Y offset (where the cursor points within the icon).
        hotspot_y: i32,
    },
}

/// A rectangle defining a drop zone within a surface.
#[derive(Debug, Clone, Default)]
pub struct DropZone {
    /// Unique identifier for this drop zone.
    pub id: u128,
    /// X coordinate of the top-left corner (relative to surface).
    pub x: f32,
    /// Y coordinate of the top-left corner (relative to surface).
    pub y: f32,
    /// Width of the drop zone.
    pub width: f32,
    /// Height of the drop zone.
    pub height: f32,
    /// Formats accepted by this drop zone (e.g., "text/plain", "text/uri-list").
    pub accepted_formats: Vec<Cow<'static, str>>,
    /// Actions supported by this drop zone.
    pub accepted_actions: DndAction,
    /// Preferred action for this drop zone.
    pub preferred_action: DndAction,
}

impl DropZone {
    /// Create a new drop zone with the given bounds.
    pub fn new(id: u128, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            id,
            x,
            y,
            width,
            height,
            accepted_formats: Vec::new(),
            accepted_actions: DndAction::Copy,
            preferred_action: DndAction::Copy,
        }
    }

    /// Set the accepted formats.
    pub fn formats(mut self, types: Vec<Cow<'static, str>>) -> Self {
        self.accepted_formats = types;
        self
    }

    /// Set the accepted actions.
    pub fn actions(mut self, actions: DndAction) -> Self {
        self.accepted_actions = actions;
        self
    }

    /// Set the preferred action.
    pub fn preferred(mut self, action: DndAction) -> Self {
        self.preferred_action = action;
        self
    }

    /// Check if a point is within this drop zone.
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

/// Events from an active drag source operation.
#[derive(Debug, Clone, PartialEq)]
pub enum DragSourceEvent {
    /// The drag operation started.
    Started,
    /// A format was accepted by the drop target.
    FormatAccepted(Option<String>),
    /// The action was updated by the drop target.
    ActionChanged(DndAction),
    /// The drag was dropped on a target.
    Dropped,
    /// The drag operation finished successfully.
    Finished(DndAction),
    /// The drag operation was cancelled.
    Cancelled,
}

/// Events for a drop target during a drag operation.
#[derive(Debug, Clone, PartialEq)]
pub enum DropTargetEvent {
    /// A drag entered the drop zone.
    Entered {
        /// X coordinate relative to the surface.
        x: f32,
        /// Y coordinate relative to the surface.
        y: f32,
        /// Formats offered by the drag source.
        formats: Vec<String>,
    },
    /// The drag moved within the drop zone.
    Motion {
        /// X coordinate relative to the surface.
        x: f32,
        /// Y coordinate relative to the surface.
        y: f32,
    },
    /// The drag left the drop zone.
    Left,
    /// Data was dropped.
    Dropped {
        /// X coordinate where the drop occurred.
        x: f32,
        /// Y coordinate where the drop occurred.
        y: f32,
        /// The dropped data.
        data: Vec<u8>,
        /// The format of the dropped data.
        format: String,
    },
}
