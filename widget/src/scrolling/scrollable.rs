//! Scrollables let users navigate an endless amount of content with a scrollbar.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } }
//! # pub type State = ();
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! use iced::widget::{column, scrollable, space};
//!
//! enum Message {
//!     // ...
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     scrollable(column![
//!         "Scroll me!",
//!         space().height(3000),
//!         "You did it!",
//!     ]).into()
//! }
//! ```
use crate::container;
use crate::core::alignment;
use crate::core::border::{self, Border};
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::text;
use crate::core::time::{Duration, Instant};
use crate::core::touch;
use crate::core::widget;
use crate::core::widget::operation::{self, Operation};
use crate::core::widget::tree::{self, Tree};
use crate::core::window;
use crate::core::{
    self, Animation, Background, Clipboard, Color, Element, Event, InputMethod, Layout, Length,
    Padding, Pixels, Point, Rectangle, Shadow, Shell, Size, Theme, Vector, Widget,
};

pub use operation::scrollable::{AbsoluteOffset, RelativeOffset};

/// A widget that can vertically display an infinite amount of content with a
/// scrollbar.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::{column, scrollable, space};
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     scrollable(column![
///         "Scroll me!",
///         space().height(3000),
///         "You did it!",
///     ]).into()
/// }
/// ```
pub struct Scrollable<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    id: Option<widget::Id>,
    width: Length,
    height: Length,
    direction: Direction,
    auto_scroll: bool,
    content: Element<'a, Message, Theme, Renderer>,
    on_scroll: Option<Box<dyn Fn(Viewport) -> Message + 'a>>,
    class: Theme::Class<'a>,
    last_status: Option<Status>,
}

impl<'a, Message, Theme, Renderer> Scrollable<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// Creates a new vertical [`Scrollable`].
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self::with_direction(content, Direction::default())
    }

    /// Creates a new [`Scrollable`] with the given [`Direction`].
    pub fn with_direction(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        direction: impl Into<Direction>,
    ) -> Self {
        Scrollable {
            id: None,
            width: Length::Shrink,
            height: Length::Shrink,
            direction: direction.into(),
            auto_scroll: false,
            content: content.into(),
            on_scroll: None,
            class: Theme::default(),
            last_status: None,
        }
        .enclose()
    }

    fn enclose(mut self) -> Self {
        let size_hint = self.content.as_widget().size_hint();

        if self.direction.horizontal().is_none() {
            self.width = self.width.enclose(size_hint.width);
        }

        if self.direction.vertical().is_none() {
            self.height = self.height.enclose(size_hint.height);
        }

        self
    }

    /// Makes the [`Scrollable`] scroll horizontally, with default [`Scrollbar`] settings.
    pub fn horizontal(self) -> Self {
        self.direction(Direction::Horizontal(Scrollbar::default()))
    }

    /// Sets the [`Direction`] of the [`Scrollable`].
    pub fn direction(mut self, direction: impl Into<Direction>) -> Self {
        self.direction = direction.into();
        self.enclose()
    }

    /// Sets the [`widget::Id`] of the [`Scrollable`].
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the width of the [`Scrollable`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Scrollable`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets a function to call when the [`Scrollable`] is scrolled.
    ///
    /// The function takes the [`Viewport`] of the [`Scrollable`]
    pub fn on_scroll(mut self, f: impl Fn(Viewport) -> Message + 'a) -> Self {
        self.on_scroll = Some(Box::new(f));
        self
    }

    /// Anchors the vertical [`Scrollable`] direction to the top.
    pub fn anchor_top(self) -> Self {
        self.anchor_y(Anchor::Start)
    }

    /// Anchors the vertical [`Scrollable`] direction to the bottom.
    pub fn anchor_bottom(self) -> Self {
        self.anchor_y(Anchor::End)
    }

    /// Anchors the horizontal [`Scrollable`] direction to the left.
    pub fn anchor_left(self) -> Self {
        self.anchor_x(Anchor::Start)
    }

    /// Anchors the horizontal [`Scrollable`] direction to the right.
    pub fn anchor_right(self) -> Self {
        self.anchor_x(Anchor::End)
    }

    /// Sets the [`Anchor`] of the horizontal direction of the [`Scrollable`], if applicable.
    pub fn anchor_x(mut self, alignment: Anchor) -> Self {
        match &mut self.direction {
            Direction::Horizontal(horizontal) | Direction::Both { horizontal, .. } => {
                horizontal.alignment = alignment;
            }
            Direction::Vertical { .. } => {}
        }

        self
    }

    /// Sets the [`Anchor`] of the vertical direction of the [`Scrollable`], if applicable.
    pub fn anchor_y(mut self, alignment: Anchor) -> Self {
        match &mut self.direction {
            Direction::Vertical(vertical) | Direction::Both { vertical, .. } => {
                vertical.alignment = alignment;
            }
            Direction::Horizontal { .. } => {}
        }

        self
    }

    /// Embeds the [`Scrollbar`] into the [`Scrollable`], instead of floating on top of the
    /// content.
    ///
    /// The `spacing` provided will be used as space between the [`Scrollbar`] and the contents
    /// of the [`Scrollable`].
    pub fn spacing(mut self, new_spacing: impl Into<Pixels>) -> Self {
        match &mut self.direction {
            Direction::Horizontal(scrollbar) | Direction::Vertical(scrollbar) => {
                scrollbar.spacing = Some(new_spacing.into().0);
            }
            Direction::Both { .. } => {}
        }

        self
    }

    /// Sets whether the user should be allowed to auto-scroll the [`Scrollable`]
    /// with the middle mouse button.
    ///
    /// By default, it is disabled.
    pub fn auto_scroll(mut self, auto_scroll: bool) -> Self {
        self.auto_scroll = auto_scroll;
        self
    }

    /// Sets the style of this [`Scrollable`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Scrollable`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

/// The direction of [`Scrollable`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    /// Vertical scrolling
    Vertical(Scrollbar),
    /// Horizontal scrolling
    Horizontal(Scrollbar),
    /// Both vertical and horizontal scrolling
    Both {
        /// The properties of the vertical scrollbar.
        vertical: Scrollbar,
        /// The properties of the horizontal scrollbar.
        horizontal: Scrollbar,
    },
}

impl Direction {
    /// Returns the horizontal [`Scrollbar`], if any.
    pub fn horizontal(&self) -> Option<&Scrollbar> {
        match self {
            Self::Horizontal(scrollbar) => Some(scrollbar),
            Self::Both { horizontal, .. } => Some(horizontal),
            Self::Vertical(_) => None,
        }
    }

    /// Returns the vertical [`Scrollbar`], if any.
    pub fn vertical(&self) -> Option<&Scrollbar> {
        match self {
            Self::Vertical(scrollbar) => Some(scrollbar),
            Self::Both { vertical, .. } => Some(vertical),
            Self::Horizontal(_) => None,
        }
    }

    /// Aligns a scroll delta according to the anchor configuration.
    pub fn align(&self, delta: Vector) -> Vector {
        let horizontal_alignment = self.horizontal().map(|p| p.alignment).unwrap_or_default();

        let vertical_alignment = self.vertical().map(|p| p.alignment).unwrap_or_default();

        let align = |alignment: Anchor, delta: f32| match alignment {
            Anchor::Start => delta,
            Anchor::End => -delta,
        };

        Vector::new(
            align(horizontal_alignment, delta.x),
            align(vertical_alignment, delta.y),
        )
    }
}

impl Default for Direction {
    fn default() -> Self {
        Self::Vertical(Scrollbar::default())
    }
}

/// A scrollbar within a [`Scrollable`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scrollbar {
    /// The width of the scrollbar.
    pub(crate) width: f32,
    /// The margin around the scrollbar.
    pub(crate) margin: f32,
    /// The width of the scroller (the draggable part).
    pub(crate) scroller_width: f32,
    /// The alignment/anchor of the scrollbar.
    pub alignment: Anchor,
    /// The spacing when embedded.
    pub(crate) spacing: Option<f32>,
}

impl Default for Scrollbar {
    fn default() -> Self {
        Self {
            width: 10.0,
            margin: 0.0,
            scroller_width: 10.0,
            alignment: Anchor::Start,
            spacing: None,
        }
    }
}

impl Scrollbar {
    /// Creates new [`Scrollbar`] for use in a [`Scrollable`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a [`Scrollbar`] with zero width to allow a [`Scrollable`] to scroll without a visible
    /// scroller.
    pub fn hidden() -> Self {
        Self::default().width(0).scroller_width(0)
    }

    /// Sets the scrollbar width of the [`Scrollbar`] .
    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = width.into().0.max(0.0);
        self
    }

    /// Sets the scrollbar margin of the [`Scrollbar`] .
    pub fn margin(mut self, margin: impl Into<Pixels>) -> Self {
        self.margin = margin.into().0;
        self
    }

    /// Sets the scroller width of the [`Scrollbar`] .
    pub fn scroller_width(mut self, scroller_width: impl Into<Pixels>) -> Self {
        self.scroller_width = scroller_width.into().0.max(0.0);
        self
    }

    /// Sets the [`Anchor`] of the [`Scrollbar`] .
    pub fn anchor(mut self, alignment: Anchor) -> Self {
        self.alignment = alignment;
        self
    }

    /// Sets whether the [`Scrollbar`] should be embedded in the [`Scrollable`], using
    /// the given spacing between itself and the contents.
    ///
    /// An embedded [`Scrollbar`] will always be displayed, will take layout space,
    /// and will not float over the contents.
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = Some(spacing.into().0);
        self
    }
}

