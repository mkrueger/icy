//! A modal overlay helper.
//!
//! This module provides a `Modal` widget that displays some content above the
//! rest of the UI using the runtime overlay system.
//!
//! When the modal is open, it captures input events to block background
//! interaction as much as possible.

use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::tree::{self, Tree};
use crate::core::widget::{Operation, Widget};
use crate::core::{
    Alignment, Clipboard, Element, Event, Layout, Rectangle, Shell, Size, Vector,
};

use std::borrow::Borrow;
use std::rc::Rc;

/// A widget that displays a modal dialog above its content.
///
/// The modal is shown using the overlay system, so it can block background
/// interactions.
pub struct Modal<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Renderer: crate::core::Renderer,
{
    open: bool,
    content: Element<'a, Message, Theme, Renderer>,
    modal: Element<'a, Message, Theme, Renderer>,
    on_blur: Option<Rc<dyn Fn() -> Message + 'a>>,
    on_escape: Option<Rc<dyn Fn() -> Message + 'a>>,
    shade: Option<crate::core::Color>,
}

impl<'a, Message, Theme, Renderer> Modal<'a, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
{
    /// Creates a new [`Modal`].
    ///
    /// If `open` is `true`, the `modal` element will be displayed on top of the
    /// `content`.
    pub fn new(
        open: bool,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        modal: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            open,
            content: content.into(),
            modal: modal.into(),
            on_blur: None,
            on_escape: None,
            shade: None,
        }
    }

    /// Sets whether the modal is open.
    #[must_use]
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Publishes `message` when the user clicks outside the modal.
    ///
    /// This is commonly used to dismiss the modal.
    #[must_use]
    pub fn on_blur(self, message: Message) -> Self
    where
        Message: Clone + 'a,
    {
        self.on_blur_with(move || message.clone())
    }

    /// Publishes a message when the user clicks outside the modal.
    #[must_use]
    pub fn on_blur_with(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_blur = Some(Rc::new(f));
        self
    }

    /// Publishes `message` when the user presses <kbd>Esc</kbd> while the modal
    /// is open.
    #[must_use]
    pub fn on_escape(self, message: Message) -> Self
    where
        Message: Clone + 'a,
    {
        self.on_escape_with(move || message.clone())
    }

    /// Publishes a message when the user presses <kbd>Esc</kbd> while the modal
    /// is open.
    #[must_use]
    pub fn on_escape_with(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_escape = Some(Rc::new(f));
        self
    }

    /// Overrides the scrim/shade color used behind the modal.
    ///
    /// By default, it uses `theme.shade`.
    #[must_use]
    pub fn shade(mut self, color: crate::core::Color) -> Self {
        self.shade = Some(color);
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Modal<'a, Message, Theme, Renderer>
where
    Theme: Borrow<crate::Theme>,
    Renderer: crate::core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<()>()
    }

    fn state(&self) -> tree::State {
        tree::State::None
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content), Tree::new(&self.modal)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.content, &self.modal]);
    }

    fn size(&self) -> Size<crate::core::Length> {
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

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
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

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        if !self.open {
            return self.content.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            );
        }

        Some(overlay::Element::new(Box::new(ModalOverlay {
            state: &mut tree.children[1],
            element: &mut self.modal,
            on_blur: self.on_blur.as_deref(),
            on_escape: self.on_escape.as_deref(),
            shade: self.shade,
        })))
    }
}

impl<'a, Message, Theme, Renderer> From<Modal<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Borrow<crate::Theme> + 'a,
    Renderer: crate::core::Renderer + 'a,
{
    fn from(modal: Modal<'a, Message, Theme, Renderer>) -> Self {
        Element::new(modal)
    }
}

struct ModalOverlay<'a, 'b, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
{
    state: &'b mut Tree,
    element: &'b mut Element<'a, Message, Theme, Renderer>,
    on_blur: Option<&'b dyn Fn() -> Message>,
    on_escape: Option<&'b dyn Fn() -> Message>,
    shade: Option<crate::core::Color>,
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for ModalOverlay<'_, '_, Message, Theme, Renderer>
where
    Theme: Borrow<crate::Theme>,
    Renderer: crate::core::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds);

        let node = self
            .element
            .as_widget_mut()
            .layout(self.state, renderer, &limits)
            .align(Alignment::Center, Alignment::Center, bounds);

        layout::Node::with_children(bounds, vec![node])
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();

        renderer.with_layer(bounds, |renderer| {
            let shade = self.shade.unwrap_or_else(|| theme.borrow().shade);
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    ..renderer::Quad::default()
                },
                shade,
            );

            let child_layout = layout.children().next().unwrap();

            self.element.as_widget().draw(
                self.state,
                renderer,
                theme,
                style,
                child_layout,
                cursor,
                &bounds,
            );
        });
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        use crate::core::keyboard;
        use crate::core::keyboard::key::Named;
        use crate::core::keyboard::Key;
        use crate::core::mouse::Button;

        let child_layout = layout.children().next().unwrap();
        let modal_bounds = child_layout.bounds();

        let mut handled_dismiss = false;

        match event {
            Event::Mouse(crate::core::mouse::Event::ButtonPressed {
                button: Button::Left,
                ..
            }) => {
                if cursor.position().is_some() && !cursor.is_over(modal_bounds) {
                    if let Some(on_blur) = self.on_blur {
                        shell.publish(on_blur());
                        shell.request_redraw();
                    }

                    handled_dismiss = true;
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                if matches!(key, Key::Named(Named::Escape)) {
                    if let Some(on_escape) = self.on_escape {
                        shell.publish(on_escape());
                        shell.request_redraw();
                    }

                    handled_dismiss = true;
                }
            }
            _ => {}
        }

        if !handled_dismiss {
            self.element.as_widget_mut().update(
                self.state,
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                &layout.bounds(),
            );
        }

        // Always capture while the modal is open to block background interaction.
        shell.capture_event();
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let child_layout = layout.children().next().unwrap();

        let child_interaction = self.element.as_widget().mouse_interaction(
            self.state,
            child_layout,
            cursor,
            &layout.bounds(),
            renderer,
        );

        if cursor.position().is_some() {
            child_interaction.max(mouse::Interaction::Idle)
        } else {
            child_interaction
        }
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'c>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
        self.element.as_widget_mut().overlay(
            self.state,
            layout.children().next().unwrap(),
            renderer,
            &layout.bounds(),
            Vector::ZERO,
        )
    }

    fn index(&self) -> f32 {
        1000.0
    }
}
