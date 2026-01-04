use crate::InputMethod;
use crate::Point;
use crate::event;
use crate::menu::ContextMenuItem;
use crate::window;

/// A request to show a context menu.
#[derive(Debug, Clone)]
pub struct ContextMenuRequest {
    /// The position to show the menu at (in window coordinates).
    pub position: Point,
    /// The menu items to display.
    pub items: Vec<ContextMenuItem>,
}

/// A connection to the state of a shell.
///
/// A [`Widget`] can leverage a [`Shell`] to trigger changes in an application,
/// like publishing messages or invalidating the current layout.
///
/// [`Widget`]: crate::Widget
#[derive(Debug)]
pub struct Shell<'a, Message> {
    messages: &'a mut Vec<Message>,
    event_status: event::Status,
    redraw_request: window::RedrawRequest,
    input_method: InputMethod,
    is_layout_invalid: bool,
    are_widgets_invalid: bool,
    context_menu_request: Option<ContextMenuRequest>,
    #[cfg(feature = "accessibility")]
    a11y_focus_request: Option<crate::accessibility::NodeId>,
}

impl<'a, Message> Shell<'a, Message> {
    /// Creates a new [`Shell`] with the provided buffer of messages.
    pub fn new(messages: &'a mut Vec<Message>) -> Self {
        Self {
            messages,
            event_status: event::Status::Ignored,
            redraw_request: window::RedrawRequest::Wait,
            is_layout_invalid: false,
            are_widgets_invalid: false,
            input_method: InputMethod::Disabled,
            context_menu_request: None,
            #[cfg(feature = "accessibility")]
            a11y_focus_request: None,
        }
    }

