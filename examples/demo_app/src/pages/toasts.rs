//! Toasts page

use icy_ui::widget::{button, column, space, text, toaster};
use icy_ui::{Element, Task};

use crate::{Message, ToastKind};

pub struct ToastsState {
    pub toasts: toaster::Toasts<Message>,
    pub toast_counter: usize,
}

impl Default for ToastsState {
    fn default() -> Self {
        Self {
            toasts: toaster::Toasts::new(Message::CloseToast),
            toast_counter: 0,
        }
    }
}

pub fn update_toasts(state: &mut ToastsState, message: &Message) -> Option<Task<Message>> {
    match message {
        Message::AddToast(kind) => {
            state.toast_counter += 1;
            let (text, style): (String, toaster::StyleFn) = match kind {
                ToastKind::Info => (
                    format!("Info #{}: This is an informational message.", state.toast_counter),
                    toaster::info_style,
                ),
                ToastKind::Success => (
                    format!("Success #{}: Operation completed!", state.toast_counter),
                    toaster::success_style,
                ),
                ToastKind::Warning => (
                    format!("Warning #{}: Please review this.", state.toast_counter),
                    toaster::warning_style,
                ),
                ToastKind::Error => (
                    format!("Error #{}: Something went wrong.", state.toast_counter),
                    toaster::danger_style,
                ),
            };

            Some(state.toasts.push(toaster::Toast::new(text).style(style)))
        }
        Message::CloseToast(id) => {
            state.toasts.remove(*id);
            None
        }
        _ => None,
    }
}

pub fn view_toasts(state: &ToastsState) -> Element<'_, Message> {
    let toast_count = state.toasts.len();
    let content = column![
        text("Toast Notifications").size(18),
        space().height(10),
        text("Click the buttons below to show toast notifications:").size(14),
        space().height(10),
        icy_ui::widget::row![
            button("ℹ️ Info").on_press(Message::AddToast(ToastKind::Info)).style(button::secondary),
            button("✅ Success").on_press(Message::AddToast(ToastKind::Success)).style(button::success),
            button("⚠️ Warning").on_press(Message::AddToast(ToastKind::Warning)),
            button("❌ Error").on_press(Message::AddToast(ToastKind::Error)).style(button::danger),
        ]
        .spacing(10),
        space().height(20),
        text(format!("Active toasts: {}", toast_count)).size(14),
        text("Toasts will appear at the bottom-center of the window.").size(12),
    ]
    .spacing(4);

    toaster::toaster(&state.toasts, content).into()
}
