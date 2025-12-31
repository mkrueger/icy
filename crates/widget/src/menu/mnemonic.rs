// Mnemonic support for menus
//
// Mnemonics allow keyboard navigation using Alt+letter shortcuts.
// Labels use '&' as the mnemonic marker (e.g., "&File" underlines 'F').
// Use "&&" for a literal ampersand.

use std::borrow::Cow;
use std::cell::Cell;

use crate::core::text::Span;
use crate::text;
use crate::core::{
    Clipboard, Element, Layout, Length, Rectangle, Shell, Size, Widget,
    event,
    layout::{Limits, Node},
    mouse::Cursor,
    overlay, renderer,
    widget::{Tree, tree},
};

type RichType<Message> = text::Rich<'static, (), Message, crate::Theme, crate::Renderer>;

// Thread-local state for mnemonic underline visibility.
// This is updated by MenuBar when Alt is pressed/released.
thread_local! {
    static SHOW_MNEMONIC_UNDERLINES: Cell<bool> = const { Cell::new(false) };
}

/// Set whether mnemonic underlines should be shown.
/// Called by MenuBar when Alt key state changes.
pub(crate) fn set_show_underlines(show: bool) {
    SHOW_MNEMONIC_UNDERLINES.with(|cell| cell.set(show));
}

/// Get whether mnemonic underlines should be shown.
pub(crate) fn get_show_underlines() -> bool {
    SHOW_MNEMONIC_UNDERLINES.with(|cell| cell.get())
}

/// How mnemonics are displayed in menu labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MnemonicDisplay {
    /// Never show mnemonic underlines.
    Hide,
    /// Always show mnemonic underlines.
    Show,
    /// Show mnemonic underlines only when Alt key is pressed (Windows/Linux default).
    #[default]
    OnAlt,
}

/// Parsed mnemonic information from a label.
#[derive(Debug, Clone)]
pub struct ParsedMnemonic<'a> {
    /// The display text with mnemonic marker removed.
    pub display_text: Cow<'a, str>,
    /// The mnemonic character (lowercase for matching).
    pub mnemonic_char: Option<char>,
    /// Index of the underlined character in display_text.
    pub underline_index: Option<usize>,
}

/// Parse a label for mnemonic marker ('&').
///
/// # Examples
/// - `"&File"` → `("File", Some('f'), Some(0))`
/// - `"E&xit"` → `("Exit", Some('x'), Some(1))`
/// - `"Save && Close"` → `("Save & Close", None, None)` (escaped)
/// - `"No mnemonic"` → `("No mnemonic", None, None)`
pub fn parse_mnemonic(label: &str) -> ParsedMnemonic<'_> {
    const MARKER: char = '&';

    let mut result = String::with_capacity(label.len());
    let mut mnemonic_char = None;
    let mut underline_index = None;
    let mut chars = label.chars().peekable();

    while let Some(c) = chars.next() {
        if c == MARKER {
            if chars.peek() == Some(&MARKER) {
                // Escaped marker (&&) -> single &
                result.push(MARKER);
                let _ = chars.next();
            } else if let Some(next) = chars.next() {
                // This is the mnemonic character (only take the first one)
                if mnemonic_char.is_none() {
                    underline_index = Some(result.len());
                    mnemonic_char = Some(next.to_ascii_lowercase());
                }
                result.push(next);
            }
        } else {
            result.push(c);
        }
    }

    ParsedMnemonic {
        display_text: if result == label {
            Cow::Borrowed(label)
        } else {
            Cow::Owned(result)
        },
        mnemonic_char,
        underline_index,
    }
}

/// Create a text element with the mnemonic character underlined.
///
/// The underline visibility is controlled by the MenuBar's internal state,
/// which tracks Alt key presses automatically.
pub fn mnemonic_text<'a, Message>(
    label: &str,
) -> crate::core::Element<'a, Message, crate::Theme, crate::Renderer>
where
    Message: Clone + 'a,
{
    MnemonicLabel::<Message>::new(label).into()
}

struct MnemonicLabel<Message> {
    before: String,
    mnemonic: String,
    after: String,
    underline: bool,
    rich: text::Rich<'static, (), Message, crate::Theme, crate::Renderer>,
}

impl<Message> MnemonicLabel<Message> {
    fn new(label: &str) -> Self {
        let parsed = parse_mnemonic(label);
        let display = parsed.display_text.as_ref();

        let (before, mnemonic, after) = if let Some(idx) = parsed.underline_index {
            // Get the character at idx (handling multi-byte chars)
            let before = &display[..idx];
            let mnemonic_end = display[idx..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| idx + i)
                .unwrap_or(display.len());
            let mnemonic = &display[idx..mnemonic_end];
            let after = &display[mnemonic_end..];
            (before.to_owned(), mnemonic.to_owned(), after.to_owned())
        } else {
            (display.to_owned(), String::new(), String::new())
        };

        let underline = get_show_underlines();

        let rich = Self::build_rich(&before, &mnemonic, &after, underline);

        Self {
            before,
            mnemonic,
            after,
            underline,
            rich,
        }
    }