/// The anchor of the scroller of the [`Scrollable`] relative to its [`Viewport`]
/// on a given axis.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Anchor {
    /// Scroller is anchoer to the start of the [`Viewport`].
    #[default]
    Start,
    /// Content is aligned to the end of the [`Viewport`].
    End,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Scrollable<'_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let mut layout = |right_padding, bottom_padding| {
            layout::padded(
                limits,
                self.width,
                self.height,
                Padding {
                    right: right_padding,
                    bottom: bottom_padding,
                    ..Padding::ZERO
                },
                |limits| {
                    let is_horizontal = self.direction.horizontal().is_some();
                    let is_vertical = self.direction.vertical().is_some();

                    let child_limits = layout::Limits::with_compression(
                        limits.min(),
                        Size::new(
                            if is_horizontal {
                                f32::INFINITY
                            } else {
                                limits.max().width
                            },
                            if is_vertical {
                                f32::INFINITY
                            } else {
                                limits.max().height
                            },
                        ),
                        Size::new(is_horizontal, is_vertical),
                    );

                    self.content.as_widget_mut().layout(
                        &mut tree.children[0],
                        renderer,
                        &child_limits,
                    )
                },
            )
        };

        match self.direction {
            Direction::Vertical(Scrollbar {
                width,
                margin,
                spacing: Some(spacing),
                ..
            })
            | Direction::Horizontal(Scrollbar {
                width,
                margin,
                spacing: Some(spacing),
                ..
            }) => {
                let is_vertical = matches!(self.direction, Direction::Vertical(_));

                let padding = width + margin * 2.0 + spacing;
                let state = tree.state.downcast_mut::<State>();

                let status_quo = layout(
                    if is_vertical && state.is_scrollbar_visible {
                        padding
                    } else {
                        0.0
                    },
                    if !is_vertical && state.is_scrollbar_visible {
                        padding
                    } else {
                        0.0
                    },
                );

                let is_scrollbar_visible = if is_vertical {
                    status_quo.children()[0].size().height > status_quo.size().height
                } else {
                    status_quo.children()[0].size().width > status_quo.size().width
                };

                if state.is_scrollbar_visible == is_scrollbar_visible {
                    status_quo
                } else {
                    log::trace!("Scrollbar status quo has changed");
                    state.is_scrollbar_visible = is_scrollbar_visible;

                    layout(
                        if is_vertical && state.is_scrollbar_visible {
                            padding
                        } else {
                            0.0
                        },
                        if !is_vertical && state.is_scrollbar_visible {
                            padding
                        } else {
                            0.0
                        },
                    )
                }
            }
            _ => layout(0.0, 0.0),
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<State>();

        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();
        let content_bounds = content_layout.bounds();
        let translation = state.translation(self.direction, bounds, content_bounds);

        operation.scrollable(self.id.as_ref(), bounds, content_bounds, translation, state);

        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
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
        _viewport: &Rectangle,
    ) {
        const AUTOSCROLL_DEADZONE: f32 = 20.0;
        const AUTOSCROLL_SMOOTHNESS: f32 = 1.5;

        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let cursor_over_scrollable = cursor.position_over(bounds);

        let content = layout.children().next().unwrap();
        let content_bounds = content.bounds();

        let scrollbars = Scrollbars::new(state, self.direction, bounds, content_bounds);

        let (mouse_over_y_scrollbar, mouse_over_x_scrollbar) = scrollbars.is_mouse_over(cursor);

        let last_offsets = (state.offset_x, state.offset_y);

        if let Some(last_scrolled) = state.last_scrolled {
            let clear_transaction = match event {
                Event::Mouse(
                    mouse::Event::ButtonPressed { .. }
                    | mouse::Event::ButtonReleased { .. }
                    | mouse::Event::CursorLeft,
                ) => true,
                Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                    last_scrolled.elapsed() > Duration::from_millis(100)
                }
                _ => last_scrolled.elapsed() > Duration::from_millis(1500),
            };

            if clear_transaction {
                state.last_scrolled = None;
            }
        }

        let mut update = || {
            if let Some(scroller_grabbed_at) = state.y_scroller_grabbed_at() {
                match event {
                    Event::Mouse(mouse::Event::CursorMoved { .. })
                    | Event::Touch(touch::Event::FingerMoved { .. }) => {
                        if let Some(scrollbar) = scrollbars.y {
                            let Some(cursor_position) = cursor.land().position() else {
                                return;
                            };

                            state.scroll_y_to(
                                scrollbar.scroll_percentage_y(scroller_grabbed_at, cursor_position),
                                bounds,
                                content_bounds,
                            );

                            let _ = notify_scroll(
                                state,
                                &self.on_scroll,
                                bounds,
                                content_bounds,
                                shell,
                            );

                            shell.capture_event();
                        }
                    }
                    _ => {}
                }
            } else if mouse_over_y_scrollbar {
                match event {
                    Event::Mouse(mouse::Event::ButtonPressed {
                        button: mouse::Button::Left,
                        ..
                    })
                    | Event::Touch(touch::Event::FingerPressed { .. }) => {
                        let Some(cursor_position) = cursor.position() else {
                            return;
                        };

                        if let (Some(scroller_grabbed_at), Some(scrollbar)) =
                            (scrollbars.grab_y_scroller(cursor_position), scrollbars.y)
                        {
                            state.scroll_y_to(
                                scrollbar.scroll_percentage_y(scroller_grabbed_at, cursor_position),
                                bounds,
                                content_bounds,
                            );

                            state.interaction = Interaction::YScrollerGrabbed(scroller_grabbed_at);

                            let _ = notify_scroll(
                                state,
                                &self.on_scroll,
                                bounds,
                                content_bounds,
                                shell,
                            );
                        }

                        shell.capture_event();
                    }
                    _ => {}
                }
            }

            if let Some(scroller_grabbed_at) = state.x_scroller_grabbed_at() {
                match event {
                    Event::Mouse(mouse::Event::CursorMoved { .. })
                    | Event::Touch(touch::Event::FingerMoved { .. }) => {
                        let Some(cursor_position) = cursor.land().position() else {
                            return;
                        };

                        if let Some(scrollbar) = scrollbars.x {
                            state.scroll_x_to(
                                scrollbar.scroll_percentage_x(scroller_grabbed_at, cursor_position),
                                bounds,
                                content_bounds,
                            );

                            let _ = notify_scroll(
                                state,
                                &self.on_scroll,
                                bounds,
                                content_bounds,
                                shell,
                            );
                        }

                        shell.capture_event();
                    }
                    _ => {}
                }
            } else if mouse_over_x_scrollbar {
                match event {
                    Event::Mouse(mouse::Event::ButtonPressed {
                        button: mouse::Button::Left,
                        ..
                    })
                    | Event::Touch(touch::Event::FingerPressed { .. }) => {
                        let Some(cursor_position) = cursor.position() else {
                            return;
                        };

                        if let (Some(scroller_grabbed_at), Some(scrollbar)) =
                            (scrollbars.grab_x_scroller(cursor_position), scrollbars.x)
                        {
                            state.scroll_x_to(
                                scrollbar.scroll_percentage_x(scroller_grabbed_at, cursor_position),
                                bounds,
                                content_bounds,
                            );

                            state.interaction = Interaction::XScrollerGrabbed(scroller_grabbed_at);

                            let _ = notify_scroll(
                                state,
                                &self.on_scroll,
                                bounds,
                                content_bounds,
                                shell,
                            );

                            shell.capture_event();
                        }
                    }
                    _ => {}
                }
            }

            if matches!(state.interaction, Interaction::AutoScrolling { .. })
                && matches!(
                    event,
                    Event::Mouse(
                        mouse::Event::ButtonPressed { .. } | mouse::Event::WheelScrolled { .. }
                    ) | Event::Touch(_)
                        | Event::Keyboard(_)
                )
            {
                state.interaction = Interaction::None;
                shell.capture_event();
                shell.invalidate_layout();
                shell.request_redraw();
                return;
            }

            if state.last_scrolled.is_none()
                || !matches!(event, Event::Mouse(mouse::Event::WheelScrolled { .. }))
            {
                let translation = state.translation(self.direction, bounds, content_bounds);

                let cursor = match cursor_over_scrollable {
                    Some(cursor_position)
                        if !(mouse_over_x_scrollbar || mouse_over_y_scrollbar) =>
                    {
                        mouse::Cursor::Available(cursor_position + translation)
                    }
                    _ => cursor.levitate() + translation,
                };

                let had_input_method = shell.input_method().is_enabled();

                self.content.as_widget_mut().update(
                    &mut tree.children[0],
                    event,
                    content,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    &Rectangle {
                        y: bounds.y + translation.y,
                        x: bounds.x + translation.x,
                        ..bounds
                    },
                );

                if !had_input_method
                    && let InputMethod::Enabled { cursor, .. } = shell.input_method_mut()
                {
                    *cursor = *cursor - translation;
                }
            };

            if matches!(
                event,
                Event::Mouse(mouse::Event::ButtonReleased {
                    button: mouse::Button::Left,
                    ..
                }) | Event::Touch(
                    touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. }
                )
            ) {
                // Start kinetic scrolling if we were touch scrolling
                if matches!(state.interaction, Interaction::TouchScrolling { .. }) {
                    // Apply direction constraints to velocity
                    state.velocity = self.direction.align(state.velocity);
                    state.last_kinetic_update = Some(Instant::now());
                    // Request redraw to animate kinetic scrolling
                    if state.is_kinetic_active() {
                        shell.request_redraw();
                    }
                }
                state.interaction = Interaction::None;
                return;
            }

            if shell.is_event_captured() {
                return;
            }

            match event {
                Event::Mouse(mouse::Event::WheelScrolled { delta, modifiers }) => {
                    if cursor_over_scrollable.is_none() {
                        return;
                    }

                    // Stop kinetic scrolling and animated scroll-to when user scrolls with wheel
                    state.velocity = Vector::new(0.0, 0.0);
                    state.scroll_to_target = None;
                    state.scroll_to_animation = None;

                    let delta = match *delta {
                        mouse::ScrollDelta::Lines { x, y } => {
                            let is_shift_pressed = modifiers.shift();

                            // macOS automatically inverts the axes when Shift is pressed
                            let (x, y) = if cfg!(target_os = "macos") && is_shift_pressed {
                                (y, x)
                            } else {
                                (x, y)
                            };

                            let movement = if !is_shift_pressed {
                                Vector::new(x, y)
                            } else {
                                Vector::new(y, x)
                            };

                            // TODO: Configurable speed/friction (?)
                            -movement * 60.0
                        }
                        mouse::ScrollDelta::Pixels { x, y } => -Vector::new(x, y),
                    };

                    state.scroll(self.direction.align(delta), bounds, content_bounds);

                    let has_scrolled =
                        notify_scroll(state, &self.on_scroll, bounds, content_bounds, shell);

                    let in_transaction = state.last_scrolled.is_some();

                    if has_scrolled || in_transaction {
                        shell.capture_event();
                    }
                }
                Event::Mouse(mouse::Event::ButtonPressed {
                    button: mouse::Button::Middle,
                    ..
                }) if self.auto_scroll && matches!(state.interaction, Interaction::None) => {
                    let Some(origin) = cursor_over_scrollable else {
                        return;
                    };

                    state.interaction = Interaction::AutoScrolling {
                        origin,
                        current: origin,
                        last_frame: None,
                    };

                    shell.capture_event();
                    shell.invalidate_layout();
                    shell.request_redraw();
                }
                Event::Touch(event)
                    if matches!(state.interaction, Interaction::TouchScrolling { .. })
                        || (!mouse_over_y_scrollbar && !mouse_over_x_scrollbar) =>
                {
                    match event {
                        touch::Event::FingerPressed { .. } => {
                            let Some(position) = cursor_over_scrollable else {
                                return;
                            };

                            // Stop any kinetic scrolling
                            state.velocity = Vector::new(0.0, 0.0);
                            state.last_kinetic_update = None;
                            // Cancel any animated scroll
                            state.scroll_to_target = None;
                            state.scroll_to_animation = None;

                            state.interaction = Interaction::TouchScrolling {
                                last_position: position,
                                last_time: Instant::now(),
                            };
                        }
                        touch::Event::FingerMoved { .. } => {
                            let Interaction::TouchScrolling {
                                last_position,
                                last_time,
                            } = state.interaction
                            else {
                                return;
                            };

                            let Some(cursor_position) = cursor.position() else {
                                return;
                            };

                            let now = Instant::now();
                            let dt = (now - last_time).as_secs_f32().max(0.001);

                            let delta = Vector::new(
                                last_position.x - cursor_position.x,
                                last_position.y - cursor_position.y,
                            );

                            // Update velocity with exponential smoothing
                            let instant_velocity = Vector::new(-delta.x / dt, -delta.y / dt);
                            let smoothing = 0.3; // Lower = smoother
                            state.velocity.x = state.velocity.x * (1.0 - smoothing)
                                + instant_velocity.x * smoothing;
                            state.velocity.y = state.velocity.y * (1.0 - smoothing)
                                + instant_velocity.y * smoothing;

                            state.scroll(self.direction.align(delta), bounds, content_bounds);

                            state.interaction = Interaction::TouchScrolling {
                                last_position: cursor_position,
                                last_time: now,
                            };

                            // TODO: bubble up touch movements if not consumed.
                            let _ = notify_scroll(
                                state,
                                &self.on_scroll,
                                bounds,
                                content_bounds,
                                shell,
                            );
                        }
                        _ => {}
                    }

                    shell.capture_event();
                }
                Event::Mouse(mouse::Event::CursorMoved { position, .. }) => {
                    if let Interaction::AutoScrolling {
                        origin, last_frame, ..
                    } = state.interaction
                    {
                        let delta = *position - origin;

                        state.interaction = Interaction::AutoScrolling {
                            origin,
                            current: *position,
                            last_frame,
                        };

                        if (delta.x.abs() >= AUTOSCROLL_DEADZONE
                            || delta.y.abs() >= AUTOSCROLL_DEADZONE)
                            && last_frame.is_none()
                        {
                            shell.request_redraw();
                        }
                    }
                }
                Event::Window(window::Event::RedrawRequested(now)) => {
                    if let Interaction::AutoScrolling {
                        origin,
                        current,
                        last_frame,
                    } = state.interaction
                    {
                        if last_frame == Some(*now) {
                            shell.request_redraw();
                            return;
                        }

                        state.interaction = Interaction::AutoScrolling {
                            origin,
                            current,
                            last_frame: None,
                        };

                        let mut delta = current - origin;

                        if delta.x.abs() < AUTOSCROLL_DEADZONE {
                            delta.x = 0.0;
                        }

                        if delta.y.abs() < AUTOSCROLL_DEADZONE {
                            delta.y = 0.0;
                        }

                        if delta.x != 0.0 || delta.y != 0.0 {
                            let time_delta = if let Some(last_frame) = last_frame {
                                *now - last_frame
                            } else {
                                Duration::ZERO
                            };

                            let scroll_factor = time_delta.as_secs_f32();

                            state.scroll(
                                self.direction.align(Vector::new(
                                    delta.x.signum()
                                        * delta.x.abs().powf(AUTOSCROLL_SMOOTHNESS)
                                        * scroll_factor,
                                    delta.y.signum()
                                        * delta.y.abs().powf(AUTOSCROLL_SMOOTHNESS)
                                        * scroll_factor,
                                )),
                                bounds,
                                content_bounds,
                            );

                            let has_scrolled = notify_scroll(
                                state,
                                &self.on_scroll,
                                bounds,
                                content_bounds,
                                shell,
                            );

                            if has_scrolled || time_delta.is_zero() {
                                state.interaction = Interaction::AutoScrolling {
                                    origin,
                                    current,
                                    last_frame: Some(*now),
                                };

                                shell.request_redraw();
                            }

                            return;
                        }
                    }

                    // Update kinetic scrolling
                    if state.is_kinetic_active() {
                        if state.update_kinetic(*now, bounds, content_bounds) {
                            let _ = notify_scroll(
                                state,
                                &self.on_scroll,
                                bounds,
                                content_bounds,
                                shell,
                            );
                            if state.is_kinetic_active() {
                                shell.request_redraw();
                            }
                        }
                    }

                    // Update scroll-to animation
                    if state.is_scroll_to_animating(*now) {
                        if state.update_scroll_to_animation(*now, bounds, content_bounds) {
                            let _ = notify_scroll(
                                state,
                                &self.on_scroll,
                                bounds,
                                content_bounds,
                                shell,
                            );
                            shell.request_redraw();
                        }
                    }

                    let _ = notify_viewport(state, &self.on_scroll, bounds, content_bounds, shell);
                }
                _ => {}
            }
        };

        update();

        // Update hover animation state
        let is_mouse_over = cursor_over_scrollable.is_some();
        let now = Instant::now();

        if is_mouse_over != state.is_mouse_over_area {
            state.is_mouse_over_area = is_mouse_over;
            state.hover_animation.go_mut(is_mouse_over, now);
        }

        // Calculate hover factor from animation
        let hover_factor = state.hover_animation.interpolate(0.0, 1.0, now);

        // Request redraw while animating
        if state.hover_animation.is_animating(now) {
            shell.request_redraw();
        }

        let status = if state.scrollers_grabbed() {
            Status::Dragged {
                hover_factor,
                is_horizontal_scrollbar_dragged: state.x_scroller_grabbed_at().is_some(),
                is_vertical_scrollbar_dragged: state.y_scroller_grabbed_at().is_some(),
                is_horizontal_scrollbar_disabled: scrollbars.is_x_disabled(),
                is_vertical_scrollbar_disabled: scrollbars.is_y_disabled(),
            }
        } else if cursor_over_scrollable.is_some() {
            Status::Hovered {
                hover_factor,
                is_horizontal_scrollbar_hovered: mouse_over_x_scrollbar,
                is_vertical_scrollbar_hovered: mouse_over_y_scrollbar,
                is_horizontal_scrollbar_disabled: scrollbars.is_x_disabled(),
                is_vertical_scrollbar_disabled: scrollbars.is_y_disabled(),
            }
        } else {
            Status::Active {
                hover_factor,
                is_horizontal_scrollbar_disabled: scrollbars.is_x_disabled(),
                is_vertical_scrollbar_disabled: scrollbars.is_y_disabled(),
            }
        };

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            self.last_status = Some(status);
        }

        if last_offsets != (state.offset_x, state.offset_y)
            || self
                .last_status
                .is_some_and(|last_status| last_status != status)
        {
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();
        let content_bounds = content_layout.bounds();

        let Some(visible_bounds) = bounds.intersection(viewport) else {
            return;
        };

        let scrollbars = Scrollbars::new(state, self.direction, bounds, content_bounds);

        let cursor_over_scrollable = cursor.position_over(bounds);
        let (mouse_over_y_scrollbar, mouse_over_x_scrollbar) = scrollbars.is_mouse_over(cursor);

        let translation = state.translation(self.direction, bounds, content_bounds);

        let cursor = match cursor_over_scrollable {
            Some(cursor_position) if !(mouse_over_x_scrollbar || mouse_over_y_scrollbar) => {
                mouse::Cursor::Available(cursor_position + translation)
            }
            _ => mouse::Cursor::Unavailable,
        };

        let status = self.last_status.unwrap_or(Status::Active {
            hover_factor: 0.0,
            is_horizontal_scrollbar_disabled: false,
            is_vertical_scrollbar_disabled: false,
        });

        let style = theme.style(&self.class, status);

        container::draw_background(renderer, &style.container, layout.bounds());

        // Draw inner content
        if scrollbars.active() {
            let scale_factor = renderer.scale_factor().unwrap_or(1.0);
            let translation = (translation * scale_factor).round() / scale_factor;

            renderer.with_layer(visible_bounds, |renderer| {
                renderer.with_translation(
                    Vector::new(-translation.x, -translation.y),
                    |renderer| {
                        self.content.as_widget().draw(
                            &tree.children[0],
                            renderer,
                            theme,
                            defaults,
                            content_layout,
                            cursor,
                            &Rectangle {
                                y: visible_bounds.y + translation.y,
                                x: visible_bounds.x + translation.x,
                                ..visible_bounds
                            },
                        );
                    },
                );
            });

            let scroll_style = &style.scroll;
            let corner_radius = border::rounded(scroll_style.corner_radius as u32);

            let draw_scrollbar = |renderer: &mut Renderer,
                                  scroll_style: &ScrollStyle,
                                  scrollbar: &internals::Scrollbar,
                                  is_hovered: bool,
                                  is_dragged: bool| {
                // Draw rail background
                if scrollbar.bounds.width > 0.0
                    && scrollbar.bounds.height > 0.0
                    && scroll_style.rail_background.is_some()
                {
                    let bg_color = scroll_style.rail_background.unwrap_or(Color::TRANSPARENT);
                    if bg_color != Color::TRANSPARENT {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: scrollbar.bounds,
                                border: corner_radius,
                                ..renderer::Quad::default()
                            },
                            Background::Color(bg_color),
                        );
                    }
                }

                // Draw handle/scroller
                if let Some(scroller) = scrollbar.scroller
                    && scroller.bounds.width > 0.0
                    && scroller.bounds.height > 0.0
                {
                    let handle_color = if is_dragged {
                        scroll_style.handle_color_dragged
                    } else if is_hovered {
                        scroll_style.handle_color_hovered
                    } else {
                        scroll_style.handle_color
                    };

                    if handle_color != Color::TRANSPARENT {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: scroller.bounds,
                                border: corner_radius,
                                ..renderer::Quad::default()
                            },
                            Background::Color(handle_color),
                        );
                    }
                }
            };

            // Determine hover/drag state for each scrollbar
            let (is_v_hovered, is_h_hovered) = match status {
                Status::Hovered {
                    is_vertical_scrollbar_hovered,
                    is_horizontal_scrollbar_hovered,
                    ..
                } => (
                    is_vertical_scrollbar_hovered,
                    is_horizontal_scrollbar_hovered,
                ),
                _ => (false, false),
            };

            let (is_v_dragged, is_h_dragged) = match status {
                Status::Dragged {
                    is_vertical_scrollbar_dragged,
                    is_horizontal_scrollbar_dragged,
                    ..
                } => (
                    is_vertical_scrollbar_dragged,
                    is_horizontal_scrollbar_dragged,
                ),
                _ => (false, false),
            };

            renderer.with_layer(
                Rectangle {
                    width: (visible_bounds.width + 2.0).min(viewport.width),
                    height: (visible_bounds.height + 2.0).min(viewport.height),
                    ..visible_bounds
                },
                |renderer| {
                    if let Some(scrollbar) = scrollbars.y {
                        draw_scrollbar(
                            renderer,
                            scroll_style,
                            &scrollbar,
                            is_v_hovered,
                            is_v_dragged,
                        );
                    }

                    if let Some(scrollbar) = scrollbars.x {
                        draw_scrollbar(
                            renderer,
                            scroll_style,
                            &scrollbar,
                            is_h_hovered,
                            is_h_dragged,
                        );
                    }

                    if let (Some(x), Some(y)) = (scrollbars.x, scrollbars.y) {
                        let background = style.gap.or(style.container.background);

                        if let Some(background) = background {
                            renderer.fill_quad(
                                renderer::Quad {
                                    bounds: Rectangle {
                                        x: y.bounds.x,
                                        y: x.bounds.y,
                                        width: y.bounds.width,
                                        height: x.bounds.height,
                                    },
                                    ..renderer::Quad::default()
                                },
                                background,
                            );
                        }
                    }
                },
            );
        } else {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                defaults,
                content_layout,
                cursor,
                &Rectangle {
                    x: visible_bounds.x + translation.x,
                    y: visible_bounds.y + translation.y,
                    ..visible_bounds
                },
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let cursor_over_scrollable = cursor.position_over(bounds);

        let content_layout = layout.children().next().unwrap();
        let content_bounds = content_layout.bounds();

        let scrollbars = Scrollbars::new(state, self.direction, bounds, content_bounds);

        let (mouse_over_y_scrollbar, mouse_over_x_scrollbar) = scrollbars.is_mouse_over(cursor);

        if state.scrollers_grabbed() {
            return mouse::Interaction::None;
        }

        let translation = state.translation(self.direction, bounds, content_bounds);

        let cursor = match cursor_over_scrollable {
            Some(cursor_position) if !(mouse_over_x_scrollbar || mouse_over_y_scrollbar) => {
                mouse::Cursor::Available(cursor_position + translation)
            }
            _ => cursor.levitate() + translation,
        };

        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            content_layout,
            cursor,
            &Rectangle {
                y: bounds.y + translation.y,
                x: bounds.x + translation.x,
                ..bounds
            },
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
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();
        let content_bounds = content_layout.bounds();
        let visible_bounds = bounds.intersection(viewport).unwrap_or(*viewport);
        let offset = state.translation(self.direction, bounds, content_bounds);

        let overlay = self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            &visible_bounds,
            translation - offset,
        );

        let icon = if let Interaction::AutoScrolling { origin, .. } = state.interaction {
            let scrollbars = Scrollbars::new(state, self.direction, bounds, content_bounds);

            Some(overlay::Element::new(Box::new(AutoScrollIcon {
                origin,
                vertical: scrollbars.y.is_some(),
                horizontal: scrollbars.x.is_some(),
                class: &self.class,
            })))
        } else {
            None
        };

        match (overlay, icon) {
            (None, None) => None,
            (None, Some(icon)) => Some(icon),
            (Some(overlay), None) => Some(overlay),
            (Some(overlay), Some(icon)) => Some(overlay::Element::new(Box::new(
                overlay::Group::with_children(vec![overlay, icon]),
            ))),
        }
    }
}

