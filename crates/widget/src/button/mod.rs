//! Button widgets and variants.
//!
//! This module provides the core [`Button`] widget and specialized variants:
//! - [`text`] - Text button with optional leading/trailing icons
//! - [`icon`] - Icon-only button with minimal styling
//! - [`hyperlink`] - Hyperlink that opens URLs in the browser (shows underline on hover)
//! - [`image`] - Image button with optional selection/removal
//!
//! # Example
//! ```no_run
//! use icy_ui::widget::button;
//!
//! // Standard button
//! let btn = button("Click me").on_press(Message::Click);
//!
//! // Text button with icons
//! let save = button::text::text_button("Save")
//!     .leading_icon(save_icon)
//!     .on_press(Message::Save);
//!
//! // Icon-only button
//! let settings = button::icon::icon_button(settings_icon)
//!     .on_press(Message::Settings);
//!
//! // Hyperlink (opens URL in browser, shows underline on hover)
//! let link = button::hyperlink("Visit GitHub", "https://github.com");
//! ```

mod widget;

pub mod icon;
#[cfg(feature = "image")]
pub mod image;
pub mod link;
pub mod text;

pub use icon::icon_button;
#[cfg(feature = "image")]
pub use image::image_button;
pub use link::{hyperlink, Hyperlink};
pub use text::text_button;
pub use widget::*;
