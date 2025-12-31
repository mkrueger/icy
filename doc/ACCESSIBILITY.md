# Accessibility Support

icy provides accessibility support for screen readers and assistive technologies via [AccessKit](https://accesskit.dev/).

## Overview

Accessibility support is built on AccessKit, which provides cross-platform accessibility via native APIs:
- **Windows**: UI Automation
- **macOS**: NSAccessibility  
- **Linux**: AT-SPI (via D-Bus)

## Enabling Accessibility

Accessibility is behind a feature flag. Enable it in your `Cargo.toml`:

```toml
[dependencies]
iced = { version = "0.15", features = ["accessibility"] }
```

## Automatic Widget Support

Most widgets automatically provide accessibility information:

| Widget | Role | Information Provided |
|--------|------|---------------------|
| `Text` | StaticText | Text content |
| `Button` | Button | Label from child text, enabled state |
| `Checkbox` | CheckBox | Label, checked state |
| `TextInput` | TextField | Current value, placeholder |
| `Slider` | Slider | Value, min, max, step |
| `VerticalSlider` | Slider | Value, min, max, step |

### Examples

```rust
use iced::widget::{button, checkbox, text_input, slider};

// Button - label automatically derived from text child
button("Press me!")  // Screen reader: "Press me!, Button"

// Checkbox with label
checkbox(is_checked)
    .label("Enable notifications")  // Screen reader: "Enable notifications, Checkbox, checked/unchecked"

// Text input with placeholder
text_input("Enter your name...", &name)  // Screen reader announces value and placeholder

// Slider with range
slider(0.0..=100.0, volume, Message::VolumeChanged)  // Screen reader: "50, Slider, 0 to 100"
```

## Handling Accessibility Events

When a screen reader user interacts with your app, you receive `Event::Accessibility` events:

```rust
use iced::{Event, Task};
use iced::accessibility::Event as AccessibilityEvent;

fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::Event(Event::Accessibility(event)) => {
            if event.is_click() {
                // Handle screen reader "click" action
            }
            if event.is_focus() {
                // Widget received accessibility focus
            }
            if event.is_blur() {
                // Widget lost accessibility focus
            }
            Task::none()
        }
        _ => Task::none()
    }
}
```

### Available Actions

Screen readers can trigger these actions:

| Action | Description |
|--------|-------------|
| `Click` | Activate a button or control |
| `Focus` | Move accessibility focus to widget |
| `Blur` | Remove accessibility focus |
| `SetValue` | Set a value (sliders, text inputs) |
| `Increment` | Increase value (sliders) |
| `Decrement` | Decrease value (sliders) |

## Screen Reader Announcements

You can send announcements to the screen reader:

```rust
use iced::accessibility::{announce, Priority};

fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::SaveSuccessful => {
            // Polite announcement - waits for current speech
            announce("File saved successfully", Priority::Polite)
        }
        Message::Error(msg) => {
            // Assertive announcement - interrupts current speech
            announce(msg, Priority::Assertive)
        }
        _ => Task::none()
    }
}
```

## Custom Widget Accessibility

To make a custom widget accessible, implement the `accessibility()` method:

```rust
use iced::advanced::Widget;
use iced::accessibility::WidgetInfo;

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for MyWidget {
    // ... other methods ...

    #[cfg(feature = "accessibility")]
    fn accessibility(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
    ) -> Option<WidgetInfo> {
        Some(
            WidgetInfo::button("My Custom Button")
                .with_bounds(layout.bounds())
                .with_enabled(self.is_enabled)
        )
    }
}
```

### WidgetInfo Builders

```rust
use iced::accessibility::WidgetInfo;

// Button
WidgetInfo::button("Click me")

// Checkbox
WidgetInfo::checkbox("Option", is_checked)

// Text input
WidgetInfo::text_input(current_value)
    .with_placeholder("Enter text...")

// Slider
WidgetInfo::slider(value, min, max)
    .with_step(1.0)

// Static text/label
WidgetInfo::label("Some text")

// Generic container
WidgetInfo::container()
    .with_children(child_ids)
```

### Providing Labels for Container Widgets

If your widget contains other widgets (like Button contains Text), implement `accessibility_label()`:

```rust
#[cfg(feature = "accessibility")]
fn accessibility_label(&self) -> Option<std::borrow::Cow<'_, str>> {
    Some(std::borrow::Cow::Borrowed(&self.label))
}
```

Parent widgets can then query this to get their label automatically.

## Testing Accessibility

### Windows
- **NVDA**: Free, open-source screen reader
- **Narrator**: Built into Windows

### macOS
- **VoiceOver**: Built into macOS (Cmd+F5 to toggle)

### Linux
- **Orca**: Common screen reader for GNOME
- **Accerciser**: AT-SPI explorer for debugging

## Best Practices

1. **Provide meaningful labels**: Avoid generic labels like "Button" - use descriptive text
2. **Handle all actions**: Respond to `Click`, `Focus`, etc. from accessibility events
3. **Announce important changes**: Use `announce()` for status updates, errors, confirmations
4. **Test with screen readers**: Actually test with NVDA, VoiceOver, or Orca
5. **Keyboard navigation**: Ensure all interactive widgets are focusable and operable via keyboard

## API Reference

### Types (from `iced::accessibility`)

| Type | Description |
|------|-------------|
| `WidgetInfo` | Accessibility information for a widget |
| `Event` | Accessibility event from screen reader |
| `Action` | Action type (Click, Focus, SetValue, etc.) |
| `ActionData` | Optional data for actions (e.g., new value) |
| `Priority` | Announcement priority (Polite, Assertive) |
| `NodeId` | Unique identifier for accessibility nodes |
| `Role` | Widget role (Button, CheckBox, Slider, etc.) |

### Functions (from `iced::accessibility`)

| Function | Description |
|----------|-------------|
| `announce(message, priority)` | Announce message to screen reader |
| `focus(node_id)` | Programmatically focus an accessible element |
| `node_id(u64)` | Create a NodeId from a numeric value |
