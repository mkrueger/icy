// From iced_aw, license MIT
// Ported from libcosmic

//! Menu bounds and adaptive open direction types

use super::types::Direction;
use crate::core::widget::Tree;
use crate::core::{Point, Rectangle, Size, Vector, renderer};
use crate::menu::menu_tree::MenuTree;

/// Adaptive open direction
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub(super) struct Aod {
    pub(super) horizontal: bool,
    pub(super) vertical: bool,
    pub(super) horizontal_overlap: bool,
    pub(super) vertical_overlap: bool,
    pub(super) horizontal_direction: Direction,
    pub(super) vertical_direction: Direction,
    pub(super) horizontal_offset: f32,
    pub(super) vertical_offset: f32,
}

impl Aod {
    /// Returns child position and offset position
    #[allow(clippy::too_many_arguments)]
    fn adaptive(
        parent_pos: f32,
        parent_size: f32,
        child_size: f32,
        max_size: f32,
        offset: f32,
        on: bool,
        overlap: bool,
        direction: Direction,
    ) -> (f32, f32) {
        match direction {
            Direction::Positive => {
                let space_negative = parent_pos;
                let space_positive = max_size - parent_pos - parent_size;

                if overlap {
                    let overshoot = child_size - parent_size;
                    if on && space_negative > space_positive && overshoot > space_positive {
                        (parent_pos - overshoot, parent_pos - overshoot)
                    } else {
                        (parent_pos, parent_pos)
                    }
                } else {
                    let overshoot = child_size + offset;
                    if on && space_negative > space_positive && overshoot > space_positive {
                        (parent_pos - overshoot, parent_pos - offset)
                    } else {
                        (parent_pos + parent_size + offset, parent_pos + parent_size)
                    }
                }
            }
            Direction::Negative => {
                let space_positive = parent_pos;
                let space_negative = max_size - parent_pos - parent_size;

                if overlap {
                    let overshoot = child_size - parent_size;
                    if on && space_negative > space_positive && overshoot > space_positive {
                        (parent_pos, parent_pos)
                    } else {
                        (parent_pos - overshoot, parent_pos - overshoot)
                    }
                } else {
                    let overshoot = child_size + offset;
                    if on && space_negative > space_positive && overshoot > space_positive {
                        (parent_pos + parent_size + offset, parent_pos + parent_size)
                    } else {
                        (parent_pos - overshoot, parent_pos - offset)
                    }
                }
            }
        }
    }

    pub(super) fn resolve(
        &self,
        parent_bounds: Rectangle,
        children_size: Size,
        viewport_size: Size,
    ) -> (Point, Point) {
        let (x, ox) = Self::adaptive(
            parent_bounds.x,
            parent_bounds.width,
            children_size.width,
            viewport_size.width,
            self.horizontal_offset,
            self.horizontal,
            self.horizontal_overlap,
            self.horizontal_direction,
        );
        let (y, oy) = Self::adaptive(
            parent_bounds.y,
            parent_bounds.height,
            children_size.height,
            viewport_size.height,
            self.vertical_offset,
            self.vertical,
            self.vertical_overlap,
            self.vertical_direction,
        );

        ([x, y].into(), [ox, oy].into())
    }
}

/// A part of a menu where items are displayed.
#[derive(Debug, Clone, Copy)]
pub(super) struct MenuSlice {
    pub(super) start_index: usize,
    pub(super) end_index: usize,
    pub(super) lower_bound_rel: f32,
    pub(super) upper_bound_rel: f32,
}

/// Menu bounds in overlay space
#[derive(Debug, Clone)]
pub(in crate::menu) struct MenuBounds {
    pub(super) child_positions: Vec<f32>,
    pub(super) child_sizes: Vec<Size>,
    pub(super) children_bounds: Rectangle,
    pub(in crate::menu) parent_bounds: Rectangle,
    pub(super) check_bounds: Rectangle,
    pub(super) offset_bounds: Rectangle,
}

impl MenuBounds {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new<'a, Message, Theme, Renderer>(
        menu_tree: &mut MenuTree<'a, Message, Theme, Renderer>,
        renderer: &Renderer,
        item_width: super::types::ItemWidth,
        item_height: super::types::ItemHeight,
        viewport_size: Size,
        overlay_offset: Vector,
        aod: &Aod,
        bounds_expand: u16,
        parent_bounds: Rectangle,
        tree: &mut [Tree],
        is_overlay: bool,
    ) -> Self
    where
        Message: Clone,
        Renderer: renderer::Renderer,
    {
        let (children_size, child_positions, child_sizes) =
            super::helpers::get_children_layout(menu_tree, renderer, item_width, item_height, tree);

        // viewport space parent bounds
        let view_parent_bounds = parent_bounds + overlay_offset;

        // overlay space children position
        let (children_position, offset_position) = {
            let (cp, op) = aod.resolve(view_parent_bounds, children_size, viewport_size);
            if is_overlay {
                (cp - overlay_offset, op - overlay_offset)
            } else {
                (Point::ORIGIN, op - overlay_offset)
            }
        };

        // calc offset bounds
        let delta = children_position - offset_position;
        let offset_size = if delta.x.abs() > delta.y.abs() {
            Size::new(delta.x, children_size.height)
        } else {
            Size::new(children_size.width, delta.y)
        };
        let offset_bounds = Rectangle::new(offset_position, offset_size);

        let children_bounds = Rectangle::new(children_position, children_size);
        let check_bounds = super::helpers::pad_rectangle(children_bounds, bounds_expand.into());

        Self {
            child_positions,
            child_sizes,
            children_bounds,
            parent_bounds,
            check_bounds,
            offset_bounds,
        }
    }
}
