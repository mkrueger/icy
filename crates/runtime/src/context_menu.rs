//! Native context menu support.
//!
//! This module provides types for working with native platform context menus.
//! On macOS, this uses NSMenu, while on other platforms it falls back to the
//! custom widget-based overlay.
//!
//! # Usage
//!
//! Use the `context_menu` widget which automatically handles native menus:
//!
//! ```ignore
//! use icy_ui::widget::menu::context_menu;
//! use icy_ui_core::menu;
//!
//! let nodes = vec![
//!     menu::item!("Cut", Message::Cut),
//!     menu::item!("Copy", Message::Copy),
//!     menu::separator!(),
//!     menu::item!("Paste", Message::Paste),
//! ];
//!
//! // The widget automatically uses native menus on macOS
//! // and handles all event routing internally
//! context_menu(my_content, &nodes)
//! ```
//!
//! No subscription or event handling is required - the widget handles
//! everything and publishes the message directly when an item is selected.

use crate::core::menu::ContextMenuItem;

// Re-export context menu types from core for convenience
pub use crate::core::menu::{ContextMenuItem as MenuItem, ContextMenuItemKind as MenuItemKind};

/// Convert a slice of [`MenuNode`]s to a vector of [`ContextMenuItem`]s.
///
/// This extracts the label, enabled state, and structure from the nodes
/// while discarding the callbacks (which are handled by the widget).
pub fn menu_nodes_to_items<Message>(
    nodes: &[crate::core::menu::MenuNode<Message>],
) -> Vec<ContextMenuItem> {
    ContextMenuItem::from_menu_nodes(nodes)
}
