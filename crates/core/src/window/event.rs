use crate::dnd::{DndAction, DragSourceEvent};
use crate::keyboard::Modifiers;
use crate::time::Instant;
use crate::{Point, Size};

/// A window-related event.
#[derive(PartialEq, Clone, Debug)]
pub enum Event {
    /// A window was opened.
    Opened {
        /// The position of the opened window. This is relative to the top-left corner of the desktop
        /// the window is on, including virtual desktops. Refers to window's "outer" position,
        /// or the window area, in logical pixels.
        ///
        /// **Note**: Not available in Wayland.
        position: Option<Point>,
        /// The size of the created window. This is its "inner" size, or the size of the
        /// client area, in logical pixels.
        size: Size,
    },

    /// A window was closed.
    Closed,

    /// A window was moved.
    Moved(Point),

    /// A window was resized.
    Resized(Size),

    /// A window changed its scale factor.
    Rescaled(f32),

    /// A window redraw was requested.
    ///
    /// The [`Instant`] contains the current time.
    RedrawRequested(Instant),

    /// The user has requested for the window to close.
    CloseRequested,

    /// A window was focused.
    Focused,

    /// A window was unfocused.
    Unfocused,

    /// A drag entered this window.
    ///
    /// This is emitted when an external drag (from another application or window)
    /// enters this window's bounds.
    DragEntered {
        /// Position where the drag entered, relative to the window.
        position: Point,
        /// Formats offered by the drag source.
        formats: Vec<String>,
    },

    /// A drag moved within this window.
    DragMoved {
        /// Current position of the drag, relative to the window.
        position: Point,
        /// Current keyboard modifiers while dragging.
        modifiers: Modifiers,
    },

    /// Data was dropped on this window.
    DragDropped {
        /// Position where the drop occurred.
        position: Point,
        /// The dropped data bytes.
        data: Vec<u8>,
        /// The format of the dropped data.
        format: String,
        /// The action that was performed (Copy, Move, Link).
        action: DndAction,
    },

    /// A drag left this window without dropping.
    DragLeft,

    /// Event from an active drag source operation that this window started.
    DragSource(DragSourceEvent),
}
