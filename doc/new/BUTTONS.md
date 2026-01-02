# Buttons

This document covers the button widgets and related controls available in icy.

## Table of Contents

- [Core Button](#core-button)
- [Button Variants](#button-variants)
  - [Text Button](#text-button)
  - [Icon Button](#icon-button)
  - [Link Button](#link-button)
  - [Image Button](#image-button)
- [Button Status](#button-status)
- [Button Styles](#button-styles)
- [Spin Button](#spin-button)

---

## Core Button

The base `Button` widget supports all standard interactions plus additional features:

```rust
use icy_ui::widget::button;

// Basic button
let btn = button("Click me").on_press(Message::Click);

// With on_press_down for immediate response
let btn = button("Hold me")
    .on_press_down(Message::PressDown)
    .on_press(Message::Released);

// Selected state (toggle button pattern)
let btn = button("Toggle")
    .selected(is_selected)
    .on_press(Message::Toggle);
```

### Core Button Methods

| Method | Description |
|--------|-------------|
| `.on_press(msg)` | Message sent when button is clicked (mouse up) |
| `.on_press_maybe(Option<msg>)` | Conditional on_press |
| `.on_press_down(msg)` | Message sent immediately on mouse down |
| `.selected(bool)` | Sets selected state, changes status to `Status::Selected` |
| `.width(Length)` | Sets button width |
| `.height(Length)` | Sets button height |
| `.padding(Padding)` | Sets internal padding |
| `.clip(bool)` | Clips content to button bounds |
| `.style(fn)` | Sets custom style function |

---

## Button Variants

### Text Button

Text buttons display a label with optional leading and trailing icons. Good for actions in toolbars, dialogs, and forms.

```rust
use icy_ui::widget::button::text_button;

// Simple text button
let save = text_button("Save").on_press(Message::Save);

// With leading icon
let download = text_button("Download")
    .leading_icon(download_icon)
    .on_press(Message::Download);

// With trailing icon
let next = text_button("Next")
    .trailing_icon(arrow_right)
    .on_press(Message::Next);

// With both icons
let btn = text_button("Open File")
    .leading_icon(folder_icon)
    .trailing_icon(chevron_icon)
    .on_press(Message::OpenFile);

// Styled variants
let primary = text_button("Submit").primary().on_press(Message::Submit);
let danger = text_button("Delete").danger().on_press(Message::Delete);
```

#### Text Button Methods

| Method | Description |
|--------|-------------|
| `.leading_icon(element)` | Icon displayed before label |
| `.trailing_icon(element)` | Icon displayed after label |
| `.spacing(f32)` | Space between icon and text (default: 8.0) |
| `.selected(bool)` | Toggle selected state |
| `.primary()` | Accent/primary style |
| `.success()` | Success/green style |
| `.warning()` | Warning/yellow style |
| `.danger()` | Danger/red style |
| `.text_style()` | Text-only, no background |

---

### Icon Button

Icon buttons display only an icon, ideal for toolbars and compact UIs.

```rust
use icy_ui::widget::button::icon_button;

// Simple icon button
let settings = icon_button(settings_icon).on_press(Message::Settings);

// Styled variants
let primary = icon_button(add_icon).primary().on_press(Message::Add);
let danger = icon_button(trash_icon).danger().on_press(Message::Delete);

// Selected state (for toggle buttons)
let bold = icon_button(bold_icon)
    .selected(is_bold)
    .on_press(Message::ToggleBold);
```

#### Icon Button Methods

| Method | Description |
|--------|-------------|
| `.padding(Padding)` | Padding around icon (default: 8.0) |
| `.selected(bool)` | Toggle selected state |
| `.primary()` | Accent/primary style |
| `.secondary()` | Secondary style |
| `.success()` | Success/green style |
| `.warning()` | Warning/yellow style |
| `.danger()` | Danger/red style |

---

### Link Button

Link buttons look like hyperlinks - minimal styling with just text color changes on hover.

```rust
use icy_ui::widget::button::link_button;

// Simple link
let docs = link_button("Read documentation")
    .on_press(Message::OpenDocs);

// With icon
let external = link_button("Visit website")
    .trailing_icon(external_link_icon)
    .on_press(Message::OpenWebsite);
```

#### Link Button Methods

| Method | Description |
|--------|-------------|
| `.leading_icon(element)` | Icon before text |
| `.trailing_icon(element)` | Icon after text |
| `.spacing(f32)` | Space between icon and text (default: 4.0) |
| `.padding(Padding)` | Padding (default: none) |

---

### Image Button

Image buttons display an image with optional selection and remove functionality. Requires the `image` feature.

```rust
use icy_ui::widget::button::image_button;

// Basic image button
let thumb = image_button(handle)
    .on_press(Message::SelectImage(id));

// With selection state
let thumb = image_button(handle)
    .selected(is_selected)
    .on_press(Message::SelectImage(id));

// With remove button
let thumb = image_button(handle)
    .on_remove(Message::RemoveImage(id))
    .on_press(Message::SelectImage(id));
```

#### Image Button Methods

| Method | Description |
|--------|-------------|
| `.selected(bool)` | Shows selection indicator |
| `.on_remove(msg)` | Adds remove button overlay |
| `.width(Length)` | Image width |
| `.height(Length)` | Image height |
| `.content_fit(ContentFit)` | How image fits in bounds |
| `.filter_method(FilterMethod)` | Image scaling filter |

---

## Button Status

Buttons have a `Status` enum that describes their current interaction state:

```rust
pub enum Status {
    Active,     // Default state
    Hovered,    // Mouse over button
    Pressed,    // Mouse button held down
    Disabled,   // No on_press handler set
    Selected,   // Button is in selected state
}
```

The `Selected` status is activated when `.selected(true)` is set on a button. This is useful for toggle buttons, tab bars, and toolbar buttons that show an "active" state.

---

## Button Styles

Built-in style functions available for all button variants:

| Style Function | Description |
|----------------|-------------|
| `button::primary` | Accent-colored, high emphasis |
| `button::secondary` | Subtle background, medium emphasis |
| `button::success` | Green/success colored |
| `button::warning` | Yellow/warning colored |
| `button::danger` | Red/destructive colored |
| `button::text_style` | No background, text only |
| `button::link` | Hyperlink style |
| `button::icon` | Minimal style for icon buttons |

### Custom Styles

```rust
use icy_ui::widget::button::{self, Status, Style};

fn custom_style(theme: &Theme, status: Status) -> Style {
    match status {
        Status::Active => Style {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.8))),
            text_color: Color::WHITE,
            border: Border::default().rounded(8),
            ..Style::default()
        },
        Status::Selected => Style {
            background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.6))),
            text_color: Color::WHITE,
            border: Border::default().rounded(8).width(2).color(Color::WHITE),
            ..Style::default()
        },
        // ... other states
    }
}

let btn = button("Custom").style(custom_style);
```

### Style with Outline (Focus Indicator)

The `Style` struct includes an `outline` field for focus indicators:

```rust
Style {
    outline: Some(Border {
        color: theme.palette().primary,
        width: 2.0,
        radius: 4.0.into(),
    }),
    ..Style::default()
}
```

---

## Spin Button

Spin buttons allow users to increment or decrement a numeric value with `+` and `−` buttons.

```rust
use icy_ui::widget::spin_button;

// Horizontal spin button (default)
let spinner = spin_button(
    value.to_string(),  // Label (formatted value)
    value,              // Current value
    1,                  // Step
    0,                  // Minimum
    100,                // Maximum
    Message::ValueChanged,
);

// Vertical spin button
let vertical = spin_button::vertical(
    value.to_string(),
    value,
    1,
    0,
    100,
    Message::ValueChanged,
);
```

### Layout

**Horizontal:** `[−] value [+]`

**Vertical:**
```
[+]
value
[−]
```

### Generic Over Numeric Types

Spin button works with any type implementing `Copy + Add + Sub + PartialOrd`:

```rust
// Integer types
let i32_spin = spin_button("0", 0i32, 1, -10, 10, Msg::I32);
let i64_spin = spin_button("0", 0i64, 10, 0, 1000, Msg::I64);

// Floating point
let f32_spin = spin_button("0.0", 0.0f32, 0.1, 0.0, 1.0, Msg::F32);

// Custom types (if they implement the required traits)
let custom = spin_button("0.00", Decimal::ZERO, Decimal::from(0.25), min, max, Msg::Dec);
```

### Spin Button Methods

| Method | Description |
|--------|-------------|
| `.label_width(f32)` | Width of the value display area (default: 48.0) |
| `.padding(Padding)` | Padding around the entire widget |

### Behavior

- **Clamping:** Values are automatically clamped to the `[min, max]` range
- **Disabled at limits:** The `−` button is disabled at minimum, `+` at maximum
- **Styled container:** Uses `rounded_box` container style by default

### Example: Complete Usage

```rust
struct State {
    quantity: i32,
}

#[derive(Clone)]
enum Message {
    QuantityChanged(i32),
}

fn view(state: &State) -> Element<Message> {
    spin_button(
        state.quantity.to_string(),
        state.quantity,
        1,    // step
        1,    // min (at least 1)
        99,   // max
        Message::QuantityChanged,
    )
    .label_width(60.0)
    .padding(4)
    .into()
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::QuantityChanged(qty) => {
            state.quantity = qty;
        }
    }
}
```
