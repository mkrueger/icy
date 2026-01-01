//! QR Code page

use icy_ui::widget::{column, container, qr_code, space, text, text_input};
use icy_ui::{Element, Fill};

use crate::Message;

pub struct QrCodeState {
    pub input: String,
    pub data: Option<qr_code::Data>,
}

impl Clone for QrCodeState {
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            data: qr_code::Data::new(&self.input).ok(),
        }
    }
}

impl Default for QrCodeState {
    fn default() -> Self {
        Self {
            input: String::new(),
            data: None,
        }
    }
}

pub fn update_qr_code(state: &mut QrCodeState, message: &Message) -> bool {
    match message {
        Message::QrCodeInputChanged(value) => {
            state.input = value.clone();
            state.data = if value.is_empty() {
                None
            } else {
                qr_code::Data::new(value).ok()
            };
            true
        }
        _ => false,
    }
}

pub fn view_qr_code(state: &QrCodeState) -> Element<'_, Message> {
    let input_field = text_input("Enter text to encode...", &state.input)
        .on_input(Message::QrCodeInputChanged)
        .width(400);

    let qr_display: Element<'_, Message> = match &state.data {
        Some(data) => qr_code(data).cell_size(8).into(),
        None => {
            if state.input.is_empty() {
                container(text("Enter some text above to generate a QR code").size(14))
                    .width(200)
                    .height(200)
                    .center(Fill)
                    .into()
            } else {
                text("Failed to generate QR code").into()
            }
        }
    };

    column![
        text("QR Code Generator").size(18),
        space().height(10),
        text("Enter any text or URL to generate a QR code:").size(14),
        space().height(10),
        input_field,
        space().height(20),
        container(qr_display).width(Fill).center_x(Fill),
        space().height(20),
        text(format!("Characters: {}", state.input.len())).size(12),
    ]
    .spacing(5)
    .into()
}
