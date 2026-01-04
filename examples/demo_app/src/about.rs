use icy_ui::widget::{button, column, container, overlay::modal::Modal, row, rule, space, text};
use icy_ui::{Element, Length};

use super::Message;

const ABOUT_SOURCE_PATH: &str = "examples/demo_app/src/about.rs";
const MODAL_SOURCE_PATH: &str = "crates/widget/src/overlay/modal.rs";

fn github_blob_url(path: &str) -> String {
    format!("https://github.com/mkrueger/icy/blob/master/{path}")
}

pub fn wrap<'a>(open: bool, content: Element<'a, Message>) -> Element<'a, Message> {
    Modal::new(open, content, dialog())
        .on_blur(Message::SetAboutOpen(false))
        .on_escape(Message::SetAboutOpen(false))
        .into()
}

fn dialog() -> Element<'static, Message> {
    let title = text("Demo App").size(22);
    let version = text(format!("Version {}", env!("CARGO_PKG_VERSION"))).size(14);
    let subtitle = text("A comprehensive icy widget showcase").size(14);

    let links = row![
        button::hyperlink("💻 About dialog source", github_blob_url(ABOUT_SOURCE_PATH)),
        space().width(Length::Fixed(16.0)),
        button::hyperlink("🧩 Modal helper source", github_blob_url(MODAL_SOURCE_PATH)),
    ]
    .align_y(icy_ui::Center);

    let close = row![
        space().width(Length::Fill),
        button(text("Close")).on_press(Message::SetAboutOpen(false))
    ]
    .align_y(icy_ui::Center);

    let body = column![
        title,
        version,
        subtitle,
        rule::horizontal(1),
        links,
        rule::horizontal(1),
        close,
    ]
    .spacing(10)
    .padding(16);

    container(body)
        .style(container::secondary)
        .width(Length::Fixed(520.0))
        .into()
}
