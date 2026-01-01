//! Operate on widgets that can be focused.
use crate::Rectangle;
use crate::widget::Id;
use crate::widget::operation::{self, Operation, Outcome};

/// Controls which widgets participate in Tab navigation.
///
/// This mirrors macOS "Full Keyboard Access" behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusLevel {
    /// Only text inputs and editors receive Tab focus (macOS default behavior).
    TextOnly,

    /// All interactive widgets receive Tab focus (full keyboard access).
    #[default]
    AllControls,

    /// No automatic Tab navigation (app handles focus manually).
    Manual,
}

impl FocusLevel {
    /// Returns whether the given tier should be focusable at this level.
    pub fn allows(self, tier: FocusTier) -> bool {
        match self {
            FocusLevel::TextOnly => tier == FocusTier::Text,
            FocusLevel::AllControls => true,
            FocusLevel::Manual => false,
        }
    }
}

/// The focus tier of a widget, used with [`FocusLevel`] to filter Tab navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum FocusTier {
    /// Text inputs and editors — always focusable via Tab.
    Text,
    /// Buttons, checkboxes, sliders — only focusable with [`FocusLevel::AllControls`].
    #[default]
    Control,
}

/// The internal state of a widget that can be focused.
pub trait Focusable {
    /// Returns whether the widget is focused or not.
    fn is_focused(&self) -> bool;

    /// Focuses the widget.
    fn focus(&mut self);

    /// Unfocuses the widget.
    fn unfocus(&mut self);

    /// Returns the focus tier of this widget.
    ///
    /// Used with [`FocusLevel`] to determine if this widget participates
    /// in Tab navigation.
    fn focus_tier(&self) -> FocusTier {
        FocusTier::Control
    }
}

/// A summary of the focusable widgets present on a widget tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Count {
    /// The index of the current focused widget, if any.
    pub focused: Option<usize>,

    /// The total amount of focusable widgets.
    pub total: usize,
}

/// Produces an [`Operation`] that focuses the widget with the given [`Id`].
pub fn focus<T>(target: Id) -> impl Operation<T> {
    struct Focus {
        target: Id,
    }

    impl<T> Operation<T> for Focus {
        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            match id {
                Some(id) if id == &self.target => {
                    state.focus();
                }
                _ => {
                    state.unfocus();
                }
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    Focus { target }
}

/// Produces an [`Operation`] that unfocuses the focused widget.
pub fn unfocus<T>() -> impl Operation<T> {
    struct Unfocus;

    impl<T> Operation<T> for Unfocus {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            state.unfocus();
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    Unfocus
}

/// Produces an [`Operation`] that generates a [`Count`] and chains it with the
/// provided function to build a new [`Operation`].
pub fn count() -> impl Operation<Count> {
    struct CountFocusable {
        count: Count,
    }

    impl Operation<Count> for CountFocusable {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if state.is_focused() {
                self.count.focused = Some(self.count.total);
            }

            self.count.total += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Count>)) {
            operate(self);
        }

        fn finish(&self) -> Outcome<Count> {
            Outcome::Some(self.count)
        }
    }

    CountFocusable {
        count: Count::default(),
    }
}

/// Produces an [`Operation`] that searches for the current focused widget, and
/// - if found, focuses the previous focusable widget.
/// - if not found, focuses the last focusable widget.
pub fn focus_previous<T>() -> impl Operation<T>
where
    T: Send + 'static,
{
    struct FocusPrevious {
        count: Count,
        current: usize,
    }

    impl<T> Operation<T> for FocusPrevious {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if self.count.total == 0 {
                return;
            }

            match self.count.focused {
                None if self.current == self.count.total - 1 => state.focus(),
                Some(0) if self.current == 0 => state.unfocus(),
                Some(0) => {}
                Some(focused) if focused == self.current => state.unfocus(),
                Some(focused) if focused - 1 == self.current => state.focus(),
                _ => {}
            }

            self.current += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    operation::then(count(), |count| FocusPrevious { count, current: 0 })
}

/// Produces an [`Operation`] that searches for the current focused widget, and
/// - if found, focuses the next focusable widget.
/// - if not found, focuses the first focusable widget.
pub fn focus_next<T>() -> impl Operation<T>
where
    T: Send + 'static,
{
    struct FocusNext {
        count: Count,
        current: usize,
    }

    impl<T> Operation<T> for FocusNext {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            match self.count.focused {
                None if self.current == 0 => state.focus(),
                Some(focused) if focused == self.current => state.unfocus(),
                Some(focused) if focused + 1 == self.current => state.focus(),
                _ => {}
            }

            self.current += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    operation::then(count(), |count| FocusNext { count, current: 0 })
}

/// Produces an [`Operation`] that searches for the current focused widget
/// and stores its ID. This ignores widgets that do not have an ID.
pub fn find_focused() -> impl Operation<Id> {
    struct FindFocused {
        focused: Option<Id>,
    }

    impl Operation<Id> for FindFocused {
        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if state.is_focused() && id.is_some() {
                self.focused = id.cloned();
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Id>)) {
            operate(self);
        }

        fn finish(&self) -> Outcome<Id> {
            if let Some(id) = &self.focused {
                Outcome::Some(id.clone())
            } else {
                Outcome::None
            }
        }
    }

    FindFocused { focused: None }
}

/// Produces an [`Operation`] that searches for the focusable widget
/// and stores whether it is focused or not. This ignores widgets that
/// do not have an ID.
pub fn is_focused(target: Id) -> impl Operation<bool> {
    struct IsFocused {
        target: Id,
        is_focused: Option<bool>,
    }

    impl Operation<bool> for IsFocused {
        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if id.is_some_and(|id| *id == self.target) {
                self.is_focused = Some(state.is_focused());
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<bool>)) {
            if self.is_focused.is_some() {
                return;
            }

            operate(self);
        }

        fn finish(&self) -> Outcome<bool> {
            self.is_focused.map_or(Outcome::None, Outcome::Some)
        }
    }

    IsFocused {
        target,
        is_focused: None,
    }
}

/// Produces an [`Operation`] that generates a [`Count`] of focusable widgets
/// that match the given [`FocusLevel`].
pub fn count_filtered(level: FocusLevel) -> impl Operation<Count> {
    struct CountFiltered {
        level: FocusLevel,
        count: Count,
    }

    impl Operation<Count> for CountFiltered {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if !self.level.allows(state.focus_tier()) {
                return;
            }

            if state.is_focused() {
                self.count.focused = Some(self.count.total);
            }

            self.count.total += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Count>)) {
            operate(self);
        }

        fn finish(&self) -> Outcome<Count> {
            Outcome::Some(self.count)
        }
    }

    CountFiltered {
        level,
        count: Count::default(),
    }
}

