// From iced_aw, license MIT
// Ported from libcosmic

//! Menu state for tracking active menu items and scroll position

use super::bounds::{MenuBounds, MenuSlice};
use super::helpers::search_bound;
use super::types::ItemHeight;
use crate::core::layout::{Limits, Node};
use crate::core::widget::Tree;
use crate::core::{Point, Size, Vector, renderer};
use crate::menu::menu_tree::MenuTree;

/// Menu state for tracking active menu items
#[derive(Clone)]
pub(in crate::menu) struct MenuState {
    /// The index of the active menu item
    pub(in crate::menu) index: Option<usize>,
    pub(super) scroll_offset: f32,
    pub(in crate::menu) menu_bounds: MenuBounds,
}

impl MenuState {
    pub(super) fn layout<'a, Message, Theme, Renderer>(
        &self,
        overlay_offset: Vector,
        slice: MenuSlice,
        renderer: &Renderer,
        menu_tree: &mut [MenuTree<'a, Message, Theme, Renderer>],
        tree: &mut [Tree],
    ) -> Node
    where
        Message: Clone,
        Renderer: renderer::Renderer,
    {
        let MenuSlice {
            start_index,
            end_index,
            lower_bound_rel,
            upper_bound_rel,
        } = slice;

        debug_assert_eq!(menu_tree.len(), self.menu_bounds.child_positions.len());

        // viewport space children bounds
        let children_bounds = self.menu_bounds.children_bounds + overlay_offset;
        let positions = self
            .menu_bounds
            .child_positions
            .get(start_index..=end_index)
            .unwrap_or(&[]);
        let sizes = self
            .menu_bounds
            .child_sizes
            .get(start_index..=end_index)
            .unwrap_or(&[]);
        let trees = menu_tree
            .get_mut(start_index..=end_index)
            .unwrap_or(&mut []);
        let child_nodes = positions
            .iter()
            .zip(sizes.iter())
            .zip(trees.iter_mut())
            .map(|((cp, size), mt)| {
                let mut position = *cp;
                let mut size = *size;

                if position < lower_bound_rel && (position + size.height) > lower_bound_rel {
                    size.height = position + size.height - lower_bound_rel;
                    position = lower_bound_rel;
                } else if position <= upper_bound_rel && (position + size.height) > upper_bound_rel
                {
                    size.height = upper_bound_rel - position;
                }

                let limits = Limits::new(size, size);

                mt.item
                    .as_widget_mut()
                    .layout(&mut tree[mt.index], renderer, &limits)
                    .move_to(Point::new(0.0, position + self.scroll_offset))
            })
            .collect::<Vec<_>>();

        Node::with_children(children_bounds.size(), child_nodes).move_to(children_bounds.position())
    }

    pub(super) fn layout_single<'a, Message, Theme, Renderer>(
        &self,
        overlay_offset: Vector,
        index: usize,
        renderer: &Renderer,
        menu_tree: &mut MenuTree<'a, Message, Theme, Renderer>,
        tree: &mut Tree,
    ) -> Node
    where
        Message: Clone,
        Renderer: renderer::Renderer,
    {
        // viewport space children bounds
        let children_bounds = self.menu_bounds.children_bounds + overlay_offset;

        let position = self
            .menu_bounds
            .child_positions
            .get(index)
            .copied()
            .unwrap_or(0.0);
        let child_size = self
            .menu_bounds
            .child_sizes
            .get(index)
            .copied()
            .unwrap_or(Size::ZERO);
        let limits = Limits::new(Size::ZERO, child_size);
        let parent_offset = children_bounds.position() - Point::ORIGIN;
        let node = menu_tree
            .item
            .as_widget_mut()
            .layout(tree, renderer, &limits);
        node.move_to(Point::new(
            parent_offset.x,
            parent_offset.y + position + self.scroll_offset,
        ))
    }

    /// returns a slice of the menu items that are inside the viewport
    pub(super) fn slice(
        &self,
        viewport_size: Size,
        overlay_offset: Vector,
        item_height: ItemHeight,
    ) -> MenuSlice {
        // viewport space children bounds
        let children_bounds = self.menu_bounds.children_bounds + overlay_offset;

        let max_index = self.menu_bounds.child_positions.len().saturating_sub(1);

        // viewport space absolute bounds
        let lower_bound = children_bounds.y.max(0.0);
        let upper_bound = (children_bounds.y + children_bounds.height).min(viewport_size.height);

        // menu space relative bounds
        let lower_bound_rel = lower_bound - (children_bounds.y + self.scroll_offset);
        let upper_bound_rel = upper_bound - (children_bounds.y + self.scroll_offset);

        // index range
        let (start_index, end_index) = match item_height {
            ItemHeight::Uniform(u) => {
                let start_index = (lower_bound_rel / f32::from(u)).floor() as usize;
                let end_index = ((upper_bound_rel / f32::from(u)).floor() as usize).min(max_index);
                (start_index, end_index)
            }
            ItemHeight::Static(_) | ItemHeight::Dynamic(_) => {
                let positions = &self.menu_bounds.child_positions;
                let sizes = &self.menu_bounds.child_sizes;

                let start_index = search_bound(0, 0, max_index, lower_bound_rel, positions, sizes);
                let end_index = search_bound(
                    max_index,
                    start_index,
                    max_index,
                    upper_bound_rel,
                    positions,
                    sizes,
                )
                .min(max_index);

                (start_index, end_index)
            }
        };

        MenuSlice {
            start_index,
            end_index,
            lower_bound_rel,
            upper_bound_rel,
        }
    }
}
