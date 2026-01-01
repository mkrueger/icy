//! Text button variant with optional leading/trailing icons.
//!
//! # Example
//! ```no_run
//! use icy_ui::widget::button;
//!
//! #[derive(Clone)]
//! enum Message {
//!     Save,
//!     Cancel,
//! }
//!
//! // Simple text button
//! let save = button::text::text_button("Save").on_press(Message::Save);
//!
//! // Text button with leading icon
//! let cancel = button::text::text_button("Cancel")
//!     .leading_icon(some_icon)
//!     .on_press(Message::Cancel);
//! ```

use crate::core::{self, Element, Length, Padding};
use crate::{Row, text};

use super::Button;

/// A text button builder with optional icons.
pub struct TextButton<'a, Message, Renderer = crate::Renderer>
where
    Renderer: core::Renderer + core::text::Renderer,
{
    label: String,
    on_press: Option<Message>,
    on_press_down: Option<Message>,
    leading_icon: Option<Element<'a, Message, crate::Theme, Renderer>>,
    trailing_icon: Option<Element<'a, Message, crate::Theme, Renderer>>,
    width: Length,
    height: Length,
    padding: Padding,
    spacing: f32,
    selected: bool,
    class: super::StyleFn<'a, crate::Theme>,
}

impl<'a, Message, Renderer> TextButton<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: core::Renderer + core::text::Renderer + 'a,
{
    /// Creates a new text button with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            on_press: None,
            on_press_down: None,
            leading_icon: None,
            trailing_icon: None,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: super::DEFAULT_PADDING,
            spacing: 8.0,
            selected: false,
            class: Box::new(super::secondary),
        }
    }

    /// Sets the message produced when pressed.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Sets the message produced when pressed, if `Some`.
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    /// Sets the message produced when pressed down.
    pub fn on_press_down(mut self, message: Message) -> Self {
        self.on_press_down = Some(message);
        self
    }

    /// Sets an icon to display before the label.
    pub fn leading_icon(
        mut self,
        icon: impl Into<Element<'a, Message, crate::Theme, Renderer>>,
    ) -> Self {
        self.leading_icon = Some(icon.into());
        self
    }

    /// Sets an icon to display after the label.
    pub fn trailing_icon(
        mut self,
        icon: impl Into<Element<'a, Message, crate::Theme, Renderer>>,
    ) -> Self {
        self.trailing_icon = Some(icon.into());
        self
    }

    /// Sets the width of the button.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the button.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the padding of the button.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the spacing between icon and text.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets whether the button is in a selected state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Sets the style to primary (accent).
    pub fn primary(mut self) -> Self {
        self.class = Box::new(super::primary);
        self
    }

    /// Sets the style to success.
    pub fn success(mut self) -> Self {
        self.class = Box::new(super::success);
        self
    }

    /// Sets the style to warning.
    pub fn warning(mut self) -> Self {
        self.class = Box::new(super::warning);
        self
    }

    /// Sets the style to danger.
    pub fn danger(mut self) -> Self {
        self.class = Box::new(super::danger);
        self
    }

    /// Sets the style to text-only (no background).
    pub fn text_style(mut self) -> Self {
        self.class = Box::new(super::text_style);
        self
    }

    /// Sets a custom style function.
    pub fn style(
        mut self,
        style: impl Fn(&crate::Theme, super::Status) -> super::Style + 'a,
    ) -> Self {
        self.class = Box::new(style);
        self
    }
}

impl<'a, Message, Renderer> From<TextButton<'a, Message, Renderer>>
    for Element<'a, Message, crate::Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: core::Renderer + core::text::Renderer + 'a,
{
    fn from(text_button: TextButton<'a, Message, Renderer>) -> Self {
        let mut content: Vec<Element<'a, Message, crate::Theme, Renderer>> = Vec::with_capacity(3);

        if let Some(icon) = text_button.leading_icon {
            content.push(icon);
        }

        if !text_button.label.is_empty() {
            content.push(text(text_button.label).into());
        }

        if let Some(icon) = text_button.trailing_icon {
            content.push(icon);
        }

        let row = Row::with_children(content)
            .spacing(text_button.spacing)
            .align_y(core::Alignment::Center);

        let mut button = Button::new(row)
            .width(text_button.width)
            .height(text_button.height)
            .padding(text_button.padding)
            .selected(text_button.selected)
            .style(move |theme, status| (text_button.class)(theme, status));

        if let Some(on_press) = text_button.on_press {
            button = button.on_press(on_press);
        }

        if let Some(on_press_down) = text_button.on_press_down {
            button = button.on_press_down(on_press_down);
        }

        button.into()
    }
}

/// Creates a text button with the given label.
pub fn text_button<'a, Message, Renderer>(
    label: impl Into<String>,
) -> TextButton<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: core::Renderer + core::text::Renderer + 'a,
{
    TextButton::new(label)
}
