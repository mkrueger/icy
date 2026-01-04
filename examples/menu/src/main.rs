//! Menu Example - Demonstrates the application menu system
//!
//! This example shows how to use the cross-platform application menu.
//! On macOS, this renders as a native menu bar.
//! On Windows/Linux, this renders as an in-window menu widget.
//!
//! The context menu uses native macOS NSMenu on macOS, and a custom
//! widget overlay on other platforms.

use icy_ui::keyboard::Key;
use icy_ui::menu::{self, MenuShortcut};
use icy_ui::widget::menu::context_menu;
use icy_ui::widget::{column, container, text};
use icy_ui::{Center, Element, Fill, Task};

pub fn main() -> icy_ui::Result {
    icy_ui::application(App::default, App::update, App::view)
        .application_menu(App::application_menu)
        .run()
}

#[derive(Default)]
struct App {
    last_action: String,
    dark_mode: bool,
    show_toolbar: bool,
    /// Whether to use native context menus (macOS only)
    use_native_context_menu: bool,
}

#[derive(Debug, Clone)]
enum Message {
    MenuAction(MenuAction),
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
    Preferences,
}

impl App {
    fn context_menu_nodes() -> Vec<menu::MenuNode<Message>> {
        vec![
            menu::item!("Cut", Message::MenuAction(MenuAction::Cut)),
            menu::item!("Copy", Message::MenuAction(MenuAction::Copy)),
            menu::item!("Paste", Message::MenuAction(MenuAction::Paste)),
            menu::separator!(),
            menu::submenu!(
                "More Options",
                [
                    menu::item!("Undo", Message::MenuAction(MenuAction::Undo)),
                    menu::item!("Redo", Message::MenuAction(MenuAction::Redo)),
                ]
            ),
        ]
    }

    fn update(&mut self, message: Message) -> Task<Message> {
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
        }
        Task::none()
    }

    fn application_menu(
        state: &App,
        _context: &menu::MenuContext,
    ) -> Option<menu::AppMenu<Message>> {
        // File menu - using macro syntax for stable IDs with keyboard shortcuts
        // Use & before a letter to mark it as mnemonic (Alt+letter to activate)
        let file_menu = menu::submenu!(
            "&File",
            [
                menu::item!(
                    "&New",
                    Message::MenuAction(MenuAction::New),
                    MenuShortcut::cmd(Key::Character("n".into()))
                ),
                menu::item!(
                    "&Open",
                    Message::MenuAction(MenuAction::Open),
                    MenuShortcut::cmd(Key::Character("o".into()))
                ),
                menu::separator!(),
                menu::item!(
                    "&Save",
                    Message::MenuAction(MenuAction::Save),
                    MenuShortcut::cmd(Key::Character("s".into()))
                ),
                menu::item!(
                    "Save &As…",
                    Message::MenuAction(MenuAction::SaveAs),
                    MenuShortcut::cmd_shift(Key::Character("s".into()))
                ),
                menu::separator!(),
                // Quit uses MenuRole::Quit - on macOS moves to app menu with ⌘Q
                menu::quit!(Message::MenuAction(MenuAction::Exit)),
            ]
        );

        // Edit menu
        let edit_menu = menu::submenu!(
            "&Edit",
            [
                menu::item!(
                    "&Undo",
                    Message::MenuAction(MenuAction::Undo),
                    MenuShortcut::cmd(Key::Character("z".into()))
                ),
                menu::item!(
                    "&Redo",
                    Message::MenuAction(MenuAction::Redo),
                    MenuShortcut::cmd_shift(Key::Character("z".into()))
                ),
                menu::separator!(),
                menu::item!(
                    "Cu&t",
                    Message::MenuAction(MenuAction::Cut),
                    MenuShortcut::cmd(Key::Character("x".into()))
                ),
                menu::item!(
                    "&Copy",
                    Message::MenuAction(MenuAction::Copy),
                    MenuShortcut::cmd(Key::Character("c".into()))
                ),
                menu::item!(
                    "&Paste",
                    Message::MenuAction(MenuAction::Paste),
                    MenuShortcut::cmd(Key::Character("v".into()))
                ),
                menu::separator!(),
                // Preferences uses MenuRole::Preferences - on macOS moves to app menu with ⌘,
                menu::preferences!(
                    "&Preferences…",
                    Message::MenuAction(MenuAction::Preferences)
                ),
            ]
        );

        // View menu - using check_item! macro for checkboxes
        let view_menu = menu::submenu!(
            "&View",
            [
                menu::check_item!(
                    "&Dark Mode",
                    Some(state.dark_mode),
                    Message::MenuAction(MenuAction::ToggleDarkMode),
                    MenuShortcut::cmd(Key::Character("d".into()))
                ),
                menu::check_item!(
                    "Show &Toolbar",
                    Some(state.show_toolbar),
                    Message::MenuAction(MenuAction::ToggleToolbar),
                    MenuShortcut::cmd(Key::Character("t".into()))
                ),
            ]
        );

        // Help menu
        let help_menu = menu::submenu!(
            "&Help",
            [
                // About uses MenuRole::About - on macOS moves to app menu
                menu::about!("About Menu Example", Message::MenuAction(MenuAction::About)),
            ]
        );

        Some(menu::AppMenu::new(vec![
            file_menu, edit_menu, view_menu, help_menu,
        ]))
    }

    fn view(&self) -> Element<'_, Message> {
        // Main content
        let content = column![
            text("Application Menu Example").size(24),
            text("").size(10),
            text("This example demonstrates the cross-platform application menu.").size(16),
            text("On macOS: Native menu bar with items relocated to app menu.").size(14),
            text("On Windows/Linux: In-window menu bar widget.").size(14),
            text("").size(10),
            text("Right-click anywhere in this area to see the context menu.").size(14),
            text("").size(20),
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
            text("").size(10),
            text(format!(
                "Context menu: {}",
                if self.use_native_context_menu {
                    "Native (macOS)"
                } else {
                    "Widget overlay"
                }
            )),
        ]
        .spacing(10)
        .align_x(Center);

        let main_area = container(content).width(Fill).height(Fill).center(Fill);

        // Use widget-based context menu (works on all platforms)
        // For native macOS context menus, use the subscription + ShowNativeContextMenu pattern
        let context_menu_nodes = Self::context_menu_nodes();
        context_menu(main_area, &context_menu_nodes).into()
    }
}
