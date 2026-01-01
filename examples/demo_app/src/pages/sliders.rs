//! Sliders page

use icy_ui::widget::{column, progress_bar, row, slider, space, text, vertical_slider};
use icy_ui::{Center, Element};

use crate::Message;

#[derive(Clone)]
pub struct SlidersState {
    pub slider_value: f32,
    pub vertical_slider_value: f32,
    pub progress_value: f32,
}

impl Default for SlidersState {
    fn default() -> Self {
        Self {
            slider_value: 50.0,
            vertical_slider_value: 50.0,
            progress_value: 0.0,
        }
    }
}

pub fn update_sliders(state: &mut SlidersState, message: &Message) -> bool {
    match message {
        Message::SliderChanged(value) => {
            state.slider_value = *value;
            true
        }
        Message::VerticalSliderChanged(value) => {
            state.vertical_slider_value = *value;
            true
        }
        Message::ProgressTick => {
            state.progress_value = (state.progress_value + 0.01) % 1.0;
            true
        }
        _ => false,
    }
}

pub fn view_sliders(state: &SlidersState) -> Element<'static, Message> {
    // Horizontal Slider section
    let horizontal_slider_section = column![
        text("Horizontal Slider").size(18),
        space().height(10),
        row![
            slider(0.0..=100.0, state.slider_value, Message::SliderChanged).width(300),
            text(format!("{:.1}", state.slider_value)),
        ]
        .spacing(10)
        .align_y(Center),
    ]
    .spacing(4);

    // Vertical Slider section
    let vertical_slider_section = column![
        text("Vertical Slider").size(18),
        space().height(10),
        row![
            vertical_slider(
                0.0..=100.0,
                state.vertical_slider_value,
                Message::VerticalSliderChanged
            )
            .height(150),
            space().width(10),
            text(format!("{:.1}", state.vertical_slider_value)),
        ]
        .align_y(Center),
    ]
    .spacing(4);

    // Sliders row (horizontal + vertical side by side)
    let sliders_row = row![
        horizontal_slider_section,
        space().width(40),
        vertical_slider_section,
    ]
    .align_y(Center);

    // Progress bar section
    let progress_section = column![
        text("Progress Bar").size(18),
        space().height(10),
        text("Animated progress (auto-incrementing):").size(14),
        progress_bar(0.0..=1.0, state.progress_value).length(300),
        text(format!("{:.0}%", state.progress_value * 100.0)).size(12),
    ]
    .spacing(4);

    // Static progress examples
    let static_progress = column![
        text("Static Progress Examples").size(18),
        space().height(10),
        row![text("0%"), progress_bar(0.0..=1.0, 0.0).length(100),]
            .spacing(10)
            .align_y(Center),
        row![text("25%"), progress_bar(0.0..=1.0, 0.25).length(100),]
            .spacing(10)
            .align_y(Center),
        row![text("50%"), progress_bar(0.0..=1.0, 0.5).length(100),]
            .spacing(10)
            .align_y(Center),
        row![text("75%"), progress_bar(0.0..=1.0, 0.75).length(100),]
            .spacing(10)
            .align_y(Center),
        row![text("100%"), progress_bar(0.0..=1.0, 1.0).length(100),]
            .spacing(10)
            .align_y(Center),
    ]
    .spacing(4);

    column![
        sliders_row,
        space().height(20),
        progress_section,
        space().height(20),
        static_progress,
    ]
    .spacing(4)
    .into()
}
