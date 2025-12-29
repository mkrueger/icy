// From iced_aw, license MIT
// Ported from libcosmic

//! A tree structure for constructing a hierarchical menu

use std::borrow::Cow;
use std::collections::HashMap;

use crate::core::{Alignment, Element, Length, renderer};

use super::action::MenuAction;
use super::key_bind::KeyBind;
use crate::{Button, Row, Space, button, text};

/// Nested menu is essentially a tree of items, a menu is a collection of items
/// a menu itself can also be an item of another menu.
///
/// A `MenuTree` represents a node in the tree, it holds a widget as a menu item
/// for its parent, and a list of menu tree as child nodes.
/// Conceptually a node is either a menu(inner node) or an item(leaf node),
/// but there's no need to explicitly distinguish them here, if a menu tree
/// has children, it's a menu, otherwise it's an item
#[allow(missing_debug_implementations)]
pub struct MenuTree<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Renderer: renderer::Renderer,
{
    /// The menu tree will be flatten into a vector to build a linear widget tree,
    /// the `index` field is the index of the item in that vector
    pub(crate) index: usize,

    /// The item of the menu tree
    pub(crate) item: Element<'a, Message, Theme, Renderer>,

    /// The children of the menu tree
    pub(crate) children: Vec<MenuTree<'a, Message, Theme, Renderer>>,

    /// The width of the menu tree
    pub(crate) width: Option<u16>,

    /// The height of the menu tree
    pub(crate) height: Option<u16>,

    /// Whether this item is a separator (divider)
    pub(crate) is_separator: bool,
}

impl<'a, Message, Theme, Renderer> MenuTree<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: renderer::Renderer + 'a,
{
    /// Create a new menu tree from a widget
    pub fn new(item: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            index: 0,
            item: item.into(),
            children: Vec::new(),
            width: None,
            height: None,
            is_separator: false,
        }
    }

    /// Create a new separator menu tree
    pub fn separator(item: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            index: 0,
            item: item.into(),
            children: Vec::new(),
            width: None,
            height: None,
            is_separator: true,
        }
    }

    /// Create a menu tree from a widget and a vector of sub trees
    pub fn with_children(
        item: impl Into<Element<'a, Message, Theme, Renderer>>,
        children: Vec<MenuTree<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            index: 0,
            item: item.into(),
            children,
            width: None,
            height: None,
            is_separator: false,
        }
    }

    /// Sets the width of the menu tree.
    /// See [`ItemWidth`]
    ///
    /// [`ItemWidth`]:`super::ItemWidth`
    #[must_use]
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the height of the menu tree.
    /// See [`ItemHeight`]
    ///
    /// [`ItemHeight`]: `super::ItemHeight`
    #[must_use]
    pub fn height(mut self, height: u16) -> Self {
        self.height = Some(height);
        self
    }

    /* Keep `set_index()` and `flatten()` recurse in the same order */

    /// Set the index of each item
    pub(super) fn set_index(&mut self) {
        /// inner counting function.
        fn rec<Message, Theme, Renderer>(
            mt: &mut MenuTree<'_, Message, Theme, Renderer>,
            count: &mut usize,
        ) where
            Renderer: renderer::Renderer,
        {
            // keep items under the same menu line up
            mt.children.iter_mut().for_each(|c| {
                c.index = *count;
                *count += 1;
            });

            mt.children.iter_mut().for_each(|c| rec(c, count));
        }

        let mut count = 0;
        self.index = count;
        count += 1;
        rec(self, &mut count);
    }

    /// Flatten the menu tree
    pub(crate) fn flatten(&self) -> Vec<&Self> {
        /// Inner flattening function
        fn rec<'a, 'b, Message, Theme, Renderer>(
            mt: &'a MenuTree<'b, Message, Theme, Renderer>,
            flat: &mut Vec<&'a MenuTree<'b, Message, Theme, Renderer>>,
        ) where
            Renderer: renderer::Renderer,
        {
            mt.children.iter().for_each(|c| {
                flat.push(c);
            });

            mt.children.iter().for_each(|c| {
                rec(c, flat);
            });
        }

        let mut flat = Vec::new();
        flat.push(self);
        rec(self, &mut flat);

        flat
    }
}

/// Creates a menu button with the given children.
pub fn menu_button<'a, Message>(
    children: Vec<Element<'a, Message, crate::Theme, crate::Renderer>>,
) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    button(
        Row::from_vec(children)
            .align_y(Alignment::Center)
            .height(Length::Fill)
            .width(Length::Fill),
    )
    .height(36.0)
    .padding([4, 16])
    .width(Length::Fill)
    .style(super::style::menu_item)
}

