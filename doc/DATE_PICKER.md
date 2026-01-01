# Date Picker

The date picker widget provides a calendar-style interface for selecting dates.

## Table of Contents

- [Basic Usage](#basic-usage)
- [Date Struct](#date-struct)
- [DatePicker Widget](#datepicker-widget)
- [First Day of Week](#first-day-of-week)
- [Full Example](#full-example)

---

## Basic Usage

```rust
use icy_ui::widget::date_picker::{self, Date, DatePicker};

#[derive(Debug, Clone)]
enum Message {
    DateSelected(Date),
    PrevMonth,
    NextMonth,
}

fn view(selected: Date, visible: Date) -> Element<'_, Message> {
    date_picker::date_picker(
        selected,              // Currently selected date
        visible,               // Currently visible month
        Message::DateSelected, // fn(Date) -> Message
        Message::PrevMonth,    // Message for previous month button
        Message::NextMonth,    // Message for next month button
    )
    .into()
}
```

---

## Date Struct

A simple date structure representing year, month, and day.

### Creating Dates

```rust
use icy_ui::widget::date_picker::Date;

// Create a specific date
let date = Date::new(2025, 12, 31);

// Get today's date (returns a default/fallback date)
let today = Date::today();

// Default is the same as today()
let date = Date::default();
```

### Date Fields

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `year` | `i32` | any | The year (e.g., 2025) |
| `month` | `u32` | 1-12 | The month (January=1, December=12) |
| `day` | `u32` | 1-31 | The day of the month |

### Date Methods

| Method | Description |
|--------|-------------|
| `Date::new(year, month, day)` | Create a new date |
| `Date::today()` | Get today's date (fallback date) |
| `.weekday()` | Get weekday (0=Monday, 6=Sunday) |
| `.days_in_month()` | Number of days in the current month |
| `.is_leap_year()` | Check if the year is a leap year |
| `.first_of_month()` | Get the first day of this month |
| `.prev_month()` | Get the same day in the previous month |
| `.next_month()` | Get the same day in the next month |
| `.month_name()` | Get the month name as a string |

### Display

The `Date` struct implements `Display`:

```rust
let date = Date::new(2025, 12, 31);
println!("{}", date); // Output: "2025-12-31"
```

---

## DatePicker Widget

The date picker displays a calendar grid with:
- **Header**: Month name, year, and navigation arrows
- **Weekday row**: Day names (Mon-Sun or Sun-Sat)
- **Calendar grid**: 6 rows × 7 columns of day buttons

### Constructor

```rust
use icy_ui::widget::date_picker;

let picker = date_picker::date_picker(
    selected_date,           // The currently selected Date
    visible_date,            // The month currently being displayed
    Message::DateSelected,   // fn(Date) -> Message when a date is clicked
    Message::PrevMonth,      // Message for the ◀ button
    Message::NextMonth,      // Message for the ▶ button
);
```

### Methods

| Method | Description |
|--------|-------------|
| `.first_day_of_week(FirstDayOfWeek)` | Set whether week starts on Sunday or Monday |
| `.width(Length)` | Set the widget width (default: 280px) |

---

## First Day of Week

Configure whether the calendar week starts on Sunday or Monday:

```rust
use icy_ui::widget::date_picker::{self, FirstDayOfWeek};

let picker = date_picker::date_picker(selected, visible, on_select, on_prev, on_next)
    .first_day_of_week(FirstDayOfWeek::Sunday);
```

| Value | Week Order |
|-------|------------|
| `FirstDayOfWeek::Monday` (default) | Mon, Tue, Wed, Thu, Fri, Sat, Sun |
| `FirstDayOfWeek::Sunday` | Sun, Mon, Tue, Wed, Thu, Fri, Sat |

---

## Full Example

```rust
use icy_ui::widget::{button, column, container, date_picker, row, text};
use icy_ui::{Element, Length, Task};

#[derive(Debug, Clone)]
enum Message {
    TogglePicker,
    DateSelected(date_picker::Date),
    PrevMonth,
    NextMonth,
}

struct State {
    selected: date_picker::Date,
    visible: date_picker::Date,
    picker_open: bool,
}

impl Default for State {
    fn default() -> Self {
        let today = date_picker::Date::new(2025, 1, 15);
        Self {
            selected: today,
            visible: today,
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
            Message::DateSelected(date) => {
                self.selected = date;
                self.visible = date; // Navigate to selected month
            }
            Message::PrevMonth => {
                self.visible = self.visible.prev_month();
            }
            Message::NextMonth => {
                self.visible = self.visible.next_month();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let date_str = format!("{}", self.selected);

        let mut content = column![
            row![
                text("Selected date:"),
                text(date_str).size(16),
            ]
            .spacing(10),
            button(if self.picker_open { "Close" } else { "Pick Date" })
                .on_press(Message::TogglePicker),
        ]
        .spacing(12);

        if self.picker_open {
            content = content.push(
                date_picker::date_picker(
                    self.selected,
                    self.visible,
                    Message::DateSelected,
                    Message::PrevMonth,
                    Message::NextMonth,
                )
                .first_day_of_week(date_picker::FirstDayOfWeek::Monday)
                .width(Length::Fixed(300.0)),
            );
        }

        container(content).padding(20).into()
    }
}
```

---

## Visual Layout

```
┌────────────────────────────────────┐
│  January 2025           ◀    ▶    │  ← Header with navigation
├────────────────────────────────────┤
│  Mon  Tue  Wed  Thu  Fri  Sat  Sun │  ← Weekday headers
├────────────────────────────────────┤
│   30   31    1    2    3    4    5 │  ← Previous month days (grayed)
│    6    7    8    9   10   11   12 │
│   13   14  [15]  16   17   18   19 │  ← [15] = selected date
│   20   21   22   23   24   25   26 │
│   27   28   29   30   31    1    2 │  ← Next month days (grayed)
│    3    4    5    6    7    8    9 │
└────────────────────────────────────┘
```

---

## Notes

- The calendar always shows 6 weeks (42 days) to maintain consistent height
- Days from adjacent months are shown grayed out and non-interactive
- The selected date is highlighted with the primary button style
- Month navigation wraps the year automatically (Dec → Jan = year + 1)
- For real-time "today" detection, use the `chrono` crate in your application:

```rust
// In your application code:
use chrono::{Datelike, Local};

fn get_today() -> date_picker::Date {
    let now = Local::now();
    date_picker::Date::new(now.year(), now.month(), now.day())
}
```