struct AutoScrollIcon<'a, Class> {
    origin: Point,
    vertical: bool,
    horizontal: bool,
    class: &'a Class,
}

impl<Class> AutoScrollIcon<'_, Class> {
    const SIZE: f32 = 40.0;
    const DOT: f32 = Self::SIZE / 10.0;
    const PADDING: f32 = Self::SIZE / 10.0;
}

impl<Message, Theme, Renderer> core::Overlay<Message, Theme, Renderer>
    for AutoScrollIcon<'_, Theme::Class<'_>>
where
    Renderer: text::Renderer,
    Theme: Catalog,
{
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        layout::Node::new(Size::new(Self::SIZE, Self::SIZE))
            .move_to(self.origin - Vector::new(Self::SIZE, Self::SIZE) / 2.0)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();
        let style = theme
            .style(
                self.class,
                Status::Active {
                    hover_factor: 0.0,
                    is_horizontal_scrollbar_disabled: false,
                    is_vertical_scrollbar_disabled: false,
                },
            )
            .auto_scroll;

        renderer.with_layer(Rectangle::INFINITE, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.border,
                    shadow: style.shadow,
                    snap: false,
                },
                style.background,
            );

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        bounds.center() - Vector::new(Self::DOT, Self::DOT) / 2.0,
                        Size::new(Self::DOT, Self::DOT),
                    ),
                    border: border::rounded(bounds.width),
                    snap: false,
                    ..renderer::Quad::default()
                },
                style.icon,
            );

            let arrow = core::Text {
                content: String::new(),
                bounds: bounds.size(),
                size: Pixels::from(12),
                line_height: text::LineHeight::Relative(1.0),
                font: Renderer::ICON_FONT,
                align_x: text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                shaping: text::Shaping::Basic,
                wrapping: text::Wrapping::None,
                hint_factor: None,
            };

            if self.vertical {
                renderer.fill_text(
                    core::Text {
                        content: Renderer::SCROLL_UP_ICON.to_string(),
                        align_y: alignment::Vertical::Top,
                        ..arrow
                    },
                    Point::new(bounds.center_x(), bounds.y + Self::PADDING),
                    style.icon,
                    bounds,
                );

                renderer.fill_text(
                    core::Text {
                        content: Renderer::SCROLL_DOWN_ICON.to_string(),
                        align_y: alignment::Vertical::Bottom,
                        ..arrow
                    },
                    Point::new(
                        bounds.center_x(),
                        bounds.y + bounds.height - Self::PADDING - 0.5,
                    ),
                    style.icon,
                    bounds,
                );
            }

            if self.horizontal {
                renderer.fill_text(
                    core::Text {
                        content: Renderer::SCROLL_LEFT_ICON.to_string(),
                        align_x: text::Alignment::Left,
                        ..arrow
                    },
                    Point::new(bounds.x + Self::PADDING + 1.0, bounds.center_y() + 1.0),
                    style.icon,
                    bounds,
                );

                renderer.fill_text(
                    core::Text {
                        content: Renderer::SCROLL_RIGHT_ICON.to_string(),
                        align_x: text::Alignment::Right,
                        ..arrow
                    },
                    Point::new(
                        bounds.x + bounds.width - Self::PADDING - 1.0,
                        bounds.center_y() + 1.0,
                    ),
                    style.icon,
                    bounds,
                );
            }
        });
    }

    fn index(&self) -> f32 {
        f32::MAX
    }
}

