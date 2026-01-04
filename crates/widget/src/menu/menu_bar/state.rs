// From iced_aw, license MIT
// Ported from libcosmic

//! Menu bar state management

use std::cell::RefCell;
use std::rc::Rc;

use crate::core::{mouse::Cursor, widget::Tree};

use super::super::menu_inner::{Direction, MenuState};

/// Reference-counted wrapper for menu bar state
pub(crate) struct RcWrapper<T> {
    inner: Rc<RefCell<T>>,
}

impl<T> Clone for RcWrapper<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T: Default> Default for RcWrapper<T> {
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(T::default())),
        }
    }
}

impl<T> RcWrapper<T> {
    pub fn with_data<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&self.inner.borrow())
    }

    pub fn with_data_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        f(&mut self.inner.borrow_mut())
    }
}

/// Menu bar state wrapper
#[derive(Clone, Default)]
pub(crate) struct MenuBarState {
    pub(crate) inner: RcWrapper<MenuBarStateInner>,
}

/// Inner state data for the menu bar
pub(crate) struct MenuBarStateInner {
    pub(crate) tree: Tree,
    pub(crate) pressed: bool,
    pub(crate) view_cursor: Cursor,
    pub(crate) open: bool,
    pub(crate) active_root: Vec<usize>,
    pub(crate) horizontal_direction: Direction,
    pub(crate) vertical_direction: Direction,
    pub(crate) menu_states: Vec<MenuState>,
    /// Whether the Alt key is currently pressed
    pub(crate) alt_pressed: bool,
    /// Whether to show mnemonic underlines (based on Alt state and display mode)
    pub(crate) show_mnemonics: bool,
    /// Index of the menu root that has accessibility focus (for VoiceOver)
    #[cfg(feature = "accessibility")]
    pub(crate) a11y_focused_root: Option<usize>,
}

impl MenuBarStateInner {
    /// get the list of indices hovered for the menu
    pub(crate) fn get_trimmed_indices(&self, index: usize) -> impl Iterator<Item = usize> + '_ {
        self.menu_states
            .iter()
            .skip(index)
            .take_while(|ms| ms.index.is_some())
            .map(|ms| ms.index.expect("No indices were found in the menu state."))
    }

    pub(crate) fn reset(&mut self) {
        self.open = false;
        self.active_root = Vec::new();
        self.menu_states.clear();
        self.alt_pressed = false;
        self.show_mnemonics = false;
    }
}

impl Default for MenuBarStateInner {
    fn default() -> Self {
        Self {
            tree: Tree::empty(),
            pressed: false,
            view_cursor: Cursor::Available([-0.5, -0.5].into()),
            open: false,
            active_root: Vec::new(),
            horizontal_direction: Direction::Positive,
            vertical_direction: Direction::Positive,
            menu_states: Vec::new(),
            alt_pressed: false,
            show_mnemonics: false,
            #[cfg(feature = "accessibility")]
            a11y_focused_root: None,
        }
    }
}