/// Produces an [`Operation`] that focuses the previous focusable widget
/// that matches the given [`FocusLevel`].
pub fn focus_previous_filtered<T>(level: FocusLevel) -> impl Operation<T>
where
    T: Send + 'static,
{
    struct CountPreviousFiltered {
        level: FocusLevel,
        count: Count,
    }

    struct ApplyPreviousFiltered {
        level: FocusLevel,
        count: Count,
        current: usize,
    }

    impl<T> Operation<T> for CountPreviousFiltered {
        fn focusable(
            &mut self,
            _id: Option<&Id>,
            _bounds: Rectangle,
            state: &mut dyn Focusable,
        ) {
            if !self.level.allows(state.focus_tier()) {
                return;
            }


            if state.is_focused() {
                self.count.focused = Some(self.count.total);
            }

            self.count.total += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }

        fn finish(&self) -> Outcome<T> {

            Outcome::Chain(Box::new(ApplyPreviousFiltered {
                level: self.level,
                count: self.count,
                current: 0,
            }))
        }
    }

    impl<T> Operation<T> for ApplyPreviousFiltered {
        fn focusable(
            &mut self,
            _id: Option<&Id>,
            _bounds: Rectangle,
            state: &mut dyn Focusable,
        ) {
            if !self.level.allows(state.focus_tier()) {
                return;
            }

            if self.count.total == 0 {
                return;
            }


            match self.count.focused {
                None if self.current == self.count.total - 1 => {
                    state.focus()
                }
                Some(0) if self.current == 0 => {
                    state.unfocus()
                }
                // Wrap: first element focused, now focus last
                Some(0) if self.current == self.count.total - 1 => {
                    state.focus()
                }
                Some(0) => {}
                Some(focused) if focused == self.current => {
                    state.unfocus()
                }
                Some(focused) if focused - 1 == self.current => {
                    state.focus()
                }
                _ => {}
            }

            self.current += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    CountPreviousFiltered {
        level,
        count: Count::default(),
    }
}

/// Produces an [`Operation`] that focuses the next focusable widget
/// that matches the given [`FocusLevel`].
pub fn focus_next_filtered<T>(level: FocusLevel) -> impl Operation<T>
where
    T: Send + 'static,
{
    struct CountNextFiltered {
        level: FocusLevel,
        count: Count,
    }

    struct ApplyNextFiltered {
        level: FocusLevel,
        count: Count,
        current: usize,
    }

    impl<T> Operation<T> for CountNextFiltered {
        fn focusable(
            &mut self,
            _id: Option<&Id>,
            _bounds: Rectangle,
            state: &mut dyn Focusable,
        ) {
            if !self.level.allows(state.focus_tier()) {
                return;
            }


            if state.is_focused() {
                self.count.focused = Some(self.count.total);
            }

            self.count.total += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }

        fn finish(&self) -> Outcome<T> {

            Outcome::Chain(Box::new(ApplyNextFiltered {
                level: self.level,
                count: self.count,
                current: 0,
            }))
        }
    }

    impl<T> Operation<T> for ApplyNextFiltered {
        fn focusable(
            &mut self,
            _id: Option<&Id>,
            _bounds: Rectangle,
            state: &mut dyn Focusable,
        ) {
            if !self.level.allows(state.focus_tier()) {
                return;
            }

            if self.count.total == 0 {
                return;
            }


            match self.count.focused {
                None if self.current == 0 => {
                    state.focus()
                }
                // Wrap: last element focused, now focus first
                Some(focused) if focused == self.count.total - 1 && self.current == 0 => {
                    state.focus()
                }
                Some(focused) if focused == self.current => {
                    state.unfocus()
                }
                Some(focused) if focused + 1 == self.current => {
                    state.focus()
                }
                _ => {}
            }

            self.current += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    CountNextFiltered {
        level,
        count: Count::default(),
    }
}
