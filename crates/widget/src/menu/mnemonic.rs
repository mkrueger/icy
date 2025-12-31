// Mnemonic support for menus
//
// Mnemonics allow keyboard navigation using Alt+letter shortcuts.
// Labels use '&' as the mnemonic marker (e.g., "&File" underlines 'F').
// Use "&&" for a literal ampersand.

use std::borrow::Cow;

use crate::core::text::Span;
use crate::text;

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
/// If `show_underline` is false or there's no mnemonic, the underline is not shown,
/// but the same widget type (Rich text with 3 spans) is always returned to avoid tree state issues.
pub fn mnemonic_text<'a, Message>(
    label: &str,
    show_underline: bool,
) -> crate::core::Element<'a, Message, crate::Theme, crate::Renderer>
where
    Message: Clone + 'a,
{
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
        (before, mnemonic, after)
    } else {
        // No mnemonic - use the whole text as "before", empty mnemonic and after
        (display, "", "")
    };

    // Always use Rich text with 3 spans to keep widget structure consistent
    text::Rich::<'a, (), Message>::with_spans([
        Span::<'a, ()>::new(before.to_owned()),
        Span::<'a, ()>::new(mnemonic.to_owned()).underline(show_underline),
        Span::<'a, ()>::new(after.to_owned()),
    ])
    .into()
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
