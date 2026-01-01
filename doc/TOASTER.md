# Toaster (Toast Notifications)

The toaster widget provides temporary notification messages that appear at the bottom of the screen and automatically disappear after a configurable duration.

## Table of Contents

- [Basic Usage](#basic-usage)
- [Toast Notifications](#toast-notifications)
- [Toasts Collection](#toasts-collection)
- [Toaster Widget](#toaster-widget)
- [Handling Timeouts](#handling-timeouts)
- [Full Example](#full-example)

---

## Basic Usage

```rust
use icy_ui::widget::toaster::{self, Toast, Toasts};

#[derive(Debug, Clone)]
enum Message {
    ShowNotification,
    CloseToast(toaster::Id),
}

struct State {
    toasts: Toasts<Message>,
}

impl State {
    fn new() -> Self {
        Self {
            toasts: Toasts::new(Message::CloseToast),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ShowNotification => {
                self.toasts.push(Toast::new("Operation completed!"));
            }
            Message::CloseToast(id) => {
                self.toasts.remove(id);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        toaster::toaster(&self.toasts, your_app_content).into()
    }
}
```

---

## Toast Notifications

A `Toast` represents a single notification message.

### Creating Toasts

```rust
use icy_ui::widget::toaster::{Toast, Duration};

// Simple toast
let toast = Toast::new("File saved successfully");

// Toast with custom duration
let toast = Toast::new("Processing...")
    .duration(Duration::Long);

// Toast with action button
let toast = Toast::new("Message sent")
    .action("Undo", |id| Message::UndoSend(id));

// Toast with custom duration
use std::time::Duration as StdDuration;
let toast = Toast::new("Quick notification")
    .duration(StdDuration::from_secs(3));
```

### Toast Methods

| Method | Description |
|--------|-------------|
| `Toast::new(message)` | Create a new toast with the given message |
| `.duration(Duration)` | Set how long the toast is displayed |
| `.action(label, fn)` | Add an action button to the toast |

### Duration Options

| Variant | Duration |
|---------|----------|
| `Duration::Short` (default) | 5 seconds |
| `Duration::Long` | 15 seconds |
| `Duration::Custom(std::time::Duration)` | Custom duration |

```rust
use icy_ui::widget::toaster::Duration;
use std::time::Duration as StdDuration;

let short = Duration::Short;
let long = Duration::Long;
let custom = Duration::Custom(StdDuration::from_secs(10));

// You can also convert from std::time::Duration
let custom: Duration = StdDuration::from_secs(8).into();
```

---

## Toasts Collection

The `Toasts` struct manages a collection of active toasts.

### Creating a Toasts Collection

```rust
use icy_ui::widget::toaster::Toasts;

// The on_close function is called when a toast should be removed
let toasts: Toasts<Message> = Toasts::new(Message::CloseToast);

// Optionally limit the number of visible toasts
let toasts = Toasts::new(Message::CloseToast).limit(3);
```

### Toasts Methods

| Method | Description |
|--------|-------------|
| `Toasts::new(on_close)` | Create a new collection with a close handler |
| `.limit(n)` | Set maximum visible toasts (default: 5) |
| `.push(toast)` | Add a toast, returns its `Id` |
| `.remove(id)` | Remove a toast by ID |
| `.is_empty()` | Check if there are any toasts |
| `.len()` | Get the number of active toasts |
| `.expired()` | Get IDs of toasts that have timed out |

### Toast ID

Each toast has a unique `Id` that can be used to:
- Remove the toast manually
- Identify which toast triggered an action

```rust
// Push returns the toast's ID
let id = toasts.push(Toast::new("Hello"));

// Remove by ID later
toasts.remove(id);
```

---

## Toaster Widget

The `toaster` function creates a widget that wraps your application content and displays toasts as an overlay.

```rust
use icy_ui::widget::toaster;

fn view(&self) -> Element<'_, Message> {
    let content = column![
        text("My Application"),
        button("Show Toast").on_press(Message::ShowToast),
    ];

    // Wrap your content with the toaster
    toaster::toaster(&self.toasts, content).into()
}
```

The toasts appear at the bottom-center of the wrapped content area.

---

## Handling Timeouts

Toasts need to be removed when they expire. There are several approaches:

### Approach 1: Subscription with Timer

```rust
use icy_ui::time;
use std::time::Duration;

impl State {
    fn subscription(&self) -> icy_ui::Subscription<Message> {
        if self.toasts.is_empty() {
            icy_ui::Subscription::none()
        } else {
            // Check every second for expired toasts
            time::every(Duration::from_secs(1)).map(|_| Message::CheckExpiredToasts)
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CheckExpiredToasts => {
                for id in self.toasts.expired() {
                    self.toasts.remove(id);
                }
            }
            // ... other messages
        }
        Task::none()
    }
}
```

### Approach 2: Manual Removal on Action

For simpler cases, just remove toasts when the user clicks the close button or an action:

```rust
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::CloseToast(id) => {
            self.toasts.remove(id);
        }
        Message::UndoAction(id) => {
            // Perform undo logic
            self.toasts.remove(id);
        }
    }
    Task::none()
}
```

---

## Full Example

```rust
use icy_ui::widget::{button, column, container, text, toaster};
use icy_ui::{Element, Length, Subscription, Task};
use std::time::Duration;

pub fn main() -> icy_ui::Result {
    icy_ui::application(State::default, State::update, State::view)
        .subscription(State::subscription)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    ShowSuccess,
    ShowError,
    ShowWithAction,
    CloseToast(toaster::Id),
    UndoAction(toaster::Id),
    CheckExpired,
}

struct State {
    toasts: toaster::Toasts<Message>,
    counter: u32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            toasts: toaster::Toasts::new(Message::CloseToast).limit(5),
            counter: 0,
        }
    }
}

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShowSuccess => {
                self.counter += 1;
                self.toasts.push(
                    toaster::Toast::new(format!("Success #{}!", self.counter))
                        .duration(toaster::Duration::Short),
                );
            }
            Message::ShowError => {
                self.toasts.push(
                    toaster::Toast::new("An error occurred")
                        .duration(toaster::Duration::Long),
                );
            }
            Message::ShowWithAction => {
                self.toasts.push(
                    toaster::Toast::new("File deleted")
                        .action("Undo", Message::UndoAction),
                );
            }
            Message::CloseToast(id) => {
                self.toasts.remove(id);
            }
            Message::UndoAction(id) => {
                // Handle undo logic here
                self.toasts.remove(id);
                self.toasts.push(toaster::Toast::new("Action undone"));
            }
            Message::CheckExpired => {
                for id in self.toasts.expired() {
                    self.toasts.remove(id);
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let content = container(
            column![
                text("Toast Notifications Demo").size(24),
                button("Show Success").on_press(Message::ShowSuccess),
                button("Show Error").on_press(Message::ShowError),
                button("Show with Undo").on_press(Message::ShowWithAction),
            ]
            .spacing(12),
        )
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill);

        toaster::toaster(&self.toasts, content).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.toasts.is_empty() {
            Subscription::none()
        } else {
            icy_ui::time::every(Duration::from_secs(1)).map(|_| Message::CheckExpired)
        }
    }
}
```

---

## Visual Layout

Toasts appear as a stack at the bottom-center of the toaster area:

```
┌────────────────────────────────────────────┐
│                                            │
│           Your Application                 │
│              Content                       │
│                                            │
│                                            │
├────────────────────────────────────────────┤
│  ┌──────────────────────────────────────┐  │
│  │ Message sent                  [Undo] │  │  ← Toast with action
│  └──────────────────────────────────────┘  │
│  ┌──────────────────────────────────────┐  │
│  │ File saved successfully          [×] │  │  ← Toast with close button
│  └──────────────────────────────────────┘  │
└────────────────────────────────────────────┘
```

---

## Notes

- Toasts are displayed in order they were added (oldest at top)
- When the limit is reached, the oldest toast is automatically removed
- Each toast has a unique `Id` for identification
- The close button (×) is always shown on each toast
- Action buttons appear next to the close button
- Toast styling follows the current theme
