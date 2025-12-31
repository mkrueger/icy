// From iced_aw, license MIT
// Ported from libcosmic

//! Menu tree overlay
//!
//! This module contains the implementation for menu overlays, including:
//! - Menu bounds calculation
//! - Menu state management
//! - Menu rendering and event handling

mod bounds;
mod helpers;
mod menu;
mod overlay;
mod state;
mod types;

// Public re-exports
pub use types::{CloseCondition, ItemHeight, ItemWidth, PathHighlight};

// Crate-internal re-exports - use pub(in crate::menu) for menu-module visibility
pub(in crate::menu) use menu::Menu;
pub(in crate::menu) use state::MenuState;
pub(super) use types::Direction;
