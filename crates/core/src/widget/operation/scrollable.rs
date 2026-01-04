//! Operate on widgets that can be scrolled.
use crate::widget::{Id, Operation};
use crate::{Rectangle, Vector};

/// The internal state of a widget that can be scrolled.
pub trait Scrollable {
    /// Snaps the scroll of the widget to the given `percentage` along the horizontal & vertical axis.
    fn snap_to(&mut self, offset: RelativeOffset<Option<f32>>);

    /// Scroll the widget to the given [`AbsoluteOffset`] along the horizontal & vertical axis.
    fn scroll_to(&mut self, offset: AbsoluteOffset<Option<f32>>);

    /// Scroll the widget by the given [`AbsoluteOffset`] along the horizontal & vertical axis.
    fn scroll_by(&mut self, offset: AbsoluteOffset, bounds: Rectangle, content_bounds: Rectangle);

    /// Scroll the widget to the given [`AbsoluteOffset`] with smooth animation.
    /// Default implementation falls back to immediate scroll_to.
    fn scroll_to_animated(
        &mut self,
        offset: AbsoluteOffset<Option<f32>>,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        // Default: fall back to immediate scroll
        self.scroll_to(offset);
        let _ = bounds;
        let _ = content_bounds;
    }

    /// Scroll the widget by the given [`AbsoluteOffset`] with smooth animation.
    /// Default implementation falls back to immediate scroll_by.
    fn scroll_by_animated(
        &mut self,
        offset: AbsoluteOffset,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        // Default: fall back to immediate scroll
        self.scroll_by(offset, bounds, content_bounds);
    }
}

/// Produces an [`Operation`] that snaps the widget with the given [`Id`] to
/// the provided `percentage`.
pub fn snap_to<T>(target: Id, offset: RelativeOffset<Option<f32>>) -> impl Operation<T> {
    struct SnapTo {
        target: Id,
        offset: RelativeOffset<Option<f32>>,
    }

    impl<T> Operation<T> for SnapTo {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            _bounds: Rectangle,
            _content_bounds: Rectangle,
            _translation: Vector,
            state: &mut dyn Scrollable,
        ) {
            if Some(&self.target) == id {
                state.snap_to(self.offset);
            }
        }
    }

    SnapTo { target, offset }
}

/// Produces an [`Operation`] that scrolls the widget with the given [`Id`] to
/// the provided [`AbsoluteOffset`].
pub fn scroll_to<T>(target: Id, offset: AbsoluteOffset<Option<f32>>) -> impl Operation<T> {
    struct ScrollTo {
        target: Id,
        offset: AbsoluteOffset<Option<f32>>,
    }

    impl<T> Operation<T> for ScrollTo {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            _bounds: Rectangle,
            _content_bounds: Rectangle,
            _translation: Vector,
            state: &mut dyn Scrollable,
        ) {
            if Some(&self.target) == id {
                state.scroll_to(self.offset);
            }
        }
    }

    ScrollTo { target, offset }
}

/// Produces an [`Operation`] that scrolls the widget with the given [`Id`] by
/// the provided [`AbsoluteOffset`].
pub fn scroll_by<T>(target: Id, offset: AbsoluteOffset) -> impl Operation<T> {
    struct ScrollBy {
        target: Id,
        offset: AbsoluteOffset,
    }

    impl<T> Operation<T> for ScrollBy {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            content_bounds: Rectangle,
            _translation: Vector,
            state: &mut dyn Scrollable,
        ) {
            if Some(&self.target) == id {
                state.scroll_by(self.offset, bounds, content_bounds);
            }
        }
    }

    ScrollBy { target, offset }
}

/// Produces an [`Operation`] that scrolls the widget with the given [`Id`] to
/// the provided [`AbsoluteOffset`] with smooth animation.
pub fn scroll_to_animated<T>(target: Id, offset: AbsoluteOffset<Option<f32>>) -> impl Operation<T> {
    struct ScrollToAnimated {
        target: Id,
        offset: AbsoluteOffset<Option<f32>>,
    }

    impl<T> Operation<T> for ScrollToAnimated {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            content_bounds: Rectangle,
            _translation: Vector,
            state: &mut dyn Scrollable,
        ) {
            if Some(&self.target) == id {
                state.scroll_to_animated(self.offset, bounds, content_bounds);
            }
        }
    }

    ScrollToAnimated { target, offset }
}

/// Produces an [`Operation`] that scrolls the widget with the given [`Id`] by
/// the provided [`AbsoluteOffset`] with smooth animation.
pub fn scroll_by_animated<T>(target: Id, offset: AbsoluteOffset) -> impl Operation<T> {
    struct ScrollByAnimated {
        target: Id,
        offset: AbsoluteOffset,
    }

    impl<T> Operation<T> for ScrollByAnimated {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            content_bounds: Rectangle,
            _translation: Vector,
            state: &mut dyn Scrollable,
        ) {
            if Some(&self.target) == id {
                state.scroll_by_animated(self.offset, bounds, content_bounds);
            }
        }
    }

    ScrollByAnimated { target, offset }
}

