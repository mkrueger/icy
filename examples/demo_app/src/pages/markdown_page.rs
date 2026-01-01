//! Markdown page - demonstrates the markdown widget
//!
//! The markdown widget can parse and display Markdown text with formatting,
//! links, code blocks with syntax highlighting, and more.

use icy_ui::widget::{column, container, markdown, row, scrollable, text, text_editor};
use icy_ui::{Color, Element, Fill, Theme};

// =============================================================================
// Markdown Page State
// =============================================================================

/// State for the markdown demo page
#[derive(Clone)]
pub struct MarkdownPageState {
    /// The markdown content (parsed items)
    pub items: Vec<markdown::Item>,
    /// The raw markdown text for editing
    pub editor_content: text_editor::Content,
}

impl Default for MarkdownPageState {
    fn default() -> Self {
        let default_markdown = EXAMPLE_MARKDOWN;
        Self {
            items: markdown::parse(default_markdown).collect(),
            editor_content: text_editor::Content::with_text(default_markdown),
        }
    }
}

const EXAMPLE_MARKDOWN: &str = r#"# Markdown Demo

This is a demonstration of the **markdown widget** in icy_ui.

## Text Formatting

You can use *italic*, **bold**, and ***bold italic*** text.
You can also use `inline code` for technical terms.

## Lists

### Unordered List
- First item
- Second item
- Third item with **bold**

### Ordered List
1. First step
2. Second step
3. Third step

## Code Blocks

Here's a Rust code example with syntax highlighting:

```rust
fn main() {
    println!("Hello, icy_ui!");
    
    let numbers = vec![1, 2, 3, 4, 5];
    let sum: i32 = numbers.iter().sum();
    println!("Sum: {}", sum);
}
```

And some Python:

```python
def greet(name):
    return f"Hello, {name}!"

print(greet("World"))
```

## Links

Check out the [icy_ui repository](https://github.com/mkrueger/icy) for more information.

## Blockquotes

> This is a blockquote.
> It can span multiple lines.
>
> And have multiple paragraphs.

## Horizontal Rule

---

## Tables

| Feature | Status |
|---------|--------|
| Bold | ✅ |
| Italic | ✅ |
| Code | ✅ |
| Links | ✅ |
| Lists | ✅ |

## Checkboxes

- [x] Completed task
- [ ] Pending task
- [ ] Another pending task

---

*Edit the markdown on the left to see live preview!*
"#;

// =============================================================================
// Update Function
// =============================================================================

/// Update the markdown page state based on messages
pub fn update_markdown(state: &mut MarkdownPageState, message: &crate::Message) -> bool {
    match message {
        crate::Message::MarkdownEditorAction(action) => {
            state.editor_content.perform(action.clone());
            // Re-parse markdown when content changes
            let text = state.editor_content.text();
            state.items = markdown::parse(&text).collect();
            true
        }
        crate::Message::MarkdownLinkClicked(url) => {
            // Open the URL in the default browser
            let _ = open::that(url);
            true
        }
        _ => false,
    }
}

// =============================================================================
// View Function
// =============================================================================

/// Create the view for the markdown page
pub fn view_markdown(state: &MarkdownPageState) -> Element<'_, crate::Message> {
    // Editor panel (left side)
    let editor = text_editor(&state.editor_content)
        .on_action(crate::Message::MarkdownEditorAction)
        .height(Fill)
        .padding(10);

    let editor_panel = column![
        text("Markdown Source").size(16),
        container(editor)
            .width(Fill)
            .height(Fill)
            .style(container::bordered_box),
    ]
    .spacing(5)
    .width(Fill)
    .height(Fill);

    // Preview panel (right side)
    let preview_content: Element<'_, markdown::Uri> = markdown::view(
        &state.items,
        markdown::Settings::with_text_size(14, Theme::dark()),
    )
    .into();

    let preview =
        scrollable(container(preview_content.map(crate::Message::MarkdownLinkClicked)).padding(15))
            .height(Fill);

    let preview_panel = column![
        text("Preview").size(16),
        container(preview)
            .width(Fill)
            .height(Fill)
            .style(container::bordered_box),
    ]
    .spacing(5)
    .width(Fill)
    .height(Fill);

    // Main layout - side by side
    let content = column![
        text("Markdown Widget Demo").size(24),
        text("Edit markdown on the left and see the rendered preview on the right. Supports syntax highlighting for code blocks!")
            .size(14)
            .color(Color::from_rgb(0.6, 0.6, 0.6)),
        row![editor_panel, preview_panel]
            .spacing(10)
            .height(Fill),
    ]
    .spacing(10)
    .padding(10)
    .width(Fill)
    .height(Fill);

    content.into()
}
