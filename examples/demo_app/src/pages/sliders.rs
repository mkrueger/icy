//! Sliders page

use icy_ui::widget::{column, progress_bar, row, slider, space, text};
use icy_ui::{Center, Element};

use crate::Message;

#[derive(Clone)]
pub struct SlidersState {
    pub slider_value: f32,
    pub progress_value: f32,
}

impl Default for SlidersState {
    fn default() -> Self {
        Self {
            slider_value: 50.0,
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
        Message::ProgressTick => {
            state.progress_value = (state.progress_value + 0.01) % 1.0;
            true
        }
        _ => false,
    }
}

pub fn view_sliders(state: &SlidersState) -> Element<'static, Message> {
    column![
        text("Slider").size(18),
        space().height(10),
        row![
            slider(0.0..=100.0, state.slider_value, Message::SliderChanged)
                .width(300),
            text(format!("{:.1}", state.slider_value)),
        ]
        .spacing(10)
        .align_y(Center),
        space().height(20),
        text("Progress Bar").size(18),
        space().height(10),
        text("Animated progress (auto-incrementing):").size(14),
        progress_bar(0.0..=1.0, state.progress_value).length(300),
        text(format!("{:.0}%", state.progress_value * 100.0)).size(12),
        space().height(20),
        text("Static Progress Examples").size(18),
        space().height(10),
        row![
            text("0%"),
            progress_bar(0.0..=1.0, 0.0).length(100),
        ]
        .spacing(10)
        .align_y(Center),
        row![
            text("25%"),
            progress_bar(0.0..=1.0, 0.25).length(100),
        ]
        .spacing(10)
        .align_y(Center),
        row![
            text("50%"),
            progress_bar(0.0..=1.0, 0.5).length(100),
        ]
        .spacing(10)
        .align_y(Center),
        row![
            text("75%"),
            progress_bar(0.0..=1.0, 0.75).length(100),
        ]
        .spacing(10)
        .align_y(Center),
        row![
            text("100%"),
            progress_bar(0.0..=1.0, 1.0).length(100),
        ]
        .spacing(10)
        .align_y(Center),
    ]
    .spacing(4)
    .into()
}
