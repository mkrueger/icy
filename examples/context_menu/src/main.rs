use iced::widget::{button, column, container, text};
use iced::{Center, Element, Fill};

use std::collections::HashMap;

pub fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view).run()
}

#[derive(Default)]
struct App {
    last_action: String,
    counter: i32,
}

#[derive(Debug, Clone)]
enum Message {
    ButtonClicked,
    ContextAction(ContextAction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextAction {
    Increment,
    Decrement,
    Reset,
    Double,
    Halve,
}

impl iced::widget::menu::Action for ContextAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        Message::ContextAction(*self)
    }
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::ButtonClicked => {
                self.last_action = "Button clicked!".to_string();
            }
            Message::ContextAction(action) => {
                self.last_action = format!("{:?}", action);
                match action {
                    ContextAction::Increment => self.counter += 1,
                    ContextAction::Decrement => self.counter -= 1,
                    ContextAction::Reset => self.counter = 0,
                    ContextAction::Double => self.counter *= 2,
                    ContextAction::Halve => self.counter /= 2,
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        use iced::widget::menu::{context_menu, items, Item};

        let key_binds: HashMap<iced::widget::menu::KeyBind, ContextAction> = HashMap::new();

        // Create a button with a context menu
        let interactive_button = context_menu(
            button(text("Right-click me!").size(20))
                .padding(20)
                .on_press(Message::ButtonClicked),
            Some(items(
                &key_binds,
                vec![
                    Item::Button("Increment (+1)", ContextAction::Increment),
                    Item::Button("Decrement (-1)", ContextAction::Decrement),
                    Item::Divider,
                    Item::Button("Double (×2)", ContextAction::Double),
                    Item::Button("Halve (÷2)", ContextAction::Halve),
                    Item::Divider,
                    Item::Button("Reset to 0", ContextAction::Reset),
                ],
            )),
        );

        // Another widget with a different context menu
        let counter_display = context_menu(
            container(
                text(format!("Counter: {}", self.counter))
                    .size(40)
            )
            .padding(20)
            .style(container::rounded_box),
            Some(items(
                &key_binds,
                vec![
                    Item::Button("Reset", ContextAction::Reset),
                    Item::Folder(
                        "Quick Set",
                        vec![
                            Item::Button("+10", ContextAction::Increment),
                            Item::Button("-10", ContextAction::Decrement),
                        ],
                    ),
                ],
            )),
        );

        let content = column![
            text("Context Menu Example").size(30),
            text("Right-click on the elements below to see context menus").size(16),
            text("").size(20),
            interactive_button,
            text("").size(20),
            counter_display,
            text("").size(20),
            if self.last_action.is_empty() {
                text("No action yet")
            } else {
                text(format!("Last action: {}", self.last_action))
            },
        ]
        .spacing(10)
        .align_x(Center);

        container(content)
            .width(Fill)
            .height(Fill)
            .center(Fill)
            .into()
    }
}