impl<'a, Message, Theme, Renderer> From<Scrollable<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a + Catalog,
    Renderer: 'a + text::Renderer,
{
    fn from(
        text_input: Scrollable<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(text_input)
    }
}

fn notify_scroll<Message>(
    state: &mut State,
    on_scroll: &Option<Box<dyn Fn(Viewport) -> Message + '_>>,
    bounds: Rectangle,
    content_bounds: Rectangle,
    shell: &mut Shell<'_, Message>,
) -> bool {
    if notify_viewport(state, on_scroll, bounds, content_bounds, shell) {
        state.last_scrolled = Some(Instant::now());

        true
    } else {
        false
    }
}

fn notify_viewport<Message>(
    state: &mut State,
    on_scroll: &Option<Box<dyn Fn(Viewport) -> Message + '_>>,
    bounds: Rectangle,
    content_bounds: Rectangle,
    shell: &mut Shell<'_, Message>,
) -> bool {
    if content_bounds.width <= bounds.width && content_bounds.height <= bounds.height {
        return false;
    }

    let viewport = Viewport {
        offset_x: state.offset_x,
        offset_y: state.offset_y,
        bounds,
        content_bounds,
    };

    // Don't publish redundant viewports to shell
    if let Some(last_notified) = state.last_notified {
        let last_relative_offset = last_notified.relative_offset();
        let current_relative_offset = viewport.relative_offset();

        let last_absolute_offset = last_notified.absolute_offset();
        let current_absolute_offset = viewport.absolute_offset();

        let unchanged =
            |a: f32, b: f32| (a - b).abs() <= f32::EPSILON || (a.is_nan() && b.is_nan());

        if last_notified.bounds == bounds
            && last_notified.content_bounds == content_bounds
            && unchanged(last_relative_offset.x, current_relative_offset.x)
            && unchanged(last_relative_offset.y, current_relative_offset.y)
            && unchanged(last_absolute_offset.x, current_absolute_offset.x)
            && unchanged(last_absolute_offset.y, current_absolute_offset.y)
        {
            return false;
        }
    }

    state.last_notified = Some(viewport);

    if let Some(on_scroll) = on_scroll {
        shell.publish(on_scroll(viewport));
    }

    true
}

