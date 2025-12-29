// From iced_aw, license MIT
// Ported from libcosmic

//! A widget that handles menu trees

use std::cell::RefCell;
use std::rc::Rc;

use super::{
    menu_inner::{
        CloseCondition, Direction, ItemHeight, ItemWidth, Menu, MenuState, PathHighlight,
    },
    menu_tree::MenuTree,
    mnemonic::{MnemonicDisplay, mnemonics_enabled},
    style::StyleSheet,
};

use crate::core::{Border, Shadow};
use crate::core::{
    Clipboard, Element, Layout, Length, Padding, Point, Rectangle, Shell, Size, Vector, Widget,
    event, keyboard,
    layout::{Limits, Node},
    mouse::{self, Cursor},
    overlay, renderer, touch,
    widget::{Tree, tree},
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

#[derive(Clone, Default)]
pub(super) struct MenuBarState {
    pub(super) inner: RcWrapper<MenuBarStateInner>,
}

/// Reference-counted wrapper for menu bar state
pub(super) struct RcWrapper<T> {
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
}

impl MenuBarStateInner {
    /// get the list of indices hovered for the menu
    pub(super) fn get_trimmed_indices(&self, index: usize) -> impl Iterator<Item = usize> + '_ {
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
        }
    }
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

pub(super) fn menu_roots_diff<'a, Message, Theme, Renderer>(
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
    width: Length,
    height: Length,
    spacing: f32,
    padding: Padding,
    bounds_expand: u16,
    main_offset: i32,
    cross_offset: i32,
    close_condition: CloseCondition,
    item_width: ItemWidth,
    item_height: ItemHeight,
    path_highlight: Option<PathHighlight>,
    menu_roots: Vec<MenuTree<'a, Message, Theme, Renderer>>,
    style: Theme::Style,
    mnemonic_display: MnemonicDisplay,
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
            item_width: ItemWidth::Uniform(180),
            item_height: ItemHeight::Dynamic(36),
            path_highlight: Some(PathHighlight::MenuActive),
            menu_roots,
            style: Theme::Style::default(),
            mnemonic_display: MnemonicDisplay::default(),
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
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for MenuBar<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer,
    Theme: StyleSheet,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<MenuBarState>();
        state
            .inner
            .with_data_mut(|inner| menu_roots_diff(&self.menu_roots, &mut inner.tree));
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<MenuBarState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(MenuBarState::default())
    }

    fn children(&self) -> Vec<Tree> {
        menu_roots_children(&self.menu_roots)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let limits = limits.width(self.width).height(self.height);

        // Layout each menu root item directly
        let padding = self.padding;
        let spacing = self.spacing;

        let max_size = limits.max();
        let available_width = max_size.width - padding.x();
        let available_height = max_size.height - padding.y();

        let mut children_nodes = Vec::with_capacity(self.menu_roots.len());
        let mut x = padding.left;
        let mut max_height: f32 = 0.0;

        for (i, root) in self.menu_roots.iter_mut().enumerate() {
            if let Some(child_tree) = tree.children.get_mut(i) {
                let child_limits =
                    Limits::new(Size::ZERO, Size::new(available_width - x, available_height));

                let mut node = root.item.as_widget_mut().layout(
                    &mut child_tree.children[root.index],
                    renderer,
                    &child_limits,
                );

                let node_size = node.size();
                max_height = max_height.max(node_size.height);
                node = node.move_to(Point::new(x, padding.top));

                x += node_size.width + spacing;
                children_nodes.push(node);
            }
        }

        // Align children vertically in center
        for node in &mut children_nodes {
            let node_height = node.size().height;
            let y_offset = (max_height - node_height) / 2.0;
            *node = node
                .clone()
                .move_to(Point::new(node.bounds().x, padding.top + y_offset));
        }

        let total_width = x - spacing + padding.right;
        let total_height = max_height + padding.y();

        Node::with_children(
            limits.resolve(
                self.width,
                self.height,
                Size::new(total_width, total_height),
            ),
            children_nodes,
        )
    }

    #[allow(clippy::too_many_lines)]
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &event::Event,
        layout: Layout<'_>,
        view_cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        use event::Event::{Keyboard, Mouse, Touch};
        use keyboard::key::Named;
        use mouse::Event::ButtonReleased;
        use touch::Event::{FingerLifted, FingerLost};

        process_root_events(
            &mut self.menu_roots,
            view_cursor,
            tree,
            event,
            layout,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let my_state = tree.state.downcast_mut::<MenuBarState>();

        match event {
            // Alt key pressed - update mnemonic display state
            Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::Alt),
                ..
            }) if mnemonics_enabled() => {
                my_state.inner.with_data_mut(|state| {
                    state.alt_pressed = true;
                    if self.mnemonic_display == MnemonicDisplay::OnAlt {
                        state.show_mnemonics = true;
                    }
                });
                shell.request_redraw();
            }

            // Alt key released - update mnemonic display state
            Keyboard(keyboard::Event::KeyReleased {
                key: keyboard::Key::Named(Named::Alt),
                ..
            }) if mnemonics_enabled() => {
                my_state.inner.with_data_mut(|state| {
                    state.alt_pressed = false;
                    // Keep mnemonics visible if menu is open
                    if !state.open && self.mnemonic_display == MnemonicDisplay::OnAlt {
                        state.show_mnemonics = false;
                    }
                });
                shell.request_redraw();
            }

            // Alt+letter for mnemonic activation (opens root menu)
            Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(c),
                modifiers,
                ..
            }) if mnemonics_enabled()
                && modifiers.alt()
                && !modifiers.control()
                && !modifiers.logo() =>
            {
                let char_lower = c.chars().next().map(|ch| ch.to_ascii_lowercase());

                if let Some(ch) = char_lower {
                    // Find menu root with matching mnemonic
                    for (idx, root) in self.menu_roots.iter().enumerate() {
                        if root.mnemonic == Some(ch) {
                            // Open this menu
                            my_state.inner.with_data_mut(|state| {
                                state.active_root = vec![idx];
                                state.open = true;
                                state.view_cursor = view_cursor;
                                // Show mnemonics while menu is open
                                if self.mnemonic_display == MnemonicDisplay::OnAlt {
                                    state.show_mnemonics = true;
                                }
                            });
                            shell.request_redraw();
                            shell.capture_event();
                            break;
                        }
                    }
                }
            }

            // Escape key closes the menu
            Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::Escape),
                ..
            }) => {
                let was_open = my_state.inner.with_data(|state| state.open);
                if was_open {
                    my_state.inner.with_data_mut(|state| {
                        state.menu_states.clear();
                        state.active_root.clear();
                        state.open = false;
                    });
                    shell.request_redraw();
                    shell.capture_event();
                }
            }

            // Left/Right arrow key navigation is handled by the overlay (menu_inner.rs)
            Mouse(ButtonReleased {
                button: mouse::Button::Left,
                ..
            })
            | Touch(FingerLifted { .. } | FingerLost { .. }) => {
                my_state.inner.with_data_mut(|state| {
                    if state.menu_states.is_empty() && view_cursor.is_over(layout.bounds()) {
                        state.view_cursor = view_cursor;
                        state.open = true;
                    } else {
                        state.menu_states.clear();
                        state.active_root.clear();
                        state.open = false;
                        state.view_cursor = view_cursor;
                    }
                });
                // Request redraw to update the visual highlight
                shell.request_redraw();
            }
            _ => (),
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        view_cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<MenuBarState>();
        let cursor_pos = view_cursor.position().unwrap_or_default();

        state.inner.with_data_mut(|state| {
            let position = if state.open && (cursor_pos.x < 0.0 || cursor_pos.y < 0.0) {
                state.view_cursor
            } else {
                view_cursor
            };

            // draw path highlight
            if self.path_highlight.is_some() {
                let styling = theme.appearance(&self.style);

                // Determine which item to highlight: either active (menu open) or hovered
                let highlight_index = if let Some(active) = state.active_root.first() {
                    Some(*active)
                } else {
                    // Check if cursor is hovering over any menu item
                    layout.children().enumerate().find_map(|(i, child_layout)| {
                        if view_cursor.is_over(child_layout.bounds()) {
                            Some(i)
                        } else {
                            None
                        }
                    })
                };

                if let Some(index) = highlight_index {
                    if let Some(item_layout) = layout.children().nth(index) {
                        let item_bounds = item_layout.bounds();

                        // Apply vertical padding to shrink the highlight
                        let [p_top, p_right, p_bottom, p_left] = styling.menu_content_padding;
                        let highlight_bounds = Rectangle {
                            x: item_bounds.x + p_left,
                            y: item_bounds.y + p_top,
                            width: item_bounds.width - p_left - p_right,
                            height: item_bounds.height - p_top - p_bottom,
                        };

                        println!(
                            "Highlighting menu item at index p_top={} p_bottom={}",
                            p_top, p_bottom
                        );

                        let [tl, tr, br, bl] = styling.path_border_radius;
                        let path_quad = renderer::Quad {
                            bounds: highlight_bounds,
                            border: Border {
                                radius: crate::core::border::Radius {
                                    top_left: tl,
                                    top_right: tr,
                                    bottom_right: br,
                                    bottom_left: bl,
                                },
                                ..Default::default()
                            },
                            shadow: Shadow::default(),
                            snap: true,
                        };

                        renderer.fill_quad(path_quad, styling.bar_path);
                    }
                }
            }

            self.menu_roots
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
                .for_each(|((root, t), lo)| {
                    root.item.as_widget().draw(
                        &t.children[root.index],
                        renderer,
                        theme,
                        style,
                        lo,
                        position,
                        viewport,
                    );
                });
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_ref::<MenuBarState>();
        if state.inner.with_data(|state| !state.open) {
            return None;
        }

        Some(
            Menu {
                tree: state.clone(),
                menu_roots: &mut self.menu_roots,
                bounds_expand: self.bounds_expand,
                menu_overlays_parent: false,
                close_condition: self.close_condition,
                item_width: self.item_width,
                item_height: self.item_height,
                bar_bounds: layout.bounds(),
                main_offset: self.main_offset,
                cross_offset: self.cross_offset,
                root_bounds_list: layout.children().map(|lo| lo.bounds()).collect(),
                path_highlight: self.path_highlight,
                style: self.style.clone(),
                position: Point::new(translation.x, translation.y),
                is_overlay: true,
            }
            .overlay(),
        )
    }
}

impl<'a, Message, Theme, Renderer> From<MenuBar<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: renderer::Renderer + 'a,
    Theme: StyleSheet + 'a,
{
    fn from(value: MenuBar<'a, Message, Theme, Renderer>) -> Self {
        Self::new(value)
    }
}

#[allow(clippy::too_many_arguments)]
fn process_root_events<'a, Message, Theme, Renderer>(
    menu_roots: &mut [MenuTree<'a, Message, Theme, Renderer>],
    view_cursor: Cursor,
    tree: &mut Tree,
    event: &event::Event,
    layout: Layout<'_>,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    viewport: &Rectangle,
) where
    Message: Clone,
    Renderer: renderer::Renderer,
{
    for ((root, t), lo) in menu_roots
        .iter_mut()
        .zip(&mut tree.children)
        .zip(layout.children())
    {
        root.item.as_widget_mut().update(
            &mut t.children[root.index],
            event,
            lo,
            view_cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
}
