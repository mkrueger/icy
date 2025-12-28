use iced::widget::{column, container, text};
use iced::{Center, Element, Fill};
use iced::keyboard::{Key, key::Named};

use std::collections::HashMap;

pub fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .subscription(App::subscription)
        .run()
}

#[derive(Default)]
struct App {
    last_action: String,
    dark_mode: bool,
    show_toolbar: bool,
}

#[derive(Debug, Clone)]
enum Message {
    MenuAction(MenuAction),
    NoOp, // Used to enable menu root buttons visually
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuAction {
    New,
    Open,
    Save,
    SaveAs,
    Exit,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    ToggleDarkMode,
    ToggleToolbar,
    About,
}

impl iced::widget::menu::Action for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        Message::MenuAction(*self)
    }
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::MenuAction(action) => {
                self.last_action = format!("{:?}", action);
                match action {
                    MenuAction::ToggleDarkMode => self.dark_mode = !self.dark_mode,
                    MenuAction::ToggleToolbar => self.show_toolbar = !self.show_toolbar,
                    MenuAction::Exit => std::process::exit(0),
                    _ => {}
                }
            }
            Message::NoOp => {} // Do nothing - used for menu root buttons
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        use iced::keyboard;
        
        fn handle_hotkey(event: keyboard::Event) -> Option<Message> {
            let keyboard::Event::KeyPressed { key, modifiers, .. } = event else {
                return None;
            };
            
            let ctrl = modifiers.control();
            let shift = modifiers.shift();
            
            match key.as_ref() {
                Key::Character("q") if ctrl => Some(Message::MenuAction(MenuAction::Exit)),
                Key::Character("n") if ctrl => Some(Message::MenuAction(MenuAction::New)),
                Key::Character("o") if ctrl => Some(Message::MenuAction(MenuAction::Open)),
                Key::Character("s") if ctrl && shift => Some(Message::MenuAction(MenuAction::SaveAs)),
                Key::Character("s") if ctrl => Some(Message::MenuAction(MenuAction::Save)),
                Key::Character("z") if ctrl => Some(Message::MenuAction(MenuAction::Undo)),
                Key::Character("y") if ctrl => Some(Message::MenuAction(MenuAction::Redo)),
                Key::Character("x") if ctrl => Some(Message::MenuAction(MenuAction::Cut)),
                Key::Character("c") if ctrl => Some(Message::MenuAction(MenuAction::Copy)),
                Key::Character("v") if ctrl => Some(Message::MenuAction(MenuAction::Paste)),
                Key::Named(Named::F1) => Some(Message::MenuAction(MenuAction::About)),
                _ => None,
            }
        }
        
        keyboard::listen().filter_map(handle_hotkey)
    }

    fn view(&self) -> Element<'_, Message> {
        use iced::widget::menu::{bar, items, root, Item, KeyBind, MenuBar, Modifier, Tree};

        // Define key bindings for display in menu
        let key_binds: HashMap<KeyBind, MenuAction> = [
            (KeyBind { modifiers: vec![Modifier::Ctrl], key: Key::Character("n".into()) }, MenuAction::New),
            (KeyBind { modifiers: vec![Modifier::Ctrl], key: Key::Character("o".into()) }, MenuAction::Open),
            (KeyBind { modifiers: vec![Modifier::Ctrl], key: Key::Character("s".into()) }, MenuAction::Save),
            (KeyBind { modifiers: vec![Modifier::Ctrl, Modifier::Shift], key: Key::Character("s".into()) }, MenuAction::SaveAs),
            (KeyBind { modifiers: vec![Modifier::Ctrl], key: Key::Character("q".into()) }, MenuAction::Exit),
            (KeyBind { modifiers: vec![Modifier::Ctrl], key: Key::Character("z".into()) }, MenuAction::Undo),
            (KeyBind { modifiers: vec![Modifier::Ctrl], key: Key::Character("y".into()) }, MenuAction::Redo),
            (KeyBind { modifiers: vec![Modifier::Ctrl], key: Key::Character("x".into()) }, MenuAction::Cut),
            (KeyBind { modifiers: vec![Modifier::Ctrl], key: Key::Character("c".into()) }, MenuAction::Copy),
            (KeyBind { modifiers: vec![Modifier::Ctrl], key: Key::Character("v".into()) }, MenuAction::Paste),
            (KeyBind { modifiers: vec![], key: Key::Named(Named::F1) }, MenuAction::About),
        ].into_iter().collect();

        // Build menu structure using Tree::with_children
        let file_menu = Tree::with_children(
            root("File", Message::NoOp),
            items(
                &key_binds,
                vec![
                    Item::Button("New", MenuAction::New),
                    Item::Button("Open", MenuAction::Open),
                    Item::Divider,
                    Item::Button("Save", MenuAction::Save),
                    Item::Button("Save As...", MenuAction::SaveAs),
                    Item::Divider,
                    Item::Button("Exit", MenuAction::Exit),
                ],
            ),
        );

        let edit_menu = Tree::with_children(
            root("Edit", Message::NoOp),
            items(
                &key_binds,
                vec![
                    Item::Button("Undo", MenuAction::Undo),
                    Item::Button("Redo", MenuAction::Redo),
                    Item::Divider,
                    Item::Button("Cut", MenuAction::Cut),
                    Item::Button("Copy", MenuAction::Copy),
                    Item::Button("Paste", MenuAction::Paste),
                ],
            ),
        );

        let view_menu = Tree::with_children(
            root("View", Message::NoOp),
            items(
                &key_binds,
                vec![
                    Item::CheckBox("Dark Mode", self.dark_mode, MenuAction::ToggleDarkMode),
                    Item::CheckBox("Show Toolbar", self.show_toolbar, MenuAction::ToggleToolbar),
                ],
            ),
        );

        let help_menu = Tree::with_children(
            root("Help", Message::NoOp),
            items(
                &key_binds,
                vec![Item::Button("About", MenuAction::About)],
            ),
        );

        // Create the menu bar
        let menu_bar: MenuBar<'_, Message> = bar(vec![file_menu, edit_menu, view_menu, help_menu]);

        // Main content
        let content = column![
            text("Click on the menu bar above to open menus").size(20),
            text("").size(10),
            if self.last_action.is_empty() {
                text("No action selected yet")
            } else {
                text(format!("Last action: {}", self.last_action))
            },
            text("").size(20),
            text(format!("Dark mode: {}", if self.dark_mode { "ON" } else { "OFF" })),
            text(format!("Toolbar: {}", if self.show_toolbar { "Visible" } else { "Hidden" })),
        ]
        .spacing(10)
        .align_x(Center);

        let main_area = container(content)
            .width(Fill)
            .height(Fill)
            .center(Fill);

        column![menu_bar, main_area].into()
    }
}