#[derive(Debug, Clone)]
struct State {
    offset_y: Offset,
    offset_x: Offset,
    interaction: Interaction,
    last_notified: Option<Viewport>,
    last_scrolled: Option<Instant>,
    is_scrollbar_visible: bool,
    /// Animation for scrollbar hover state (0.0 = dormant, 1.0 = fully visible)
    hover_animation: Animation<bool>,
    /// Whether the mouse is currently over the scroll area
    is_mouse_over_area: bool,
    /// Current scroll velocity for kinetic scrolling (pixels per second)
    velocity: Vector<f32>,
    /// Last time kinetic scrolling was updated
    last_kinetic_update: Option<Instant>,
    /// Target offset for smooth scroll-to animation
    scroll_to_target: Option<AbsoluteOffset>,
    /// Animation for smooth scroll-to
    scroll_to_animation: Option<Animation<bool>>,
    /// Starting offset for smooth scroll-to animation
    scroll_to_start: Option<AbsoluteOffset>,
}

#[derive(Debug, Clone, Copy)]
enum Interaction {
    None,
    YScrollerGrabbed(f32),
    XScrollerGrabbed(f32),
    TouchScrolling {
        last_position: Point,
        last_time: Instant,
    },
    AutoScrolling {
        origin: Point,
        current: Point,
        last_frame: Option<Instant>,
    },
}

impl Default for State {
    fn default() -> Self {
        Self {
            offset_y: Offset::Absolute(0.0),
            offset_x: Offset::Absolute(0.0),
            interaction: Interaction::None,
            last_notified: None,
            last_scrolled: None,
            is_scrollbar_visible: true,
            hover_animation: Animation::new(false).quick(),
            is_mouse_over_area: false,
            velocity: Vector::new(0.0, 0.0),
            last_kinetic_update: None,
            scroll_to_target: None,
            scroll_to_animation: None,
            scroll_to_start: None,
        }
    }
}

impl operation::Scrollable for State {
    fn snap_to(&mut self, offset: RelativeOffset<Option<f32>>) {
        State::snap_to(self, offset);
    }

    fn scroll_to(&mut self, offset: AbsoluteOffset<Option<f32>>) {
        State::scroll_to(self, offset);
    }

    fn scroll_by(&mut self, offset: AbsoluteOffset, bounds: Rectangle, content_bounds: Rectangle) {
        State::scroll_by(self, offset, bounds, content_bounds);
    }

    fn scroll_to_animated(
        &mut self,
        offset: AbsoluteOffset<Option<f32>>,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        let target = AbsoluteOffset {
            x: offset
                .x
                .unwrap_or_else(|| self.offset_x.absolute(bounds.width, content_bounds.width)),
            y: offset
                .y
                .unwrap_or_else(|| self.offset_y.absolute(bounds.height, content_bounds.height)),
        };
        State::scroll_to_animated(self, target, bounds, content_bounds);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Offset {
    Absolute(f32),
    Relative(f32),
}

impl Offset {
    fn absolute(self, viewport: f32, content: f32) -> f32 {
        match self {
            Offset::Absolute(absolute) => absolute.min((content - viewport).max(0.0)),
            Offset::Relative(percentage) => ((content - viewport) * percentage).max(0.0),
        }
    }

    fn translation(self, viewport: f32, content: f32, alignment: Anchor) -> f32 {
        let offset = self.absolute(viewport, content);

        match alignment {
            Anchor::Start => offset,
            Anchor::End => ((content - viewport).max(0.0) - offset).max(0.0),
        }
    }
}

/// The current [`Viewport`] of the [`Scrollable`].
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    offset_x: Offset,
    offset_y: Offset,
    bounds: Rectangle,
    content_bounds: Rectangle,
}

impl Viewport {
    /// Creates a new [`Viewport`] from absolute offset values.
    /// This is used internally by the virtual scrollable.
    pub(crate) fn from_absolute(
        offset: AbsoluteOffset,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) -> Self {
        Self {
            offset_x: Offset::Absolute(offset.x),
            offset_y: Offset::Absolute(offset.y),
            bounds,
            content_bounds,
        }
    }

    /// Returns the [`AbsoluteOffset`] of the current [`Viewport`].
    pub fn absolute_offset(&self) -> AbsoluteOffset {
        let x = self
            .offset_x
            .absolute(self.bounds.width, self.content_bounds.width);
        let y = self
            .offset_y
            .absolute(self.bounds.height, self.content_bounds.height);

        AbsoluteOffset { x, y }
    }

    /// Returns the [`AbsoluteOffset`] of the current [`Viewport`], but with its
    /// alignment reversed.
    ///
    /// This method can be useful to switch the alignment of a [`Scrollable`]
    /// while maintaining its scrolling position.
    pub fn absolute_offset_reversed(&self) -> AbsoluteOffset {
        let AbsoluteOffset { x, y } = self.absolute_offset();

        AbsoluteOffset {
            x: (self.content_bounds.width - self.bounds.width).max(0.0) - x,
            y: (self.content_bounds.height - self.bounds.height).max(0.0) - y,
        }
    }

    /// Returns the [`RelativeOffset`] of the current [`Viewport`].
    pub fn relative_offset(&self) -> RelativeOffset {
        let AbsoluteOffset { x, y } = self.absolute_offset();

        let x = x / (self.content_bounds.width - self.bounds.width);
        let y = y / (self.content_bounds.height - self.bounds.height);

        RelativeOffset { x, y }
    }

