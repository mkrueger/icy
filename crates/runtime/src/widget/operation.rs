//! Change internal widget state.
use crate::core::widget::Id;
use crate::core::widget::operation;
use crate::core::Rectangle;
use crate::task;
use crate::{Action, Task};

pub use crate::core::widget::operation::focusable::{FocusLevel, FocusTier};
pub use crate::core::widget::operation::scrollable::{AbsoluteOffset, RelativeOffset};

/// Snaps the scrollable with the given [`Id`] to the provided [`RelativeOffset`].
pub fn snap_to<T>(id: impl Into<Id>, offset: impl Into<RelativeOffset<Option<f32>>>) -> Task<T> {
    task::effect(Action::widget(operation::scrollable::snap_to(
        id.into(),
        offset.into(),
    )))
}

/// Snaps the scrollable with the given [`Id`] to the [`RelativeOffset::END`].
pub fn snap_to_end<T>(id: impl Into<Id>) -> Task<T> {
    task::effect(Action::widget(operation::scrollable::snap_to(
        id.into(),
        RelativeOffset::END.into(),
    )))
}

/// Scrolls the scrollable with the given [`Id`] to the provided [`AbsoluteOffset`].
pub fn scroll_to<T>(id: impl Into<Id>, offset: impl Into<AbsoluteOffset<Option<f32>>>) -> Task<T> {
    task::effect(Action::widget(operation::scrollable::scroll_to(
        id.into(),
        offset.into(),
    )))
}

/// Scrolls the scrollable with the given [`Id`] by the provided [`AbsoluteOffset`].
pub fn scroll_by<T>(id: impl Into<Id>, offset: AbsoluteOffset) -> Task<T> {
    task::effect(Action::widget(operation::scrollable::scroll_by(
        id.into(),
        offset,
    )))
}

/// Scrolls the scrollable with the given [`Id`] to the provided [`AbsoluteOffset`] with smooth animation.
pub fn scroll_to_animated<T>(
    id: impl Into<Id>,
    offset: impl Into<AbsoluteOffset<Option<f32>>>,
) -> Task<T> {
    task::effect(Action::widget(operation::scrollable::scroll_to_animated(
        id.into(),
        offset.into(),
    )))
}

/// Scrolls the scrollable with the given [`Id`] by the provided [`AbsoluteOffset`] with smooth animation.
pub fn scroll_by_animated<T>(id: impl Into<Id>, offset: AbsoluteOffset) -> Task<T> {
    task::effect(Action::widget(operation::scrollable::scroll_by_animated(
        id.into(),
        offset,
    )))
}

/// Scrolls the scrollable with the given [`Id`] the minimum amount to make the
/// given [`Rectangle`] visible.
///
/// The rectangle should be in content coordinates. If already visible, no scrolling occurs.
pub fn ensure_visible<T>(id: impl Into<Id>, target_rect: Rectangle) -> Task<T> {
    task::effect(Action::widget(operation::scrollable::ensure_visible(
        id.into(),
        target_rect,
    )))
}

/// Scrolls the scrollable with the given [`Id`] the minimum amount to make the
/// given [`Rectangle`] visible, with smooth animation.
///
/// The rectangle should be in content coordinates. If already visible, no scrolling occurs.
pub fn ensure_visible_animated<T>(id: impl Into<Id>, target_rect: Rectangle) -> Task<T> {
    task::effect(Action::widget(
        operation::scrollable::ensure_visible_animated(id.into(), target_rect),
    ))
}

/// Focuses the previous focusable widget.
pub fn focus_previous<T>() -> Task<T> {
    task::effect(Action::widget(operation::focusable::focus_previous()))
}

/// Focuses the next focusable widget.
pub fn focus_next<T>() -> Task<T> {
    task::effect(Action::widget(operation::focusable::focus_next()))
}

/// Focuses the previous focusable widget that matches the given [`FocusLevel`].
pub fn focus_previous_filtered<T>(level: FocusLevel) -> Task<T> {
    task::effect(Action::widget(
        operation::focusable::focus_previous_filtered(level),
    ))
}

/// Focuses the next focusable widget that matches the given [`FocusLevel`].
pub fn focus_next_filtered<T>(level: FocusLevel) -> Task<T> {
    task::effect(Action::widget(operation::focusable::focus_next_filtered(
        level,
    )))
}

/// Unfocuses the currently focused widget.
pub fn unfocus<T>() -> Task<T> {
    task::effect(Action::widget(operation::focusable::unfocus()))
}

/// Returns whether the widget with the given [`Id`] is focused or not.
pub fn is_focused(id: impl Into<Id>) -> Task<bool> {
    task::widget(operation::focusable::is_focused(id.into()))
}

/// Focuses the widget with the given [`Id`].
pub fn focus<T>(id: impl Into<Id>) -> Task<T> {
    task::effect(Action::widget(operation::focusable::focus(id.into())))
}

/// Moves the cursor of the widget with the given [`Id`] to the end.
pub fn move_cursor_to_end<T>(id: impl Into<Id>) -> Task<T> {
    task::effect(Action::widget(operation::text_input::move_cursor_to_end(
        id.into(),
    )))
}

/// Moves the cursor of the widget with the given [`Id`] to the front.
pub fn move_cursor_to_front<T>(id: impl Into<Id>) -> Task<T> {
    task::effect(Action::widget(operation::text_input::move_cursor_to_front(
        id.into(),
    )))
}

/// Moves the cursor of the widget with the given [`Id`] to the provided position.
pub fn move_cursor_to<T>(id: impl Into<Id>, position: usize) -> Task<T> {
    task::effect(Action::widget(operation::text_input::move_cursor_to(
        id.into(),
        position,
    )))
}

/// Selects all the content of the widget with the given [`Id`].
pub fn select_all<T>(id: impl Into<Id>) -> Task<T> {
    task::effect(Action::widget(operation::text_input::select_all(id.into())))
}

/// Selects the given content range of the widget with the given [`Id`].
pub fn select_range<T>(id: impl Into<Id>, start: usize, end: usize) -> Task<T> {
    task::effect(Action::widget(operation::text_input::select_range(
        id.into(),
        start,
        end,
    )))
}
