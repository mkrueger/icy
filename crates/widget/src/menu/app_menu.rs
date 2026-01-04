//! Adapter from `icy_ui_core::menu` models to the widget `MenuBar`.

use std::borrow::Cow;

use crate::core::menu as app_menu;
use crate::core::{Alignment, Element, Length};

use super::menu_item_line::{MenuItemIcon, menu_item_line, menu_item_line_with_icon};
use super::mnemonic::{mnemonic_text, parse_mnemonic};
use super::style::menu_folder;
use super::{MenuBar, Tree, menu_button, menu_root_style};
use crate::{Row, button};

fn shortcut_text_style(theme: &crate::Theme) -> crate::text::Style {
    let mut color = theme.on_background();
    color.a *= 0.6;
    crate::text::Style { color: Some(color) }
}

/// Creates a submenu button with folder style (never appears disabled).
fn submenu_button<'a, Message>(
    children: Vec<Element<'a, Message, crate::Theme, crate::Renderer>>,
) -> crate::Button<'a, Message>
where
    Message: Clone + 'a,
{
    button(
        Row::from_vec(children)
            .align_y(Alignment::Center)
            .height(Length::Fill)
            .width(Length::Shrink),
    )
    .height(36.0)
    .padding([4, 16])
    .width(Length::Shrink)
    .style(menu_folder)
}

fn format_shortcut(shortcut: &app_menu::MenuShortcut) -> String {
    let mut parts: Vec<&'static str> = Vec::new();

    let m = shortcut.modifiers;
    if m.control() {
        parts.push("Ctrl");
    }
    if m.alt() {
        parts.push("Alt");
    }
    if m.shift() {
        parts.push("Shift");
    }
    if m.logo() {
        parts.push("Super");
    }

    let key = match shortcut.key.as_ref() {
        crate::core::keyboard::Key::Character(c) => c.to_uppercase(),
        crate::core::keyboard::Key::Named(named) => format!("{named:?}"),
        crate::core::keyboard::Key::Unidentified => String::new(),
    };

    if key.is_empty() {
        parts.join(" + ")
    } else if parts.is_empty() {
        key
    } else {
        format!("{} + {key}", parts.join(" + "))
    }
}

pub(super) fn convert_children<'a, Message>(
    children: &[app_menu::MenuNode<Message>],
) -> Vec<Tree<'a, Message, crate::Theme, crate::Renderer>>
where
    Message: Clone + 'static,
{
    let mut out: Vec<Tree<'a, Message, crate::Theme, crate::Renderer>> = Vec::new();

    for node in children {
        match &node.kind {
            app_menu::MenuKind::Separator => {
                // Avoid repeated separators
                if matches!(out.last(), Some(t) if t.is_separator) {
                    continue;
                }

                out.push(Tree::separator(
                    crate::container(crate::rule::horizontal(1)).padding([4, 8]),
                ));
            }
            _ => {
                if let Some(tree) = convert_node(node) {
                    out.push(tree);
                }
            }
        }
    }

    // Trim trailing separators
    while matches!(out.last(), Some(t) if t.is_separator) {
        let _ = out.pop();
    }

    out
}

