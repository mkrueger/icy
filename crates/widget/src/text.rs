//! Draw and interact with text.
mod rich;

pub use crate::core::text::{Fragment, Highlighter, IntoFragment, Span};
pub use crate::core::widget::text::*;
pub use rich::Rich;

/// A bunch of text.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use icy_ui_widget::*; } pub use icy_ui_widget::Renderer; pub use icy_ui_widget::core::*; }
/// # pub type State = ();
/// # pub type Element<'a, Message> = icy_ui_widget::core::Element<'a, Message, icy_ui_widget::Theme, icy_ui_widget::Renderer>;
/// use icy_ui::widget::text;
/// use icy_ui::color;
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     text("Hello, this is iced!")
///         .size(20)
///         .color(color!(0x0000ff))
///         .into()
/// }
/// ```
pub type Text<'a, Theme = crate::Theme, Renderer = crate::Renderer> =
    crate::core::widget::Text<'a, Theme, Renderer>;
