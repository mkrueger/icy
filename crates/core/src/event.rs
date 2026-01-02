//! Handle events of a user interface.
use crate::input_method;
use crate::keyboard;
use crate::menu;
use crate::mouse;
use crate::touch;
use crate::window;

#[cfg(feature = "accessibility")]
use crate::accessibility;

/// A user interface event.
///
/// _**Note:** This type is largely incomplete! If you need to track
/// additional events, feel free to [open an issue] and share your use case!_
///
/// [open an issue]: https://github.com/iced-rs/iced/issues
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A keyboard event
    Keyboard(keyboard::Event),

    /// A mouse event
    Mouse(mouse::Event),

    /// A window event
    Window(window::Event),

    /// A touch event
    Touch(touch::Event),

    /// An input method event
    InputMethod(input_method::Event),

    /// A context menu item was selected.
    ///
    /// This event is dispatched when a user selects an item from a native
    /// context menu. The widget that opened the menu should handle this
    /// event and map the [`MenuId`](menu::MenuId) back to the appropriate message.
    ContextMenuItemSelected(menu::MenuId),

    /// An accessibility event from a screen reader or assistive technology.
    ///
    /// This event is triggered when a user interacts with the UI through
    /// accessibility tools like screen readers (Narrator, VoiceOver, Orca).
    #[cfg(feature = "accessibility")]
    Accessibility(accessibility::Event),
}

/// The status of an [`Event`] after being processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`Event`] was **NOT** handled by any widget.
    Ignored,

    /// The [`Event`] was handled and processed by a widget.
    Captured,
}

impl Status {
    /// Merges two [`Status`] into one.
    ///
    /// `Captured` takes precedence over `Ignored`:
    ///
    /// ```
    /// use icy_ui_core::event::Status;
    ///
    /// assert_eq!(Status::Ignored.merge(Status::Ignored), Status::Ignored);
    /// assert_eq!(Status::Ignored.merge(Status::Captured), Status::Captured);
    /// assert_eq!(Status::Captured.merge(Status::Ignored), Status::Captured);
    /// assert_eq!(Status::Captured.merge(Status::Captured), Status::Captured);
    /// ```
    pub fn merge(self, b: Self) -> Self {
        match self {
            Status::Ignored => b,
            Status::Captured => Status::Captured,
        }
    }
}