    /// Returns the bounds of the current [`Viewport`].
    pub fn bounds(&self) -> Rectangle {
        self.bounds
    }

    /// Returns the content bounds of the current [`Viewport`].
    pub fn content_bounds(&self) -> Rectangle {
        self.content_bounds
    }

    /// Returns the visible rectangle in content-local coordinates.
    pub fn visible_rect(&self) -> Rectangle {
        let offset = self.absolute_offset();
        Rectangle {
            x: offset.x,
            y: offset.y,
            width: self.bounds.width,
            height: self.bounds.height,
        }
    }
}

impl State {
    fn new() -> Self {
        State::default()
    }

    fn scroll(&mut self, delta: Vector<f32>, bounds: Rectangle, content_bounds: Rectangle) {
        if bounds.height < content_bounds.height {
            self.offset_y = Offset::Absolute(
                (self.offset_y.absolute(bounds.height, content_bounds.height) + delta.y)
                    .clamp(0.0, content_bounds.height - bounds.height),
            );
        }

        if bounds.width < content_bounds.width {
            self.offset_x = Offset::Absolute(
                (self.offset_x.absolute(bounds.width, content_bounds.width) + delta.x)
                    .clamp(0.0, content_bounds.width - bounds.width),
            );
        }
    }

    fn scroll_y_to(&mut self, percentage: f32, bounds: Rectangle, content_bounds: Rectangle) {
        self.offset_y = Offset::Relative(percentage.clamp(0.0, 1.0));
        self.unsnap(bounds, content_bounds);
    }

    fn scroll_x_to(&mut self, percentage: f32, bounds: Rectangle, content_bounds: Rectangle) {
        self.offset_x = Offset::Relative(percentage.clamp(0.0, 1.0));
        self.unsnap(bounds, content_bounds);
    }

    fn snap_to(&mut self, offset: RelativeOffset<Option<f32>>) {
        if let Some(x) = offset.x {
            self.offset_x = Offset::Relative(x.clamp(0.0, 1.0));
        }

        if let Some(y) = offset.y {
            self.offset_y = Offset::Relative(y.clamp(0.0, 1.0));
        }
    }

    fn scroll_to(&mut self, offset: AbsoluteOffset<Option<f32>>) {
        if let Some(x) = offset.x {
            self.offset_x = Offset::Absolute(x.max(0.0));
        }

        if let Some(y) = offset.y {
            self.offset_y = Offset::Absolute(y.max(0.0));
        }
    }

    /// Scroll by the provided [`AbsoluteOffset`].
    fn scroll_by(&mut self, offset: AbsoluteOffset, bounds: Rectangle, content_bounds: Rectangle) {
        self.scroll(Vector::new(offset.x, offset.y), bounds, content_bounds);
    }

    /// Unsnaps the current scroll position, if snapped, given the bounds of the
    /// [`Scrollable`] and its contents.
    fn unsnap(&mut self, bounds: Rectangle, content_bounds: Rectangle) {
        self.offset_x =
            Offset::Absolute(self.offset_x.absolute(bounds.width, content_bounds.width));
        self.offset_y =
            Offset::Absolute(self.offset_y.absolute(bounds.height, content_bounds.height));
    }

    /// Returns the scrolling translation of the [`State`], given a [`Direction`],
    /// the bounds of the [`Scrollable`] and its contents.
    fn translation(
        &self,
        direction: Direction,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) -> Vector {
        Vector::new(
            if let Some(horizontal) = direction.horizontal() {
                self.offset_x
                    .translation(bounds.width, content_bounds.width, horizontal.alignment)
                    .round()
            } else {
                0.0
            },
            if let Some(vertical) = direction.vertical() {
                self.offset_y
                    .translation(bounds.height, content_bounds.height, vertical.alignment)
                    .round()
            } else {
                0.0
            },
        )
    }

    /// Starts an animated scroll to the given absolute offset.
    fn scroll_to_animated(
        &mut self,
        target: AbsoluteOffset,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        // Get current absolute offset
        let current = AbsoluteOffset {
            x: self.offset_x.absolute(bounds.width, content_bounds.width),
            y: self.offset_y.absolute(bounds.height, content_bounds.height),
        };

        // Clamp target to valid range
        let target = AbsoluteOffset {
            x: target
                .x
                .clamp(0.0, (content_bounds.width - bounds.width).max(0.0)),
            y: target
                .y
                .clamp(0.0, (content_bounds.height - bounds.height).max(0.0)),
        };

        self.scroll_to_start = Some(current);
        self.scroll_to_target = Some(target);
        self.scroll_to_animation = Some(Animation::new(false).slow().go(true, Instant::now()));

        // Stop any kinetic scrolling
        self.velocity = Vector::new(0.0, 0.0);
    }

    /// Updates kinetic scrolling, returning true if the scroll position changed.
    fn update_kinetic(
        &mut self,
        now: Instant,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) -> bool {
        const FRICTION: f32 = 5.0; // Deceleration factor
        const MIN_VELOCITY: f32 = 1.0; // Stop when velocity is below this (px/s)

        // Skip if velocity is negligible
        if self.velocity.x.abs() < MIN_VELOCITY && self.velocity.y.abs() < MIN_VELOCITY {
            self.velocity = Vector::new(0.0, 0.0);
            self.last_kinetic_update = None;
            return false;
        }

        let dt = if let Some(last) = self.last_kinetic_update {
            (now - last).as_secs_f32()
        } else {
            0.016 // ~60fps default
        };

        self.last_kinetic_update = Some(now);

        // Apply velocity to scroll position
        let delta = Vector::new(self.velocity.x * dt, self.velocity.y * dt);

        let old_x = self.offset_x;
        let old_y = self.offset_y;

        self.scroll(delta, bounds, content_bounds);

        // Apply friction (exponential decay)
        let decay = (-FRICTION * dt).exp();
        self.velocity.x *= decay;
        self.velocity.y *= decay;

        // Stop if we hit the edge
        if self.offset_x == old_x {
            self.velocity.x = 0.0;
        }
        if self.offset_y == old_y {
            self.velocity.y = 0.0;
        }

        true
    }

    /// Updates animated scroll-to, returning true if still animating.
    fn update_scroll_to_animation(
        &mut self,
        now: Instant,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) -> bool {
        let (Some(target), Some(start), Some(animation)) = (
            self.scroll_to_target,
            self.scroll_to_start,
            &self.scroll_to_animation,
        ) else {
            return false;
        };

        if !animation.is_animating(now) {
            // Animation complete, snap to target
            self.offset_x = Offset::Absolute(target.x);
            self.offset_y = Offset::Absolute(target.y);
            self.scroll_to_target = None;
            self.scroll_to_start = None;
            self.scroll_to_animation = None;
            return false;
        }

        // Interpolate using ease-out
        let t = animation.interpolate(0.0, 1.0, now);
        // Apply ease-out cubic for smoother deceleration
        let eased = 1.0 - (1.0 - t).powi(3);

        let current_x = start.x + (target.x - start.x) * eased;
        let current_y = start.y + (target.y - start.y) * eased;

        self.offset_x =
            Offset::Absolute(current_x.clamp(0.0, (content_bounds.width - bounds.width).max(0.0)));
        self.offset_y = Offset::Absolute(
            current_y.clamp(0.0, (content_bounds.height - bounds.height).max(0.0)),
        );

        true
    }

    /// Returns true if kinetic scrolling is active.
    fn is_kinetic_active(&self) -> bool {
        self.velocity.x.abs() > 1.0 || self.velocity.y.abs() > 1.0
    }

    /// Returns true if a scroll-to animation is active.
    fn is_scroll_to_animating(&self, now: Instant) -> bool {
        self.scroll_to_animation
            .as_ref()
            .is_some_and(|a| a.is_animating(now))
    }

    fn scrollers_grabbed(&self) -> bool {
        matches!(
            self.interaction,
            Interaction::YScrollerGrabbed(_) | Interaction::XScrollerGrabbed(_),
        )
    }

    pub fn y_scroller_grabbed_at(&self) -> Option<f32> {
        let Interaction::YScrollerGrabbed(at) = self.interaction else {
            return None;
        };

        Some(at)
    }

