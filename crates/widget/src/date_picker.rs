//! A widget that displays an interactive date picker / calendar.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use icy_ui_widget::*; } pub use icy_ui_widget::core::*; }
//! # pub type Element<'a, Message> = icy_ui_widget::core::Element<'a, Message, icy_ui_widget::Theme, icy_ui_widget::Renderer>;
//! use icy_ui::widget::date_picker::{self, DatePicker};
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     DateSelected(date_picker::Date),
//!     PrevMonth,
//!     NextMonth,
//! }
//!
//! struct State {
//!     selected_date: date_picker::Date,
//!     visible_date: date_picker::Date,
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     date_picker::date_picker(
//!         state.selected_date,
//!         state.visible_date,
//!         Message::DateSelected,
//!         Message::PrevMonth,
//!         Message::NextMonth,
//!     ).into()
//! }
//! ```

use crate::button;
use crate::container;
use crate::core::alignment::Horizontal;
use crate::core::{Alignment, Element, Length, Padding};
use crate::{text, Column, Row};

/// A simple date representation (year, month, day).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Date {
    /// The year.
    pub year: i32,
    /// The month (1-12).
    pub month: u32,
    /// The day of month (1-31).
    pub day: u32,
}

impl Date {
    /// Create a new date.
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    /// Get today's date.
    /// 
    /// Note: Returns a fixed fallback date. For actual current date,
    /// consider using the `chrono` crate in your application code.
    pub fn today() -> Self {
        Self {
            year: 2025,
            month: 1,
            day: 1,
        }
    }

    /// Get the weekday (0 = Monday, 6 = Sunday).
    pub fn weekday(&self) -> u32 {
        // Zeller's congruence for Gregorian calendar
        let mut y = self.year;
        let mut m = self.month as i32;

        if m < 3 {
            m += 12;
            y -= 1;
        }

        let q = self.day as i32;
        let k = y % 100;
        let j = y / 100;

        let h = (q + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 - 2 * j) % 7;
        // Convert from Zeller (0=Sat, 1=Sun, ..., 6=Fri) to (0=Mon, ..., 6=Sun)
        ((h + 5) % 7) as u32
    }

    /// Get the number of days in the month.
    pub fn days_in_month(&self) -> u32 {
        match self.month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if self.is_leap_year() {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    /// Check if the year is a leap year.
    pub fn is_leap_year(&self) -> bool {
        (self.year % 4 == 0 && self.year % 100 != 0) || (self.year % 400 == 0)
    }

    /// Get the previous month's date.
    pub fn prev_month(&self) -> Self {
        if self.month == 1 {
            Self {
                year: self.year - 1,
                month: 12,
                day: 1,
            }
        } else {
            Self {
                year: self.year,
                month: self.month - 1,
                day: 1,
            }
        }
    }

    /// Get the next month's date.
    pub fn next_month(&self) -> Self {
        if self.month == 12 {
            Self {
                year: self.year + 1,
                month: 1,
                day: 1,
            }
        } else {
            Self {
                year: self.year,
                month: self.month + 1,
                day: 1,
            }
        }
    }

    /// Get the first day of the month.
    pub fn first_of_month(&self) -> Self {
        Self {
            year: self.year,
            month: self.month,
            day: 1,
        }
    }

    /// Get the month name.
    pub fn month_name(&self) -> &'static str {
        match self.month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown",
        }
    }
}

impl Default for Date {
    fn default() -> Self {
        Self::today()
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// The first day of the week.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FirstDayOfWeek {
    /// Week starts on Sunday.
    Sunday,
    /// Week starts on Monday.
    #[default]
    Monday,
}

/// A date picker widget.
pub struct DatePicker<'a, Message> {
    selected: Date,
    visible: Date,
    on_select: Box<dyn Fn(Date) -> Message + 'a>,
    on_prev: Message,
    on_next: Message,
    first_day_of_week: FirstDayOfWeek,
    width: Length,
}

/// Create a new date picker widget.
pub fn date_picker<'a, Message: Clone + 'static>(
    selected: Date,
    visible: Date,
    on_select: impl Fn(Date) -> Message + 'a,
    on_prev: Message,
    on_next: Message,
) -> DatePicker<'a, Message> {
    DatePicker {
        selected,
        visible,
        on_select: Box::new(on_select),
        on_prev,
        on_next,
        first_day_of_week: FirstDayOfWeek::default(),
        width: Length::Fixed(280.0),
    }
}

impl<'a, Message: Clone + 'static> DatePicker<'a, Message> {
    /// Set the first day of the week.
    #[must_use]
    pub fn first_day_of_week(mut self, first_day: FirstDayOfWeek) -> Self {
        self.first_day_of_week = first_day;
        self
    }

    /// Set the width of the date picker.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

