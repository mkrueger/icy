//! Example demonstrating the color picker and date picker widgets.

use icy_ui::widget::{button, color_picker, column, container, date_picker, row, space, text};
use icy_ui::{Color, Element, Length, Task};

pub fn main() -> icy_ui::Result {
    icy_ui::application(State::default, State::update, State::view)
        .title("Picker Widgets Demo")
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ActivePicker {
    #[default]
    None,
    Color,
    Date,
}

#[derive(Debug, Clone)]
enum Message {
    TogglePicker(ActivePicker),
    ColorChanged(Color),
    DateSelected(date_picker::Date),
    PrevMonth,
    NextMonth,
}

struct State {
    active_picker: ActivePicker,
    selected_color: Color,
    selected_date: date_picker::Date,
    visible_date: date_picker::Date,
}

impl Default for State {
    fn default() -> Self {
        let today = date_picker::Date::new(2025, 1, 15);
        Self {
            active_picker: ActivePicker::None,
            selected_color: Color::from_rgb(0.2, 0.6, 0.9),
            selected_date: today,
            visible_date: today,
        }
    }
}

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TogglePicker(picker) => {
                if self.active_picker == picker {
                    self.active_picker = ActivePicker::None;
                } else {
                    self.active_picker = picker;
                }
            }
            Message::ColorChanged(color) => {
                self.selected_color = color;
            }
            Message::DateSelected(date) => {
                self.selected_date = date;
                self.visible_date = date;
            }
            Message::PrevMonth => {
                self.visible_date = self.visible_date.prev_month();
            }
            Message::NextMonth => {
                self.visible_date = self.visible_date.next_month();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let color_section = {
            let preview = container(text(""))
                .width(Length::Fixed(40.0))
                .height(Length::Fixed(40.0))
                .style(move |_theme| container::Style {
                    background: Some(icy_ui::Background::Color(self.selected_color)),
                    border: icy_ui::Border {
                        color: Color::from_rgb(0.3, 0.3, 0.3),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                });

            let hex = format!(
                "#{:02X}{:02X}{:02X}",
                (self.selected_color.r * 255.0) as u8,
                (self.selected_color.g * 255.0) as u8,
                (self.selected_color.b * 255.0) as u8
            );

            let header = row![
                text("Color Picker").size(18),
                space::horizontal(),
                preview,
                text(hex).size(14),
            ]
            .spacing(10)
            .align_y(icy_ui::Alignment::Center);

            let toggle_btn = button(if self.active_picker == ActivePicker::Color {
                "Close Color Picker"
            } else {
                "Open Color Picker"
            })
            .on_press(Message::TogglePicker(ActivePicker::Color));

            let picker = if self.active_picker == ActivePicker::Color {
                Some(
                    color_picker::color_picker(self.selected_color, Message::ColorChanged)
                        .width(Length::Fixed(280.0))
                        .height(180.0),
                )
            } else {
                None
            };

            let mut col = column![header, toggle_btn].spacing(12);
            if let Some(p) = picker {
                col = col.push(p);
            }
            col
        };

        let date_section = {
            let date_str = format!(
                "{}-{:02}-{:02}",
                self.selected_date.year, self.selected_date.month, self.selected_date.day
            );

            let header = row![
                text("Date Picker").size(18),
                space::horizontal(),
                text(format!("Selected: {}", date_str)).size(14),
            ]
            .spacing(10)
            .align_y(icy_ui::Alignment::Center);

            let toggle_btn = button(if self.active_picker == ActivePicker::Date {
                "Close Date Picker"
            } else {
                "Open Date Picker"
            })
            .on_press(Message::TogglePicker(ActivePicker::Date));

            let picker = if self.active_picker == ActivePicker::Date {
                Some(date_picker::date_picker(
                    self.selected_date,
                    self.visible_date,
                    Message::DateSelected,
                    Message::PrevMonth,
                    Message::NextMonth,
                ))
            } else {
                None
            };

            let mut col = column![header, toggle_btn].spacing(12);
            if let Some(p) = picker {
                col = col.push(p);
            }
            col
        };

        let content = column![
            text("Picker Widgets Demo").size(24),
            color_section,
            icy_ui::widget::rule::horizontal(1),
            date_section,
        ]
        .spacing(24)
        .padding(24)
        .max_width(400);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill)
            .into()
    }
}
