// From iced_aw, license MIT
// Ported from libcosmic

//! Helper functions for menu layout and event processing

use super::bounds::{Aod, MenuBounds};
use super::menu::Menu;
use super::state::MenuState;
use super::types::{Direction, ItemHeight, ItemWidth};
use crate::core::Clipboard;
use crate::core::layout::Layout;
use crate::core::mouse::Cursor;
use crate::core::widget::Tree;
use crate::core::{Length, Padding, Point, Rectangle, Shell, Size, Vector, event, mouse, renderer};
use crate::menu::menu_tree::MenuTree;
use crate::menu::style::StyleSheet;

pub(super) fn pad_rectangle(rect: Rectangle, padding: Padding) -> Rectangle {
    Rectangle {
        x: rect.x - padding.left,
        y: rect.y - padding.top,
        width: rect.width + padding.x(),
        height: rect.height + padding.y(),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn init_root_menu<'a, 'b, Message, Theme, Renderer>(
    menu: &mut Menu<'a, 'b, Message, Theme, Renderer>,
    renderer: &Renderer,
    shell: &mut Shell<'_, Message>,
    overlay_cursor: Point,
    viewport_size: Size,
    overlay_offset: Vector,
    bar_bounds: Rectangle,
    main_offset: f32,
) where
    Message: Clone,
    Renderer: renderer::Renderer,
    Theme: StyleSheet,
{
    menu.tree.inner.with_data_mut(|state| {
        // DEBUG
        eprintln!("[init_root_menu] open={}, menu_states.len={}, active_root={:?}, tree.children.len={}", 
            state.open, state.menu_states.len(), state.active_root, state.tree.children.len());
        
        if !state.open {
            return;
        }

        // Case 1: menu_states is empty but active_root is set (mnemonic activation)
        // We need to initialize menu_states for the active root
        if state.menu_states.is_empty() && !state.active_root.is_empty() {
            let Some(&i) = state.active_root.first() else {
                return;
            };
            if let Some((&root_bounds, mt)) =
                menu.root_bounds_list.get(i).zip(menu.menu_roots.get_mut(i))
            {
                if !mt.children.is_empty() {
                    let Some(tree_entry) = state.tree.children.get_mut(i) else {
                        return;
                    };
                    let view_center = viewport_size.width * 0.5;
                    let rb_center = root_bounds.center_x();

                    state.horizontal_direction = if menu.is_overlay && rb_center > view_center {
                        Direction::Negative
                    } else {
                        Direction::Positive
                    };

                    let aod = Aod {
                        horizontal: true,
                        vertical: true,
                        horizontal_overlap: true,
                        vertical_overlap: false,
                        horizontal_direction: state.horizontal_direction,
                        vertical_direction: state.vertical_direction,
                        horizontal_offset: 0.0,
                        vertical_offset: main_offset,
                    };

                    let menu_bounds = MenuBounds::new(
                        mt,
                        renderer,
                        menu.item_width,
                        menu.item_height,
                        viewport_size,
                        overlay_offset,
                        &aod,
                        menu.bounds_expand,
                        root_bounds,
                        &mut tree_entry.children,
                        menu.is_overlay,
                    );

                    let ms = MenuState {
                        index: Some(0),
                        scroll_offset: 0.0,
                        menu_bounds,
                    };
                    state.menu_states.push(ms);

                    shell.invalidate_layout();
                }
            }
            return;
        }

        // Case 2: Normal cursor-based initialization
        if !(state.menu_states.is_empty()
            && (!menu.is_overlay || bar_bounds.contains(overlay_cursor)))
        {
            return;
        }

        for (i, (&root_bounds, mt)) in menu
            .root_bounds_list
            .iter()
            .zip(menu.menu_roots.iter_mut())
            .enumerate()
        {
            if mt.children.is_empty() {
                continue;
            }

            if root_bounds.contains(overlay_cursor) {
                eprintln!("[init_root_menu] Cursor over root {}, mt.children.len={}", i, mt.children.len());
                let Some(tree_entry) = state.tree.children.get_mut(i) else {
                    eprintln!("[init_root_menu] ERROR: tree.children.get_mut({}) failed, tree has {} children", i, state.tree.children.len());
                    continue;
                };
                eprintln!("[init_root_menu] tree_entry.children.len={}", tree_entry.children.len());
                let view_center = viewport_size.width * 0.5;
                let rb_center = root_bounds.center_x();

                state.horizontal_direction = if menu.is_overlay && rb_center > view_center {
                    Direction::Negative
                } else {
                    Direction::Positive
                };

                let aod = Aod {
                    horizontal: true,
                    vertical: true,
                    horizontal_overlap: true,
                    vertical_overlap: false,
                    horizontal_direction: state.horizontal_direction,
                    vertical_direction: state.vertical_direction,
                    horizontal_offset: 0.0,
                    vertical_offset: main_offset,
                };

                let menu_bounds = MenuBounds::new(
                    mt,
                    renderer,
                    menu.item_width,
                    menu.item_height,
                    viewport_size,
                    overlay_offset,
                    &aod,
                    menu.bounds_expand,
                    root_bounds,
                    &mut tree_entry.children,
                    menu.is_overlay,
                );

                state.active_root.push(i);
                let ms = MenuState {
                    index: None,
                    scroll_offset: 0.0,
                    menu_bounds,
                };
                state.menu_states.push(ms);

                // Hack to ensure menu opens properly
                shell.invalidate_layout();

                break;
            }
        }
    });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn process_menu_events<'a, 'b, Message, Theme, Renderer>(
    menu: &mut Menu<'a, 'b, Message, Theme, Renderer>,
    event: &event::Event,
    view_cursor: Cursor,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    overlay_offset: Vector,
) -> event::Status
where
    Message: Clone,
    Renderer: renderer::Renderer,
    Theme: StyleSheet,
{
    use event::Status;

    let menu_roots = &mut menu.menu_roots;
    let my_state = &mut menu.tree;

    my_state.inner.with_data_mut(|state| {
        if state.active_root.is_empty() {
            return event::Status::Ignored;
        }

        let Some(hover) = state.menu_states.last_mut() else {
            return Status::Ignored;
        };

        let Some(hover_index) = hover.index else {
            return Status::Ignored;
        };

        let Some(first_root) = state.active_root.first().copied() else {
            return Status::Ignored;
        };
        let Some(root_entry) = menu_roots.get_mut(first_root) else {
            return Status::Ignored;
        };
        let mt = state
            .active_root
            .iter()
            .skip(1)
            .fold(root_entry, |mt, &next_active_root| {
                if next_active_root < mt.children.len() {
                    &mut mt.children[next_active_root]
                } else {
                    mt
                }
            });

        let Some(mt) = mt.children.get_mut(hover_index) else {
            return Status::Ignored;
        };
        let Some(tree_entry) = state.tree.children.get_mut(first_root) else {
            return Status::Ignored;
        };
        let Some(tree) = tree_entry.children.get_mut(mt.index) else {
            return Status::Ignored;
        };

        // get layout
        let child_node = hover.layout_single(
            overlay_offset,
            hover.index.expect("missing index within menu state."),
            renderer,
            mt,
            tree,
        );
        let child_layout = Layout::new(&child_node);

        // process only the last widget
        mt.item.as_widget_mut().update(
            tree,
            event,
            child_layout,
            view_cursor,
            renderer,
            clipboard,
            shell,
            &Rectangle::default(),
        );

        if shell.is_event_captured() {
            Status::Captured
        } else {
            Status::Ignored
        }
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn process_overlay_events<'a, 'b, Message, Theme, Renderer>(
    menu: &mut Menu<'a, 'b, Message, Theme, Renderer>,
    renderer: &Renderer,
    layout: Layout<'_>,
    viewport_size: Size,
    overlay_offset: Vector,
    view_cursor: Cursor,
    overlay_cursor: Point,
    cross_offset: f32,
    shell: &mut Shell<'_, Message>,
) -> event::Status
where
    Message: Clone,
    Renderer: renderer::Renderer,
    Theme: StyleSheet,
{
    use event::Status::{Captured, Ignored};

    menu.tree.inner.with_data_mut(|state| {
        state.view_cursor = view_cursor;

        // remove invalid menus
        let mut prev_bounds = std::iter::once(menu.bar_bounds)
            .chain(
                state.menu_states[..state.menu_states.len().saturating_sub(1)]
                    .iter()
                    .map(|s| s.menu_bounds.children_bounds),
            )
            .collect::<Vec<_>>();

        if menu.is_overlay && menu.close_condition.leave {
            for i in (0..state.menu_states.len()).rev() {
                let mb = &state.menu_states[i].menu_bounds;

                if mb.parent_bounds.contains(overlay_cursor)
                    || menu.is_overlay && mb.children_bounds.contains(overlay_cursor)
                    || mb.offset_bounds.contains(overlay_cursor)
                    || (mb.check_bounds.contains(overlay_cursor)
                        && prev_bounds.iter().all(|pvb| !pvb.contains(overlay_cursor)))
                {
                    break;
                }
                let _ = prev_bounds.pop();
                let _ = state.active_root.pop();
                let _ = state.menu_states.pop();
            }
        } else if menu.is_overlay {
            for i in (0..state.menu_states.len()).rev() {
                let mb = &state.menu_states[i].menu_bounds;

                if mb.parent_bounds.contains(overlay_cursor)
                    || mb.children_bounds.contains(overlay_cursor)
                    || prev_bounds.iter().all(|pvb| !pvb.contains(overlay_cursor))
                {
                    break;
                }
                let _ = prev_bounds.pop();
                let _ = state.active_root.pop();
                let _ = state.menu_states.pop();
            }
        }

        // update active item
        let last_menu_state_index = match state.menu_states.len() {
            0 => {
                if menu.is_overlay && !menu.bar_bounds.contains(overlay_cursor) {
                    state.open = false;
                    shell.request_redraw();
                }
                return Captured;
            }
            n => n - 1,
        };

        let Some(last_menu_state) = state.menu_states.get_mut(last_menu_state_index) else {
            return Captured;
        };

        let last_menu_bounds = &last_menu_state.menu_bounds;
        let last_parent_bounds = last_menu_bounds.parent_bounds;
        let last_children_bounds = last_menu_bounds.children_bounds;

        if (menu.is_overlay
            && !menu.menu_overlays_parent
            && last_parent_bounds.contains(overlay_cursor))
            || menu.is_overlay && !last_children_bounds.contains(overlay_cursor)
        {
            last_menu_state.index = None;
            shell.request_redraw();
            return Captured;
        }

        let active_root = &state.active_root;

        if state.pressed {
            return Ignored;
        }

        let Some(first_root) = active_root.first().copied() else {
            last_menu_state.index = None;
            shell.request_redraw();
            return Captured;
        };
        let Some(root_entry) = menu.menu_roots.get_mut(first_root) else {
            last_menu_state.index = None;
            shell.request_redraw();
            return Captured;
        };
        let roots =
            active_root
                .iter()
                .skip(1)
                .fold(&mut root_entry.children, |mt, &next_active_root| {
                    if next_active_root < mt.len() {
                        &mut mt[next_active_root].children
                    } else {
                        mt
                    }
                });
        let Some(tree_entry) = state.tree.children.get_mut(first_root) else {
            last_menu_state.index = None;
            shell.request_redraw();
            return Captured;
        };
        let tree = &mut tree_entry.children;

        let active_menu = roots;
        // Determine hovered item by checking real layout bounds (iced_aw-style).
        // This is robust with dynamic heights and separators.
        let Some(last_menu_layout) = layout.children().nth(last_menu_state_index) else {
            last_menu_state.index = None;
            shell.request_redraw();
            return Captured;
        };

        let slice = last_menu_state.slice(viewport_size, overlay_offset, menu.item_height);
        let start_index = slice.start_index;

        // Use overlay_cursor (in overlay space) to match the layout bounds
        let cursor_point = Point::new(overlay_cursor.x, overlay_cursor.y);

        let mut hovered_index: Option<usize> = None;
        for (i, item_layout) in last_menu_layout.children().enumerate() {
            let bounds = item_layout.bounds();
            if bounds.contains(cursor_point) {
                hovered_index = Some(start_index + i);
                break;
            }
        }

        let Some(new_index) = hovered_index else {
            last_menu_state.index = None;
            shell.request_redraw();
            return Captured;
        };

        if new_index >= active_menu.len()
            || active_menu.get(new_index).map_or(true, |m| m.is_separator)
        {
            last_menu_state.index = None;
            shell.request_redraw();
            return Captured;
        }

        let remove = last_menu_state.index.as_ref().is_some_and(|i| {
            *i != new_index
                && active_menu
                    .get(*i)
                    .map_or(false, |m| !m.children.is_empty())
        });

        let Some(item) = active_menu.get_mut(new_index) else {
            last_menu_state.index = None;
            shell.request_redraw();
            return Captured;
        };
        let old_index = last_menu_state.index.replace(new_index);

        // add new menu if the new item is a menu
        if !item.children.is_empty() && old_index.is_none_or(|i| i != new_index) {
            let Some(&item_pos) = last_menu_bounds.child_positions.get(new_index) else {
                return Captured;
            };
            let Some(&item_size) = last_menu_bounds.child_sizes.get(new_index) else {
                return Captured;
            };
            let item_position = Point::new(0.0, item_pos + last_menu_state.scroll_offset);

            let item_bounds = Rectangle::new(item_position, item_size)
                + (last_menu_bounds.children_bounds.position() - Point::ORIGIN);

            let aod = Aod {
                horizontal: true,
                vertical: true,
                horizontal_overlap: false,
                vertical_overlap: true,
                horizontal_direction: state.horizontal_direction,
                vertical_direction: state.vertical_direction,
                horizontal_offset: cross_offset,
                vertical_offset: 0.0,
            };

            let ms = MenuState {
                index: None,
                scroll_offset: 0.0,
                menu_bounds: MenuBounds::new(
                    item,
                    renderer,
                    menu.item_width,
                    menu.item_height,
                    viewport_size,
                    overlay_offset,
                    &aod,
                    menu.bounds_expand,
                    item_bounds,
                    tree,
                    menu.is_overlay,
                ),
            };

            if menu.is_overlay {
                state.active_root.push(new_index);
            } else {
                state.menu_states.truncate(1);
            }
            state.menu_states.push(ms);
        } else if !menu.is_overlay && remove {
            state.menu_states.truncate(1);
        }

        // Request redraw to update the visual highlight
        shell.request_redraw();

        Captured
    })
}

pub(super) fn process_scroll_events<'a, 'b, Message, Theme, Renderer>(
    menu: &mut Menu<'a, 'b, Message, Theme, Renderer>,
    delta: mouse::ScrollDelta,
    overlay_cursor: Point,
    viewport_size: Size,
    overlay_offset: Vector,
) -> event::Status
where
    Message: Clone,
    Renderer: renderer::Renderer,
    Theme: StyleSheet,
{
    use event::Status::{Captured, Ignored};
    use mouse::ScrollDelta;

    menu.tree.inner.with_data_mut(|state| {
        let delta_y = match delta {
            ScrollDelta::Lines { y, .. } => y * 60.0,
            ScrollDelta::Pixels { y, .. } => y,
        };

        let calc_offset_bounds = |menu_state: &MenuState, viewport_size: Size| -> (f32, f32) {
            let children_bounds = menu_state.menu_bounds.children_bounds + overlay_offset;

            let max_offset = (0.0 - children_bounds.y).max(0.0);
            let min_offset =
                (viewport_size.height - (children_bounds.y + children_bounds.height)).min(0.0);
            (max_offset, min_offset)
        };

        if state.menu_states.is_empty() {
            return Ignored;
        } else if state.menu_states.len() == 1 {
            let Some(last_ms) = state.menu_states.first_mut() else {
                return Ignored;
            };

            if last_ms.index.is_none() {
                return Captured;
            }

            let (max_offset, min_offset) = calc_offset_bounds(last_ms, viewport_size);
            last_ms.scroll_offset = (last_ms.scroll_offset + delta_y).clamp(min_offset, max_offset);
        } else {
            let max_index = state.menu_states.len() - 1;
            let Some(last_two) = state.menu_states.get_mut(max_index - 1..=max_index) else {
                return Ignored;
            };

            if last_two[1].index.is_some() {
                let (max_offset, min_offset) = calc_offset_bounds(&last_two[1], viewport_size);
                last_two[1].scroll_offset =
                    (last_two[1].scroll_offset + delta_y).clamp(min_offset, max_offset);
            } else {
                if !last_two[0]
                    .menu_bounds
                    .children_bounds
                    .contains(overlay_cursor)
                {
                    return Captured;
                }

                let (max_offset, min_offset) = calc_offset_bounds(&last_two[0], viewport_size);
                let scroll_offset =
                    (last_two[0].scroll_offset + delta_y).clamp(min_offset, max_offset);
                let clamped_delta_y = scroll_offset - last_two[0].scroll_offset;
                last_two[0].scroll_offset = scroll_offset;

                last_two[1].menu_bounds.parent_bounds.y += clamped_delta_y;
                last_two[1].menu_bounds.children_bounds.y += clamped_delta_y;
                last_two[1].menu_bounds.check_bounds.y += clamped_delta_y;
            }
        }
        Captured
    })
}

/// Returns (children_size, child_positions, child_sizes)
pub(super) fn get_children_layout<'a, Message, Theme, Renderer>(
    menu_tree: &mut MenuTree<'a, Message, Theme, Renderer>,
    renderer: &Renderer,
    item_width: ItemWidth,
    item_height: ItemHeight,
    tree: &mut [Tree],
) -> (Size, Vec<f32>, Vec<Size>)
where
    Message: Clone,
    Renderer: renderer::Renderer,
{
    use crate::core::layout::Limits;

    let width = match item_width {
        ItemWidth::Uniform(u) => f32::from(u),
        ItemWidth::Static(s) => f32::from(menu_tree.width.unwrap_or(s)),
    };

    let child_sizes: Vec<Size> = match item_height {
        ItemHeight::Uniform(u) => {
            let count = menu_tree.children.len();
            vec![Size::new(width, f32::from(u)); count]
        }
        ItemHeight::Static(s) => menu_tree
            .children
            .iter()
            .map(|mt| Size::new(width, f32::from(mt.height.unwrap_or(s))))
            .collect(),
        ItemHeight::Dynamic(d) => menu_tree
            .children
            .iter_mut()
            .map(|mt| {
                let w = mt.item.as_widget_mut();
                if let Length::Fixed(f) = w.size().height {
                    return Size::new(width, f);
                }

                // Always measure the actual layout height.
                // Relying on `w.size().height` is not sufficient because many widgets report
                // `Fill` while still having a well-defined intrinsic height (e.g. separators).
                let l_height = w
                    .layout(
                        &mut tree[mt.index],
                        renderer,
                        &Limits::new(Size::ZERO, Size::new(width, f32::MAX)),
                    )
                    .size()
                    .height;

                let fallback = mt.height.map_or_else(|| f32::from(d), |h| f32::from(h));
                let height = if !l_height.is_finite() || (f32::MAX - l_height).abs() < 0.001 {
                    fallback
                } else {
                    l_height
                };

                Size::new(width, height)
            })
            .collect(),
    };

    let max_index = menu_tree.children.len().saturating_sub(1);
    let child_positions: Vec<f32> = std::iter::once(0.0)
        .chain(child_sizes[0..max_index].iter().scan(0.0, |acc, x| {
            *acc += x.height;
            Some(*acc)
        }))
        .collect();

    let height = child_sizes.iter().fold(0.0, |acc, x| acc + x.height);

    (Size::new(width, height), child_positions, child_sizes)
}

pub(super) fn search_bound(
    default: usize,
    default_left: usize,
    default_right: usize,
    bound: f32,
    positions: &[f32],
    sizes: &[Size],
) -> usize {
    let mut left = default_left;
    let mut right = default_right;

    let mut index = default;
    while left != right {
        let m = ((left + right) / 2) + 1;
        if positions[m] > bound {
            right = m - 1;
        } else {
            left = m;
        }
    }

    let height = sizes[left].height;
    if positions[left] + height > bound {
        index = left;
    }
    index
}
