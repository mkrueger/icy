//! A control for incremental adjustments of a value.
//!
//! # Example
//! ```no_run
//! use icy_ui::widget::spin_button;
//!
//! #[derive(Clone)]
//! enum Message {
//!     ValueChanged(i32),
//! }
//!
//! let spinner = spin_button("42", 42, 1, 0, 100, Message::ValueChanged);
//! ```

use crate::core::{Alignment, Element, Length, Padding};
use crate::{button, column, container, row, text, Button, Container, Text};
use std::borrow::Cow;
use std::ops::{Add, Sub};

/// Creates a horizontal spin button widget.
///
/// A spin button allows users to increment or decrement a value
/// using `+` and `-` buttons.
pub fn spin_button<'a, T, M>(
    label: impl Into<Cow<'a, str>>,
    value: T,
    step: T,
    min: T,
    max: T,
    on_change: impl Fn(T) -> M + 'static,
) -> SpinButton<'a, T, M>
where
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    SpinButton::new(label, value, step, min, max, Orientation::Horizontal, on_change)
}

/// Creates a vertical spin button widget.
///
/// The increment button is on top, the decrement button on bottom.
pub fn vertical<'a, T, M>(
    label: impl Into<Cow<'a, str>>,
    value: T,
    step: T,
    min: T,
    max: T,
    on_change: impl Fn(T) -> M + 'static,
) -> SpinButton<'a, T, M>
where
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    SpinButton::new(label, value, step, min, max, Orientation::Vertical, on_change)
}

/// Orientation of the spin button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    /// Horizontal layout: `[−] value [+]`
    #[default]
    Horizontal,
    /// Vertical layout with `+` on top, `−` on bottom
    Vertical,
}

/// A widget for incrementally adjusting a numeric value.
pub struct SpinButton<'a, T, M>
where
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    /// The formatted value of the spin button.
    label: Cow<'a, str>,
    /// The current value.
    value: T,
    /// The amount to increment or decrement.
    step: T,
    /// The minimum value permitted.
    min: T,
    /// The maximum value permitted.
    max: T,
    /// Layout orientation.
    orientation: Orientation,
    /// Callback when value changes.
    on_change: Box<dyn Fn(T) -> M>,
    /// Width of the label area.
    label_width: f32,
    /// Padding around the widget.
    padding: Padding,
}

impl<'a, T, M> SpinButton<'a, T, M>
where
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    /// Creates a new spin button.
    fn new(
        label: impl Into<Cow<'a, str>>,
        value: T,
        step: T,
        min: T,
        max: T,
        orientation: Orientation,
        on_change: impl Fn(T) -> M + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            value: clamp(value, min, max),
            step,
            min,
            max,
            orientation,
            on_change: Box::new(on_change),
            label_width: 48.0,
            padding: Padding::ZERO,
        }
    }

    /// Sets the width of the value label area.
    pub fn label_width(mut self, width: f32) -> Self {
        self.label_width = width;
        self
    }

    /// Sets the padding around the widget.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }
}

fn clamp<T>(value: T, min: T, max: T) -> T
where
    T: PartialOrd,
{
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn increment<T>(value: T, step: T, _min: T, max: T) -> T
where
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    let new_value = value + step;
    if new_value > max {
        max
    } else {
        new_value
    }
}

fn decrement<T>(value: T, step: T, min: T, _max: T) -> T
where
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    // Check if subtracting would go below min
    // We need to be careful with underflow for unsigned types
    if value < min + step {
        min
    } else {
        value - step
    }
}

impl<'a, T, Message> From<SpinButton<'a, T, Message>> for Element<'a, Message, crate::Theme, crate::Renderer>
where
    Message: Clone + 'static,
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd + 'static,
{
    fn from(spin: SpinButton<'a, T, Message>) -> Self {
        match spin.orientation {
            Orientation::Horizontal => horizontal_variant(spin),
            Orientation::Vertical => vertical_variant(spin),
        }
    }
}

fn make_button<'a, T, Message>(
    label: &'static str,
    spin: &SpinButton<'_, T, Message>,
    operation: fn(T, T, T, T) -> T,
) -> Button<'a, Message, crate::Theme, crate::Renderer>
where
    Message: Clone + 'static,
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd,
{
    let new_value = operation(spin.value, spin.step, spin.min, spin.max);
    let is_at_limit = new_value == spin.value;

    let btn = button(text(label).center())
        .padding(Padding::new(4.0).left(10.0).right(10.0));

    if is_at_limit {
        btn
    } else {
        btn.on_press((spin.on_change)(new_value))
    }
}

fn horizontal_variant<'a, T, Message>(
    spin: SpinButton<'a, T, Message>,
) -> Element<'a, Message, crate::Theme, crate::Renderer>
where
    Message: Clone + 'static,
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd + 'static,
{
    let decrement_btn = make_button("−", &spin, decrement);
    let increment_btn = make_button("+", &spin, increment);

    let label: Text<'a, crate::Theme, crate::Renderer> = text(spin.label);
    let label_container: Container<'a, Message, crate::Theme, crate::Renderer> =
        container(label.center())
            .center_x(Length::Fixed(spin.label_width))
            .center_y(Length::Shrink);

    container(
        row![decrement_btn, label_container, increment_btn]
            .align_y(Alignment::Center)
            .spacing(2),
    )
    .padding(spin.padding)
    .style(crate::container::rounded_box)
    .into()
}

fn vertical_variant<'a, T, Message>(
    spin: SpinButton<'a, T, Message>,
) -> Element<'a, Message, crate::Theme, crate::Renderer>
where
    Message: Clone + 'static,
    T: Copy + Sub<Output = T> + Add<Output = T> + PartialOrd + 'static,
{
    let decrement_btn = make_button("−", &spin, decrement);
    let increment_btn = make_button("+", &spin, increment);

    let label: Text<'a, crate::Theme, crate::Renderer> = text(spin.label);
    let label_container: Container<'a, Message, crate::Theme, crate::Renderer> =
        container(label.center())
            .center_x(Length::Fixed(spin.label_width))
            .center_y(Length::Shrink);

    container(
        column![increment_btn, label_container, decrement_btn]
            .align_x(Alignment::Center)
            .spacing(2),
    )
    .padding(spin.padding)
    .style(crate::container::rounded_box)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment() {
        assert_eq!(increment(5i32, 1, 0, 10), 6);
        assert_eq!(increment(10i32, 1, 0, 10), 10); // at max
        assert_eq!(increment(9i32, 5, 0, 10), 10); // would exceed max
    }

    #[test]
    fn test_decrement() {
        assert_eq!(decrement(5i32, 1, 0, 10), 4);
        assert_eq!(decrement(0i32, 1, 0, 10), 0); // at min
        assert_eq!(decrement(2i32, 5, 0, 10), 0); // would go below min
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5i32, 0, 10), 5);
        assert_eq!(clamp(-5i32, 0, 10), 0);
        assert_eq!(clamp(15i32, 0, 10), 10);
    }
}
