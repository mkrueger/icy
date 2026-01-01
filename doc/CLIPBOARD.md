# Clipboard API

Icy provides a clean, target-based clipboard API that works across platforms, including proper support for Wayland's security model.

## Overview

The clipboard API is built around the concept of **targets** - specifically `STANDARD` and `PRIMARY`:

- **`STANDARD`** - The main system clipboard (Ctrl+C/Ctrl+V)
- **`PRIMARY`** - The X11/Wayland selection clipboard (middle-click paste, selected text)

All clipboard operations return a `Task` that must be executed by the runtime. This design is required because on Wayland, clipboard access requires a connection to the display server through the window handle.

## Basic Usage

```rust
use icy_ui::clipboard::{STANDARD, PRIMARY, Format};

// Read text from the standard clipboard
let task = STANDARD.read_text();

// Write text to the standard clipboard
let task = STANDARD.write_text("Hello, world!".to_string());

// Read from the primary selection (X11/Wayland)
let task = PRIMARY.read_text();

// Write to the primary selection
let task = PRIMARY.write_text("Selected text".to_string());

// Use format constants
let task = STANDARD.read_format(Format::Image.formats());

// Use the builder pattern for complex writes
let task = STANDARD.write()
    .html("<b>Hello</b>".to_string())
    .text("Hello".to_string())
    .finish();
```

## Format Constants

The `Format` enum provides **platform-appropriate** format strings for common content types:

```rust
use icy_ui::clipboard::Format;

Format::Text   // Plain text (UTF-8)
Format::Html   // HTML content
Format::Rtf    // Rich Text Format
Format::Png    // PNG image
Format::Jpeg   // JPEG image
Format::Image  // Any image (PNG, JPEG, TIFF, etc.)
Format::Files  // File paths/URIs

// Get format strings for reading (returns &'static [&'static str])
let formats = Format::Image.formats();

// Get primary format string for writing (returns &'static str)
let primary = Format::Png.primary();

// Use with read_format
let task = STANDARD.read_format(Format::Image.formats());
```

### Platform-Specific Behavior

The format strings returned by `formats()` and `primary()` vary by platform:

| Format | Linux (X11/Wayland) | macOS (UTI) | Windows |
|--------|---------------------|-------------|---------|
| `Text` | `text/plain;charset=utf-8`, `UTF8_STRING` | `public.utf8-plain-text` | `CF_UNICODETEXT` |
| `Html` | `text/html` | `public.html` | `HTML Format` |
| `Rtf` | `text/rtf`, `application/rtf` | `public.rtf` | `Rich Text Format` |
| `Png` | `image/png` | `public.png` | `PNG` |
| `Jpeg` | `image/jpeg` | `public.jpeg` | `JFIF` |
| `Image` | `image/png`, `image/jpeg`, ... | `public.png`, `public.tiff`, ... | `PNG`, `CF_DIBV5` |
| `Files` | `text/uri-list` | `public.file-url` | `CF_HDROP` |

This abstraction allows you to write cross-platform clipboard code without worrying about
the underlying format identifiers.

## Write Builder

For writing multiple formats at once, use the builder pattern:

```rust
use icy_ui::clipboard::STANDARD;

// Write HTML with plain text fallback
let task = STANDARD.write()
    .html("<b>Bold</b> and <i>italic</i>".to_string())
    .text("Bold and italic".to_string())
    .finish();

// Write image with multiple formats
let task = STANDARD.write()
    .png(png_data)
    .custom("application/x-myapp", app_data)
    .finish();
```

### Builder Methods

```rust
.text(text: String)           // Add plain text
.html(html: String)           // Add HTML
.rtf(rtf: String)             // Add RTF
.png(data: Vec<u8>)           // Add PNG image
.jpeg(data: Vec<u8>)          // Add JPEG image
.custom(mime: &str, data)     // Add custom format
.with_formats(data, formats)  // Add data with multiple MIME types
.finish()                     // Build and return the Task
```

## Error Handling

The `Error` enum is available for detailed error information in future Result-based APIs:

```rust
use icy_ui::clipboard::Error;

pub enum Error {
    Empty,                                    // Clipboard is empty
    FormatNotAvailable { available: Vec<String> }, // Requested format not available
    AccessDenied,                             // Wayland: window not focused
    Unavailable,                              // Clipboard not available
    DecodingFailed,                           // Failed to decode data
}
```

Currently, most read methods return `Option` for simplicity. Use `available_formats()` to check what's available before reading.

## Available Methods

