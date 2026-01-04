# Accessibility Support

icy_ui provides accessibility support for screen readers and assistive technologies via [AccessKit](https://accesskit.dev/).

## Overview

Accessibility support is built on AccessKit, which provides cross-platform accessibility via native APIs:
- **Windows**: UI Automation
- **macOS**: NSAccessibility  
- **Linux**: AT-SPI (via D-Bus)

## Accessibility Mode

When a screen reader connects (e.g., VoiceOver, NVDA), the application automatically enters **accessibility mode**. This mode changes several behaviors:

| Behavior | Normal Mode | Accessibility Mode |
|----------|-------------|-------------------|
| **Tab Navigation** | Respects `FocusLevel` setting | Traverses ALL interactive controls |
| **Focus Handling** | Widget focus only | Separate a11y focus (VoiceOver cursor) |
| **Announcements** | Disabled | Focus changes are announced |

The mode is tracked via `accessibility::Mode`:
```rust
use icy_ui::accessibility::Mode;

// Mode::Inactive - no screen reader connected
// Mode::Active   - screen reader is connected
```

## Accessibility Focus vs Widget Focus

In accessibility mode, there are **two separate focus concepts**:

1. **Widget Focus**: The standard keyboard focus managed by widgets (text inputs, buttons, etc.)
2. **Accessibility Focus**: The "VoiceOver cursor" controlled by the screen reader

The a11y focus can point to non-interactive elements (like static text), while widget focus only applies to interactive widgets. When widget focus changes, the a11y focus syncs automatically.

## Enabling Accessibility

Accessibility is behind a feature flag. Enable it in your `Cargo.toml`:

```toml
[dependencies]
icy_ui = { version = "0.1", features = ["accessibility"] }
```

## Automatic Widget Support

Most widgets automatically provide accessibility information and handle accessibility events:

| Widget | Role | Information Provided | Handled Events |
|--------|------|---------------------|----------------|
| `Button` | Button | Label, enabled state | Click, Focus, Blur |
| `Checkbox` | CheckBox | Label, checked state | Click, Focus, Blur |
| `TextInput` | TextField | Value, placeholder | Focus, Blur, SetValue |
| `Slider` | Slider | Value, min, max, step | Increment, Decrement, SetValue, Focus, Blur |
| `VerticalSlider` | Slider | Value, min, max, step | Increment, Decrement, SetValue, Focus, Blur |

### Examples

```rust
use icy_ui::widget::{button, checkbox, text_input, slider};

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

Standard widgets handle accessibility events automatically. For custom widgets, you receive `Event::Accessibility` events:

```rust
use icy_ui::{Event, Task};
use icy_ui::accessibility::Event as AccessibilityEvent;

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
use icy_ui::accessibility::{announce, Priority};

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
use icy_ui::advanced::Widget;
use icy_ui::accessibility::WidgetInfo;

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
use icy_ui::accessibility::WidgetInfo;

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
2. **Widgets handle events**: Standard widgets handle accessibility events automatically
3. **Announce important changes**: Use `announce()` for status updates, errors, confirmations
4. **Test with screen readers**: Actually test with NVDA, VoiceOver, or Orca
5. **Keyboard navigation**: Ensure all interactive widgets are focusable and operable via keyboard
6. **Give widgets IDs**: Widgets with `.id()` can be targeted by accessibility events

## API Reference

### Core Types (from `icy_ui::accessibility`)

| Type | Description |
|------|-------------|
| `Mode` | Accessibility mode (`Inactive` or `Active`) |
| `Focus` | Accessibility focus state (separate from widget focus) |
| `AccessibilityState` | Full state including mode, focus, announcements |
| `AnnouncementPriority` | Priority for announcements (`Polite`, `Assertive`) |
| `WidgetInfo` | Accessibility information for a widget |
| `Event` | Accessibility event from screen reader |
| `Action` | Action type (Click, Focus, SetValue, etc.) |
| `ActionData` | Optional data for actions (e.g., new value) |
| `NodeId` | Unique identifier for accessibility nodes |
| `Role` | Widget role (Button, CheckBox, Slider, etc.) |

### Runtime Functions (from `icy_ui::accessibility`)

| Function | Description |
|----------|-------------|
| `announce(message, priority)` | Announce message to screen reader |
| `focus(node_id)` | Programmatically focus an accessible element |
| `node_id(u64)` | Create a NodeId from a numeric value |
| `node_id_from_widget_id(&Id)` | Convert widget ID to NodeId |
