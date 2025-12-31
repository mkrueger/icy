//! Drag and drop widget wrappers.
//!
//! This module provides widgets that enable drag and drop functionality:
//! - [`Draggable`] - Makes a widget draggable
//! - [`DropTarget`] - Makes a widget accept drops

use crate::core::clipboard::Format;
use crate::core::dnd::{DndAction, DragData};
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::touch;
use crate::core::widget::{Operation, Tree, tree};
use crate::core::{
    Clipboard, Element, Event, Layout, Length, Point, Rectangle, Shell, Size, Vector, Widget,
};

// ============================================================================
// Draggable Widget
// ============================================================================

/// A widget wrapper that makes its content draggable.
///
/// When the user presses and drags, this widget will emit a message to start
/// a drag operation.
pub struct Draggable<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    /// Data provider called when drag starts
    on_drag: Option<Box<dyn Fn() -> DragData + 'a>>,
    /// Message to emit when drag starts
    on_drag_start: Option<Box<dyn Fn(DragData) -> Message + 'a>>,
    /// Message to emit when drag ends
    on_drag_end: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    /// Minimum distance before drag starts (platform drag threshold)
    drag_threshold: f32,
    /// Allowed actions for this drag
    allowed_actions: DndAction,
}

impl<'a, Message, Theme, Renderer> Draggable<'a, Message, Theme, Renderer> {
    /// Creates a new [`Draggable`] wrapper around the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Draggable {
            content: content.into(),
            on_drag: None,
            on_drag_start: None,
            on_drag_end: None,
            drag_threshold: 5.0,
            allowed_actions: DndAction::Copy,
        }
    }

    /// Sets the data provider for when a drag starts.
    ///
    /// This function is called when the drag threshold is exceeded.
    #[must_use]
    pub fn on_drag(mut self, f: impl Fn() -> DragData + 'a) -> Self {
        self.on_drag = Some(Box::new(f));
        self
    }

    /// Sets the message to emit when a drag starts.
    ///
    /// The message receives the [`DragData`] that will be dragged.
    #[must_use]
    pub fn on_drag_start(mut self, f: impl Fn(DragData) -> Message + 'a) -> Self {
        self.on_drag_start = Some(Box::new(f));
        self
    }

    /// Sets the message to emit when a drag ends.
    ///
    /// The boolean indicates whether the drag was successful (dropped) or cancelled.
    #[must_use]
    pub fn on_drag_end(mut self, f: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_drag_end = Some(Box::new(f));
        self
    }

    /// Sets the minimum distance the cursor must move before a drag starts.
    ///
    /// Default is 5.0 pixels.
    #[must_use]
    pub fn drag_threshold(mut self, threshold: f32) -> Self {
        self.drag_threshold = threshold;
        self
    }

    /// Sets the allowed actions for this drag.
    #[must_use]
    pub fn allowed_actions(mut self, actions: DndAction) -> Self {
        self.allowed_actions = actions;
        self
    }
}