    pub fn x_scroller_grabbed_at(&self) -> Option<f32> {
        let Interaction::XScrollerGrabbed(at) = self.interaction else {
            return None;
        };

        Some(at)
    }
}

#[derive(Debug)]
/// State of both [`Scrollbar`]s.
struct Scrollbars {
    y: Option<internals::Scrollbar>,
    x: Option<internals::Scrollbar>,
}

impl Scrollbars {
    /// Create y and/or x scrollbar(s) if content is overflowing the [`Scrollable`] bounds.
    fn new(
        state: &State,
        direction: Direction,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) -> Self {
        let translation = state.translation(direction, bounds, content_bounds);

        let show_scrollbar_x = direction
            .horizontal()
            .filter(|_scrollbar| content_bounds.width > bounds.width);

        let show_scrollbar_y = direction
            .vertical()
            .filter(|_scrollbar| content_bounds.height > bounds.height);

        let y_scrollbar = if let Some(vertical) = show_scrollbar_y {
            let Scrollbar {
                width,
                margin,
                scroller_width,
                ..
            } = *vertical;

            // Adjust the height of the vertical scrollbar if the horizontal scrollbar
            // is present
            let x_scrollbar_height =
                show_scrollbar_x.map_or(0.0, |h| h.width.max(h.scroller_width) + h.margin);

            let total_scrollbar_width = width.max(scroller_width) + 2.0 * margin;

            // Total bounds of the scrollbar + margin + scroller width
            let total_scrollbar_bounds = Rectangle {
                x: bounds.x + bounds.width - total_scrollbar_width,
                y: bounds.y,
                width: total_scrollbar_width,
                height: (bounds.height - x_scrollbar_height).max(0.0),
            };

            // Bounds of just the scrollbar
            let scrollbar_bounds = Rectangle {
                x: bounds.x + bounds.width - total_scrollbar_width / 2.0 - width / 2.0,
                y: bounds.y,
                width,
                height: (bounds.height - x_scrollbar_height).max(0.0),
            };

            let ratio = bounds.height / content_bounds.height;

            let scroller = if ratio >= 1.0 {
                None
            } else {
                // min height for easier grabbing with super tall content
                let scroller_height = (scrollbar_bounds.height * ratio).max(2.0);
                let scroller_offset =
                    translation.y * ratio * scrollbar_bounds.height / bounds.height;

                let scroller_bounds = Rectangle {
                    x: bounds.x + bounds.width - total_scrollbar_width / 2.0 - scroller_width / 2.0,
                    y: (scrollbar_bounds.y + scroller_offset).max(0.0),
                    width: scroller_width,
                    height: scroller_height,
                };

                Some(internals::Scroller {
                    bounds: scroller_bounds,
                })
            };

            Some(internals::Scrollbar {
                total_bounds: total_scrollbar_bounds,
                bounds: scrollbar_bounds,
                scroller,
                alignment: vertical.alignment,
                disabled: content_bounds.height <= bounds.height,
            })
        } else {
            None
        };

        let x_scrollbar = if let Some(horizontal) = show_scrollbar_x {
            let Scrollbar {
                width,
                margin,
                scroller_width,
                ..
            } = *horizontal;

            // Need to adjust the width of the horizontal scrollbar if the vertical scrollbar
            // is present
            let scrollbar_y_width =
                y_scrollbar.map_or(0.0, |scrollbar| scrollbar.total_bounds.width);

            let total_scrollbar_height = width.max(scroller_width) + 2.0 * margin;

            // Total bounds of the scrollbar + margin + scroller width
            let total_scrollbar_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + bounds.height - total_scrollbar_height,
                width: (bounds.width - scrollbar_y_width).max(0.0),
                height: total_scrollbar_height,
            };

            // Bounds of just the scrollbar
            let scrollbar_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + bounds.height - total_scrollbar_height / 2.0 - width / 2.0,
                width: (bounds.width - scrollbar_y_width).max(0.0),
                height: width,
            };

            let ratio = bounds.width / content_bounds.width;

            let scroller = if ratio >= 1.0 {
                None
            } else {
                // min width for easier grabbing with extra wide content
                let scroller_length = (scrollbar_bounds.width * ratio).max(2.0);
                let scroller_offset = translation.x * ratio * scrollbar_bounds.width / bounds.width;

                let scroller_bounds = Rectangle {
                    x: (scrollbar_bounds.x + scroller_offset).max(0.0),
                    y: bounds.y + bounds.height
                        - total_scrollbar_height / 2.0
                        - scroller_width / 2.0,
                    width: scroller_length,
                    height: scroller_width,
                };

                Some(internals::Scroller {
                    bounds: scroller_bounds,
                })
            };

            Some(internals::Scrollbar {
                total_bounds: total_scrollbar_bounds,
                bounds: scrollbar_bounds,
                scroller,
                alignment: horizontal.alignment,
                disabled: content_bounds.width <= bounds.width,
            })
        } else {
            None
        };

        Self {
            y: y_scrollbar,
            x: x_scrollbar,
        }
    }

    fn is_mouse_over(&self, cursor: mouse::Cursor) -> (bool, bool) {
        if let Some(cursor_position) = cursor.position() {
            (
                self.y
                    .as_ref()
                    .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
                    .unwrap_or(false),
                self.x
                    .as_ref()
                    .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
                    .unwrap_or(false),
            )
        } else {
            (false, false)
        }
    }

    fn is_y_disabled(&self) -> bool {
        self.y.map(|y| y.disabled).unwrap_or(false)
    }

    fn is_x_disabled(&self) -> bool {
        self.x.map(|x| x.disabled).unwrap_or(false)
    }

    fn grab_y_scroller(&self, cursor_position: Point) -> Option<f32> {
        let scrollbar = self.y?;
        let scroller = scrollbar.scroller?;

        if scrollbar.total_bounds.contains(cursor_position) {
            Some(if scroller.bounds.contains(cursor_position) {
                (cursor_position.y - scroller.bounds.y) / scroller.bounds.height
            } else {
                0.5
            })
        } else {
            None
        }
    }

    fn grab_x_scroller(&self, cursor_position: Point) -> Option<f32> {
        let scrollbar = self.x?;
        let scroller = scrollbar.scroller?;

        if scrollbar.total_bounds.contains(cursor_position) {
            Some(if scroller.bounds.contains(cursor_position) {
                (cursor_position.x - scroller.bounds.x) / scroller.bounds.width
            } else {
                0.5
            })
        } else {
            None
        }
    }

    fn active(&self) -> bool {
        self.y.is_some() || self.x.is_some()
    }
}

pub(super) mod internals {
    use crate::core::{Point, Rectangle};

    use super::Anchor;

    #[derive(Debug, Copy, Clone)]
    pub struct Scrollbar {
        pub total_bounds: Rectangle,
        pub bounds: Rectangle,
        pub scroller: Option<Scroller>,
        pub alignment: Anchor,
        pub disabled: bool,
    }

    impl Scrollbar {
        /// Returns whether the mouse is over the scrollbar or not.
        pub fn is_mouse_over(&self, cursor_position: Point) -> bool {
            self.total_bounds.contains(cursor_position)
        }

        /// Returns the y-axis scrolled percentage from the cursor position.
        pub fn scroll_percentage_y(&self, grabbed_at: f32, cursor_position: Point) -> f32 {
            if let Some(scroller) = self.scroller {
                let percentage =
                    (cursor_position.y - self.bounds.y - scroller.bounds.height * grabbed_at)
                        / (self.bounds.height - scroller.bounds.height);

                match self.alignment {
                    Anchor::Start => percentage,
                    Anchor::End => 1.0 - percentage,
                }
            } else {
                0.0
            }
        }

        /// Returns the x-axis scrolled percentage from the cursor position.
        pub fn scroll_percentage_x(&self, grabbed_at: f32, cursor_position: Point) -> f32 {
            if let Some(scroller) = self.scroller {
                let percentage =
                    (cursor_position.x - self.bounds.x - scroller.bounds.width * grabbed_at)
                        / (self.bounds.width - scroller.bounds.width);

                match self.alignment {
                    Anchor::Start => percentage,
                    Anchor::End => 1.0 - percentage,
                }
            } else {
                0.0
            }
        }
    }

    /// The handle of a [`Scrollbar`].
    #[derive(Debug, Clone, Copy)]
    pub struct Scroller {
        /// The bounds of the [`Scroller`].
        pub bounds: Rectangle,
    }
}

/// The possible status of a [`Scrollable`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    /// The [`Scrollable`] can be interacted with.
    Active {
        /// The animated hover factor (0.0 = dormant, 1.0 = fully hovered).
        hover_factor: f32,
        /// Whether or not the horizontal scrollbar is disabled meaning the content isn't overflowing.
        is_horizontal_scrollbar_disabled: bool,
        /// Whether or not the vertical scrollbar is disabled meaning the content isn't overflowing.
        is_vertical_scrollbar_disabled: bool,
    },
    /// The [`Scrollable`] is being hovered.
    Hovered {
        /// The animated hover factor (0.0 = dormant, 1.0 = fully hovered).
        hover_factor: f32,
        /// Indicates if the horizontal scrollbar is being hovered.
        is_horizontal_scrollbar_hovered: bool,
        /// Indicates if the vertical scrollbar is being hovered.
        is_vertical_scrollbar_hovered: bool,
        /// Whether or not the horizontal scrollbar is disabled meaning the content isn't overflowing.
        is_horizontal_scrollbar_disabled: bool,
        /// Whether or not the vertical scrollbar is disabled meaning the content isn't overflowing.
        is_vertical_scrollbar_disabled: bool,
    },
    /// The [`Scrollable`] is being dragged.
    Dragged {
        /// The animated hover factor (0.0 = dormant, 1.0 = fully hovered).
        hover_factor: f32,
        /// Indicates if the horizontal scrollbar is being dragged.
        is_horizontal_scrollbar_dragged: bool,
        /// Indicates if the vertical scrollbar is being dragged.
        is_vertical_scrollbar_dragged: bool,
        /// Whether or not the horizontal scrollbar is disabled meaning the content isn't overflowing.
        is_horizontal_scrollbar_disabled: bool,
        /// Whether or not the vertical scrollbar is disabled meaning the content isn't overflowing.
        is_vertical_scrollbar_disabled: bool,
    },
}

