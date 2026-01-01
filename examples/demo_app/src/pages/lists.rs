//! Lists page

use icy_ui::widget::{button, column, pick_list, row, space, text};
use icy_ui::Element;

use crate::{Language, Message};

#[derive(Default, Clone)]
pub struct ListsState {
    pub selected_language: Option<Language>,
}

pub fn update_lists(state: &mut ListsState, message: &Message) -> Option<String> {
    match message {
        Message::LanguageSelected(lang) => {
            state.selected_language = Some(*lang);
            Some(format!("Selected: {}", lang))
        }
        _ => None,
    }
}

pub fn view_lists(state: &ListsState) -> Element<'static, Message> {
    column![
        text("Pick List").size(18),
        space().height(10),
        row![
            pick_list(
                Language::ALL.as_slice(),
                state.selected_language,
                Message::LanguageSelected,
            )
            .placeholder("Choose a language..."),
            text(
                state.selected_language
                    .map(|l| format!("Selected: {}", l))
                    .unwrap_or_else(|| "No selection".into())
            ),
        ]
        .spacing(20)
        .align_y(icy_ui::Alignment::Center),
        space().height(20),
        text("Pick List with all options visible:").size(14),
        space().height(10),
        column(
            Language::ALL
                .iter()
                .map(|lang| {
                    let is_selected = state.selected_language == Some(*lang);
                    button(
                        row![
                            text(if is_selected { "●" } else { "○" }),
                            text(lang.to_string()),
                        ]
                        .spacing(8)
                    )
                    .on_press(Message::LanguageSelected(*lang))
                    .width(200)
                    .style(if is_selected {
                        button::primary
                    } else {
                        button::secondary
                    })
                    .into()
                })
                .collect::<Vec<Element<'_, Message>>>()
        )
        .spacing(4),
    ]
    .spacing(4)
    .into()
}
