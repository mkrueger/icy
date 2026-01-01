//! Hyperlink widget that opens URLs in the default browser.
//!
//! Inspired by egui's `Hyperlink` widget - shows underline on hover and
//! opens the URL automatically when clicked.
//!
//! # Example
//! ```no_run
//! use icy_ui::widget::button;
//!
//! // Simple hyperlink
//! let link = button::hyperlink("Visit GitHub", "https://github.com");
//!
//! // Using the builder pattern
//! let link = button::Hyperlink::new("Documentation", "https://docs.rs")
//!     .size(14.0);
//! ```

use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::text::paragraph::{self, Paragraph};
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    self, Clipboard, Element, Event, Layout, Length, Pixels, Rectangle, Shell, Size, Widget,
};
use crate::Theme;

/// A clickable hyperlink that opens a URL in the default browser.
///
/// Shows underline on hover (like egui) and automatically opens the URL when clicked.
pub struct Hyperlink<'a, Message, Renderer = crate::Renderer>
where
    Renderer: core::text::Renderer,
{
    label: String,
    url: String,
    size: Option<Pixels>,
    on_open: Option<Box<dyn Fn(&str) -> Message + 'a>>,
    _marker: std::marker::PhantomData<Renderer>,
}

impl<'a, Message, Renderer> Hyperlink<'a, Message, Renderer>
where
    Renderer: core::text::Renderer,
{
    /// Creates a new hyperlink with the given label and URL.
    pub fn new(label: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            url: url.into(),
            size: None,
            on_open: None,
            _marker: std::marker::PhantomData,
        }
    }

    /// Sets the text size of the hyperlink.
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets a callback that's called when the link is opened.
    ///
    /// This allows the application to be notified when a link is clicked,
    /// in addition to opening the URL in the browser.
    pub fn on_open<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> Message + 'a,
    {
        self.on_open = Some(Box::new(f));
        self
    }
}

/// The local state of a [`Hyperlink`].
#[derive(Debug, Clone, Default)]
struct State<P: Paragraph> {
    is_hovered: bool,
    paragraph: paragraph::Plain<P>,
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Hyperlink<'_, Message, Renderer>
where
    Renderer: core::text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph>::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        let size = self.size.unwrap_or_else(|| renderer.default_size());

        let _ = state.paragraph.update(core::text::Text {
            content: &self.label,
            bounds: limits.max(),
            size,
            line_height: core::text::LineHeight::default(),
            font: renderer.default_font(),
            align_x: core::text::Alignment::Default,
            align_y: core::alignment::Vertical::Top,
            shaping: core::text::Shaping::Advanced,
            wrapping: core::text::Wrapping::None,
            hint_factor: renderer.scale_factor(),
        });

        let size = state.paragraph.min_bounds();

        layout::Node::new(size)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let bounds = layout.bounds();

        let color = theme.accent.base;

        // Draw the text
        renderer.fill_paragraph(state.paragraph.raw(), bounds.position(), color, *viewport);

        // Draw underline when hovered
        if state.is_hovered {
            let text_size = state.paragraph.min_bounds();
            let underline_y = bounds.y + text_size.height;

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x,
                        y: underline_y - 1.0,
                        width: text_size.width,
                        height: 1.0,
                    },
                    ..renderer::Quad::default()
                },
                color,
            );
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let was_hovered = state.is_hovered;
                state.is_hovered = cursor.is_over(bounds);

                if was_hovered != state.is_hovered {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed {
                button: mouse::Button::Left,
                ..
            }) => {
                if cursor.is_over(bounds) {
                    // Open the URL in the default browser
                    if let Err(e) = opener::open(&self.url) {
                        log::error!("Failed to open URL '{}': {}", self.url, e);
                    }

                    // Notify the application if a callback was set
                    if let Some(on_open) = &self.on_open {
                        shell.publish((on_open)(&self.url));
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
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();

        if state.is_hovered || cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message, Renderer> From<Hyperlink<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core::text::Renderer + 'a,
{
    fn from(hyperlink: Hyperlink<'a, Message, Renderer>) -> Self {
        Element::new(hyperlink)
    }
}

/// Creates a hyperlink with the given label and URL.
///
/// The hyperlink will open the URL in the default browser when clicked,
/// and show an underline on hover.
///
/// # Example
/// ```no_run
/// use icy_ui::widget::button;
///
/// let link = button::hyperlink("Visit GitHub", "https://github.com");
/// ```
pub fn hyperlink<'a, Message, Renderer>(
    label: impl Into<String>,
    url: impl Into<String>,
) -> Hyperlink<'a, Message, Renderer>
where
    Renderer: core::text::Renderer,
{
    Hyperlink::new(label, url)
}