/// The appearance of a scrollable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The [`container::Style`] of a scrollable.
    pub container: container::Style,
    /// The scroll style configuration.
    pub scroll: ScrollStyle,
    /// The [`Background`] of the gap between a horizontal and vertical scrollbar.
    pub gap: Option<Background>,
    /// The appearance of the [`AutoScroll`] overlay.
    pub auto_scroll: AutoScroll,
}

/// Controls the spacing and visuals of scrollbars.
///
/// There are three presets to choose from:
/// * [`ScrollStyle::solid`] - Always visible, allocates space
/// * [`ScrollStyle::thin`] - Thin bars that expand on hover
/// * [`ScrollStyle::floating`] - Hidden until hover, floats over content
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollStyle {
    /// If `true`, scroll bars float above the content, partially covering it.
    /// If `false`, the scroll bars allocate space, shrinking the area available to the contents.
    pub floating: bool,

    /// The width of the scroll bar at its largest.
    pub bar_width: f32,

    /// Make sure the scroll handle is at least this big.
    pub handle_min_length: f32,

    /// Margin between contents and scroll bar.
    pub bar_inner_margin: f32,

    /// Margin between scroll bar and the outer container (e.g. right of a vertical scroll bar).
    pub bar_outer_margin: f32,

    /// The thin width of floating scroll bars that the user is NOT hovering.
    pub floating_width: f32,

    /// How much space is allocated for a floating scroll bar?
    /// Normally this is zero, but you could set this to something small.
    pub floating_allocated_width: f32,

    /// If true, use colors with more contrast. Good for floating scroll bars.
    pub foreground_color: bool,

    /// Corner radius of the scrollbar and handle.
    pub corner_radius: f32,

    /// The opaqueness of the background when the user is neither scrolling
    /// nor hovering the scroll area. (Only for floating scroll bars.)
    pub dormant_background_opacity: f32,

    /// The opaqueness of the background when the user is hovering
    /// the scroll area, but not the scroll bar. (Only for floating scroll bars.)
    pub active_background_opacity: f32,

    /// The opaqueness of the background when the user is hovering
    /// over the scroll bars. (Only for floating scroll bars.)
    pub interact_background_opacity: f32,

    /// The opaqueness of the handle when the user is neither scrolling
    /// nor hovering the scroll area. (Only for floating scroll bars.)
    pub dormant_handle_opacity: f32,

    /// The opaqueness of the handle when the user is hovering
    /// the scroll area, but not the scroll bar. (Only for floating scroll bars.)
    pub active_handle_opacity: f32,

    /// The opaqueness of the handle when the user is hovering
    /// over the scroll bars. (Only for floating scroll bars.)
    pub interact_handle_opacity: f32,

    /// The background color of the scrollbar rail.
    pub rail_background: Option<Color>,

    /// The color of the scrollbar handle.
    pub handle_color: Color,

    /// The color of the scrollbar handle when hovered.
    pub handle_color_hovered: Color,

    /// The color of the scrollbar handle when dragged.
    pub handle_color_dragged: Color,
}

impl Default for ScrollStyle {
    fn default() -> Self {
        Self::floating()
    }
}

impl ScrollStyle {
    /// Solid scroll bars that always use up space.
    pub fn solid() -> Self {
        Self {
            floating: false,
            bar_width: 6.0,
            handle_min_length: 12.0,
            bar_inner_margin: 4.0,
            bar_outer_margin: 0.0,
            floating_width: 2.0,
            floating_allocated_width: 0.0,
            foreground_color: false,
            corner_radius: 2.0,
            dormant_background_opacity: 0.0,
            active_background_opacity: 0.4,
            interact_background_opacity: 0.7,
            dormant_handle_opacity: 0.0,
            active_handle_opacity: 0.6,
            interact_handle_opacity: 1.0,
            rail_background: None,
            handle_color: Color::from_rgb(0.5, 0.5, 0.5),
            handle_color_hovered: Color::from_rgb(0.6, 0.6, 0.6),
            handle_color_dragged: Color::from_rgb(0.7, 0.7, 0.7),
        }
    }

    /// Thin scroll bars that expand on hover.
    pub fn thin() -> Self {
        Self {
            floating: true,
            bar_width: 10.0,
            floating_allocated_width: 6.0,
            foreground_color: false,
            dormant_background_opacity: 1.0,
            dormant_handle_opacity: 1.0,
            active_background_opacity: 1.0,
            active_handle_opacity: 1.0,
            // Be translucent when expanded so we can see the content
            interact_background_opacity: 0.6,
            interact_handle_opacity: 0.6,
            ..Self::solid()
        }
    }

    /// No scroll bars until you hover the scroll area,
    /// at which time they appear faintly, and then expand
    /// when you hover the scroll bars.
    pub fn floating() -> Self {
        Self {
            floating: true,
            bar_width: 10.0,
            foreground_color: true,
            floating_allocated_width: 0.0,
            dormant_background_opacity: 0.0,
            dormant_handle_opacity: 0.0,
            ..Self::solid()
        }
    }

    /// Width of a solid vertical scrollbar, or height of a horizontal scroll bar, when it is at its widest.
    pub fn allocated_width(&self) -> f32 {
        if self.floating {
            self.floating_allocated_width
        } else {
            self.bar_inner_margin + self.bar_width + self.bar_outer_margin
        }
    }

    /// Returns the current width based on the hover factor (0.0 = dormant, 1.0 = fully expanded).
    pub fn current_width(&self, hover_factor: f32) -> f32 {
        if self.floating {
            self.floating_width + (self.bar_width - self.floating_width) * hover_factor
        } else {
            self.bar_width
        }
    }

    /// Returns the background opacity based on the interaction state.
    pub fn background_opacity(&self, hover_factor: f32, is_interacting: bool) -> f32 {
        if self.floating {
            if is_interacting {
                self.interact_background_opacity
            } else {
                self.dormant_background_opacity
                    + (self.active_background_opacity - self.dormant_background_opacity)
                        * hover_factor
            }
        } else {
            1.0
        }
    }

    /// Returns the handle opacity based on the interaction state.
    pub fn handle_opacity(&self, hover_factor: f32, is_interacting: bool) -> f32 {
        if self.floating {
            if is_interacting {
                self.interact_handle_opacity
            } else {
                self.dormant_handle_opacity
                    + (self.active_handle_opacity - self.dormant_handle_opacity) * hover_factor
            }
        } else {
            1.0
        }
    }
}

/// The appearance of the autoscroll overlay of a scrollable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoScroll {
    /// The [`Background`] of the [`AutoScroll`] overlay.
    pub background: Background,
    /// The [`Border`] of the [`AutoScroll`] overlay.
    pub border: Border,
    /// Thje [`Shadow`] of the [`AutoScroll`] overlay.
    pub shadow: Shadow,
    /// The [`Color`] for the arrow icons of the [`AutoScroll`] overlay.
    pub icon: Color,
}

/// The theme catalog of a [`Scrollable`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`Scrollable`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default style of a [`Scrollable`].
pub fn default(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();

    // Get the hover factor from any status variant
    let hover_factor = match status {
        Status::Active { hover_factor, .. } => hover_factor,
        Status::Hovered { hover_factor, .. } => hover_factor,
        Status::Dragged { hover_factor, .. } => hover_factor,
    };

    // Determine if we're interacting (hovering scrollbar or dragging)
    let (is_h_interacting, is_v_interacting) = match status {
        Status::Active { .. } => (false, false),
        Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
            ..
        } => (
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
        ),
        Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
            ..
        } => (
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
        ),
    };

    let is_interacting = is_h_interacting || is_v_interacting;

    // Create base scroll style with theme colors
    let mut scroll_style = ScrollStyle::floating();
    scroll_style.rail_background = Some(palette.background.weak.color);
    scroll_style.handle_color = palette.background.strongest.color;
    scroll_style.handle_color_hovered = palette.primary.strong.color;
    scroll_style.handle_color_dragged = palette.primary.base.color;

    // Adjust handle color based on interaction state
    let handle_color = if is_interacting {
        if matches!(status, Status::Dragged { .. }) {
            scroll_style.handle_color_dragged
        } else {
            scroll_style.handle_color_hovered
        }
    } else {
        scroll_style.handle_color
    };

    // Apply opacity based on hover state
    let handle_opacity = scroll_style.handle_opacity(hover_factor, is_interacting);
    let bg_opacity = scroll_style.background_opacity(hover_factor, is_interacting);

    scroll_style.handle_color = handle_color.scale_alpha(handle_opacity);
    scroll_style.handle_color_hovered = scroll_style
        .handle_color_hovered
        .scale_alpha(handle_opacity);
    scroll_style.handle_color_dragged = scroll_style
        .handle_color_dragged
        .scale_alpha(handle_opacity);

    if let Some(ref mut bg) = scroll_style.rail_background {
        *bg = bg.scale_alpha(bg_opacity);
    }

    let auto_scroll = AutoScroll {
        background: palette.background.base.color.scale_alpha(0.9).into(),
        border: border::rounded(u32::MAX)
            .width(1)
            .color(palette.background.base.text.scale_alpha(0.8)),
        shadow: Shadow {
            color: Color::BLACK.scale_alpha(0.7),
            offset: Vector::ZERO,
            blur_radius: 2.0,
        },
        icon: palette.background.base.text.scale_alpha(0.8),
    };

    Style {
        container: container::Style::default(),
        scroll: scroll_style,
        gap: None,
        auto_scroll,
    }
}