    /// Returns true if the [`Shell`] contains no published messages
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Publish the given `Message` for an application to process it.
    pub fn publish(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// Marks the current event as captured. Prevents "event bubbling".
    ///
    /// A widget should capture an event when no ancestor should
    /// handle it.
    pub fn capture_event(&mut self) {
        self.event_status = event::Status::Captured;
    }

    /// Returns the current [`event::Status`] of the [`Shell`].
    #[must_use]
    pub fn event_status(&self) -> event::Status {
        self.event_status
    }

    /// Returns whether the current event has been captured.
    #[must_use]
    pub fn is_event_captured(&self) -> bool {
        self.event_status == event::Status::Captured
    }

    /// Resets the event status to [`event::Status::Ignored`].
    ///
    /// This is useful when you need to simulate multiple events in sequence
    /// and want each event to be processed independently.
    pub fn uncapture_event(&mut self) {
        self.event_status = event::Status::Ignored;
    }

    /// Requests a new frame to be drawn as soon as possible.
    pub fn request_redraw(&mut self) {
        self.redraw_request = window::RedrawRequest::NextFrame;
    }

    /// Requests a new frame to be drawn at the given [`window::RedrawRequest`].
    pub fn request_redraw_at(&mut self, redraw_request: impl Into<window::RedrawRequest>) {
        self.redraw_request = self.redraw_request.min(redraw_request.into());
    }

    /// Returns the request a redraw should happen, if any.
    #[must_use]
    pub fn redraw_request(&self) -> window::RedrawRequest {
        self.redraw_request
    }

    /// Replaces the redraw request of the [`Shell`]; without conflict resolution.
    ///
    /// This is useful if you want to overwrite the redraw request to a previous value.
    /// Since it's a fairly advanced use case and should rarely be used, it is a static
    /// method.
    pub fn replace_redraw_request(shell: &mut Self, redraw_request: window::RedrawRequest) {
        shell.redraw_request = redraw_request;
    }

    /// Requests the current [`InputMethod`] strategy.
    ///
    /// __Important__: This request will only be honored by the
    /// [`Shell`] only during a [`window::Event::RedrawRequested`].
    pub fn request_input_method<T: AsRef<str>>(&mut self, ime: &InputMethod<T>) {
        self.input_method.merge(ime);
    }

    /// Returns the current [`InputMethod`] strategy.
    #[must_use]
    pub fn input_method(&self) -> &InputMethod {
        &self.input_method
    }

    /// Returns the current [`InputMethod`] strategy.
    #[must_use]
    pub fn input_method_mut(&mut self) -> &mut InputMethod {
        &mut self.input_method
    }

    /// Returns whether the current layout is invalid or not.
    #[must_use]
    pub fn is_layout_invalid(&self) -> bool {
        self.is_layout_invalid
    }

    /// Invalidates the current application layout.
    ///
    /// The shell will relayout the application widgets.
    pub fn invalidate_layout(&mut self) {
        self.is_layout_invalid = true;
    }

    /// Triggers the given function if the layout is invalid, cleaning it in the
    /// process.
    pub fn revalidate_layout(&mut self, f: impl FnOnce()) {
        if self.is_layout_invalid {
            self.is_layout_invalid = false;

            f();
        }
    }

    /// Returns whether the widgets of the current application have been
    /// invalidated.
    #[must_use]
    pub fn are_widgets_invalid(&self) -> bool {
        self.are_widgets_invalid
    }

    /// Invalidates the current application widgets.
    ///
    /// The shell will rebuild and relayout the widget tree.
    pub fn invalidate_widgets(&mut self) {
        self.are_widgets_invalid = true;
    }

    /// Merges the current [`Shell`] with another one by applying the given
    /// function to the messages of the latter.
    ///
    /// This method is useful for composition.
    pub fn merge<B>(&mut self, other: Shell<'_, B>, f: impl Fn(B) -> Message) {
        self.messages.extend(other.messages.drain(..).map(f));

        self.is_layout_invalid = self.is_layout_invalid || other.is_layout_invalid;

        self.are_widgets_invalid = self.are_widgets_invalid || other.are_widgets_invalid;

        self.redraw_request = self.redraw_request.min(other.redraw_request);
        self.event_status = self.event_status.merge(other.event_status);
        self.input_method.merge(&other.input_method);

        // Merge context menu request (last one wins)
        if other.context_menu_request.is_some() {
            self.context_menu_request = other.context_menu_request;
        }

        #[cfg(feature = "accessibility")]
        {
            // Merge a11y focus request (last one wins)
            if other.a11y_focus_request.is_some() {
                self.a11y_focus_request = other.a11y_focus_request;
            }
        }
    }

    /// Requests a native context menu to be shown at the given position.
    ///
    /// On platforms that support native context menus (macOS), this will
    /// display a native menu. On other platforms, widgets should fall back
    /// to overlay menus.
    ///
    /// This is typically called by the `context_menu` widget when a right-click
    /// is detected.
    pub fn request_context_menu(&mut self, position: Point, items: Vec<ContextMenuItem>) {
        self.context_menu_request = Some(ContextMenuRequest { position, items });
    }

    /// Takes the pending context menu request, if any.
    ///
    /// This is called by the runtime to process context menu requests from widgets.
    pub fn take_context_menu_request(&mut self) -> Option<ContextMenuRequest> {
        self.context_menu_request.take()
    }

    /// Requests programmatic accessibility focus (VoiceOver cursor) to move to the given NodeId.
    ///
    /// This is used by widgets that implement internal cursor navigation (e.g. menus) so that
    /// arrow-key navigation also moves the screen reader focus.
    #[cfg(feature = "accessibility")]
    pub fn request_a11y_focus(&mut self, target: crate::accessibility::NodeId) {
        self.a11y_focus_request = Some(target);
    }

    /// Takes the pending accessibility focus request, if any.
    #[cfg(feature = "accessibility")]
    pub fn take_a11y_focus_request(&mut self) -> Option<crate::accessibility::NodeId> {
        self.a11y_focus_request.take()
    }

    /// Returns whether there is a pending context menu request.
    #[must_use]
    pub fn has_context_menu_request(&self) -> bool {
        self.context_menu_request.is_some()
    }
}
