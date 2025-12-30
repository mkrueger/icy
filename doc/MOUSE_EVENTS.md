# Mouse Events with Modifiers

Icy extends iced's mouse events by adding keyboard modifier information to all mouse events. This allows widgets to respond differently based on whether Shift, Ctrl, Alt, or other modifier keys are held during mouse interactions.

## Overview

All mouse events now include a `modifiers` field containing the current keyboard modifier state:

```rust
use iced::mouse;
use iced::keyboard::Modifiers;

match event {
    mouse::Event::ButtonPressed { button, modifiers } => {
        if modifiers.shift() {
            // Shift+click behavior
        } else if modifiers.control() {
            // Ctrl+click behavior
        } else {
            // Normal click
        }
    }
    mouse::Event::CursorMoved { position, modifiers } => {
        // Modifier-aware cursor tracking
    }
    mouse::Event::WheelScrolled { delta, modifiers } => {
        if modifiers.shift() {
            // Horizontal scroll
        } else if modifiers.control() {
            // Zoom
        }
    }
    _ => {}
}
```

## Event Types

All mouse events now carry modifier information:

| Event | Fields |
|-------|--------|
| `CursorMoved` | `position: Point`, `modifiers: Modifiers` |
| `ButtonPressed` | `button: Button`, `modifiers: Modifiers` |
| `ButtonReleased` | `button: Button`, `modifiers: Modifiers` |
| `WheelScrolled` | `delta: ScrollDelta`, `modifiers: Modifiers` |
| `CursorEntered` | _(no fields)_ |
| `CursorLeft` | _(no fields)_ |

## Modifiers

The `Modifiers` struct provides methods to check individual modifier keys:

```rust
modifiers.shift()    // Shift key
modifiers.control()  // Ctrl key (Cmd on macOS)
modifiers.alt()      // Alt key (Option on macOS)
modifiers.logo()     // Windows/Super/Cmd key
```

## Use Cases

### Shift+Click for Range Selection

```rust
mouse::Event::ButtonPressed { button: mouse::Button::Left, modifiers } => {
    if modifiers.shift() {
        // Extend selection from anchor to clicked position
        self.extend_selection(cursor_position);
    } else {
        // Start new selection
        self.start_selection(cursor_position);
    }
}
```

### Ctrl+Click for Multi-Selection

```rust
mouse::Event::ButtonPressed { button: mouse::Button::Left, modifiers } => {
    if modifiers.control() {
        // Toggle item in selection
        self.toggle_selection(item);
    } else {
        // Replace selection
        self.select_only(item);
    }
}
```

### Scroll Wheel Zoom

```rust
mouse::Event::WheelScrolled { delta, modifiers } => {
    if modifiers.control() {
        // Zoom in/out
        let zoom_delta = match delta {
            ScrollDelta::Lines { y, .. } => y * 0.1,
            ScrollDelta::Pixels { y, .. } => y * 0.001,
        };
        self.zoom += zoom_delta;
    } else if modifiers.shift() {
        // Horizontal scroll
        self.scroll_horizontal(delta);
    } else {
        // Normal vertical scroll
        self.scroll_vertical(delta);
    }
}
```

### Fine-Grained Slider Control

The built-in slider widget uses modifiers for precision control:

```rust
// In slider widget
mouse::Event::ButtonPressed { modifiers, .. } => {
    if modifiers.control() {
        // Fine adjustment mode - smaller increments
        self.step_size = self.base_step / 10.0;
    }
}
```

## Why This Change?

In stock iced, modifier information is only available through separate keyboard events. 
To handle this properly, you'd need to track modifier state manually - which was error prone and hard to do (ModifiersChanged didn't contain the correct modifier mask, only the modifier that got pressed):

```rust
// The old way (error-prone)
struct State {
    current_modifiers: Modifiers,
}

fn update(&mut self, event: Event) {
    match event {
        Event::Keyboard(keyboard::Event::ModifiersChanged(m)) => {
            self.current_modifiers = m;
        }
        Event::Mouse(mouse::Event::ButtonPressed(button)) => {
            // Hope current_modifiers is up-to-date...
            if self.current_modifiers.shift() { ... }
        }
    }
}
```

With icy's approach, the modifier state is **always available at the moment of the mouse event**, eliminating race conditions and simplifying widget code.
