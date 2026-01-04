// From iced_aw, license MIT
// Ported from libcosmic

//! Widget implementation for MenuBar

use crate::core::{Border, Shadow};
use crate::core::{
    Clipboard, Element, Layout, Length, Point, Rectangle, Shell, Size, Vector, Widget, event,
    keyboard,
    layout::{Limits, Node},
    mouse::{self, Cursor},
    overlay, renderer, touch,
    widget::{Tree, tree},
};

use super::super::{
    menu_inner::{Direction, Menu},
    menu_tree::MenuTree,
    mnemonic::{MnemonicDisplay, mnemonics_enabled, set_show_underlines},
    style::StyleSheet,
};

use super::state::MenuBarState;
use super::{MenuBar, menu_roots_children, menu_roots_diff};

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

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<MenuBarState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(MenuBarState::default())
    }

    fn children(&self) -> Vec<Tree> {
        menu_roots_children(&self.menu_roots)
    }

    fn diff(&self, tree: &mut Tree) {
        menu_roots_diff(&self.menu_roots, tree);
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        // Ensure inner.tree is initialized (diff() may not have been called)
        let state = tree.state.downcast_mut::<MenuBarState>();
        state.inner.with_data_mut(|inner| {
            if inner.tree.children.len() != self.menu_roots.len() {
                menu_roots_diff(&self.menu_roots, &mut inner.tree);
            }
        });

        let limits = limits.width(self.width).height(self.height);

        // Layout each menu root item directly
        let padding = self.padding;
        let spacing = self.spacing;

        let max_size = limits.max();
        let available_width = max_size.width - padding.x();
        let available_height = max_size.height - padding.y();

        // Use widget override, or global layout direction
        let is_rtl = self
            .layout_direction
            .unwrap_or_else(crate::core::layout_direction)
            .is_rtl();

        let mut children_nodes = Vec::with_capacity(self.menu_roots.len());
        let mut max_height: f32 = 0.0;

        // First pass: layout all children to get their sizes
        let mut node_sizes = Vec::with_capacity(self.menu_roots.len());
        for (i, root) in self.menu_roots.iter_mut().enumerate() {
            if let Some(child_tree) = tree.children.get_mut(i) {
                let child_limits =
                    Limits::new(Size::ZERO, Size::new(available_width, available_height));

                let node = root.item.as_widget_mut().layout(
                    &mut child_tree.children[root.index],
                    renderer,
                    &child_limits,
                );

                let node_size = node.size();
                max_height = max_height.max(node_size.height);
                node_sizes.push((node, node_size));
            }
        }

        // Second pass: position children based on direction
        if is_rtl {
            // RTL: start from right edge
            let mut x = max_size.width - padding.right;
            for (node, node_size) in node_sizes {
                x -= node_size.width;
                let positioned_node = node.move_to(Point::new(x, padding.top));
                x -= spacing;
                children_nodes.push(positioned_node);
            }
        } else {
            // LTR: start from left edge
            let mut x = padding.left;
            for (node, node_size) in node_sizes {
                let positioned_node = node.move_to(Point::new(x, padding.top));
                x += node_size.width + spacing;
                children_nodes.push(positioned_node);
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

        let total_height = max_height + padding.y();

        Node::with_children(
            limits.resolve(
                self.width,
                self.height,
                Size::new(max_size.width, total_height),
            ),
            children_nodes,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn crate::core::widget::Operation,
    ) {
        operation.container(None, layout.bounds());

        // Report MenuBar as a MenuBar role for accessibility
        #[cfg(feature = "accessibility")]
        {
            // Collect root IDs for the menubar children
            let root_ids: Vec<_> = (0..self.menu_roots.len())
                .map(|idx| {
                    let root_id =
                        crate::core::widget::Id::from(format!("icy_ui.menubar/root/{}", idx));
                    crate::core::accessibility::node_id_from_widget_id(&root_id)
                })
                .collect();

            let info = crate::core::accessibility::WidgetInfo::new(
                crate::core::accessibility::Role::MenuBar,
            )
            .with_bounds(layout.bounds())
            .with_children(root_ids);
            operation.accessibility(None, layout.bounds(), info);
        }

        // Operate on each menu root item - emit explicit accessibility nodes
        for (idx, ((root, child_tree), child_layout)) in self
            .menu_roots
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .enumerate()
        {
            #[cfg(feature = "accessibility")]
            {
                // Emit an explicit accessibility node for the menu root
                let root_id = crate::core::widget::Id::from(format!("icy_ui.menubar/root/{}", idx));
                let label = root
                    .item
                    .as_widget()
                    .accessibility_label()
                    .map(|s| s.into_owned())
                    .unwrap_or_default();

                let bounds = child_layout.bounds();
                let has_children = !root.children.is_empty();

                // Use menu_item() to get focusable=true and proper actions
                let mut info =
                    crate::core::accessibility::WidgetInfo::menu_item(label).with_bounds(bounds);

                if has_children {
                    info = info.with_expanded(Some(false));
                }

                operation.accessibility(Some(&root_id), bounds, info);
            }

            // Always run the normal widget operations (keyboard, mouse, etc.)
            operation.traverse(&mut |op| {
                root.item.as_widget_mut().operate(
                    &mut child_tree.children[root.index],
                    child_layout,
                    renderer,
                    op,
                );
            });
        }

        operation.leave_container();
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

        // Handle accessibility events for menu bar roots (VoiceOver navigation)
        #[cfg(feature = "accessibility")]
        if let event::Event::Accessibility(accessibility_event) = event {
            use crate::core::accessibility::node_id_from_widget_id;

            // Check if the event targets a menu bar root
            for idx in 0..self.menu_roots.len() {
                let root_id = crate::core::widget::Id::from(format!("icy_ui.menubar/root/{}", idx));
                let node_id = node_id_from_widget_id(&root_id);

                if node_id == accessibility_event.target {
                    let my_state = tree.state.downcast_mut::<MenuBarState>();

                    if accessibility_event.is_focus() {
                        // VoiceOver cursor focus: open this menu and focus first item
                        my_state.inner.with_data_mut(|state| {
                            state.a11y_focused_root = Some(idx);
                            state.menu_states.clear();
                            state.active_root = vec![idx];
                            state.open = true;
                            state.view_cursor = view_cursor;
                        });

                        // Request focus on the first item in the menu
                        if !self.menu_roots[idx].children.is_empty() {
                            let first_item_id = crate::core::widget::Id::from(format!(
                                "icy_ui.menu/{}/panel/0/item/0",
                                idx
                            ));
                            let first_item_node_id = node_id_from_widget_id(&first_item_id);
                            shell.request_a11y_focus(first_item_node_id);
                        }

                        shell.invalidate_layout();
                        shell.request_redraw();
                        shell.capture_event();
                        return;
                    }

                    if accessibility_event.is_click() {
                        // Open this menu and select first item
                        my_state.inner.with_data_mut(|state| {
                            state.menu_states.clear();
                            state.active_root = vec![idx];
                            state.open = true;
                            state.view_cursor = view_cursor;
                        });

                        // Request focus on the first item in the menu
                        if !self.menu_roots[idx].children.is_empty() {
                            let first_item_id = crate::core::widget::Id::from(format!(
                                "icy_ui.menu/{}/panel/0/item/0",
                                idx
                            ));
                            let first_item_node_id = node_id_from_widget_id(&first_item_id);
                            shell.request_a11y_focus(first_item_node_id);
                        }

                        shell.invalidate_layout();
                        shell.request_redraw();
                        shell.capture_event();
                    }
                    return;
                }
            }
        }

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

        let was_open = my_state.inner.with_data(|state| state.open);

        // Handle Space/Enter when a menu root has accessibility focus (VoiceOver)
        #[cfg(feature = "accessibility")]
        if let Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(named_key),
            ..
        }) = event
        {
            if matches!(named_key, Named::Space | Named::Enter) {
                let maybe_idx = my_state.inner.with_data(|state| state.a11y_focused_root);
                if let Some(idx) = maybe_idx {
                    if idx < self.menu_roots.len() {
                        // Open this menu
                        my_state.inner.with_data_mut(|state| {
                            state.menu_states.clear();
                            state.active_root = vec![idx];
                            state.open = true;
                            state.view_cursor = view_cursor;
                        });

                        // Request focus on the first item in the menu
                        if !self.menu_roots[idx].children.is_empty() {
                            use crate::core::accessibility::node_id_from_widget_id;
                            let first_item_id = crate::core::widget::Id::from(format!(
                                "icy_ui.menu/{}/panel/0/item/0",
                                idx
                            ));
                            let first_item_node_id = node_id_from_widget_id(&first_item_id);
                            shell.request_a11y_focus(first_item_node_id);
                        }

                        shell.invalidate_layout();
                        shell.request_redraw();
                        shell.capture_event();
                        return;
                    }
                }
            }
        }

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
                        set_show_underlines(true);
                    }
                });
                shell.invalidate_layout();
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
                        set_show_underlines(false);
                    }
                });
                shell.invalidate_layout();
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
                                // Clear existing menu states to force reinitialization
                                // with correct menu bounds for the new root
                                state.menu_states.clear();
                                state.active_root = vec![idx];
                                state.open = true;
                                state.view_cursor = view_cursor;
                                // Show mnemonics while menu is open
                                if self.mnemonic_display == MnemonicDisplay::OnAlt {
                                    state.show_mnemonics = true;
                                    set_show_underlines(true);
                                }
                            });
                            shell.invalidate_layout();
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

                        if self.mnemonic_display == MnemonicDisplay::OnAlt && !state.alt_pressed {
                            state.show_mnemonics = false;
                            set_show_underlines(false);
                        }
                    });
                    shell.invalidate_layout();
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
                let opened = my_state.inner.with_data_mut(|state| {
                    if state.menu_states.is_empty() && view_cursor.is_over(layout.bounds()) {
                        state.view_cursor = view_cursor;
                        state.open = true;
                        true
                    } else {
                        state.menu_states.clear();
                        state.active_root.clear();
                        state.open = false;
                        state.view_cursor = view_cursor;

                        if self.mnemonic_display == MnemonicDisplay::OnAlt && !state.alt_pressed {
                            state.show_mnemonics = false;
                            set_show_underlines(false);
                        }
                        false
                    }
                });

                // Prevent the just-opened menu from immediately receiving the same
                // release event in the overlay and closing itself.
                if opened {
                    shell.capture_event();
                }

                // Layout invalidation also triggers a redraw; needed for mnemonic underline updates.
                shell.invalidate_layout();
                shell.request_redraw();
            }
            _ => (),
        }

        // If the menu has just opened, move accessibility focus to the first item.
        //
        // This is a key fallback for VoiceOver-style interaction where we may not
        // receive an AccessKit ActionRequested(Focus) for the menubar root.
        #[cfg(feature = "accessibility")]
        {
            let is_open = my_state.inner.with_data(|state| state.open);
            if !was_open && is_open {
                let active_root = my_state
                    .inner
                    .with_data(|state| state.active_root.first().copied());

                if let Some(idx) = active_root {
                    if idx < self.menu_roots.len() && !self.menu_roots[idx].children.is_empty() {
                        use crate::core::accessibility::node_id_from_widget_id;

                        let first_item_id = crate::core::widget::Id::from(format!(
                            "icy_ui.menu/{}/panel/0/item/0",
                            idx
                        ));
                        let first_item_node_id = node_id_from_widget_id(&first_item_id);

                        shell.request_a11y_focus(first_item_node_id);
                        shell.request_redraw();
                    }
                }
            }
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
        // Update global mnemonic underline state based on display mode
        match self.mnemonic_display {
            MnemonicDisplay::Show => set_show_underlines(true),
            MnemonicDisplay::Hide => set_show_underlines(false),
            MnemonicDisplay::OnAlt => {
                state
                    .inner
                    .with_data(|s| set_show_underlines(s.show_mnemonics));
            }
        }

        state.inner.with_data_mut(|inner_state| {
            // Set horizontal direction: use widget override or fallback to global
            let direction = self
                .layout_direction
                .unwrap_or_else(crate::core::layout_direction);
            inner_state.horizontal_direction = if direction.is_rtl() {
                Direction::Negative
            } else {
                Direction::Positive
            };

            let position = if inner_state.open && (cursor_pos.x < 0.0 || cursor_pos.y < 0.0) {
                inner_state.view_cursor
            } else {
                view_cursor
            };

            // draw path highlight
            if self.path_highlight.is_some() {
                let styling = theme.appearance(&self.style);

                // Determine which item to highlight: either active (menu open) or hovered
                let highlight_index = if let Some(active) = inner_state.active_root.first() {
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