impl<'a, Message: Clone + 'static> From<DatePicker<'a, Message>>
    for Element<'a, Message, crate::Theme, crate::Renderer>
{
    fn from(picker: DatePicker<'a, Message>) -> Self {
        let weekday_names = match picker.first_day_of_week {
            FirstDayOfWeek::Monday => ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
            FirstDayOfWeek::Sunday => ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
        };

        // Header with month/year and navigation
        let header = Row::new()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(
                text(format!(
                    "{} {}",
                    picker.visible.month_name(),
                    picker.visible.year
                ))
                .size(16),
            )
            .push(crate::Space::new().width(Length::Fill))
            .push(
                button::Button::new(text("◀"))
                    .on_press(picker.on_prev.clone())
                    .padding(Padding::new(4.0).left(8.0).right(8.0)),
            )
            .push(
                button::Button::new(text("▶"))
                    .on_press(picker.on_next.clone())
                    .padding(Padding::new(4.0).left(8.0).right(8.0)),
            );

        // Weekday header row
        let weekday_row = Row::with_children(
            weekday_names
                .iter()
                .map(|name| {
                    text(*name)
                        .size(12)
                        .width(Length::Fixed(36.0))
                        .align_x(Horizontal::Center)
                        .into()
                })
                .collect::<Vec<_>>(),
        )
        .spacing(2);

        // Calculate the first day to display
        let first_of_month = picker.visible.first_of_month();
        let first_weekday = first_of_month.weekday();

        // Adjust for first day of week
        let offset = match picker.first_day_of_week {
            FirstDayOfWeek::Monday => first_weekday,
            FirstDayOfWeek::Sunday => (first_weekday + 1) % 7,
        };

        // Build the calendar grid (6 rows x 7 columns = 42 cells)
        let mut rows: Vec<Element<'a, Message, crate::Theme, crate::Renderer>> = Vec::new();
        let days_in_month = picker.visible.days_in_month();
        let prev_month = picker.visible.prev_month();
        let days_in_prev_month = prev_month.days_in_month();

        for week in 0..6 {
            let mut week_cells: Vec<Element<'a, Message, crate::Theme, crate::Renderer>> =
                Vec::new();

            for day_of_week in 0..7 {
                let cell_index = week * 7 + day_of_week;

                let (day_num, is_current_month, date) = if cell_index < offset {
                    // Previous month
                    let d = days_in_prev_month - (offset - cell_index - 1);
                    (
                        d,
                        false,
                        Date::new(prev_month.year, prev_month.month, d),
                    )
                } else if cell_index - offset < days_in_month {
                    // Current month
                    let d = cell_index - offset + 1;
                    (d, true, Date::new(picker.visible.year, picker.visible.month, d))
                } else {
                    // Next month
                    let next_month = picker.visible.next_month();
                    let d = cell_index - offset - days_in_month + 1;
                    (
                        d,
                        false,
                        Date::new(next_month.year, next_month.month, d),
                    )
                };

                let is_selected = date == picker.selected;

                let btn = if is_current_month {
                    let date_to_select = date;
                    let on_select = &picker.on_select;
                    let msg = on_select(date_to_select);

                    if is_selected {
                        button::Button::new(
                            text(day_num.to_string())
                                .width(Length::Fill)
                                .align_x(Horizontal::Center),
                        )
                        .on_press(msg)
                        .width(Length::Fixed(36.0))
                        .padding(8)
                        .style(button::primary)
                    } else {
                        button::Button::new(
                            text(day_num.to_string())
                                .width(Length::Fill)
                                .align_x(Horizontal::Center),
                        )
                        .on_press(msg)
                        .width(Length::Fixed(36.0))
                        .padding(8)
                        .style(button::text_style)
                    }
                } else {
                    // Disabled style for other months
                    button::Button::new(
                        text(day_num.to_string())
                            .width(Length::Fill)
                            .align_x(Horizontal::Center)
                            .color(crate::core::Color::from_rgba(0.5, 0.5, 0.5, 0.5)),
                    )
                    .width(Length::Fixed(36.0))
                    .padding(8)
                    .style(button::text_style)
                };

                week_cells.push(btn.into());
            }

            rows.push(Row::with_children(week_cells).spacing(2).into());
        }

        let calendar_grid = Column::with_children(rows).spacing(2);

        // Combine everything
        let content = Column::new()
            .spacing(8)
            .padding(12)
            .width(picker.width)
            .push(header)
            .push(weekday_row)
            .push(calendar_grid);

        container::Container::new(content)
            .style(container::rounded_box)
            .into()
    }
}
