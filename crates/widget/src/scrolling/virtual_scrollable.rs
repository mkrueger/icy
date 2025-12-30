//! Virtual scrolling widgets for efficiently displaying large content.
//!
//! This module provides two approaches for virtual scrolling:
//!
//! - [`show_viewport`] - For custom virtualization where you control what's rendered
//!   based on the visible viewport rectangle.
//! - [`show_rows`] - For simple uniform-height row virtualization.
//!
//! # Example: Virtual List with `show_rows`
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } }
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! use iced::widget::{column, text, virtual_scrollable};
//!
//! enum Message {}
//!
//! fn view(items: &[String]) -> Element<'_, Message> {
//!     virtual_scrollable::show_rows(
//!         30.0,  // row height
//!         items.len(),
//!         |visible_range| {
//!             column(
//!                 items[visible_range.clone()]
//!                     .iter()
//!                     .map(|item| text(item).into())
//!             ).into()
//!         }
//!     ).into()
//! }
//! ```
//!
//! # Example: Custom Virtualization with `show_viewport`
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::core::Size; }
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! use iced::widget::{column, text, virtual_scrollable};
//! use iced::Size;
//!
//! enum Message {}
//!
//! fn view(items: &[String]) -> Element<'_, Message> {
//!     let row_height = 30.0;
//!     let total_height = items.len() as f32 * row_height;
//!
//!     virtual_scrollable::show_viewport(
//!         Size::new(400.0, total_height),
//!         |viewport| {
//!             let first = (viewport.y / row_height).floor() as usize;
//!             let last = ((viewport.y + viewport.height) / row_height).ceil() as usize;
//!             let visible = first..last.min(items.len());
//!
//!             column(
//!                 items[visible]
//!                     .iter()
//!                     .map(|item| text(item).into())
//!             ).into()
//!         }
//!     ).into()
//! }
//! ```
use crate::container;
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
    Animation, Background, Clipboard, Color, Element, Event, Layout, Length, Point, Rectangle,
    Shell, Size, Vector, Widget,
};

use super::scrollable::{
    Anchor, Catalog, Direction, ScrollStyle, Scrollbar, Status, Style, StyleFn, Viewport,
};

pub use super::scrollable::{AbsoluteOffset, RelativeOffset};

use std::ops::Range;

/// Creates a virtual scrollable optimized for uniform-height rows.
///
/// This is a convenience wrapper around [`show_viewport`] for the common case
/// of a list with rows of equal height. The callback receives the range of
/// visible row indices.
///
/// # Arguments
/// * `row_height` - The height of each row in pixels
/// * `total_rows` - The total number of rows
/// * `view` - A callback that receives the visible row range and returns content
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::{column, text, virtual_scrollable};
///
/// enum Message {}
///
/// let items: Vec<String> = (0..10_000).map(|i| format!("Item {}", i)).collect();
///
/// virtual_scrollable::show_rows(
///     30.0,  // each row is 30 pixels tall
///     items.len(),
///     |visible_range| {
///         column(
///             items[visible_range]
///                 .iter()
///                 .map(|item| text(item).into())
///         ).into()
///     }
/// );
/// ```
pub fn show_rows<'a, Message, Theme, Renderer>(
    row_height: f32,
    total_rows: usize,
    view: impl Fn(Range<usize>) -> Element<'a, Message, Theme, Renderer> + 'a,
) -> VirtualScrollable<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    VirtualScrollable::with_rows(row_height, total_rows, view)
}

/// A scrollable that only renders visible content for efficient large content display.
///
/// Use [`show_viewport`] or [`show_rows`] to create instances.
pub struct VirtualScrollable<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    id: Option<widget::Id>,
    width: Length,
    height: Length,
    direction: Direction,
    content_size: Size,
    view: Box<dyn Fn(Rectangle) -> Element<'a, Message, Theme, Renderer> + 'a>,
    on_scroll: Option<Box<dyn Fn(Viewport) -> Message + 'a>>,
    class: Theme::Class<'a>,
    last_status: Option<Status>,
    /// Cell size for smooth sub-cell scrolling.
    /// For rows, only height is used. For 2D grids, both width and height.
    cell_size: Option<Size>,
}

