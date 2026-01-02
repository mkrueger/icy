# Color Picker

The color picker widget provides an interactive way for users to select colors using an HSV (Hue, Saturation, Value) color model.

## Table of Contents

- [Basic Usage](#basic-usage)
- [HSV Color Space](#hsv-color-space)
- [ColorPicker Widget](#colorpicker-widget)
- [Color Button Helper](#color-button-helper)
- [Full Example](#full-example)

---

## Basic Usage

```rust
use icy_ui::widget::color_picker::{self, ColorPicker};
use icy_ui::Color;

#[derive(Debug, Clone)]
enum Message {
    ColorChanged(Color),
}

fn view(selected_color: Color) -> Element<'_, Message> {
    color_picker::color_picker(selected_color, Message::ColorChanged).into()
}
```

---

## HSV Color Space

The color picker uses HSV (Hue, Saturation, Value) color space internally, which provides a more intuitive way to select colors compared to RGB.

### Hsv Struct

```rust
use icy_ui::widget::color_picker::Hsv;

// Create from components
let hsv = Hsv::new(180.0, 0.8, 0.9); // Cyan-ish color

// Convert to RGB Color
let color = hsv.to_color();

// Create from RGB Color
let hsv = Hsv::from_color(Color::from_rgb(0.2, 0.6, 0.8));
```

### Hsv Fields

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `hue` | `f32` | 0-360 | The color's hue in degrees (red=0, green=120, blue=240) |
| `saturation` | `f32` | 0-1 | Color saturation (0=gray, 1=full color) |
| `value` | `f32` | 0-1 | Brightness (0=black, 1=full brightness) |

### Hsv Methods

| Method | Description |
|--------|-------------|
| `Hsv::new(h, s, v)` | Create a new HSV color |
| `Hsv::default()` | Returns red (hue=0, sat=1, val=1) |
| `.to_color()` | Convert to `icy_ui::Color` (RGB) |
| `Hsv::from_color(color)` | Create from `icy_ui::Color` |

---

## ColorPicker Widget

The main color picker widget displays:
- A **saturation-value area** (rectangular gradient)
- A **hue slider** (rainbow bar)
- A **color preview** box
- A **hex value** display

### Constructor

```rust
use icy_ui::widget::color_picker;

let picker = color_picker::color_picker(
    current_color,           // Current Color value
    Message::ColorChanged,   // fn(Color) -> Message
);
```

### Methods

| Method | Description |
|--------|-------------|
| `.width(Length)` | Set the overall width (default: 250px) |
| `.height(f32)` | Set the height of the SV area in pixels (default: 150px) |

### Example with Customization

```rust
use icy_ui::widget::color_picker;
use icy_ui::Length;

let picker = color_picker::color_picker(color, Message::ColorChanged)
    .width(Length::Fixed(300.0))
    .height(200.0);
```

---

## Color Button Helper

A convenience function to create a button that displays a color swatch. Useful for showing the currently selected color and triggering a color picker.

```rust
use icy_ui::widget::color_picker;

// Create a clickable color button
let btn = color_picker::color_button(selected_color, Some(Message::OpenPicker));

// Create a display-only color button (no interaction)
let btn = color_picker::color_button(selected_color, None);
```

---

## Full Example

```rust
use icy_ui::widget::{button, column, color_picker, container, row, text};
use icy_ui::{Color, Element, Length, Task};

#[derive(Debug, Clone)]
enum Message {
    TogglePicker,
    ColorChanged(Color),
}

struct State {
    color: Color,
    picker_open: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            color: Color::from_rgb(0.2, 0.6, 0.9),
            picker_open: false,
        }
    }
}

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TogglePicker => {
                self.picker_open = !self.picker_open;
            }
            Message::ColorChanged(color) => {
                self.color = color;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let hex = format!(
            "#{:02X}{:02X}{:02X}",
            (self.color.r * 255.0) as u8,
            (self.color.g * 255.0) as u8,
            (self.color.b * 255.0) as u8
        );

        let preview = color_picker::color_button(self.color, Some(Message::TogglePicker));

        let mut content = column![
            row![text("Selected:"), preview, text(hex)].spacing(10),
            button(if self.picker_open { "Close" } else { "Pick Color" })
                .on_press(Message::TogglePicker),
        ]
        .spacing(12);

        if self.picker_open {
            content = content.push(
                color_picker::color_picker(self.color, Message::ColorChanged)
                    .width(Length::Fixed(280.0))
                    .height(180.0),
            );
        }

        container(content).padding(20).into()
    }
}
```

---

## Visual Layout

The color picker is laid out vertically:

```
┌──────────────────────────────────┐
│                                  │
│    Saturation-Value Area         │  ← Click/drag to select S and V
│    (gradient from hue color      │
│     to white to black)           │
│                                  │
├──────────────────────────────────┤
│  ████████████████████████████    │  ← Hue slider (rainbow)
├──────────────────────────────────┤
│  ████████████████████████████    │  ← Color preview
├──────────────────────────────────┤
│          #3399CC                 │  ← Hex value
└──────────────────────────────────┘
```

---

## Notes

- The color picker works with `icy_ui::Color` (RGB with alpha) but internally uses HSV for the UI
- Alpha channel is not currently supported in the picker UI
- The hex display shows RGB without alpha
- Drag interactions work on both the SV area and hue slider
