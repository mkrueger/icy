// From iced_aw, license MIT
// Ported from libcosmic

//! A widget that handles menu trees

mod state;
mod widget;

pub(crate) use state::MenuBarState;

use crate::core::{Element, LayoutDirection, Length, Padding, renderer, widget::Tree};

use super::{
    menu_inner::{CloseCondition, ItemHeight, ItemWidth, PathHighlight},
    menu_tree::MenuTree,
    mnemonic::MnemonicDisplay,
    style::StyleSheet,
};

/// A `MenuBar` collects `MenuTree`s and handles all the layout, event processing, and drawing.
pub fn menu_bar<'a, Message, Theme, Renderer>(
    menu_roots: Vec<MenuTree<'a, Message, Theme, Renderer>>,
) -> MenuBar<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: renderer::Renderer + 'a,
    Theme: StyleSheet,
{
    MenuBar::new(menu_roots)
}

pub(crate) fn menu_roots_children<'a, Message, Theme, Renderer>(
    menu_roots: &[MenuTree<'a, Message, Theme, Renderer>],
) -> Vec<Tree>
where
    Message: Clone,
    Renderer: renderer::Renderer,
{
    menu_roots
        .iter()
        .map(|root| {
            let mut tree = Tree::empty();
            let flat = root
                .flatten()
                .iter()
                .map(|mt| Tree::new(&mt.item))
                .collect();
            tree.children = flat;
            tree
        })
        .collect()
}

pub(crate) fn menu_roots_diff<'a, Message, Theme, Renderer>(
    menu_roots: &[MenuTree<'a, Message, Theme, Renderer>],
    tree: &mut Tree,
) where
    Message: Clone,
    Renderer: renderer::Renderer,
{
    if tree.children.len() > menu_roots.len() {
        tree.children.truncate(menu_roots.len());
    }

    for (t, root) in tree.children.iter_mut().zip(menu_roots.iter()) {
        let flat: Vec<&Element<'_, Message, Theme, Renderer>> =
            root.flatten().iter().map(|mt| &mt.item).collect();
        t.diff_children(&flat);
    }

    if tree.children.len() < menu_roots.len() {
        let extended = menu_roots[tree.children.len()..].iter().map(|root| {
            let mut tree = Tree::empty();
            let flat = root
                .flatten()
                .iter()
                .map(|mt| Tree::new(&mt.item))
                .collect();
            tree.children = flat;
            tree
        });
        tree.children.extend(extended);
    }
}

/// A `MenuBar` collects `MenuTree`s and handles all the layout, event processing, and drawing.
#[allow(missing_debug_implementations)]
pub struct MenuBar<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer,
    Theme: StyleSheet,
{
    pub(crate) width: Length,
    pub(crate) height: Length,
    pub(crate) spacing: f32,
    pub(crate) padding: Padding,
    pub(crate) bounds_expand: u16,
    pub(crate) main_offset: i32,
    pub(crate) cross_offset: i32,
    pub(crate) close_condition: CloseCondition,
    pub(crate) item_width: ItemWidth,
    pub(crate) item_height: ItemHeight,
    pub(crate) path_highlight: Option<PathHighlight>,
    pub(crate) menu_roots: Vec<MenuTree<'a, Message, Theme, Renderer>>,
    pub(crate) style: Theme::Style,
    pub(crate) mnemonic_display: MnemonicDisplay,
    /// Override for layout direction. If `None`, uses the global style direction.
    pub(crate) layout_direction: Option<LayoutDirection>,
}

impl<'a, Message, Theme, Renderer> MenuBar<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: renderer::Renderer + 'a,
    Theme: StyleSheet,
{
    /// Creates a new [`MenuBar`] with the given menu roots
    #[must_use]
    pub fn new(menu_roots: Vec<MenuTree<'a, Message, Theme, Renderer>>) -> Self {
        let mut menu_roots = menu_roots;
        menu_roots.iter_mut().for_each(MenuTree::set_index);

        Self {
            width: Length::Shrink,
            height: Length::Shrink,
            spacing: 0.0,
            padding: Padding::ZERO,
            bounds_expand: 16,
            main_offset: 0,
            cross_offset: 0,
            close_condition: CloseCondition {
                leave: false,
                click_outside: true,
                click_inside: true,
            },
            item_width: ItemWidth::Uniform(240),
            item_height: ItemHeight::Dynamic(36),
            path_highlight: Some(PathHighlight::MenuActive),
            menu_roots,
            style: Theme::Style::default(),
            mnemonic_display: MnemonicDisplay::default(),
            layout_direction: None,
        }
    }

    /// Sets the expand value for each menu's check bounds
    ///
    /// When the cursor goes outside of a menu's check bounds,
    /// the menu will be closed automatically, this value expands
    /// the check bounds
    #[must_use]
    pub fn bounds_expand(mut self, value: u16) -> Self {
        self.bounds_expand = value;
        self
    }

    /// [`CloseCondition`]
    #[must_use]
    pub fn close_condition(mut self, close_condition: CloseCondition) -> Self {
        self.close_condition = close_condition;
        self
    }

    /// Moves each menu in the horizontal open direction
    #[must_use]
    pub fn cross_offset(mut self, value: i32) -> Self {
        self.cross_offset = value;
        self
    }

    /// Sets the height of the [`MenuBar`]
    #[must_use]
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// [`ItemHeight`]
    #[must_use]
    pub fn item_height(mut self, item_height: ItemHeight) -> Self {
        self.item_height = item_height;
        self
    }

    /// [`ItemWidth`]
    #[must_use]
    pub fn item_width(mut self, item_width: ItemWidth) -> Self {
        self.item_width = item_width;
        self
    }

    /// Moves all the menus in the vertical open direction
    #[must_use]
    pub fn main_offset(mut self, value: i32) -> Self {
        self.main_offset = value;
        self
    }

    /// Sets the [`Padding`] of the [`MenuBar`]
    #[must_use]
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the method for drawing path highlight
    #[must_use]
    pub fn path_highlight(mut self, path_highlight: Option<PathHighlight>) -> Self {
        self.path_highlight = path_highlight;
        self
    }

    /// Sets the spacing between menu roots
    #[must_use]
    pub fn spacing(mut self, units: f32) -> Self {
        self.spacing = units;
        self
    }

    /// Sets the style of the menu bar and its menus
    #[must_use]
    pub fn style(mut self, style: impl Into<Theme::Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the width of the [`MenuBar`]
    #[must_use]
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets how mnemonic underlines are displayed.
    ///
    /// - [`MnemonicDisplay::Hide`]: Never show underlines.
    /// - [`MnemonicDisplay::Show`]: Always show underlines.
    /// - [`MnemonicDisplay::OnAlt`]: Show underlines only when Alt is pressed (default).
    ///
    /// Note: Mnemonic keyboard navigation is only enabled on Windows and Linux.
    /// On macOS, mnemonics are disabled as the platform uses Cmd-based shortcuts.
    #[must_use]
    pub fn mnemonic_display(mut self, display: MnemonicDisplay) -> Self {
        self.mnemonic_display = display;
        self
    }

    /// Sets the layout direction of the [`MenuBar`].
    ///
    /// In RTL mode, submenus open to the left instead of the right.
    /// If not set, uses the global style direction.
    #[must_use]
    pub fn layout_direction(mut self, direction: LayoutDirection) -> Self {
        self.layout_direction = Some(direction);
        self
    }
}
