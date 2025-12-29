//! A group of radio buttons with keyboard navigation.
//!
//! Radio groups allow users to select a single option from a list using
//! arrow keys for navigation within the group.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! #
//! use iced::widget::radio_group;
//!
//! struct State {
//!    selection: Option<Choice>,
//! }
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! enum Message {
//!     Selected(Choice),
//! }
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! enum Choice {
//!     A,
//!     B,
//!     C,
//! }
//!
//! impl std::fmt::Display for Choice {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         match self {
//!             Choice::A => write!(f, "Option A"),
//!             Choice::B => write!(f, "Option B"),
//!             Choice::C => write!(f, "Option C"),
//!         }
//!     }
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     radio_group(
//!         [Choice::A, Choice::B, Choice::C],
//!         state.selection,
//!         Message::Selected,
//!     )
//!     .into()
//! }
//! ```
use crate::core::alignment;
use crate::core::border::{self, Border};
use crate::core::keyboard;
use crate::core::keyboard::key::{self, Key};
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::text;
use crate::core::touch;
use crate::core::widget;
use crate::core::widget::Id as WidgetId;
use crate::core::widget::operation::{self, Operation};
use crate::core::widget::tree::{self, Tree};
use crate::core::window;
use crate::core::{
    Background, Clipboard, Color, Element, Event, Layout, Length, Pixels, Rectangle, Shell, Size,
    Theme, Widget,
};
use crate::focus::FocusRing;

/// A group of radio buttons that allows selecting a single option.
///
/// This widget handles keyboard navigation internally:
/// - **Tab**: Focus enters/leaves the group (single tab stop)
/// - **Arrow Up/Down**: Navigate between options within the group
/// - **Space/Enter**: Select the currently focused option
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// #
/// use iced::widget::radio_group;
///
/// struct State {
///    selection: Option<Choice>,
/// }
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// enum Message {
///     Selected(Choice),
/// }
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// enum Choice {
///     A,
///     B,
///     C,
/// }
///
/// impl std::fmt::Display for Choice {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         match self {
///             Choice::A => write!(f, "Option A"),
///             Choice::B => write!(f, "Option B"),
///             Choice::C => write!(f, "Option C"),
///         }
///     }
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     radio_group(
///         [Choice::A, Choice::B, Choice::C],
///         state.selection,
///         Message::Selected,
///     )
///     .into()
/// }
/// ```
pub struct RadioGroup<'a, T, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    T: Clone + Eq + ToString,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    id: Option<WidgetId>,
    options: Vec<T>,
    selected: Option<T>,
    on_select: Box<dyn Fn(T) -> Message + 'a>,
    width: Length,
    spacing: f32,
    radio_size: f32,
    radio_spacing: f32,
    text_size: Option<Pixels>,
    text_line_height: text::LineHeight,
    font: Option<Renderer::Font>,
    class: Theme::Class<'a>,
    last_status: Option<Status>,
}

impl<'a, T, Message, Theme, Renderer> RadioGroup<'a, T, Message, Theme, Renderer>
where
    T: Clone + Eq + ToString,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// The default size of radio buttons.
    pub const DEFAULT_RADIO_SIZE: f32 = 16.0;

    /// The default spacing between the radio and its label.
    pub const DEFAULT_RADIO_SPACING: f32 = 8.0;

    /// The default spacing between radio options.
    pub const DEFAULT_SPACING: f32 = 5.0;

    /// Creates a new [`RadioGroup`].
    ///
    /// It expects:
    ///   * the list of options
    ///   * the currently selected option (if any)
    ///   * a function that will be called when an option is selected
    pub fn new<F>(options: impl IntoIterator<Item = T>, selected: Option<T>, on_select: F) -> Self
    where
        F: Fn(T) -> Message + 'a,
    {
        RadioGroup {
            id: None,
            options: options.into_iter().collect(),
            selected,
            on_select: Box::new(on_select),
            width: Length::Shrink,
            spacing: Self::DEFAULT_SPACING,
            radio_size: Self::DEFAULT_RADIO_SIZE,
            radio_spacing: Self::DEFAULT_RADIO_SPACING,
            text_size: None,
            text_line_height: text::LineHeight::default(),
            font: None,
            class: Theme::default(),
            last_status: None,
        }
    }

    /// Sets the unique identifier of the [`RadioGroup`].
    pub fn id(mut self, id: impl Into<WidgetId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the width of the [`RadioGroup`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the spacing between options in the [`RadioGroup`].
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }

    /// Sets the size of the radio buttons.
    pub fn radio_size(mut self, size: impl Into<Pixels>) -> Self {
        self.radio_size = size.into().0;
        self
    }

    /// Sets the spacing between each radio button and its label.
    pub fn radio_spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.radio_spacing = spacing.into().0;
        self
    }

    /// Sets the text size of the labels.
    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.text_size = Some(size.into());
        self
    }

    /// Sets the text line height of the labels.
    pub fn text_line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.text_line_height = line_height.into();
        self
    }

    /// Sets the font of the labels.
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the style of the [`RadioGroup`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status, bool) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`RadioGroup`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

/// Internal state of a [`RadioGroup`].
pub struct State<P: text::Paragraph> {
    is_focused: bool,
    last_is_focused: bool,
    /// The index of the currently focused option (for keyboard navigation)
    focused_option: Option<usize>,
    paragraphs: Vec<widget::text::State<P>>,
}

impl<P: text::Paragraph> Default for State<P> {
    fn default() -> Self {
        Self {
            is_focused: false,
            last_is_focused: false,
            focused_option: None,
            paragraphs: Vec::new(),
        }
    }
}

impl<P: text::Paragraph> operation::Focusable for State<P> {
    fn is_focused(&self) -> bool {
        self.is_focused
    }

    fn focus(&mut self) {
        self.is_focused = true;
        // When gaining focus, focus the selected option or the first one
        if self.focused_option.is_none() {
            self.focused_option = Some(0);
        }
    }

    fn unfocus(&mut self) {
        self.is_focused = false;
    }

    fn focus_tier(&self) -> operation::FocusTier {
        operation::FocusTier::Control
    }
}

impl<T, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for RadioGroup<'_, T, Message, Theme, Renderer>
where
    T: Clone + Eq + ToString,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph>::default())
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

        // Ensure we have enough paragraph states
        while state.paragraphs.len() < self.options.len() {
            state.paragraphs.push(widget::text::State::default());
        }

        // If selected option exists, set focused_option to it when first laid out
        if state.focused_option.is_none() && !self.options.is_empty() {
            if let Some(ref selected) = self.selected {
                state.focused_option = self.options.iter().position(|o| o == selected);
            }
            if state.focused_option.is_none() {
                state.focused_option = Some(0);
            }
        }

        let limits = limits.width(self.width);
        let _text_size = self.text_size.unwrap_or_else(|| renderer.default_size()).0;

        let mut children = Vec::with_capacity(self.options.len());
        let mut total_height = 0.0f32;
        let mut max_width = 0.0f32;

        for (i, option) in self.options.iter().enumerate() {
            let label = option.to_string();

            // Layout: radio circle + spacing + text
            let radio_node = layout::Node::new(Size::new(self.radio_size, self.radio_size));

            let text_limits = limits.shrink(Size::new(self.radio_size + self.radio_spacing, 0.0));

            let paragraph = &mut state.paragraphs[i];
            let text_node = widget::text::layout(
                paragraph,
                renderer,
                &text_limits,
                &label,
                widget::text::Format {
                    width: Length::Shrink,
                    height: Length::Shrink,
                    line_height: self.text_line_height,
                    size: self.text_size,
                    font: self.font,
                    align_x: text::Alignment::Default,
                    align_y: alignment::Vertical::Center,
                    shaping: text::Shaping::default(),
                    wrapping: text::Wrapping::default(),
                },
            );

            let row_height = self.radio_size.max(text_node.size().height);
            let row_width = self.radio_size + self.radio_spacing + text_node.size().width;

            // Position radio and text within the row
            let radio_y = (row_height - self.radio_size) / 2.0;
            let text_y = (row_height - text_node.size().height) / 2.0;

            let row_node = layout::Node::with_children(
                Size::new(row_width, row_height),
                vec![
                    radio_node.move_to(crate::core::Point::new(0.0, radio_y)),
                    text_node.move_to(crate::core::Point::new(
                        self.radio_size + self.radio_spacing,
                        text_y,
                    )),
                ],
            )
            .move_to(crate::core::Point::new(0.0, total_height));

            max_width = max_width.max(row_width);
            total_height += row_height;

            if i < self.options.len() - 1 {
                total_height += self.spacing;
            }

            children.push(row_node);
        }

        layout::Node::with_children(Size::new(max_width, total_height), children)
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
        let option_count = self.options.len();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed {
                button: mouse::Button::Left,
                ..
            })
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                // Check which option was clicked (don't set focus on mouse click)
                for (i, child) in layout.children().enumerate() {
                    if cursor.is_over(child.bounds()) {
                        if let Some(option) = self.options.get(i) {
                            shell.publish((self.on_select)(option.clone()));
                            shell.capture_event();
                            break;
                        }
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(key::Named::Space | key::Named::Enter),
                ..
            }) => {
                if state.is_focused {
                    if let Some(focused_idx) = state.focused_option {
                        if let Some(option) = self.options.get(focused_idx) {
                            shell.publish((self.on_select)(option.clone()));
                            shell.capture_event();
                        }
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(key::Named::ArrowUp),
                ..
            }) => {
                if state.is_focused && option_count > 0 {
                    let current = state.focused_option.unwrap_or(0);
                    let new_idx = if current == 0 {
                        option_count - 1
                    } else {
                        current - 1
                    };
                    state.focused_option = Some(new_idx);
                    // Only move focus, don't select - use Space/Enter to select
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(key::Named::ArrowDown),
                ..
            }) => {
                if state.is_focused && option_count > 0 {
                    let current = state.focused_option.unwrap_or(0);
                    let new_idx = if current >= option_count - 1 {
                        0
                    } else {
                        current + 1
                    };
                    state.focused_option = Some(new_idx);
                    // Only move focus, don't select - use Space/Enter to select
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            _ => {}
        }

        // Don't sync focused_option with selection - they are independent now
        // Focus can be on a different option than the selected one

        // Status tracking for redraw
        let current_status = {
            let is_mouse_over = cursor.is_over(layout.bounds());
            if is_mouse_over {
                Status::Hovered
            } else {
                Status::Active
            }
        };

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            self.last_status = Some(current_status);
            state.last_is_focused = state.is_focused;
        } else if self
            .last_status
            .is_some_and(|last_status| last_status != current_status)
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
        if cursor.is_over(layout.bounds()) {
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
        defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();

        for (i, (option, row_layout)) in self.options.iter().zip(layout.children()).enumerate() {
            let is_selected = self.selected.as_ref() == Some(option);
            let is_focused_option = state.is_focused && state.focused_option == Some(i);

            let status = if is_focused_option {
                Status::Hovered
            } else {
                Status::Active
            };

            let style = theme.style(&self.class, status, is_selected);

            let mut children = row_layout.children();
            let radio_layout = children.next().unwrap();
            let text_layout = children.next().unwrap();

            let radio_bounds = radio_layout.bounds();

            // Draw radio circle background
            renderer.fill_quad(
                renderer::Quad {
                    bounds: radio_bounds,
                    border: Border {
                        radius: (radio_bounds.width / 2.0).into(),
                        width: style.border_width,
                        color: style.border_color,
                    },
                    ..renderer::Quad::default()
                },
                style.background,
            );

            // Draw focus ring on the focused option
            if is_focused_option {
                FocusRing::default().draw(renderer, radio_bounds);
            }

            // Draw selection dot
            if is_selected {
                let dot_size = radio_bounds.width / 2.0;
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: radio_bounds.x + dot_size / 2.0,
                            y: radio_bounds.y + dot_size / 2.0,
                            width: radio_bounds.width - dot_size,
                            height: radio_bounds.height - dot_size,
                        },
                        border: border::rounded(dot_size / 2.0),
                        ..renderer::Quad::default()
                    },
                    style.dot_color,
                );
            }

            // Draw label text
            if let Some(paragraph) = state.paragraphs.get(i) {
                crate::text::draw(
                    renderer,
                    defaults,
                    text_layout.bounds(),
                    paragraph.raw(),
                    crate::text::Style {
                        color: style.text_color,
                    },
                    viewport,
                );
            }
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        operation.focusable(self.id.as_ref(), layout.bounds(), state);
    }
}