Both `STANDARD` and `PRIMARY` targets support the following methods:

### Text Operations

```rust
// Read plain text
fn read_text() -> Task<Option<String>>

// Write plain text
fn write_text(text: String) -> Task<()>
```

### Format Operations

```rust
// Read data for specific formats (first available wins)
fn read_format(formats: Vec<String>) -> Task<Option<ClipboardData>>

// Write data with specific formats
fn write_format(data: Vec<u8>, formats: Vec<String>) -> Task<()>

// Write multiple formats at once (for format negotiation)
fn write_multi(entries: Vec<(Vec<u8>, Vec<String>)>) -> Task<()>

// Get list of available formats
fn available_formats() -> Task<Vec<String>>

// Read all available data for multiple formats
fn read_all_formats(formats: Vec<String>) -> Task<Vec<ClipboardData>>
```

### Image Operations

```rust
// Read an image from the clipboard
fn read_image() -> Task<Option<image::RgbaImage>>

// Write an image to the clipboard
fn write_image(image: image::RgbaImage) -> Task<()>
```

### HTML Operations

```rust
// Read HTML content
fn read_html() -> Task<Option<String>>

// Write HTML content (with optional plain text fallback)
fn write_html(html: String, alt_text: Option<String>) -> Task<()>
```

### Rich Text Operations

```rust
// Read RTF content
fn read_rich_text() -> Task<Option<String>>

// Write RTF content (with optional plain text fallback)
fn write_rich_text(rtf: String, alt_text: Option<String>) -> Task<()>
```

### File Operations

```rust
// Read file URIs from the clipboard
fn read_files() -> Task<Option<Vec<String>>>

// Write file URIs to the clipboard
fn write_files(files: Vec<String>) -> Task<()>
```

### Clear

```rust
// Clear the clipboard contents
fn clear() -> Task<()>
```

### Availability Checks

```rust
// Check if the clipboard has any content
fn has_content() -> Task<bool>

// Check if any of the specified formats are available
fn has_format(formats: Vec<String>) -> Task<bool>
```

## Example: Copy/Paste in an Application

```rust
use icy_ui::{Element, Task};
use icy_ui::clipboard::STANDARD;
use icy_ui::widget::{button, column, text, text_input};

#[derive(Default)]
struct App {
    content: String,
    pasted: String,
}

#[derive(Debug, Clone)]
enum Message {
    ContentChanged(String),
    Copy,
    Paste,
    Pasted(Option<String>),
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ContentChanged(content) => {
                self.content = content;
                Task::none()
            }
            Message::Copy => {
                STANDARD.write_text(self.content.clone())
                    .discard()
            }
            Message::Paste => {
                STANDARD.read_text()
                    .map(Message::Pasted)
            }
            Message::Pasted(text) => {
                if let Some(text) = text {
                    self.pasted = text;
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        column![
            text_input("Type something...", &self.content)
                .on_input(Message::ContentChanged),
            button("Copy").on_press(Message::Copy),
            button("Paste").on_press(Message::Paste),
            text(&self.pasted),
        ]
        .into()
    }
}
```

## Platform Notes

### Format Abstraction

The `Format` enum abstracts platform differences in clipboard format identifiers:

- **Linux (X11/Wayland)**: Uses MIME types (e.g., `text/plain`, `image/png`) and X11 atom names (`UTF8_STRING`)
- **macOS**: Uses Uniform Type Identifiers (UTIs) (e.g., `public.utf8-plain-text`, `public.png`)
- **Windows**: Uses registered format names (e.g., `CF_UNICODETEXT`, `Rich Text Format`, `PNG`)

When using `Format::Text.formats()`, you get the correct identifiers for the current platform.

### Wayland

On Wayland, clipboard access is tied to window focus for security reasons. The clipboard operations are executed asynchronously through the runtime, which has access to the window handle required by `smithay_clipboard`.

### X11

Both `STANDARD` and `PRIMARY` selections work as expected. The primary selection is automatically populated when text is selected in most X11 applications.

### Windows / macOS

Only the `STANDARD` clipboard is available. Operations on `PRIMARY` will work but behave identically to `STANDARD`.

## Design Rationale

The target-based API (`STANDARD.read_text()`) was chosen over separate functions (`read()`, `read_primary()`) for several reasons:

1. **Discoverability** - All clipboard operations are accessible through the target constants
2. **Consistency** - The same methods work on both targets
3. **Clarity** - It's immediately clear which clipboard is being accessed
4. **Extensibility** - New methods can be added to `Target` without API proliferation
