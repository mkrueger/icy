use icy_ui::keyboard::{Key, key::Named};
use icy_ui::widget::{column, container, text};
use icy_ui::{Center, Element, Fill};

use std::collections::HashMap;

pub fn main() -> icy_ui::Result {
    icy_ui::application(App::default, App::update, App::view)
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

impl icy_ui::widget::menu::Action for MenuAction {
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

    fn subscription(&self) -> icy_ui::Subscription<Message> {
        use icy_ui::keyboard;

        fn handle_hotkey(event: keyboard::Event) -> Option<Message> {
            match event {
                keyboard::Event::KeyPressed { key, modifiers, .. } => {
                    let ctrl = modifiers.control();
                    let shift = modifiers.shift();

                    match key.as_ref() {
                        Key::Character("q") if ctrl => Some(Message::MenuAction(MenuAction::Exit)),
                        Key::Character("n") if ctrl => Some(Message::MenuAction(MenuAction::New)),
                        Key::Character("o") if ctrl => Some(Message::MenuAction(MenuAction::Open)),
                        Key::Character("s") if ctrl && shift => {
                            Some(Message::MenuAction(MenuAction::SaveAs))
                        }
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
                _ => None,
            }
        }

        keyboard::listen().filter_map(handle_hotkey)
    }

    fn view(&self) -> Element<'_, Message> {
        use icy_ui::widget::menu::{Item, KeyBind, MenuBar, Modifier, Tree, bar, items, root};

        // Define key bindings for display in menu
        let key_binds: HashMap<KeyBind, MenuAction> = [
            (
                KeyBind {
                    modifiers: vec![Modifier::Ctrl],
                    key: Key::Character("n".into()),
                },
                MenuAction::New,
            ),
            (
                KeyBind {
                    modifiers: vec![Modifier::Ctrl],
                    key: Key::Character("o".into()),
                },
                MenuAction::Open,
            ),
            (
                KeyBind {
                    modifiers: vec![Modifier::Ctrl],
                    key: Key::Character("s".into()),
                },
                MenuAction::Save,
            ),
            (
                KeyBind {
                    modifiers: vec![Modifier::Ctrl, Modifier::Shift],
                    key: Key::Character("s".into()),
                },
                MenuAction::SaveAs,
            ),
            (
                KeyBind {
                    modifiers: vec![Modifier::Ctrl],
                    key: Key::Character("q".into()),
                },
                MenuAction::Exit,
            ),
            (
                KeyBind {
                    modifiers: vec![Modifier::Ctrl],
                    key: Key::Character("z".into()),
                },
                MenuAction::Undo,
            ),
            (
                KeyBind {
                    modifiers: vec![Modifier::Ctrl],
                    key: Key::Character("y".into()),
                },
                MenuAction::Redo,
            ),
            (
                KeyBind {
                    modifiers: vec![Modifier::Ctrl],
                    key: Key::Character("x".into()),
                },
                MenuAction::Cut,
            ),
            (
                KeyBind {
                    modifiers: vec![Modifier::Ctrl],
                    key: Key::Character("c".into()),
                },
                MenuAction::Copy,
            ),
            (
                KeyBind {
                    modifiers: vec![Modifier::Ctrl],
                    key: Key::Character("v".into()),
                },
                MenuAction::Paste,
            ),
            (
                KeyBind {
                    modifiers: vec![],
                    key: Key::Named(Named::F1),
                },
                MenuAction::About,
            ),
        ]
        .into_iter()
        .collect();

        // Build menu structure using Tree::with_children
        // Use '&' to mark mnemonic characters (e.g., "&File" makes Alt+F open File menu)

        let (file_btn, file_mnemonic) = root("&File", Message::NoOp);
        let mut file_menu = Tree::with_children(
            file_btn,
            items(
                &key_binds,
                vec![
                    Item::Button("&New", MenuAction::New),
                    Item::Button("&Open", MenuAction::Open),
                    Item::Divider,
                    Item::Button("&Save", MenuAction::Save),
                    Item::Button("Save &As...", MenuAction::SaveAs),
                    Item::Divider,
                    Item::Button("E&xit", MenuAction::Exit),
                ],
            ),
        );
        if let Some(m) = file_mnemonic {
            file_menu = file_menu.mnemonic(m);
        }

        let (edit_btn, edit_mnemonic) = root("&Edit", Message::NoOp);
        let mut edit_menu = Tree::with_children(
            edit_btn,
            items(
                &key_binds,
                vec![
                    Item::Button("&Undo", MenuAction::Undo),
                    Item::Button("&Redo", MenuAction::Redo),
                    Item::Divider,
                    Item::Button("Cu&t", MenuAction::Cut),
                    Item::Button("&Copy", MenuAction::Copy),
                    Item::Button("&Paste", MenuAction::Paste),
                ],
            ),
        );
        if let Some(m) = edit_mnemonic {
            edit_menu = edit_menu.mnemonic(m);
        }

        let (view_btn, view_mnemonic) = root("&View", Message::NoOp);
        let mut view_menu = Tree::with_children(
            view_btn,
            items(
                &key_binds,
                vec![
                    Item::CheckBox("&Dark Mode", self.dark_mode, MenuAction::ToggleDarkMode),
                    Item::CheckBox(
                        "Show &Toolbar",
                        self.show_toolbar,
                        MenuAction::ToggleToolbar,
                    ),
                ],
            ),
        );
        if let Some(m) = view_mnemonic {
            view_menu = view_menu.mnemonic(m);
        }

        let (help_btn, help_mnemonic) = root("&Help", Message::NoOp);
        let mut help_menu = Tree::with_children(
            help_btn,
            items(&key_binds, vec![Item::Button("&About", MenuAction::About)]),
        );
        if let Some(m) = help_mnemonic {
            help_menu = help_menu.mnemonic(m);
        }

        // Create the menu bar
        let menu_bar: MenuBar<'_, Message> = bar(vec![file_menu, edit_menu, view_menu, help_menu]);

        // Main content
        let content = column![
            text("Click on the menu bar above to open menus").size(20),
            text("Or press Alt+letter to activate mnemonics (e.g., Alt+F for File)").size(14),
            text("").size(10),
            if self.last_action.is_empty() {
                text("No action selected yet")
            } else {
                text(format!("Last action: {}", self.last_action))
            },
            text("").size(20),
            text(format!(
                "Dark mode: {}",
                if self.dark_mode { "ON" } else { "OFF" }
            )),
            text(format!(
                "Toolbar: {}",
                if self.show_toolbar {
                    "Visible"
                } else {
                    "Hidden"
                }
            )),
        ]
        .spacing(10)
        .align_x(Center);

        let main_area = container(content).width(Fill).height(Fill).center(Fill);

        column![menu_bar, main_area].into()
    }
}
