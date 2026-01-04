//! Pick lists display a dropdown list of selectable options.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use icy_ui_widget::*; } pub use icy_ui_widget::Renderer; pub use icy_ui_widget::core::*; }
//! # pub type Element<'a, Message> = icy_ui_widget::core::Element<'a, Message, icy_ui_widget::Theme, icy_ui_widget::Renderer>;
//! #
//! use icy_ui::widget::pick_list;
//!
//! struct State {
//!    favorite: Option<Fruit>,
//! }
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! enum Fruit {
//!     Apple,
//!     Orange,
//!     Strawberry,
//!     Tomato,
//! }
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     FruitSelected(Fruit),
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     let fruits = [
//!         Fruit::Apple,
//!         Fruit::Orange,
//!         Fruit::Strawberry,
//!         Fruit::Tomato,
//!     ];
//!
//!     pick_list(
//!         fruits,
//!         state.favorite,
//!         Message::FruitSelected,
//!     )
//!     .placeholder("Select your favorite fruit...")
//!     .into()
//! }
//!
//! fn update(state: &mut State, message: Message) {
//!     match message {
//!         Message::FruitSelected(fruit) => {
//!             state.favorite = Some(fruit);
//!         }
//!     }
//! }
//!
//! impl std::fmt::Display for Fruit {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         f.write_str(match self {
//!             Self::Apple => "Apple",
//!             Self::Orange => "Orange",
//!             Self::Strawberry => "Strawberry",
//!             Self::Tomato => "Tomato",
//!         })
//!     }
//! }
//! ```
use crate::core::alignment;
use crate::core::keyboard;
use crate::core::keyboard::Key;
use crate::core::keyboard::key;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::text::paragraph;
use crate::core::text::{self, Text};
use crate::core::touch;
use crate::core::widget::Id;
use crate::core::widget::operation::{self, Operation};
use crate::core::widget::tree::{self, Tree};
use crate::core::window;
use crate::core::{
    Background, Border, Clipboard, Color, Element, Event, Layout, Length, Padding, Pixels, Point,
    Rectangle, Shell, Size, Theme, Vector, Widget,
};
use crate::focus::FocusRing;
use crate::overlay::menu::{self, Menu};

use std::borrow::Borrow;
use std::f32;

/// A widget for selecting a single value from a list of options.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use icy_ui_widget::*; } pub use icy_ui_widget::Renderer; pub use icy_ui_widget::core::*; }
/// # pub type Element<'a, Message> = icy_ui_widget::core::Element<'a, Message, icy_ui_widget::Theme, icy_ui_widget::Renderer>;
/// #
/// use icy_ui::widget::pick_list;
///
/// struct State {
///    favorite: Option<Fruit>,
/// }
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// enum Fruit {
///     Apple,
///     Orange,
///     Strawberry,
///     Tomato,
/// }
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     FruitSelected(Fruit),
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     let fruits = [
///         Fruit::Apple,
///         Fruit::Orange,
///         Fruit::Strawberry,
///         Fruit::Tomato,
///     ];
///
///     pick_list(
///         fruits,
///         state.favorite,
///         Message::FruitSelected,
///     )
///     .placeholder("Select your favorite fruit...")
///     .into()
/// }
///
/// fn update(state: &mut State, message: Message) {
///     match message {
///         Message::FruitSelected(fruit) => {
///             state.favorite = Some(fruit);
///         }
///     }
/// }
///
/// impl std::fmt::Display for Fruit {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         f.write_str(match self {
///             Self::Apple => "Apple",
///             Self::Orange => "Orange",
///             Self::Strawberry => "Strawberry",
///             Self::Tomato => "Tomato",
///         })
///     }
/// }
/// ```
pub struct PickList<'a, T, L, V, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    T: ToString + PartialEq + Clone,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    id: Option<Id>,
    on_select: Box<dyn Fn(T) -> Message + 'a>,
    on_open: Option<Message>,
    on_close: Option<Message>,
    options: L,
    placeholder: Option<String>,
    selected: Option<V>,
    width: Length,
    padding: Padding,
    text_size: Option<Pixels>,
    text_line_height: text::LineHeight,
    text_shaping: text::Shaping,
    font: Option<Renderer::Font>,
    handle: Handle<Renderer::Font>,
    class: <Theme as Catalog>::Class<'a>,
    menu_class: <Theme as menu::Catalog>::Class<'a>,
    last_status: Option<Status>,
    menu_height: Length,
}

impl<'a, T, L, V, Message, Theme, Renderer> PickList<'a, T, L, V, Message, Theme, Renderer>
where
    T: ToString + PartialEq + Clone,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Message: Clone,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// Creates a new [`PickList`] with the given list of options, the current
    /// selected value, and the message to produce when an option is selected.
    pub fn new(options: L, selected: Option<V>, on_select: impl Fn(T) -> Message + 'a) -> Self {
        Self {
            id: None,
            on_select: Box::new(on_select),
            on_open: None,
            on_close: None,
            options,
            placeholder: None,
            selected,
            width: Length::Shrink,
            padding: crate::button::DEFAULT_PADDING,
            text_size: None,
            text_line_height: text::LineHeight::default(),
            text_shaping: text::Shaping::default(),
            font: None,
            handle: Handle::default(),
            class: <Theme as Catalog>::default(),
            menu_class: <Theme as Catalog>::default_menu(),
            last_status: None,
            menu_height: Length::Shrink,
        }
    }

    /// Sets the [`Id`] of the [`PickList`].
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the placeholder of the [`PickList`].
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Sets the width of the [`PickList`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Menu`].
    pub fn menu_height(mut self, menu_height: impl Into<Length>) -> Self {
        self.menu_height = menu_height.into();
        self
    }

    /// Sets the [`Padding`] of the [`PickList`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size of the [`PickList`].
    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.text_size = Some(size.into());
        self
    }

    /// Sets the text [`text::LineHeight`] of the [`PickList`].
    pub fn text_line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.text_line_height = line_height.into();
        self
    }

    /// Sets the [`text::Shaping`] strategy of the [`PickList`].
    pub fn text_shaping(mut self, shaping: text::Shaping) -> Self {
        self.text_shaping = shaping;
        self
    }

    /// Sets the font of the [`PickList`].
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the [`Handle`] of the [`PickList`].
    pub fn handle(mut self, handle: Handle<Renderer::Font>) -> Self {
        self.handle = handle;
        self
    }

    /// Sets the message that will be produced when the [`PickList`] is opened.
    pub fn on_open(mut self, on_open: Message) -> Self {
        self.on_open = Some(on_open);
        self
    }

    /// Sets the message that will be produced when the [`PickList`] is closed.
    pub fn on_close(mut self, on_close: Message) -> Self {
        self.on_close = Some(on_close);
        self
    }

    /// Sets the style of the [`PickList`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        <Theme as Catalog>::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style of the [`Menu`].
    #[must_use]
    pub fn menu_style(mut self, style: impl Fn(&Theme) -> menu::Style + 'a) -> Self
    where
        <Theme as menu::Catalog>::Class<'a>: From<menu::StyleFn<'a, Theme>>,
    {
        self.menu_class = (Box::new(style) as menu::StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`PickList`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<<Theme as Catalog>::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Sets the style class of the [`Menu`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn menu_class(mut self, class: impl Into<<Theme as menu::Catalog>::Class<'a>>) -> Self {
        self.menu_class = class.into();
        self
    }
}

impl<'a, T, L, V, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for PickList<'a, T, L, V, Message, Theme, Renderer>
where
    T: Clone + ToString + PartialEq + 'a,
    L: Borrow<[T]>,
    V: Borrow<T>,
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: text::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph>::new())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
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

        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());
        let options = self.options.borrow();

        state.options.resize_with(options.len(), Default::default);

        let option_text = Text {
            content: "",
            bounds: Size::new(
                f32::INFINITY,
                self.text_line_height.to_absolute(text_size).into(),
            ),
            size: text_size,
            line_height: self.text_line_height,
            font,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Center,
            shaping: self.text_shaping,
            wrapping: text::Wrapping::default(),
            hint_factor: renderer.scale_factor(),
        };

        for (option, paragraph) in options.iter().zip(state.options.iter_mut()) {
            let label = option.to_string();

            let _ = paragraph.update(Text {
                content: &label,
                ..option_text
            });
        }

        if let Some(placeholder) = &self.placeholder {
            let _ = state.placeholder.update(Text {
                content: placeholder,
                ..option_text
            });
        }

        let max_width = match self.width {
            Length::Shrink => {
                let labels_width = state.options.iter().fold(0.0, |width, paragraph| {
                    f32::max(width, paragraph.min_width())
                });

                labels_width.max(
                    self.placeholder
                        .as_ref()
                        .map(|_| state.placeholder.min_width())
                        .unwrap_or(0.0),
                )
            }
            _ => 0.0,
        };

        let size = {
            let intrinsic = Size::new(
                max_width + text_size.0 + self.padding.left,
                f32::from(self.text_line_height.to_absolute(text_size)),
            );

            limits
                .width(self.width)
                .shrink(self.padding)
                .resolve(self.width, Length::Shrink, intrinsic)
                .expand(self.padding)
        };

        layout::Node::new(size)
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

        match event {
            Event::Mouse(mouse::Event::ButtonPressed {
                button: mouse::Button::Left,
                ..
            })
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if state.is_open {
                    // Event wasn't processed by overlay, so cursor was clicked either outside its
                    // bounds or on the drop-down, either way we close the overlay.
                    state.is_open = false;

                    if let Some(on_close) = &self.on_close {
                        shell.publish(on_close.clone());
                    }

                    shell.capture_event();
                } else if cursor.is_over(layout.bounds()) {
                    let selected = self.selected.as_ref().map(Borrow::borrow);

                    state.is_open = true;
                    state.hovered_option = self
                        .options
                        .borrow()
                        .iter()
                        .position(|option| Some(option) == selected);

                    if let Some(on_open) = &self.on_open {
                        shell.publish(on_open.clone());
                    }

                    shell.capture_event();
                } else {
                    // Unfocus when clicked outside
                    if state.is_focused {
                        state.is_focused = false;
                        shell.request_redraw();
                    }
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { y, .. },
                modifiers,
            }) => {
                if modifiers.command() && cursor.is_over(layout.bounds()) && !state.is_open {
                    fn find_next<'a, T: PartialEq>(
                        selected: &'a T,
                        mut options: impl Iterator<Item = &'a T>,
                    ) -> Option<&'a T> {
                        let _ = options.find(|&option| option == selected);

                        options.next()
                    }

                    let options = self.options.borrow();
                    let selected = self.selected.as_ref().map(Borrow::borrow);

                    let next_option = if *y < 0.0 {
                        if let Some(selected) = selected {
                            find_next(selected, options.iter())
                        } else {
                            options.first()
                        }
                    } else if *y > 0.0 {
                        if let Some(selected) = selected {
                            find_next(selected, options.iter().rev())
                        } else {
                            options.last()
                        }
                    } else {
                        None
                    };

                    if let Some(next_option) = next_option {
                        shell.publish((self.on_select)(next_option.clone()));
                    }

                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(key::Named::Space),
                ..
            }) => {
                if state.is_focused && !state.is_open {
                    let selected = self.selected.as_ref().map(Borrow::borrow);

                    state.is_open = true;
                    state.hovered_option = self
                        .options
                        .borrow()
                        .iter()
                        .position(|option| Some(option) == selected);

                    if let Some(on_open) = &self.on_open {
                        shell.publish(on_open.clone());
                    }

                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(key::Named::Enter),
                ..
            }) => {
                if state.is_focused {
                    if state.is_open {
                        // Select the hovered option
                        if let Some(index) = state.hovered_option {
                            if let Some(option) = self.options.borrow().get(index) {
                                shell.publish((self.on_select)(option.clone()));
                            }
                        }
                        state.is_open = false;

                        if let Some(on_close) = &self.on_close {
                            shell.publish(on_close.clone());
                        }

                        shell.capture_event();
                    } else {
                        // Open the menu
                        let selected = self.selected.as_ref().map(Borrow::borrow);

                        state.is_open = true;
                        state.hovered_option = self
                            .options
                            .borrow()
                            .iter()
                            .position(|option| Some(option) == selected);

                        if let Some(on_open) = &self.on_open {
                            shell.publish(on_open.clone());
                        }

                        shell.capture_event();
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(key::Named::ArrowUp),
                ..
            }) => {
                if state.is_focused && state.is_open {
                    let options_len = self.options.borrow().len();
                    if options_len > 0 {
                        state.hovered_option = Some(
                            state
                                .hovered_option
                                .map(|i| if i == 0 { options_len - 1 } else { i - 1 })
                                .unwrap_or(options_len - 1),
                        );
                        shell.request_redraw();
                    }
                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(key::Named::ArrowDown),
                ..
            }) => {
                if state.is_focused && state.is_open {
                    let options_len = self.options.borrow().len();
                    if options_len > 0 {
                        state.hovered_option = Some(
                            state
                                .hovered_option
                                .map(|i| (i + 1) % options_len)
                                .unwrap_or(0),
                        );
                        shell.request_redraw();
                    }
                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(key::Named::Escape),
                ..
            }) => {
                if state.is_focused && state.is_open {
                    state.is_open = false;

                    if let Some(on_close) = &self.on_close {
                        shell.publish(on_close.clone());
                    }

                    shell.capture_event();
                }
            }
            #[cfg(feature = "accessibility")]
            Event::Accessibility(accessibility_event) => {
                // Check if the event target matches this widget
                if let Some(id) = self.id.as_ref() {
                    if accessibility_event.target
                        != crate::core::accessibility::node_id_from_widget_id(id)
                    {
                        return;
                    }
                } else {
                    // Widget has no explicit ID - only respond if we're focused
                    if !state.is_focused {
                        return;
                    }
                }

                // Handle screen reader "click" action (open/close picker)
                if accessibility_event.is_click() {
                    if state.is_open {
                        state.is_open = false;
                        if let Some(on_close) = &self.on_close {
                            shell.publish(on_close.clone());
                        }
                    } else {
                        let selected = self.selected.as_ref().map(Borrow::borrow);
                        state.is_open = true;
                        state.hovered_option = self
                            .options
                            .borrow()
                            .iter()
                            .position(|option| Some(option) == selected);
                        if let Some(on_open) = &self.on_open {
                            shell.publish(on_open.clone());
                        }
                    }
                    shell.request_redraw();
                    shell.capture_event();
                }
                // Handle screen reader "focus" action
                if accessibility_event.is_focus() {
                    state.is_focused = true;
                    shell.request_redraw();
                    shell.capture_event();
                }
                // Handle screen reader "blur" action
                if accessibility_event.is_blur() {
                    state.is_focused = false;
                    state.is_open = false;
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            _ => {}
        };

        let status = {
            let is_hovered = cursor.is_over(layout.bounds());

            if state.is_open {
                Status::Opened { is_hovered }
            } else if is_hovered {
                Status::Hovered
            } else {
                Status::Active
            }
        };

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            self.last_status = Some(status);
            state.last_is_focused = state.is_focused;
        } else if self
            .last_status
            .is_some_and(|last_status| last_status != status)
            || state.last_is_focused != state.is_focused
        {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let is_mouse_over = cursor.is_over(bounds);

        if is_mouse_over {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
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
        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let selected = self.selected.as_ref().map(Borrow::borrow);
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();

        let bounds = layout.bounds();

        let style = Catalog::style(
            theme,
            &self.class,
            self.last_status.unwrap_or(Status::Active),
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background,
        );

        let handle = match &self.handle {
            Handle::Arrow { size } => Some((
                Renderer::ICON_FONT,
                Renderer::ARROW_DOWN_ICON,
                *size,
                text::LineHeight::default(),
                text::Shaping::Basic,
            )),
            Handle::Static(Icon {
                font,
                code_point,
                size,
                line_height,
                shaping,
            }) => Some((*font, *code_point, *size, *line_height, *shaping)),
            Handle::Dynamic { open, closed } => {
                if state.is_open {
                    Some((
                        open.font,
                        open.code_point,
                        open.size,
                        open.line_height,
                        open.shaping,
                    ))
                } else {
                    Some((
                        closed.font,
                        closed.code_point,
                        closed.size,
                        closed.line_height,
                        closed.shaping,
                    ))
                }
            }
            Handle::None => None,
        };

        if let Some((font, code_point, size, line_height, shaping)) = handle {
            let size = size.unwrap_or_else(|| renderer.default_size());
            let is_rtl = crate::core::layout_direction().is_rtl();

            // Handle position: right side for LTR, left side for RTL
            let (handle_x, handle_align) = if is_rtl {
                (bounds.x + self.padding.left, text::Alignment::Left)
            } else {
                (
                    bounds.x + bounds.width - self.padding.right,
                    text::Alignment::Right,
                )
            };

            renderer.fill_text(
                Text {
                    content: code_point.to_string(),
                    size,
                    line_height,
                    font,
                    bounds: Size::new(bounds.width, f32::from(line_height.to_absolute(size))),
                    align_x: handle_align,
                    align_y: alignment::Vertical::Center,
                    shaping,
                    wrapping: text::Wrapping::default(),
                    hint_factor: None,
                },
                Point::new(handle_x, bounds.center_y()),
                style.handle_color,
                *viewport,
            );
        }

        let label = selected.map(ToString::to_string);

        if let Some(label) = label.or_else(|| self.placeholder.clone()) {
            let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());
            let is_rtl = crate::core::layout_direction().is_rtl();

            // Label position: left side for LTR, right side for RTL
            let (label_x, label_align) = if is_rtl {
                (
                    bounds.x + bounds.width - self.padding.right,
                    text::Alignment::Right,
                )
            } else {
                (bounds.x + self.padding.left, text::Alignment::Default)
            };

            renderer.fill_text(
                Text {
                    content: label,
                    size: text_size,
                    line_height: self.text_line_height,
                    font,
                    bounds: Size::new(
                        bounds.width - self.padding.x(),
                        f32::from(self.text_line_height.to_absolute(text_size)),
                    ),
                    align_x: label_align,
                    align_y: alignment::Vertical::Center,
                    shaping: self.text_shaping,
                    wrapping: text::Wrapping::default(),
                    hint_factor: renderer.scale_factor(),
                },
                Point::new(label_x, bounds.center_y()),
                if selected.is_some() {
                    style.text_color
                } else {
                    style.placeholder_color
                },
                *viewport,
            );
        }

        // Draw focus ring when focused
        if state.is_focused {
            FocusRing::default().draw(renderer, bounds);
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        #[cfg(feature = "accessibility")]
        if let Some(info) = self.accessibility(tree, layout) {
            operation.accessibility(self.id.as_ref(), layout.bounds(), info);
        }

        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        operation.focusable(self.id.as_ref(), layout.bounds(), state);
    }

    #[cfg(feature = "accessibility")]
    fn accessibility(
        &self,
        tree: &crate::core::widget::Tree,
        layout: crate::core::Layout<'_>,
    ) -> Option<crate::core::accessibility::WidgetInfo> {
        use crate::core::accessibility::WidgetInfo;

        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();

        // Get the current value or placeholder text
        let value = self
            .selected
            .as_ref()
            .map(|s| s.borrow().to_string())
            .or_else(|| self.placeholder.clone())
            .unwrap_or_default();

        let mut info = WidgetInfo::pick_list(value).with_bounds(layout.bounds());

        // Set expanded state for screen readers
        info.expanded = Some(state.is_open);

        Some(info)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        let font = self.font.unwrap_or_else(|| renderer.default_font());

        if state.is_open {
            let bounds = layout.bounds();

            let on_select = &self.on_select;

            let mut menu = Menu::new(
                &mut state.menu,
                self.options.borrow(),
                &mut state.hovered_option,
                |option| {
                    state.is_open = false;

                    (on_select)(option)
                },
                None,
                &self.menu_class,
            )
            .width(bounds.width)
            .padding(self.padding)
            .font(font)
            .text_shaping(self.text_shaping);

            if let Some(text_size) = self.text_size {
                menu = menu.text_size(text_size);
            }

            Some(menu.overlay(
                layout.position() + translation,
                *viewport,
                bounds.height,
                self.menu_height,
            ))
        } else {
            None
        }
    }
}

impl<'a, T, L, V, Message, Theme, Renderer> From<PickList<'a, T, L, V, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    T: Clone + ToString + PartialEq + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: text::Renderer + 'a,
{
    fn from(pick_list: PickList<'a, T, L, V, Message, Theme, Renderer>) -> Self {
        Self::new(pick_list)
    }
}

#[derive(Debug)]
struct State<P: text::Paragraph> {
    menu: menu::State,
    is_open: bool,
    is_focused: bool,
    last_is_focused: bool,
    hovered_option: Option<usize>,
    options: Vec<paragraph::Plain<P>>,
    placeholder: paragraph::Plain<P>,
}

impl<P: text::Paragraph> State<P> {
    /// Creates a new [`State`] for a [`PickList`].
    fn new() -> Self {
        Self {
            menu: menu::State::default(),
            is_open: bool::default(),
            is_focused: false,
            last_is_focused: false,
            hovered_option: Option::default(),
            options: Vec::new(),
            placeholder: paragraph::Plain::default(),
        }
    }
}

impl<P: text::Paragraph> Default for State<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: text::Paragraph> operation::Focusable for State<P> {
    fn is_focused(&self) -> bool {
        self.is_focused
    }

    fn focus(&mut self) {
        self.is_focused = true;
    }

    fn unfocus(&mut self) {
        self.is_focused = false;
    }

    fn focus_tier(&self) -> operation::FocusTier {
        operation::FocusTier::Control
    }
}

/// The handle to the right side of the [`PickList`].
#[derive(Debug, Clone, PartialEq)]
pub enum Handle<Font> {
    /// Displays an arrow icon (▼).
    ///
    /// This is the default.
    Arrow {
        /// Font size of the content.
        size: Option<Pixels>,
    },
    /// A custom static handle.
    Static(Icon<Font>),
    /// A custom dynamic handle.
    Dynamic {
        /// The [`Icon`] used when [`PickList`] is closed.
        closed: Icon<Font>,
        /// The [`Icon`] used when [`PickList`] is open.
        open: Icon<Font>,
    },
    /// No handle will be shown.
    None,
}

impl<Font> Default for Handle<Font> {
    fn default() -> Self {
        Self::Arrow { size: None }
    }
}

/// The icon of a [`Handle`].
#[derive(Debug, Clone, PartialEq)]
pub struct Icon<Font> {
    /// Font that will be used to display the `code_point`,
    pub font: Font,
    /// The unicode code point that will be used as the icon.
    pub code_point: char,
    /// Font size of the content.
    pub size: Option<Pixels>,
    /// Line height of the content.
    pub line_height: text::LineHeight,
    /// The shaping strategy of the icon.
    pub shaping: text::Shaping,
}

/// The possible status of a [`PickList`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`PickList`] can be interacted with.
    Active,
    /// The [`PickList`] is being hovered.
    Hovered,
    /// The [`PickList`] is open.
    Opened {
        /// Whether the [`PickList`] is hovered, while open.
        is_hovered: bool,
    },
}

/// The appearance of a pick list.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The text [`Color`] of the pick list.
    pub text_color: Color,
    /// The placeholder [`Color`] of the pick list.
    pub placeholder_color: Color,
    /// The handle [`Color`] of the pick list.
    pub handle_color: Color,
    /// The [`Background`] of the pick list.
    pub background: Background,
    /// The [`Border`] of the pick list.
    pub border: Border,
}

/// The theme catalog of a [`PickList`].
pub trait Catalog: menu::Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> <Self as Catalog>::Class<'a>;

    /// The default class for the menu of the [`PickList`].
    fn default_menu<'a>() -> <Self as menu::Catalog>::Class<'a> {
        <Self as menu::Catalog>::default()
    }

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &<Self as Catalog>::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`PickList`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> StyleFn<'a, Self> {
        Box::new(default)
    }

    fn style(&self, class: &StyleFn<'_, Self>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default style of the field of a [`PickList`].
pub fn default(theme: &Theme, status: Status) -> Style {
    let active = Style {
        text_color: theme.background.on,
        background: theme.background.base.into(),
        placeholder_color: theme.background.on.scale_alpha(0.5),
        handle_color: theme.background.on,
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: theme.primary.divider,
        },
    };

    match status {
        Status::Active => active,
        Status::Hovered | Status::Opened { .. } => Style {
            border: Border {
                color: theme.accent.base,
                ..active.border
            },
            ..active
        },
    }
}