/// Represents a menu item that performs an action when selected or a separator between menu items.
#[derive(Clone)]
pub enum MenuItem<A: MenuAction, L: Into<Cow<'static, str>>> {
    /// Represents a button menu item.
    Button(L, A),
    /// Represents a button menu item that is disabled.
    ButtonDisabled(L, A),
    /// Represents a checkbox menu item.
    CheckBox(L, bool, A),
    /// Represents a folder menu item (submenu).
    Folder(L, Vec<MenuItem<A, L>>),
    /// Represents a divider between menu items.
    Divider,
}

/// Create a root menu item.
///
/// # Arguments
/// - `label` - The label of the menu item.
/// - `on_press` - A message to send when the menu root is activated.
///   Note: The MenuBar handles the actual click events to open menus.
///   This message is primarily used to enable the button visually.
///   You can use a "no-op" message variant that does nothing in your update function.
///
/// # Returns
/// - A button for the root menu item.
pub fn menu_root<'a, Message>(
    label: impl Into<Cow<'a, str>> + 'a,
    on_press: Message,
) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    let l: Cow<'a, str> = label.into();
    button(text(l.to_string()))
        .padding([4, 12])
        .on_press(on_press)
        .style(super::style::menu_root_style)
}

/// Create a list of menu items from a vector of `MenuItem`.
///
/// The `MenuItem` can be either an action or a separator.
///
/// # Arguments
/// - `key_binds` - A reference to a `HashMap` that maps `KeyBind` to `A`.
/// - `children` - A vector of `MenuItem`.
///
/// # Returns
/// - A vector of `MenuTree`.
#[must_use]
pub fn menu_items<'a, A, L, Message>(
    key_binds: &HashMap<KeyBind, A>,
    children: Vec<MenuItem<A, L>>,
) -> Vec<MenuTree<'a, Message, crate::Theme, crate::Renderer>>
where
    A: MenuAction<Message = Message>,
    L: Into<Cow<'static, str>> + Clone + 'static,
    Message: Clone + 'static,
{
    fn find_key<A: MenuAction>(action: &A, key_binds: &HashMap<KeyBind, A>) -> String {
        for (key_bind, key_action) in key_binds {
            if action == key_action {
                return key_bind.to_string();
            }
        }
        String::new()
    }

    let size = children.len();

    children
        .into_iter()
        .enumerate()
        .flat_map(|(i, item)| {
            let mut trees = vec![];

            match item {
                MenuItem::Button(label, action) => {
                    let l: Cow<'static, str> = label.into();
                    let key = find_key(&action, key_binds);
                    let items: Vec<Element<'_, Message, crate::Theme, crate::Renderer>> = vec![
                        text(l).into(),
                        Space::new().width(Length::Fill).into(),
                        text(key).into(),
                    ];

                    let menu_button = menu_button(items).on_press(action.message());
                    trees.push(MenuTree::new(menu_button));
                }
                MenuItem::ButtonDisabled(label, _action) => {
                    let l: Cow<'static, str> = label.into();
                    let items: Vec<Element<'_, Message, crate::Theme, crate::Renderer>> =
                        vec![text(l).into(), Space::new().width(Length::Fill).into()];

                    let menu_button = menu_button(items);
                    trees.push(MenuTree::new(menu_button));
                }
                MenuItem::CheckBox(label, value, action) => {
                    let key = find_key(&action, key_binds);
                    let l: Cow<'static, str> = label.into();

                    let check_mark = if value { "✓ " } else { "   " };

                    let items: Vec<Element<'_, Message, crate::Theme, crate::Renderer>> = vec![
                        text(check_mark).into(),
                        text(l).into(),
                        Space::new().width(Length::Fill).into(),
                        text(key).into(),
                    ];

                    trees.push(MenuTree::new(menu_button(items).on_press(action.message())));
                }
                MenuItem::Folder(label, sub_children) => {
                    let l: Cow<'static, str> = label.clone().into();

                    trees.push(MenuTree::with_children(
                        menu_button(vec![
                            text(l).into(),
                            Space::new().width(Length::Fill).into(),
                            text("▶").into(),
                        ]),
                        menu_items(key_binds, sub_children),
                    ));
                }
                MenuItem::Divider => {
                    if i != size - 1 {
                        trees.push(MenuTree::separator(
                            crate::container(crate::rule::horizontal(1)).padding([4, 8]),
                        ));
                    }
                }
            }
            trees
        })
        .collect()
}
