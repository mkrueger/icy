//! Image button variant with selection support.
//!
//! # Example
//! ```no_run
//! use icy_ui::widget::button;
//!
//! #[derive(Clone)]
//! enum Message {
//!     SelectImage(usize),
//! }
//!
//! let image_btn = button::image::image_button(my_image_handle)
//!     .selected(is_selected)
//!     .on_press(Message::SelectImage(0));
//! ```

use std::marker::PhantomData;

use crate::core::{self, Element, Length, Padding};
use crate::core::image::Handle;

use super::Button;

/// An image button builder.
pub struct ImageButton<'a, Message, Renderer = crate::Renderer>
where
    Renderer: core::Renderer + core::image::Renderer<Handle = Handle>,
{
    handle: Handle,
    on_press: Option<Message>,
    on_press_down: Option<Message>,
    width: Length,
    height: Length,
    padding: Padding,
    selected: bool,
    image_width: Option<Length>,
    image_height: Option<Length>,
    class: super::StyleFn<'a, crate::Theme>,
    _renderer: PhantomData<Renderer>,
}

impl<'a, Message, Renderer> ImageButton<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: core::Renderer + core::image::Renderer<Handle = Handle> + 'a,
{
    /// Creates a new image button with the given image handle.
    pub fn new(handle: impl Into<Handle>) -> Self {
        Self {
            handle: handle.into(),
            on_press: None,
            on_press_down: None,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::ZERO,
            selected: false,
            image_width: None,
            image_height: None,
            class: Box::new(super::secondary),
            _renderer: PhantomData,
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

    /// Sets the width of the inner image.
    pub fn image_width(mut self, width: impl Into<Length>) -> Self {
        self.image_width = Some(width.into());
        self
    }

    /// Sets the height of the inner image.
    pub fn image_height(mut self, height: impl Into<Length>) -> Self {
        self.image_height = Some(height.into());
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

    /// Sets a custom style function.
    pub fn style(
        mut self,
        style: impl Fn(&crate::Theme, super::Status) -> super::Style + 'a,
    ) -> Self {
        self.class = Box::new(style);
        self
    }
}

impl<'a, Message, Renderer> From<ImageButton<'a, Message, Renderer>>
    for Element<'a, Message, crate::Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: core::Renderer + core::image::Renderer<Handle = Handle> + 'a,
{
    fn from(image_button: ImageButton<'a, Message, Renderer>) -> Self {
        let mut img = crate::Image::new(image_button.handle);
        
        if let Some(w) = image_button.image_width {
            img = img.width(w);
        }
        if let Some(h) = image_button.image_height {
            img = img.height(h);
        }

        let mut button = Button::new(img)
            .width(image_button.width)
            .height(image_button.height)
            .padding(image_button.padding)
            .selected(image_button.selected)
            .style(move |theme, status| (image_button.class)(theme, status));

        if let Some(on_press) = image_button.on_press {
            button = button.on_press(on_press);
        }

        if let Some(on_press_down) = image_button.on_press_down {
            button = button.on_press_down(on_press_down);
        }

        button.into()
    }
}

/// Creates an image button with the given handle.
pub fn image_button<'a, Message, Renderer>(
    handle: impl Into<Handle>,
) -> ImageButton<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: core::Renderer + core::image::Renderer<Handle = Handle> + 'a,
{
    ImageButton::new(handle)
}
