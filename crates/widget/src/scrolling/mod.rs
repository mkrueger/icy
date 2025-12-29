//! Scrolling widgets for displaying content that exceeds the available space.
//!
//! This module provides three complementary scrolling approaches:
//!
//! - [`scrollable`] - Traditional scrolling with fully measured content
//! - [`scroll_area`] - Unified builder API for both regular and virtual scrolling
//! - `virtual_scrollable` (internal) - Virtual scrolling implementation for large content
//!
//! # Quick Start
//!
//! For most use cases, use the [`scroll_area()`](crate::scroll_area) helper:
//!
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } }
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! use iced::widget::{column, scroll_area, space};
//!
//! enum Message {}
//!
//! // Regular scrolling
//! fn view_regular() -> Element<'static, Message> {
//!     scroll_area()
//!         .show(column![
//!             "Scroll me!",
//!             space().height(3000),
//!             "You did it!",
//!         ])
//!         .into()
//! }
//!
//! // Virtual scrolling for large lists
//! fn view_virtual(items: &[String]) -> Element<'_, Message> {
//!     use iced::widget::text;
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

pub mod scroll_area;
pub mod scrollable;
pub(crate) mod virtual_scrollable;

pub use scroll_area::ScrollArea;
pub use scrollable::Scrollable;