pub(super) fn convert_node<'a, Message>(
    node: &app_menu::MenuNode<Message>,
) -> Option<Tree<'a, Message, crate::Theme, crate::Renderer>>
where
    Message: Clone + 'static,
{
    match &node.kind {
        app_menu::MenuKind::Item {
            label,
            enabled,
            shortcut,
            on_activate,
        } => {
            let l: Cow<'static, str> = Cow::Owned(label.clone());
            let parsed = parse_mnemonic(&l);

            let shortcut = shortcut.as_ref().map(format_shortcut).unwrap_or_default();

            let menu_button = menu_button(vec![menu_item_line(
                "",
                l.to_string(),
                shortcut,
                "",
                shortcut_text_style,
            )])
            .on_press_maybe(if *enabled {
                Some(on_activate.clone())
            } else {
                None
            });

            let mut tree = Tree::new(menu_button);
            tree.mnemonic = parsed.mnemonic_char;
            Some(tree)
        }

        app_menu::MenuKind::CheckItem {
            label,
            enabled,
            checked,
            shortcut,
            on_activate,
        } => {
            let l: Cow<'static, str> = Cow::Owned(label.clone());
            let parsed = parse_mnemonic(&l);

            let shortcut = shortcut.as_ref().map(format_shortcut).unwrap_or_default();

            let prefix_icon = match *checked {
                Some(true) => Some(MenuItemIcon::Checkmark),
                Some(false) => Some(MenuItemIcon::CheckboxBox),
                None => Some(MenuItemIcon::None),
            };

            let menu_button = menu_button(vec![menu_item_line_with_icon(
                prefix_icon,
                l.to_string(),
                shortcut,
                "",
                shortcut_text_style,
            )])
            .on_press_maybe(if *enabled {
                Some(on_activate.clone())
            } else {
                None
            });

            let mut tree = Tree::new(menu_button);
            tree.mnemonic = parsed.mnemonic_char;
            Some(tree)
        }

        app_menu::MenuKind::Submenu {
            label,
            enabled,
            children,
        } => {
            let l: Cow<'static, str> = Cow::Owned(label.clone());
            let parsed = parse_mnemonic(&l);

            if !*enabled {
                // Disabled submenu uses regular menu_button (will appear disabled)
                let mut tree = Tree::new(menu_button(vec![menu_item_line(
                    "",
                    l.to_string(),
                    "",
                    "▶",
                    shortcut_text_style,
                )]));
                tree.mnemonic = parsed.mnemonic_char;
                return Some(tree);
            }

            let child_trees = convert_children(children);

            // Enabled submenu uses submenu_button (never appears disabled)
            let mut tree = Tree::with_children(
                submenu_button(vec![menu_item_line(
                    "",
                    l.to_string(),
                    "",
                    "▶",
                    shortcut_text_style,
                )]),
                child_trees,
            );
            tree.mnemonic = parsed.mnemonic_char;
            Some(tree)
        }

        app_menu::MenuKind::Separator => None,
    }
}

/// Converts an [`app_menu::AppMenu`] into a widget [`MenuBar`].
///
/// Menu roots are rendered as passive buttons (they do not emit messages);
/// selecting a leaf item emits the `Message` stored in the menu model.
pub fn menu_bar_from<'a, Message>(
    menu: &app_menu::AppMenu<Message>,
) -> MenuBar<'a, Message, crate::Theme, crate::Renderer>
where
    Message: Clone + 'static,
{
    let roots: Vec<Tree<'a, Message, crate::Theme, crate::Renderer>> = menu
        .roots
        .iter()
        .filter_map(|node| match &node.kind {
            app_menu::MenuKind::Submenu {
                label,
                enabled: _,
                children,
            } => {
                let l: Cow<'static, str> = Cow::Owned(label.clone());
                let parsed = parse_mnemonic(&l);

                let root_button = button(
                    Row::new()
                        .push(mnemonic_text(&l))
                        .align_y(Alignment::Center)
                        .height(Length::Shrink)
                        .width(Length::Shrink),
                )
                .padding([4, 12])
                .style(menu_root_style);

                let mut tree = Tree::with_children(root_button, convert_children(children));
                tree.mnemonic = parsed.mnemonic_char;
                Some(tree)
            }
            // Non-submenu roots are supported but uncommon.
            _ => convert_node(node),
        })
        .collect();

    MenuBar::new(roots)
}

/// Creates a [`MenuBar`] directly from [`MenuNode`] items.
///
/// This is a convenience function that converts the simpler `MenuNode` API
/// into a menu bar widget, making it easier to create consistent menus
/// using the same API as `context_menu`.
///
/// # Example
///
/// ```ignore
/// use icy_ui_core::menu;
/// use icy_ui::widget::menu::menu_bar;
///
/// let nodes = vec![
///     menu::submenu!("File", [
///         menu::item!("New", Message::New),
///         menu::item!("Open", Message::Open),
///         menu::separator!(),
///         menu::quit!(Message::Quit),
///     ]),
///     menu::submenu!("Edit", [
///         menu::item!("Undo", Message::Undo),
///         menu::item!("Redo", Message::Redo),
///     ]),
/// ];
///
/// menu_bar(&nodes)
/// ```
pub fn menu_bar<Message>(
    nodes: &[app_menu::MenuNode<Message>],
) -> MenuBar<'static, Message, crate::Theme, crate::Renderer>
where
    Message: Clone + 'static,
{
    menu_bar_from(&app_menu::AppMenu::new(nodes.to_vec()))
}
