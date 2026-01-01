//! Icon-only button variant.
//!
//! # Example
//! ```no_run
//! use icy_ui::widget::button;
//!
//! #[derive(Clone)]
//! enum Message {
//!     Settings,
//! }
//!
//! let settings_btn = button::icon(settings_icon)
//!     .on_press(Message::Settings);
//! ```

use crate::core::{self, Element, Length, Padding};

use super::Button;

/// An icon button builder.
pub struct IconButton<'a, Message, Renderer = crate::Renderer>
where
    Renderer: core::Renderer,
{
    icon: Element<'a, Message, crate::Theme, Renderer>,
    on_press: Option<Message>,
    on_press_down: Option<Message>,
    width: Length,
    height: Length,
    padding: Padding,
    selected: bool,
    class: super::StyleFn<'a, crate::Theme>,
}

impl<'a, Message, Renderer> IconButton<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: core::Renderer + 'a,
{
    /// Creates a new icon button.
    pub fn new(icon: impl Into<Element<'a, Message, crate::Theme, Renderer>>) -> Self {
        Self {
            icon: icon.into(),
            on_press: None,
            on_press_down: None,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::new(8.0),
            selected: false,
            class: Box::new(super::icon),
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

    /// Sets the style to secondary.
    pub fn secondary(mut self) -> Self {
        self.class = Box::new(super::secondary);
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

    /// Sets a custom style function.
    pub fn style(
        mut self,
        style: impl Fn(&crate::Theme, super::Status) -> super::Style + 'a,
    ) -> Self {
        self.class = Box::new(style);
        self
    }
}

impl<'a, Message, Renderer> From<IconButton<'a, Message, Renderer>>
    for Element<'a, Message, crate::Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: core::Renderer + 'a,
{
    fn from(icon_button: IconButton<'a, Message, Renderer>) -> Self {
        let mut button = Button::new(icon_button.icon)
            .width(icon_button.width)
            .height(icon_button.height)
            .padding(icon_button.padding)
            .selected(icon_button.selected)
            .style(move |theme, status| (icon_button.class)(theme, status));

        if let Some(on_press) = icon_button.on_press {
            button = button.on_press(on_press);
        }

        if let Some(on_press_down) = icon_button.on_press_down {
            button = button.on_press_down(on_press_down);
        }

        button.into()
    }
}

/// Creates an icon button.
pub fn icon_button<'a, Message, Renderer>(
    icon: impl Into<Element<'a, Message, crate::Theme, Renderer>>,
) -> IconButton<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: core::Renderer + 'a,
{
    IconButton::new(icon)
}
