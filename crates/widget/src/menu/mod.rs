// From iced_aw, license MIT
// Ported from libcosmic

//! A [`MenuBar`] widget for displaying [`MenuTree`]s
//!
//! # Example
//!
//! ```ignore
//! use icy_ui::widget::button;
//! use icy_ui_widget::menu::{MenuTree, MenuBar};
//!
//! let sub_2 = MenuTree::with_children(
//!     button("Sub Menu 2"),
//!     vec![
//!         MenuTree::new(button("item_1")),
//!         MenuTree::new(button("item_2")),
//!         MenuTree::new(button("item_3")),
//!     ]
//! );
//!
//! let sub_1 = MenuTree::with_children(
//!     button("Sub Menu 1"),
//!     vec![
//!         MenuTree::new(button("item_1")),
//!         sub_2,
//!         MenuTree::new(button("item_2")),
//!         MenuTree::new(button("item_3")),
//!     ]
//! );
//!
//!
//! let root_1 = MenuTree::with_children(
//!     button("Menu 1"),
//!     vec![
//!         MenuTree::new(button("item_1")),
//!         MenuTree::new(button("item_2")),
//!         sub_1,
//!         MenuTree::new(button("item_3")),
//!     ]
//! );
//!
//! let root_2 = MenuTree::with_children(
//!     button("Menu 2"),
//!     vec![
//!         MenuTree::new(button("item_1")),
//!         MenuTree::new(button("item_2")),
//!         MenuTree::new(button("item_3")),
//!     ]
//! );
//!
//! let menu_bar = MenuBar::new(vec![root_1, root_2]);
//!
//! ```
//!

pub mod action;

pub use action::MenuAction as Action;

pub mod key_bind;
pub use key_bind::{KeyBind, Modifier};

mod menu_bar;
pub use menu_bar::{MenuBar, menu_bar as bar};

mod menu_inner;
mod menu_tree;
pub use menu_tree::{
    MenuItem as Item, MenuTree as Tree, menu_button, menu_items as items, menu_root as root,
};

mod app_menu;
pub use app_menu::{menu_bar, menu_bar_from};

pub use menu_inner::{CloseCondition, ItemHeight, ItemWidth, PathHighlight};

mod context_menu;
pub use context_menu::{ContextMenu, context_menu, context_menu_from};

mod mnemonic;
pub use mnemonic::{
    MnemonicDisplay, ParsedMnemonic, mnemonic_text, mnemonics_enabled, parse_mnemonic,
};

mod style;
pub use style::{Appearance, Style, StyleSheet, menu_folder, menu_item, menu_root_style};