    fn build_rich(
        before: &str,
        mnemonic: &str,
        after: &str,
        underline: bool,
    ) -> text::Rich<'static, (), Message, crate::Theme, crate::Renderer> {
        text::Rich::with_spans([
            Span::<'static, ()>::new(before.to_owned()),
            Span::<'static, ()>::new(mnemonic.to_owned()).underline(underline),
            Span::<'static, ()>::new(after.to_owned()),
        ])
    }

    fn sync(&mut self) {
        let underline = get_show_underlines();

        if underline == self.underline {
            return;
        }

        self.underline = underline;
        self.rich = Self::build_rich(&self.before, &self.mnemonic, &self.after, underline);
    }
}

impl<Message> Widget<Message, crate::Theme, crate::Renderer> for MnemonicLabel<Message> {
    fn tag(&self) -> tree::Tag {
        <RichType<Message> as Widget<Message, crate::Theme, crate::Renderer>>::tag(&self.rich)
    }

    fn state(&self) -> tree::State {
        <RichType<Message> as Widget<Message, crate::Theme, crate::Renderer>>::state(&self.rich)
    }

    fn size(&self) -> Size<Length> {
        <RichType<Message> as Widget<Message, crate::Theme, crate::Renderer>>::size(&self.rich)
    }

    fn children(&self) -> Vec<Tree> {
        <RichType<Message> as Widget<Message, crate::Theme, crate::Renderer>>::children(&self.rich)
    }

    fn diff(&self, tree: &mut Tree) {
        <RichType<Message> as Widget<Message, crate::Theme, crate::Renderer>>::diff(&self.rich, tree);
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &crate::Renderer, limits: &Limits) -> Node {
        self.sync();
        <RichType<Message> as Widget<Message, crate::Theme, crate::Renderer>>::layout(
            &mut self.rich,
            tree,
            renderer,
            limits,
        )
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &event::Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        <RichType<Message> as Widget<Message, crate::Theme, crate::Renderer>>::update(
            &mut self.rich,
            tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        <RichType<Message> as Widget<Message, crate::Theme, crate::Renderer>>::draw(
            &self.rich,
            tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &crate::Renderer,
        viewport: &Rectangle,
        translation: crate::core::Vector,
    ) -> Option<overlay::Element<'b, Message, crate::Theme, crate::Renderer>> {
        let _ = (tree, layout, renderer, viewport, translation);
        None
    }
}

impl<'a, Message> From<MnemonicLabel<Message>> for Element<'a, Message, crate::Theme, crate::Renderer>
where
    Message: Clone + 'a,
{
    fn from(value: MnemonicLabel<Message>) -> Self {
        Element::new(value)
    }
}

/// Check if mnemonics are enabled on the current platform.
///
/// Mnemonics are enabled on Windows and Linux, but disabled on macOS
/// (which uses Cmd-based shortcuts instead of Alt-based mnemonics).
#[inline]
pub fn mnemonics_enabled() -> bool {
    !cfg!(target_os = "macos")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mnemonic_simple() {
        let parsed = parse_mnemonic("&File");
        assert_eq!(parsed.display_text, "File");
        assert_eq!(parsed.mnemonic_char, Some('f'));
        assert_eq!(parsed.underline_index, Some(0));
    }

    #[test]
    fn test_parse_mnemonic_middle() {
        let parsed = parse_mnemonic("E&xit");
        assert_eq!(parsed.display_text, "Exit");
        assert_eq!(parsed.mnemonic_char, Some('x'));
        assert_eq!(parsed.underline_index, Some(1));
    }

    #[test]
    fn test_parse_mnemonic_escaped() {
        let parsed = parse_mnemonic("Save && Close");
        assert_eq!(parsed.display_text, "Save & Close");
        assert_eq!(parsed.mnemonic_char, None);
        assert_eq!(parsed.underline_index, None);
    }

    #[test]
    fn test_parse_mnemonic_none() {
        let parsed = parse_mnemonic("No mnemonic");
        assert_eq!(parsed.display_text, "No mnemonic");
        assert_eq!(parsed.mnemonic_char, None);
        assert_eq!(parsed.underline_index, None);
    }

    #[test]
    fn test_parse_mnemonic_multiple() {
        // Only first mnemonic is used
        let parsed = parse_mnemonic("&File &Menu");
        assert_eq!(parsed.display_text, "File Menu");
        assert_eq!(parsed.mnemonic_char, Some('f'));
        assert_eq!(parsed.underline_index, Some(0));
    }
}