/// Local state of the [`Draggable`] widget.
#[derive(Default)]
struct DraggableState {
    /// Press start position (if pressed)
    press_position: Option<Point>,
    /// Whether we're currently dragging
    is_dragging: bool,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Draggable<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<DraggableState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(DraggableState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        // First, let the content handle the event
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if shell.is_event_captured() {
            return;
        }

        let state: &mut DraggableState = tree.state.downcast_mut();
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed {
                button: mouse::Button::Left,
                ..
            })
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if cursor.is_over(bounds) && self.on_drag.is_some() {
                    state.press_position = cursor.position();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased {
                button: mouse::Button::Left,
                ..
            })
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if state.is_dragging {
                    state.is_dragging = false;
                    if let Some(on_drag_end) = &self.on_drag_end {
                        // We don't know if the drag succeeded from here
                        // The actual result comes via window events
                        shell.publish(on_drag_end(false));
                    }
                }
                state.press_position = None;
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if let Some(press_pos) = state.press_position {
                    if !state.is_dragging {
                        if let Some(current_pos) = cursor.position() {
                            let distance = ((current_pos.x - press_pos.x).powi(2)
                                + (current_pos.y - press_pos.y).powi(2))
                            .sqrt();

                            if distance >= self.drag_threshold {
                                // Start drag
                                if let Some(on_drag) = &self.on_drag {
                                    let data = on_drag();
                                    state.is_dragging = true;

                                    if let Some(on_drag_start) = &self.on_drag_start {
                                        shell.publish(on_drag_start(data));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state: &DraggableState = tree.state.downcast_ref();

        if state.is_dragging {
            mouse::Interaction::Grabbing
        } else if state.press_position.is_some() {
            mouse::Interaction::Grab
        } else {
            self.content.as_widget().mouse_interaction(
                &tree.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            renderer_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Draggable<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a + renderer::Renderer,
{
    fn from(draggable: Draggable<'a, Message, Theme, Renderer>) -> Self {
        Element::new(draggable)
    }
}

// ============================================================================
// DropTarget Widget
// ============================================================================

/// A widget wrapper that makes its content a drop target.
///
/// This widget emits messages when drags enter, move over, leave, or drop
/// on the wrapped content.
pub struct DropTarget<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    /// MIME types this target accepts
    accepted_mime_types: Vec<String>,
    /// Message when drag enters
    on_enter: Option<Box<dyn Fn(Point, Vec<String>) -> Message + 'a>>,
    /// Message when drag moves
    on_move: Option<Box<dyn Fn(Point) -> Message + 'a>>,
    /// Message when drag leaves
    on_leave: Option<Message>,
    /// Message when drop occurs
    on_drop: Option<Box<dyn Fn(Point, Vec<u8>, String) -> Message + 'a>>,
    /// Preferred action
    preferred_action: DndAction,
    /// Highlight when drag is over
    highlight_on_hover: bool,
}

impl<'a, Message, Theme, Renderer> DropTarget<'a, Message, Theme, Renderer> {
    /// Creates a new [`DropTarget`] wrapper around the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        DropTarget {
            content: content.into(),
            accepted_mime_types: Vec::new(),
            on_enter: None,
            on_move: None,
            on_leave: None,
            on_drop: None,
            preferred_action: DndAction::Copy,
            highlight_on_hover: false,
        }
    }

    /// Sets the MIME types this drop target accepts.
    #[must_use]
    pub fn mime_types(mut self, types: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.accepted_mime_types = types.into_iter().map(Into::into).collect();
        self
    }

    /// Convenience method to accept text drops.
    #[must_use]
    pub fn accept_text(mut self) -> Self {
        self.accepted_mime_types = Format::Text
            .formats()
            .iter()
            .map(|s| s.to_string())
            .collect();
        self
    }

    /// Convenience method to accept file drops.
    #[must_use]
    pub fn accept_files(mut self) -> Self {
        self.accepted_mime_types = Format::Files
            .formats()
            .iter()
            .map(|s| s.to_string())
            .collect();
        self
    }

    /// Sets the message to emit when a drag enters this target.
    ///
    /// The callback receives the position and the offered MIME types.
    #[must_use]
    pub fn on_enter(mut self, f: impl Fn(Point, Vec<String>) -> Message + 'a) -> Self {
        self.on_enter = Some(Box::new(f));
        self
    }

    /// Sets the message to emit when a drag moves within this target.
    #[must_use]
    pub fn on_move(mut self, f: impl Fn(Point) -> Message + 'a) -> Self {
        self.on_move = Some(Box::new(f));
        self
    }

    /// Sets the message to emit when a drag leaves this target.
    #[must_use]
    pub fn on_leave(mut self, message: Message) -> Self {
        self.on_leave = Some(message);
        self
    }

    /// Sets the message to emit when data is dropped on this target.
    ///
    /// The callback receives the position, the dropped data, and its MIME type.
    #[must_use]
    pub fn on_drop(mut self, f: impl Fn(Point, Vec<u8>, String) -> Message + 'a) -> Self {
        self.on_drop = Some(Box::new(f));
        self
    }

    /// Sets the preferred action for drops on this target.
    #[must_use]
    pub fn preferred_action(mut self, action: DndAction) -> Self {
        self.preferred_action = action;
        self
    }

    /// Whether to visually highlight when a drag is over this target.
    #[must_use]
    pub fn highlight_on_hover(mut self, highlight: bool) -> Self {
        self.highlight_on_hover = highlight;
        self
    }
}

/// Local state of the [`DropTarget`] widget.
#[derive(Default)]
struct DropTargetState {
    /// Whether a drag is currently over this target
    is_hovered: bool,
    /// MIME types offered by the current drag
    offered_mime_types: Vec<String>,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for DropTarget<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<DropTargetState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(DropTargetState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        // First, let the content handle the event
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if shell.is_event_captured() {
            return;
        }

        let state: &mut DropTargetState = tree.state.downcast_mut();
        let bounds = layout.bounds();

        // Handle DnD window events
        match event {
            Event::Window(crate::core::window::Event::DragEntered {
                position,
                mime_types,
            }) => {
                if bounds.contains(*position) {
                    state.is_hovered = true;
                    state.offered_mime_types = mime_types.clone();

                    if let Some(on_enter) = &self.on_enter {
                        shell.publish(on_enter(*position, mime_types.clone()));
                    }
                }
            }
            Event::Window(crate::core::window::Event::DragMoved { position }) => {
                let is_over = bounds.contains(*position);

                if is_over && !state.is_hovered {
                    // Entered
                    state.is_hovered = true;
                    if let Some(on_enter) = &self.on_enter {
                        shell.publish(on_enter(*position, state.offered_mime_types.clone()));
                    }
                } else if !is_over && state.is_hovered {
                    // Left
                    state.is_hovered = false;
                    if let Some(on_leave) = &self.on_leave {
                        shell.publish(on_leave.clone());
                    }
                } else if is_over {
                    // Moving within
                    if let Some(on_move) = &self.on_move {
                        shell.publish(on_move(*position));
                    }
                }
            }
            Event::Window(crate::core::window::Event::DragDropped {
                position,
                data,
                mime_type,
                ..
            }) => {
                if bounds.contains(*position) {
                    state.is_hovered = false;
                    if let Some(on_drop) = &self.on_drop {
                        shell.publish(on_drop(*position, data.clone(), mime_type.clone()));
                    }
                }
            }
            Event::Window(crate::core::window::Event::DragLeft) => {
                if state.is_hovered {
                    state.is_hovered = false;
                    if let Some(on_leave) = &self.on_leave {
                        shell.publish(on_leave.clone());
                    }
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state: &DropTargetState = tree.state.downcast_ref();

        // Draw content
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            renderer_style,
            layout,
            cursor,
            viewport,
        );

        // Draw highlight overlay if hovered and highlighting is enabled
        if self.highlight_on_hover && state.is_hovered {
            use crate::core::Border;

            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border: Border {
                        color: crate::core::Color::from_rgba(0.2, 0.5, 1.0, 0.3),
                        width: 2.0,
                        radius: 4.0.into(),
                    },
                    shadow: Default::default(),
                    snap: false,
                },
                crate::core::Background::Color(crate::core::Color::from_rgba(0.2, 0.5, 1.0, 0.1)),
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<DropTarget<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a + renderer::Renderer,
{
    fn from(drop_target: DropTarget<'a, Message, Theme, Renderer>) -> Self {
        Element::new(drop_target)
    }
}