impl<'a, Message, Theme, Renderer> VirtualScrollable<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// Creates a new [`VirtualScrollable`] with the given content size and view function.
    pub fn new(
        content_size: Size,
        view: impl Fn(Rectangle) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self {
        VirtualScrollable {
            id: None,
            width: Length::Fill,
            height: Length::Fill,
            direction: Direction::default(),
            content_size,
            view: Box::new(view),
            on_scroll: None,
            class: Theme::default(),
            last_status: None,
            cell_size: None,
        }
    }

    /// Creates a new [`VirtualScrollable`] optimized for uniform-height rows.
    ///
    /// This variant enables smooth sub-row scrolling by rendering one extra row
    /// and applying a fractional offset, similar to egui's `show_rows`.
    pub fn with_rows(
        row_height: f32,
        total_rows: usize,
        view: impl Fn(Range<usize>) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self {
        let total_height = row_height * total_rows as f32;

        VirtualScrollable {
            id: None,
            width: Length::Fill,
            height: Length::Fill,
            direction: Direction::default(),
            content_size: Size::new(0.0, total_height), // Width will be determined by bounds
            view: Box::new(move |viewport| {
                let first_row = (viewport.y / row_height).floor().max(0.0) as usize;
                let last_row = ((viewport.y + viewport.height) / row_height).ceil() as usize + 1;
                let visible_range = first_row..last_row.min(total_rows);

                view(visible_range)
            }),
            on_scroll: None,
            class: Theme::default(),
            last_status: None,
            cell_size: Some(Size::new(0.0, row_height)),
        }
    }

    /// Makes the [`VirtualScrollable`] scroll horizontally.
    pub fn horizontal(mut self) -> Self {
        self.direction = Direction::Horizontal(Scrollbar::default());
        self
    }

    /// Sets the [`Direction`] of the [`VirtualScrollable`].
    pub fn direction(mut self, direction: impl Into<Direction>) -> Self {
        self.direction = direction.into();
        self
    }

    /// Sets the [`widget::Id`] of the [`VirtualScrollable`].
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the width of the [`VirtualScrollable`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`VirtualScrollable`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets a function to call when the [`VirtualScrollable`] is scrolled.
    pub fn on_scroll(mut self, f: impl Fn(Viewport) -> Message + 'a) -> Self {
        self.on_scroll = Some(Box::new(f));
        self
    }

    /// Anchors the vertical [`VirtualScrollable`] direction to the top.
    pub fn anchor_top(self) -> Self {
        self.anchor_y(Anchor::Start)
    }

    /// Anchors the vertical [`VirtualScrollable`] direction to the bottom.
    pub fn anchor_bottom(self) -> Self {
        self.anchor_y(Anchor::End)
    }

    /// Anchors the horizontal [`VirtualScrollable`] direction to the left.
    pub fn anchor_left(self) -> Self {
        self.anchor_x(Anchor::Start)
    }

    /// Anchors the horizontal [`VirtualScrollable`] direction to the right.
    pub fn anchor_right(self) -> Self {
        self.anchor_x(Anchor::End)
    }

    /// Sets the [`Anchor`] of the horizontal direction.
    pub fn anchor_x(mut self, alignment: Anchor) -> Self {
        match &mut self.direction {
            Direction::Horizontal(horizontal) | Direction::Both { horizontal, .. } => {
                horizontal.alignment = alignment;
            }
            Direction::Vertical { .. } => {}
        }
        self
    }

    /// Sets the [`Anchor`] of the vertical direction.
    pub fn anchor_y(mut self, alignment: Anchor) -> Self {
        match &mut self.direction {
            Direction::Vertical(vertical) | Direction::Both { vertical, .. } => {
                vertical.alignment = alignment;
            }
            Direction::Horizontal { .. } => {}
        }
        self
    }

    /// Sets the style of this [`VirtualScrollable`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the cell size for smooth sub-cell scrolling.
    ///
    /// When using [`show_viewport`] with a grid of uniform-sized cells (like tiles),
    /// call this method with the cell size to enable smooth scrolling. The widget
    /// will calculate the sub-cell offset and translate the content appropriately.
    ///
    /// For row-based scrolling, use [`show_rows`] which sets this automatically.
    ///
    /// # Example
    /// ```no_run
    /// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::core::Size; }
    /// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
    /// use iced::widget::virtual_scrollable;
    /// use iced::Size;
    ///
    /// enum Message {}
    ///
    /// let tile_size = 200.0;
    /// let canvas_size = 100_000.0;
    ///
    /// virtual_scrollable::show_viewport(
    ///     Size::new(canvas_size, canvas_size),
    ///     move |viewport| {
    ///         // Render visible tiles based on viewport
    ///         # iced::widget::text("").into()
    ///     }
    /// )
    /// .with_cell_size(Size::new(tile_size, tile_size));
    /// ```
    #[must_use]
    pub fn with_cell_size(mut self, cell_size: Size) -> Self {
        self.cell_size = Some(cell_size);
        self
    }

    /// Sets the style class of the [`VirtualScrollable`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

#[derive(Debug, Clone)]
struct State {
    offset_y: Offset,
    offset_x: Offset,
    interaction: Interaction,
    last_notified: Option<Viewport>,
    last_scrolled: Option<Instant>,
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
}

impl Default for State {
    fn default() -> Self {
        Self {
            offset_y: Offset::Absolute(0.0),
            offset_x: Offset::Absolute(0.0),
            interaction: Interaction::None,
            last_notified: None,
            last_scrolled: None,
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

    fn scroll_by(&mut self, offset: AbsoluteOffset, bounds: Rectangle, content_bounds: Rectangle) {
        self.scroll(Vector::new(offset.x, offset.y), bounds, content_bounds);
    }

    fn unsnap(&mut self, bounds: Rectangle, content_bounds: Rectangle) {
        self.offset_x =
            Offset::Absolute(self.offset_x.absolute(bounds.width, content_bounds.width));
        self.offset_y =
            Offset::Absolute(self.offset_y.absolute(bounds.height, content_bounds.height));
    }

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

    fn y_scroller_grabbed_at(&self) -> Option<f32> {
        if let Interaction::YScrollerGrabbed(at) = self.interaction {
            Some(at)
        } else {
            None
        }
    }

    fn x_scroller_grabbed_at(&self) -> Option<f32> {
        if let Interaction::XScrollerGrabbed(at) = self.interaction {
            Some(at)
        } else {
            None
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for VirtualScrollable<'_, Message, Theme, Renderer>
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
        // We'll create the child dynamically based on viewport
        vec![]
    }

    fn diff(&self, _tree: &mut Tree) {
        // Children are rebuilt each frame based on viewport
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
        let state = tree.state.downcast_ref::<State>();

        // Calculate bounds from limits
        let bounds = limits.resolve(self.width, self.height, Size::ZERO);

        // Determine content bounds based on declared content_size
        let content_width = if self.content_size.width > 0.0 {
            self.content_size.width
        } else {
            bounds.width
        };
        let content_height = if self.content_size.height > 0.0 {
            self.content_size.height
        } else {
            bounds.height
        };

        let content_bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: content_width,
            height: content_height,
        };

        // Calculate visible viewport in content coordinates
        let translation =
            state.translation(self.direction, Rectangle::with_size(bounds), content_bounds);

        let visible_viewport = {
            let (extra_width, extra_height) = self
                .cell_size
                .map(|cell_size| {
                    let max_scroll_x = (content_bounds.width - bounds.width).max(0.0);
                    let max_scroll_y = (content_bounds.height - bounds.height).max(0.0);

                    let at_end_x = translation.x >= max_scroll_x;
                    let at_end_y = translation.y >= max_scroll_y;

                    let extra_width = if cell_size.width > 0.0
                        && !at_end_x
                        && (translation.x % cell_size.width) != 0.0
                    {
                        cell_size.width
                    } else {
                        0.0
                    };

                    let extra_height = if cell_size.height > 0.0
                        && !at_end_y
                        && (translation.y % cell_size.height) != 0.0
                    {
                        cell_size.height
                    } else {
                        0.0
                    };

                    (extra_width, extra_height)
                })
                .unwrap_or((0.0, 0.0));

            let max_width = (content_bounds.width - translation.x).max(0.0);
            let max_height = (content_bounds.height - translation.y).max(0.0);

            Rectangle {
                x: translation.x,
                y: translation.y,
                width: (bounds.width + extra_width).min(max_width),
                height: (bounds.height + extra_height).min(max_height),
            }
        };

        // Create the visible content
        let mut content = (self.view)(visible_viewport);

        // Create a temporary tree for the content
        if tree.children.is_empty() {
            tree.children.push(Tree::new(content.as_widget()));
        } else {
            tree.children[0] = Tree::new(content.as_widget());
        }

        // Layout the visible content within the visible bounds
        let content_limits = layout::Limits::new(Size::ZERO, bounds);
        let content_node =
            content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &content_limits);

        // The main node has the visible bounds, with content as child
        layout::Node::with_children(bounds, vec![content_node])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();

        let content_bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: if self.content_size.width > 0.0 {
                self.content_size.width
            } else {
                bounds.width
            },
            height: if self.content_size.height > 0.0 {
                self.content_size.height
            } else {
                bounds.height
            },
        };

        let translation = state.translation(self.direction, bounds, content_bounds);

        operation.scrollable(self.id.as_ref(), bounds, content_bounds, translation, state);
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
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let cursor_over_scrollable = cursor.position_over(bounds);

        let content_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: if self.content_size.width > 0.0 {
                self.content_size.width
            } else {
                bounds.width
            },
            height: if self.content_size.height > 0.0 {
                self.content_size.height
            } else {
                bounds.height
            },
        };

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
            // Handle Y scrollbar dragging
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

            // Handle X scrollbar dragging
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

            // Forward events to content if not interacting with scrollbars
            if state.last_scrolled.is_none()
                || !matches!(event, Event::Mouse(mouse::Event::WheelScrolled { .. }))
            {
                let translation = state.translation(self.direction, bounds, content_bounds);

                // Content is rendered without translation. Give it viewport-local coordinates.
                let cursor = match cursor_over_scrollable {
                    Some(cursor_position)
                        if !(mouse_over_x_scrollbar || mouse_over_y_scrollbar) =>
                    {
                        mouse::Cursor::Available(cursor_position)
                    }
                    _ => cursor.levitate(),
                };

                // Rebuild content for current viewport and update it
                let visible_viewport = {
                    let (extra_width, extra_height) = self
                        .cell_size
                        .map(|cell_size| {
                            let max_scroll_x = (content_bounds.width - bounds.width).max(0.0);
                            let max_scroll_y = (content_bounds.height - bounds.height).max(0.0);

                            let at_end_x = translation.x >= max_scroll_x;
                            let at_end_y = translation.y >= max_scroll_y;

                            let extra_width = if cell_size.width > 0.0
                                && !at_end_x
                                && (translation.x % cell_size.width) != 0.0
                            {
                                cell_size.width
                            } else {
                                0.0
                            };

                            let extra_height = if cell_size.height > 0.0
                                && !at_end_y
                                && (translation.y % cell_size.height) != 0.0
                            {
                                cell_size.height
                            } else {
                                0.0
                            };

                            (extra_width, extra_height)
                        })
                        .unwrap_or((0.0, 0.0));

                    let max_width = (content_bounds.width - translation.x).max(0.0);
                    let max_height = (content_bounds.height - translation.y).max(0.0);

                    Rectangle {
                        x: translation.x,
                        y: translation.y,
                        width: (bounds.width + extra_width).min(max_width),
                        height: (bounds.height + extra_height).min(max_height),
                    }
                };

                let mut content = (self.view)(visible_viewport);

                if tree.children.is_empty() {
                    tree.children.push(Tree::new(content.as_widget()));
                } else {
                    tree.children[0].diff(content.as_widget());
                }

                let content_layout = layout.children().next().unwrap();

                content.as_widget_mut().update(
                    &mut tree.children[0],
                    event,
                    content_layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    &bounds,
                );
            }

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

                    // Stop any kinetic scrolling
                    state.velocity = Vector::new(0.0, 0.0);
                    state.last_kinetic_update = None;
                    // Cancel any animated scroll
                    state.scroll_to_target = None;
                    state.scroll_to_animation = None;

                    let delta = match *delta {
                        mouse::ScrollDelta::Lines { x, y } => {
                            let is_shift_pressed = modifiers.shift();

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
                            let smoothing = 0.3;
                            state.velocity.x = state.velocity.x * (1.0 - smoothing)
                                + instant_velocity.x * smoothing;
                            state.velocity.y = state.velocity.y * (1.0 - smoothing)
                                + instant_velocity.y * smoothing;

                            state.scroll(self.direction.align(delta), bounds, content_bounds);
                            state.interaction = Interaction::TouchScrolling {
                                last_position: cursor_position,
                                last_time: now,
                            };

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
                Event::Window(window::Event::RedrawRequested(now)) => {
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

        let Some(visible_bounds) = bounds.intersection(viewport) else {
            return;
        };

        let content_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: if self.content_size.width > 0.0 {
                self.content_size.width
            } else {
                bounds.width
            },
            height: if self.content_size.height > 0.0 {
                self.content_size.height
            } else {
                bounds.height
            },
        };

        let scrollbars = Scrollbars::new(state, self.direction, bounds, content_bounds);
        let cursor_over_scrollable = cursor.position_over(bounds);
        let (mouse_over_y_scrollbar, mouse_over_x_scrollbar) = scrollbars.is_mouse_over(cursor);

        let translation = state.translation(self.direction, bounds, content_bounds);

        // Virtual scrolling: we render content for the visible viewport, but we do NOT
        // translate the rendered content. Therefore, the content receives viewport-local
        // cursor coordinates.
        let cursor = match cursor_over_scrollable {
            Some(cursor_position) if !(mouse_over_x_scrollbar || mouse_over_y_scrollbar) => {
                mouse::Cursor::Available(cursor_position)
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

        // Draw virtual content
        if scrollbars.active() {
            // Calculate visible viewport in content coordinates (+ overscan when needed)
            let visible_viewport = {
                let (extra_width, extra_height) = self
                    .cell_size
                    .map(|cell_size| {
                        let max_scroll_x = (content_bounds.width - bounds.width).max(0.0);
                        let max_scroll_y = (content_bounds.height - bounds.height).max(0.0);

                        let at_end_x = translation.x >= max_scroll_x;
                        let at_end_y = translation.y >= max_scroll_y;

                        let extra_width = if cell_size.width > 0.0
                            && !at_end_x
                            && (translation.x % cell_size.width) != 0.0
                        {
                            cell_size.width
                        } else {
                            0.0
                        };

                        let extra_height = if cell_size.height > 0.0
                            && !at_end_y
                            && (translation.y % cell_size.height) != 0.0
                        {
                            cell_size.height
                        } else {
                            0.0
                        };

                        (extra_width, extra_height)
                    })
                    .unwrap_or((0.0, 0.0));

                let max_width = (content_bounds.width - translation.x).max(0.0);
                let max_height = (content_bounds.height - translation.y).max(0.0);

                Rectangle {
                    x: translation.x,
                    y: translation.y,
                    width: (bounds.width + extra_width).min(max_width),
                    height: (bounds.height + extra_height).min(max_height),
                }
            };

            // Generate content for visible viewport
            let content = (self.view)(visible_viewport);

            // Get the tree for the content - use existing child tree if available
            let content_tree = if let Some(child) = tree.children.first() {
                child
            } else {
                // Fallback: create a temporary tree (this shouldn't happen normally)
                &Tree::new(content.as_widget())
            };

            renderer.with_layer(visible_bounds, |renderer| {
                if let Some(content_layout) = layout.children().next() {
                    // Calculate translation offset for smooth scrolling
                    // Use cell_size for sub-cell offset, or fractional pixel offset as fallback
                    let (offset_x, offset_y) = if let Some(cell_size) = self.cell_size {
                        let max_scroll_x = (content_bounds.width - bounds.width).max(0.0);
                        let max_scroll_y = (content_bounds.height - bounds.height).max(0.0);

                        let at_end_x = translation.x >= max_scroll_x;
                        let at_end_y = translation.y >= max_scroll_y;

                        // Sub-cell offset for smooth cell-based scrolling
                        let ox = if cell_size.width > 0.0 {
                            if at_end_x {
                                0.0
                            } else {
                                translation.x % cell_size.width
                            }
                        } else {
                            translation.x - translation.x.floor()
                        };
                        let oy = if cell_size.height > 0.0 {
                            if at_end_y {
                                0.0
                            } else {
                                translation.y % cell_size.height
                            }
                        } else {
                            translation.y - translation.y.floor()
                        };
                        (ox, oy)
                    } else {
                        // Fractional pixel offset for viewport-based scrolling
                        (
                            translation.x - translation.x.floor(),
                            translation.y - translation.y.floor(),
                        )
                    };

                    renderer.with_translation(Vector::new(-offset_x, -offset_y), |renderer| {
                        content.as_widget().draw(
                            content_tree,
                            renderer,
                            theme,
                            defaults,
                            content_layout,
                            cursor,
                            &visible_bounds,
                        );
                    });
                }
            });

            // Draw scrollbars
            let scroll_style = &style.scroll;
            let corner_radius = crate::core::border::rounded(scroll_style.corner_radius as u32);

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
            // No scrolling needed, render content directly
            let visible_viewport = Rectangle {
                x: 0.0,
                y: 0.0,
                width: bounds.width,
                height: bounds.height,
            };

            let content = (self.view)(visible_viewport);

            // Get the tree for the content
            let content_tree = if let Some(child) = tree.children.first() {
                child
            } else {
                &Tree::new(content.as_widget())
            };

            if let Some(content_layout) = layout.children().next() {
                content.as_widget().draw(
                    content_tree,
                    renderer,
                    theme,
                    defaults,
                    content_layout,
                    cursor,
                    &visible_bounds,
                );
            }
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

        let content_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: if self.content_size.width > 0.0 {
                self.content_size.width
            } else {
                bounds.width
            },
            height: if self.content_size.height > 0.0 {
                self.content_size.height
            } else {
                bounds.height
            },
        };

        let scrollbars = Scrollbars::new(state, self.direction, bounds, content_bounds);
        let (mouse_over_y_scrollbar, mouse_over_x_scrollbar) = scrollbars.is_mouse_over(cursor);

        if state.scrollers_grabbed() {
            return mouse::Interaction::None;
        }

        let translation = state.translation(self.direction, bounds, content_bounds);

        let cursor = match cursor_over_scrollable {
            Some(cursor_position) if !(mouse_over_x_scrollbar || mouse_over_y_scrollbar) => {
                mouse::Cursor::Available(cursor_position)
            }
            _ => cursor.levitate(),
        };

        // Get mouse interaction from content
        let visible_viewport = Rectangle {
            x: translation.x,
            y: translation.y,
            width: bounds.width,
            height: bounds.height,
        };

        let content = (self.view)(visible_viewport);

        // Get the tree for the content
        let content_tree = if let Some(child) = tree.children.first() {
            child
        } else {
            &Tree::new(content.as_widget())
        };

        if let Some(content_layout) = layout.children().next() {
            content.as_widget().mouse_interaction(
                content_tree,
                content_layout,
                cursor,
                &bounds,
                renderer,
            )
        } else {
            mouse::Interaction::None
        }
    }

    fn overlay<'b>(
        &'b mut self,
        _tree: &'b mut Tree,
        _layout: Layout<'b>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        _translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        // Virtual scrollable doesn't support overlays from content
        // (would require more complex state management)
        None
    }
}

impl<'a, Message, Theme, Renderer> From<VirtualScrollable<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a + Catalog,
    Renderer: 'a + text::Renderer,
{
    fn from(scrollable: VirtualScrollable<'a, Message, Theme, Renderer>) -> Self {
        Element::new(scrollable)
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

    // Compute absolute offset from the internal Offset values
    let offset = AbsoluteOffset {
        x: state.offset_x.absolute(bounds.width, content_bounds.width),
        y: state
            .offset_y
            .absolute(bounds.height, content_bounds.height),
    };

    let viewport = Viewport::from_absolute(offset, bounds, content_bounds);

    if let Some(last_notified) = state.last_notified {
        let last_relative_offset = last_notified.relative_offset();
        let current_relative_offset = viewport.relative_offset();

        let last_absolute_offset = last_notified.absolute_offset();
        let current_absolute_offset = viewport.absolute_offset();

        let unchanged =
            |a: f32, b: f32| (a - b).abs() <= f32::EPSILON || (a.is_nan() && b.is_nan());

        if last_notified.bounds() == bounds
            && last_notified.content_bounds() == content_bounds
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

#[derive(Debug)]
struct Scrollbars {
    y: Option<internals::Scrollbar>,
    x: Option<internals::Scrollbar>,
}

impl Scrollbars {
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

            let x_scrollbar_height =
                show_scrollbar_x.map_or(0.0, |h| h.width.max(h.scroller_width) + h.margin);

            let total_scrollbar_width = width.max(scroller_width) + 2.0 * margin;

            let total_scrollbar_bounds = Rectangle {
                x: bounds.x + bounds.width - total_scrollbar_width,
                y: bounds.y,
                width: total_scrollbar_width,
                height: (bounds.height - x_scrollbar_height).max(0.0),
            };

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

            let scrollbar_y_width =
                y_scrollbar.map_or(0.0, |scrollbar| scrollbar.total_bounds.width);

            let total_scrollbar_height = width.max(scroller_width) + 2.0 * margin;

            let total_scrollbar_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + bounds.height - total_scrollbar_height,
                width: (bounds.width - scrollbar_y_width).max(0.0),
                height: total_scrollbar_height,
            };

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

mod internals {
    use super::Anchor;
    use crate::core::{Point, Rectangle};

    #[derive(Debug, Copy, Clone)]
    pub struct Scrollbar {
        pub total_bounds: Rectangle,
        pub bounds: Rectangle,
        pub scroller: Option<Scroller>,
        pub alignment: Anchor,
        pub disabled: bool,
    }

    impl Scrollbar {
        pub fn is_mouse_over(&self, cursor_position: Point) -> bool {
            self.total_bounds.contains(cursor_position)
        }

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

    #[derive(Debug, Clone, Copy)]
    pub struct Scroller {
        pub bounds: Rectangle,
    }
}