impl<'a, T, Message, Theme, Renderer> From<RadioGroup<'a, T, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    T: 'a + Clone + Eq + ToString,
    Message: 'a,
    Theme: 'a + Catalog,
    Renderer: 'a + text::Renderer,
{
    fn from(
        radio_group: RadioGroup<'a, T, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(radio_group)
    }
}

/// The possible status of a [`RadioGroup`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`RadioGroup`] can be interacted with.
    Active,
    /// An option in the [`RadioGroup`] is being hovered or focused.
    Hovered,
}

/// The appearance of a radio button in the group.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The [`Background`] of the radio button.
    pub background: Background,
    /// The [`Color`] of the dot when selected.
    pub dot_color: Color,
    /// The border width of the radio button.
    pub border_width: f32,
    /// The border [`Color`] of the radio button.
    pub border_color: Color,
    /// The text [`Color`] of the label.
    pub text_color: Option<Color>,
}

/// The theme catalog of a [`RadioGroup`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status, is_selected: bool) -> Style;
}

/// A styling function for a [`RadioGroup`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status, bool) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status, is_selected: bool) -> Style {
        class(self, status, is_selected)
    }
}

/// The default style of a [`RadioGroup`].
pub fn default(theme: &Theme, status: Status, _is_selected: bool) -> Style {
    let active = Style {
        background: Color::TRANSPARENT.into(),
        dot_color: theme.accent.base,
        border_width: 1.0,
        border_color: theme.accent.base,
        text_color: None,
    };

    match status {
        Status::Active => active,
        Status::Hovered => Style {
            dot_color: theme.accent.hover,
            background: theme.accent.hover.scale_alpha(0.2).into(),
            ..active
        },
    }
}
