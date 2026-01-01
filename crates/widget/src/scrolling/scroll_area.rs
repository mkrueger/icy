//! A unified scroll area widget for both regular and virtualized scrolling.
//!
//! This module provides a single [`ScrollArea`] builder with three content modes:
//!
//! - [`show`](ScrollArea::show) - Regular scrolling with measured content
//! - [`show_viewport`](ScrollArea::show_viewport) - Virtual scrolling with a viewport callback
//! - [`show_rows`](ScrollArea::show_rows) - Virtual scrolling optimized for uniform-height rows
//!
//! # Example: Regular Scrolling
//! ```no_run
//! # mod iced { pub mod widget { pub use icy_ui_widget::*; } }
//! # pub type Element<'a, Message> = icy_ui_widget::core::Element<'a, Message, icy_ui_widget::Theme, icy_ui_widget::Renderer>;
//! use icy_ui::widget::{column, scroll_area, space};
//!
//! enum Message {}
//!
//! fn view() -> Element<'static, Message> {
//!     scroll_area()
//!         .show(column![
//!             "Scroll me!",
//!             space().height(3000),
//!             "You did it!",
//!         ])
//!         .into()
//! }
//! ```
//!
//! # Example: Virtual List with `show_rows`
//! ```no_run
//! # mod iced { pub mod widget { pub use icy_ui_widget::*; } }
//! # pub type Element<'a, Message> = icy_ui_widget::core::Element<'a, Message, icy_ui_widget::Theme, icy_ui_widget::Renderer>;
//! use icy_ui::widget::{column, scroll_area, text};
//!
//! enum Message {}
//!
//! fn view(items: &[String]) -> Element<'_, Message> {
//!     scroll_area()
//!         .show_rows(30.0, items.len(), |range| {
//!             column(
//!                 items[range.clone()]
//!                     .iter()
//!                     .map(|item| text(item).into())
//!             ).into()
//!         })
//!         .into()
//! }
//! ```
//!
//! # Example: Custom Virtualization with `show_viewport`
//! ```no_run
//! # mod iced { pub mod widget { pub use icy_ui_widget::*; } pub use icy_ui_widget::core::Size; }
//! # pub type Element<'a, Message> = icy_ui_widget::core::Element<'a, Message, icy_ui_widget::Theme, icy_ui_widget::Renderer>;
//! use icy_ui::widget::{column, scroll_area, text};
//! use icy_ui::Size;
//!
//! enum Message {}
//!
//! fn view(items: &[String]) -> Element<'_, Message> {
//!     let row_height = 30.0;
//!     let total_height = items.len() as f32 * row_height;
//!
//!     scroll_area()
//!         .show_viewport(Size::new(400.0, total_height), |viewport| {
//!             let first = (viewport.y / row_height).floor() as usize;
//!             let last = ((viewport.y + viewport.height) / row_height).ceil() as usize;
//!             let visible = first..last.min(items.len());
//!
//!             column(
//!                 items[visible]
//!                     .iter()
//!                     .map(|item| text(item).into())
//!             ).into()
//!         })
//!         .into()
//! }
//! ```

use crate::core::text;
use crate::core::widget;
use crate::core::{Element, Length, Pixels, Rectangle, Size};

use super::scrollable::{self, Scrollable};
use super::virtual_scrollable::VirtualScrollable;

// Re-export common types from scrollable
pub use scrollable::{
    AbsoluteOffset, Anchor, Catalog, Direction, RelativeOffset, ScrollStyle, Scrollbar, Status,
    Style, StyleFn, Viewport,
};

use std::marker::PhantomData;
use std::ops::Range;

/// A builder for creating scroll areas with different content modes.
///
/// Use [`scroll_area()`](crate::scroll_area) to create a new builder.
pub struct ScrollArea<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    id: Option<widget::Id>,
    width: Length,
    height: Length,
    direction: Direction,
    auto_scroll: bool,
    style: Option<StyleFn<'a, Theme>>,
    _phantom: PhantomData<(Message, Renderer)>,
}