/// Produces an [`Operation`] that scrolls the widget with the given [`Id`] the
/// minimum amount needed to make the given [`Rectangle`] visible.
///
/// The `target_rect` should be in content coordinates (relative to the scrollable's content).
/// If the rectangle is already fully visible, no scrolling occurs.
pub fn ensure_visible<T>(target: Id, target_rect: Rectangle) -> impl Operation<T> {
    struct EnsureVisible {
        target: Id,
        target_rect: Rectangle,
    }

    impl<T> Operation<T> for EnsureVisible {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            content_bounds: Rectangle,
            translation: Vector,
            state: &mut dyn Scrollable,
        ) {
            if Some(&self.target) == id {
                let delta = compute_visible_delta(bounds, translation, self.target_rect);

                if delta.x != 0.0 || delta.y != 0.0 {
                    state.scroll_by(delta, bounds, content_bounds);
                }
            }
        }
    }

    EnsureVisible { target, target_rect }
}

/// Produces an [`Operation`] that scrolls the widget with the given [`Id`] the
/// minimum amount needed to make the given [`Rectangle`] visible, with smooth animation.
///
/// The `target_rect` should be in content coordinates (relative to the scrollable's content).
/// If the rectangle is already fully visible, no scrolling occurs.
pub fn ensure_visible_animated<T>(target: Id, target_rect: Rectangle) -> impl Operation<T> {
    struct EnsureVisibleAnimated {
        target: Id,
        target_rect: Rectangle,
    }

    impl<T> Operation<T> for EnsureVisibleAnimated {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            content_bounds: Rectangle,
            translation: Vector,
            state: &mut dyn Scrollable,
        ) {
            if Some(&self.target) == id {
                let delta = compute_visible_delta(bounds, translation, self.target_rect);

                if delta.x != 0.0 || delta.y != 0.0 {
                    state.scroll_by_animated(delta, bounds, content_bounds);
                }
            }
        }
    }

    EnsureVisibleAnimated { target, target_rect }
}

/// Computes the scroll delta needed to make `target_rect` visible within the viewport.
fn compute_visible_delta(
    bounds: Rectangle,
    translation: Vector,
    target_rect: Rectangle,
) -> AbsoluteOffset {
    // Current visible area in content coordinates
    // translation is the scroll offset (positive = scrolled down/right)
    let visible_x = translation.x;
    let visible_y = translation.y;
    let visible_width = bounds.width;
    let visible_height = bounds.height;

    let mut delta = AbsoluteOffset::default();

    // Vertical: check if target is above or below visible area
    if target_rect.y < visible_y {
        // Target is above visible area - scroll up
        delta.y = target_rect.y - visible_y;
    } else if target_rect.y + target_rect.height > visible_y + visible_height {
        // Target is below visible area - scroll down
        delta.y = (target_rect.y + target_rect.height) - (visible_y + visible_height);
    }

    // Horizontal: check if target is left or right of visible area
    if target_rect.x < visible_x {
        // Target is left of visible area - scroll left
        delta.x = target_rect.x - visible_x;
    } else if target_rect.x + target_rect.width > visible_x + visible_width {
        // Target is right of visible area - scroll right
        delta.x = (target_rect.x + target_rect.width) - (visible_x + visible_width);
    }

    delta
}

/// The amount of absolute offset in each direction of a [`Scrollable`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AbsoluteOffset<T = f32> {
    /// The amount of horizontal offset
    pub x: T,
    /// The amount of vertical offset
    pub y: T,
}

impl From<AbsoluteOffset> for AbsoluteOffset<Option<f32>> {
    fn from(offset: AbsoluteOffset) -> Self {
        Self {
            x: Some(offset.x),
            y: Some(offset.y),
        }
    }
}

/// The amount of relative offset in each direction of a [`Scrollable`].
///
/// A value of `0.0` means start, while `1.0` means end.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RelativeOffset<T = f32> {
    /// The amount of horizontal offset
    pub x: T,
    /// The amount of vertical offset
    pub y: T,
}

impl RelativeOffset {
    /// A relative offset that points to the top-left of a [`Scrollable`].
    pub const START: Self = Self { x: 0.0, y: 0.0 };

    /// A relative offset that points to the bottom-right of a [`Scrollable`].
    pub const END: Self = Self { x: 1.0, y: 1.0 };
}

impl From<RelativeOffset> for RelativeOffset<Option<f32>> {
    fn from(offset: RelativeOffset) -> Self {
        Self {
            x: Some(offset.x),
            y: Some(offset.y),
        }
    }
}