impl<'a, Message, Theme, Renderer> ScrollArea<'a, Message, Theme, Renderer>
where
    Theme: Catalog + 'a,
    Renderer: text::Renderer,
{
    /// Creates a new [`ScrollArea`] builder with default settings.
    pub fn new() -> Self {
        ScrollArea {
            id: None,
            width: Length::Shrink,
            height: Length::Shrink,
            direction: Direction::default(),
            auto_scroll: false,
            style: None,
            _phantom: PhantomData,
        }
    }

    /// Sets the [`widget::Id`] of the [`ScrollArea`].
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the width of the [`ScrollArea`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`ScrollArea`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Makes the [`ScrollArea`] scroll horizontally with default [`Scrollbar`] settings.
    pub fn horizontal(mut self) -> Self {
        self.direction = Direction::Horizontal(Scrollbar::default());
        self
    }

    /// Sets the [`Direction`] of the [`ScrollArea`].
    pub fn direction(mut self, direction: impl Into<Direction>) -> Self {
        self.direction = direction.into();
        self
    }

    /// Anchors the vertical scrolling direction to the top.
    pub fn anchor_top(self) -> Self {
        self.anchor_y(Anchor::Start)
    }

    /// Anchors the vertical scrolling direction to the bottom.
    pub fn anchor_bottom(self) -> Self {
        self.anchor_y(Anchor::End)
    }

    /// Anchors the horizontal scrolling direction to the left.
    pub fn anchor_left(self) -> Self {
        self.anchor_x(Anchor::Start)
    }

    /// Anchors the horizontal scrolling direction to the right.
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

    /// Embeds the [`Scrollbar`] into the [`ScrollArea`], instead of floating on top.
    ///
    /// The `spacing` provided will be the space between the scrollbar and the contents.
    pub fn spacing(mut self, new_spacing: impl Into<Pixels>) -> Self {
        match &mut self.direction {
            Direction::Horizontal(scrollbar) | Direction::Vertical(scrollbar) => {
                scrollbar.spacing = Some(new_spacing.into().0);
            }
            Direction::Both {
                vertical,
                horizontal,
            } => {
                let spacing = new_spacing.into().0;
                vertical.spacing = Some(spacing);
                horizontal.spacing = Some(spacing);
            }
        }
        self
    }

    /// Sets whether auto-scroll with the middle mouse button is enabled.
    ///
    /// By default, it is disabled.
    pub fn auto_scroll(mut self, auto_scroll: bool) -> Self {
        self.auto_scroll = auto_scroll;
        self
    }

    /// Sets the style of this [`ScrollArea`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self {
        self.style = Some(Box::new(style));
        self
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Content modes
    // ─────────────────────────────────────────────────────────────────────────

    /// Shows regular scrollable content.
    ///
    /// The content is fully laid out and measured, and scroll bars appear
    /// when the content exceeds the available space.
    ///
    /// For very large content (thousands of items), consider using
    /// [`show_viewport`](Self::show_viewport) or [`show_rows`](Self::show_rows) instead.
    pub fn show(
        self,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Scrollable<'a, Message, Theme, Renderer>
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        let mut scrollable =
            Scrollable::with_direction(content, self.direction).auto_scroll(self.auto_scroll);

        if let Some(id) = self.id {
            scrollable = scrollable.id(id);
        }
        scrollable = scrollable.width(self.width).height(self.height);

        // Apply spacing if set
        match &self.direction {
            Direction::Horizontal(sb) | Direction::Vertical(sb) => {
                if let Some(spacing) = sb.spacing {
                    scrollable = scrollable.spacing(spacing);
                }
            }
            Direction::Both {
                vertical,
                horizontal,
            } => {
                if let Some(spacing) = vertical.spacing.or(horizontal.spacing) {
                    scrollable = scrollable.spacing(spacing);
                }
            }
        }

        if let Some(style) = self.style {
            scrollable = scrollable.style(style);
        }

        scrollable
    }

    /// Shows virtualized content based on the visible viewport.
    ///
    /// The `content_size` declares the total logical size of the content
    /// (used for scrollbar calculations). The `view` callback receives
    /// the visible viewport rectangle (in content-local coordinates)
    /// and must return the content to render.
    ///
    /// This is the most flexible virtualization approach - you decide what
    /// to render based on the viewport.
    ///
    /// # Example
    /// ```no_run
    /// # mod iced { pub mod widget { pub use icy_ui_widget::*; } pub use icy_ui_widget::core::Size; }
    /// # pub type Element<'a, Message> = icy_ui_widget::core::Element<'a, Message, icy_ui_widget::Theme, icy_ui_widget::Renderer>;
    /// use icy_ui::widget::scroll_area;
    /// use icy_ui::Size;
    ///
    /// enum Message {}
    ///
    /// // Virtual canvas: only render what's visible
    /// let element: Element<'_, Message> = scroll_area()
    ///     .show_viewport(Size::new(10000.0, 10000.0), |viewport| {
    ///         // Render only visible tiles/content based on viewport
    ///         # iced::widget::text("").into()
    ///     })
    ///     .into();
    /// ```
    pub fn show_viewport(
        self,
        content_size: Size,
        view: impl Fn(Rectangle) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> VirtualScrollable<'a, Message, Theme, Renderer>
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        let mut virtual_scrollable = VirtualScrollable::new(content_size, view)
            .direction(self.direction)
            .auto_scroll(self.auto_scroll);

        if let Some(id) = self.id {
            virtual_scrollable = virtual_scrollable.id(id);
        }

        // For virtual scrolling, default to Fill if Shrink was set
        let width = match self.width {
            Length::Shrink => Length::Fill,
            w => w,
        };
        let height = match self.height {
            Length::Shrink => Length::Fill,
            h => h,
        };
        virtual_scrollable = virtual_scrollable.width(width).height(height);

        if let Some(style) = self.style {
            virtual_scrollable = virtual_scrollable.style(style);
        }

        virtual_scrollable
    }

    /// Shows virtualized content optimized for uniform-height rows.
    ///
    /// This is a convenience wrapper for the common case of a list with
    /// rows of equal height. The callback receives the range of visible
    /// row indices.
    ///
    /// # Arguments
    /// * `row_height` - The height of each row in pixels
    /// * `total_rows` - The total number of rows
    /// * `view` - A callback that receives the visible row range
    ///
    /// # Example
    /// ```no_run
    /// # mod iced { pub mod widget { pub use icy_ui_widget::*; } }
    /// # pub type Element<'a, Message> = icy_ui_widget::core::Element<'a, Message, icy_ui_widget::Theme, icy_ui_widget::Renderer>;
    /// use icy_ui::widget::{column, scroll_area, text};
    ///
    /// enum Message {}
    ///
    /// let items: Vec<String> = (0..100_000).map(|i| format!("Item {}", i)).collect();
    ///
    /// let element: Element<'_, Message> = scroll_area()
    ///     .show_rows(30.0, items.len(), |visible_range| {
    ///         column(
    ///             items[visible_range]
    ///                 .iter()
    ///                 .map(|item| text(item).into())
    ///         ).into()
    ///     })
    ///     .into();
    /// ```
    pub fn show_rows(
        self,
        row_height: f32,
        total_rows: usize,
        view: impl Fn(Range<usize>) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> VirtualScrollable<'a, Message, Theme, Renderer>
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        let mut virtual_scrollable = VirtualScrollable::with_rows(row_height, total_rows, view)
            .direction(self.direction)
            .auto_scroll(self.auto_scroll);

        if let Some(id) = self.id {
            virtual_scrollable = virtual_scrollable.id(id);
        }

        // For virtual scrolling, default to Fill if Shrink was set
        let width = match self.width {
            Length::Shrink => Length::Fill,
            w => w,
        };
        let height = match self.height {
            Length::Shrink => Length::Fill,
            h => h,
        };
        virtual_scrollable = virtual_scrollable.width(width).height(height);

        if let Some(style) = self.style {
            virtual_scrollable = virtual_scrollable.style(style);
        }

        virtual_scrollable
    }
}

impl<Message, Theme, Renderer> Default for ScrollArea<'_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn default() -> Self {
        ScrollArea {
            id: None,
            width: Length::Shrink,
            height: Length::Shrink,
            direction: Direction::default(),
            auto_scroll: false,
            style: None,
            _phantom: PhantomData,
        }
    }
}

/// Creates a new [`ScrollArea`] builder.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use icy_ui_widget::*; } }
/// # pub type Element<'a, Message> = icy_ui_widget::core::Element<'a, Message, icy_ui_widget::Theme, icy_ui_widget::Renderer>;
/// use icy_ui::widget::{column, scroll_area, text};
///
/// enum Message {}
///
/// // Regular scrolling
/// let regular: Element<'static, Message> = scroll_area()
///     .show(column![
///         text("Line 1"),
///         text("Line 2"),
///         // ... many more lines
///     ])
///     .into();
///
/// // Virtual scrolling for large lists
/// let virtual_list: Element<'static, Message> = scroll_area()
///     .show_rows(30.0, 100_000, |range| {
///         column(range.map(|i| text(format!("Row {}", i)).into())).into()
///     })
///     .into();
/// ```
pub fn scroll_area<'a, Message, Theme, Renderer>() -> ScrollArea<'a, Message, Theme, Renderer>
where
    Theme: Catalog + 'a,
    Renderer: text::Renderer,
{
    ScrollArea::new()
}
