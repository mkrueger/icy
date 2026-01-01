//! Demo App - A comprehensive showcase of icy widgets
//!
//! This application demonstrates all the major widgets available in icy,
//! organized into pages accessible via a sidebar navigation.

mod pages;

use std::collections::HashMap;

use icy_ui::keyboard::Key;
use icy_ui::widget::{
    button, column, container, date_picker, row, rule, scrollable, space, text, toaster,
};
use icy_ui::widget::menu::{Item, KeyBind, Modifier, Tree, bar, items, root};
use icy_ui::{Element, Fill, Subscription, Task, Theme};

use pages::{
    AnchorPosition, ButtonsState, ComponentChoice, ContainerChoice, ContextMenuState, ListsState,
    PickersState, ScrollDirection, ScrollStylePreset, ScrollablesTab, ScrollingState, SlidersState,
    TextInputsState, ThemePage, ThemePageState, ToastsState, TogglesState,
};

pub fn main() -> icy_ui::Result {
    icy_ui::application(DemoApp::default, DemoApp::update, DemoApp::view)
        .subscription(DemoApp::subscription)
        .theme(DemoApp::theme)
        .run()
}

// =============================================================================
// Page Definitions
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Overview,
    Buttons,
    TextInputs,
    Sliders,
    Toggles,
    Pickers,
    Lists,
    Scrolling,
    Theme,
    Toasts,
    ContextMenu,
}

impl Page {
    const ALL: &'static [Page] = &[
        Page::Overview,
        Page::Buttons,
        Page::TextInputs,
        Page::Sliders,
        Page::Toggles,
        Page::Pickers,
        Page::Lists,
        Page::Scrolling,
        Page::Theme,
        Page::Toasts,
        Page::ContextMenu,
    ];

    fn name(&self) -> &'static str {
        match self {
            Page::Overview => "Overview",
            Page::Buttons => "Buttons",
            Page::TextInputs => "Text Inputs",
            Page::Sliders => "Sliders & Progress",
            Page::Toggles => "Toggles & Checkboxes",
            Page::Pickers => "Color & Date Pickers",
            Page::Lists => "Pick Lists & Combos",
            Page::Scrolling => "Scrollables",
            Page::Theme => "Theme",
            Page::Toasts => "Toasts",
            Page::ContextMenu => "Context Menu",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Page::Overview => "🏠",
            Page::Buttons => "🔘",
            Page::TextInputs => "📝",
            Page::Sliders => "🎚️",
            Page::Toggles => "✅",
            Page::Pickers => "🎨",
            Page::Lists => "📋",
            Page::Scrolling => "📜",
            Page::Theme => "🎭",
            Page::Toasts => "🔔",
            Page::ContextMenu => "📌",
        }
    }
}

impl std::fmt::Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.name())
    }
}

// =============================================================================
// Menu Actions
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    ToggleDarkMode,
    About,
    Exit,
    GoToPage(Page),
}

impl icy_ui::widget::menu::Action for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        Message::MenuAction(*self)
    }
}

// =============================================================================
// Shared Types
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadioChoice {
    Option1,
    Option2,
    Option3,
}

impl std::fmt::Display for RadioChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RadioChoice::Option1 => write!(f, "Option 1"),
            RadioChoice::Option2 => write!(f, "Option 2"),
            RadioChoice::Option3 => write!(f, "Option 3"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Cpp,
}

impl Language {
    pub const ALL: [Language; 6] = [
        Language::Rust,
        Language::Python,
        Language::JavaScript,
        Language::TypeScript,
        Language::Go,
        Language::Cpp,
    ];
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Rust => write!(f, "Rust"),
            Language::Python => write!(f, "Python"),
            Language::JavaScript => write!(f, "JavaScript"),
            Language::TypeScript => write!(f, "TypeScript"),
            Language::Go => write!(f, "Go"),
            Language::Cpp => write!(f, "C++"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

// =============================================================================
// Application State
// =============================================================================

struct DemoApp {
    current_page: Page,
    dark_mode: bool,
    status_message: String,

    // Page states
    buttons: ButtonsState,
    text_inputs: TextInputsState,
    sliders: SlidersState,
    toggles: TogglesState,
    pickers: PickersState,
    lists: ListsState,
    scrolling: ScrollingState,
    theme_page: ThemePageState,
    toasts: ToastsState,
    context_menu: ContextMenuState,
}

impl Default for DemoApp {
    fn default() -> Self {
        Self {
            current_page: Page::default(),
            dark_mode: false,
            status_message: "Welcome to Demo App!".into(),
            buttons: ButtonsState::default(),
            text_inputs: TextInputsState::default(),
            sliders: SlidersState::default(),
            toggles: TogglesState::default(),
            pickers: PickersState::default(),
            lists: ListsState::default(),
            scrolling: ScrollingState::default(),
            theme_page: ThemePageState::default(),
            toasts: ToastsState::default(),
            context_menu: ContextMenuState::default(),
        }
    }
}

// =============================================================================
// Messages
// =============================================================================

#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    GoToPage(Page),
    MenuAction(MenuAction),
    NoOp,

    // Buttons
    ButtonClicked,
    SpinValueChanged(i32),

    // Text inputs
    TextChanged(String),
    PasswordChanged(String),

    // Sliders
    SliderChanged(f32),
    ProgressTick,

    // Toggles
    CheckboxToggled(bool),
    TogglerToggled(bool),
    RadioSelected(RadioChoice),

    // Pickers
    ColorChanged(icy_ui::Color),
    ToggleColorPicker,
    DateChanged(date_picker::Date),
    DatePrevMonth,
    DateNextMonth,
    ToggleDatePicker,

    // Lists
    LanguageSelected(Language),

    // Scrolling
    ScrollablesTabSelected(ScrollablesTab),
    Scrolled(scrollable::Viewport),
    RowHeightChanged(f32),
    ScrollStylePresetChanged(ScrollStylePreset),
    ScrollDirectionChanged(ScrollDirection),
    ScrollbarWidthChanged(u32),
    ScrollbarMarginChanged(u32),
    ScrollerWidthChanged(u32),
    ScrollAnchorChanged(AnchorPosition),

    // Theme
    ThemePageChanged(ThemePage),
    ContainerChoiceChanged(ContainerChoice),
    ComponentChoiceChanged(ComponentChoice),

    // Toasts
    AddToast(ToastKind),
    CloseToast(toaster::Id),

    // Context menu
    ContextAction(String),
}

// =============================================================================
// Update Logic
// =============================================================================

impl DemoApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        // Handle global messages first
        match &message {
            Message::GoToPage(page) => {
                self.current_page = *page;
                self.status_message = format!("Switched to {} page", page.name());
                return Task::none();
            }
            Message::MenuAction(action) => {
                match action {
                    MenuAction::ToggleDarkMode => {
                        self.dark_mode = !self.dark_mode;
                        self.status_message = format!(
                            "Theme: {}",
                            if self.dark_mode { "Dark" } else { "Light" }
                        );
                    }
                    MenuAction::About => {
                        self.status_message =
                            "Demo App v1.0 - A comprehensive icy widget showcase".into();
                    }
                    MenuAction::Exit => {
                        std::process::exit(0);
                    }
                    MenuAction::GoToPage(page) => {
                        self.current_page = *page;
                        self.status_message = format!("Switched to {} page", page.name());
                    }
                }
                return Task::none();
            }
            Message::NoOp => return Task::none(),
            _ => {}
        }

        // Dispatch to page-specific update functions
        if let Some((task, status)) = pages::update_buttons(&mut self.buttons, &message) {
            self.status_message = status;
            return task;
        }

        if pages::update_text_inputs(&mut self.text_inputs, &message) {
            return Task::none();
        }

        if pages::update_sliders(&mut self.sliders, &message) {
            return Task::none();
        }

        if pages::update_toggles(&mut self.toggles, &message) {
            return Task::none();
        }

        if pages::update_pickers(&mut self.pickers, &message) {
            return Task::none();
        }

        if let Some(status) = pages::update_lists(&mut self.lists, &message) {
            self.status_message = status;
            return Task::none();
        }

        if pages::update_scrolling(&mut self.scrolling, &message) {
            return Task::none();
        }

        if pages::update_theme_page(&mut self.theme_page, &message) {
            return Task::none();
        }

        if let Some(task) = pages::update_toasts(&mut self.toasts, &message) {
            return task;
        }

        if let Some(status) = pages::update_context_menu(&mut self.context_menu, &message) {
            self.status_message = status;
            return Task::none();
        }

        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        use icy_ui::keyboard;
        use icy_ui::time;
        use std::time::Duration;

        let keyboard_sub = keyboard::listen().filter_map(|event| {
            if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                let ctrl = modifiers.control();
                match key.as_ref() {
                    Key::Character("q") if ctrl => Some(Message::MenuAction(MenuAction::Exit)),
                    Key::Character("d") if ctrl => {
                        Some(Message::MenuAction(MenuAction::ToggleDarkMode))
                    }
                    _ => None,
                }
            } else {
                None
            }
        });

        let progress_sub = time::every(Duration::from_millis(50)).map(|_| Message::ProgressTick);

        Subscription::batch([keyboard_sub, progress_sub])
    }

    fn theme(&self) -> Theme {
        if self.dark_mode {
            Theme::dark()
        } else {
            Theme::light()
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let menu_bar = self.view_menu_bar();
        let sidebar = self.view_sidebar();
        let content = self.view_content();
        let status_bar = self.view_status_bar();

        let main_area = row![content, rule::vertical(1), sidebar,].height(Fill);

        column![
            menu_bar,
            rule::horizontal(1),
            main_area,
            rule::horizontal(1),
            status_bar,
        ]
        .into()
    }

    fn view_menu_bar(&self) -> Element<'_, Message> {
        let key_binds: HashMap<KeyBind, MenuAction> = [
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
                    key: Key::Character("d".into()),
                },
                MenuAction::ToggleDarkMode,
            ),
        ]
        .into_iter()
        .collect();

        let (file_btn, file_mnemonic) = root("&File", Message::NoOp);
        let mut file_menu = Tree::with_children(
            file_btn,
            items(&key_binds, vec![Item::Button("E&xit", MenuAction::Exit)]),
        );
        if let Some(m) = file_mnemonic {
            file_menu = file_menu.mnemonic(m);
        }

        let (view_btn, view_mnemonic) = root("&View", Message::NoOp);
        let mut view_menu = Tree::with_children(
            view_btn,
            items(
                &key_binds,
                vec![Item::CheckBox(
                    "&Dark Mode",
                    self.dark_mode,
                    MenuAction::ToggleDarkMode,
                )],
            ),
        );
        if let Some(m) = view_mnemonic {
            view_menu = view_menu.mnemonic(m);
        }

        let (pages_btn, pages_mnemonic) = root("&Pages", Message::NoOp);
        let page_items: Vec<Item<MenuAction, &str>> = Page::ALL
            .iter()
            .map(|page| Item::Button(page.name(), MenuAction::GoToPage(*page)))
            .collect();
        let mut pages_menu = Tree::with_children(pages_btn, items(&key_binds, page_items));
        if let Some(m) = pages_mnemonic {
            pages_menu = pages_menu.mnemonic(m);
        }

        let (help_btn, help_mnemonic) = root("&Help", Message::NoOp);
        let mut help_menu = Tree::with_children(
            help_btn,
            items(&key_binds, vec![Item::Button("&About", MenuAction::About)]),
        );
        if let Some(m) = help_mnemonic {
            help_menu = help_menu.mnemonic(m);
        }

        bar(vec![file_menu, view_menu, pages_menu, help_menu]).into()
    }

    fn view_sidebar(&self) -> Element<'_, Message> {
        let page_buttons: Vec<Element<'_, Message>> = Page::ALL
            .iter()
            .map(|page| {
                let is_selected = *page == self.current_page;
                let label = format!("{} {}", page.icon(), page.name());

                let btn = button(text(label).size(14))
                    .on_press(Message::GoToPage(*page))
                    .width(Fill)
                    .padding([8, 12]);

                if is_selected {
                    btn.style(button::primary).into()
                } else {
                    btn.style(button::secondary).into()
                }
            })
            .collect();

        let nav = column(page_buttons).spacing(4).padding(8);

        container(
            column![
                text("Navigation").size(16),
                rule::horizontal(1),
                scrollable(nav).height(Fill),
            ]
            .spacing(8)
            .padding(8),
        )
        .width(200)
        .height(Fill)
        .into()
    }

    fn view_content(&self) -> Element<'_, Message> {
        let page_content: Element<'_, Message> = match self.current_page {
            Page::Overview => pages::view_overview(),
            Page::Buttons => pages::view_buttons(&self.buttons),
            Page::TextInputs => pages::view_text_inputs(&self.text_inputs),
            Page::Sliders => pages::view_sliders(&self.sliders),
            Page::Toggles => pages::view_toggles(&self.toggles),
            Page::Pickers => pages::view_pickers(&self.pickers),
            Page::Lists => pages::view_lists(&self.lists),
            Page::Scrolling => pages::view_scrolling(&self.scrolling),
            Page::Theme => pages::view_theme(self.theme(), &self.theme_page),
            Page::Toasts => pages::view_toasts(&self.toasts),
            Page::ContextMenu => pages::view_context_menu(&self.context_menu),
        };

        let header = text(format!(
            "{} {}",
            self.current_page.icon(),
            self.current_page.name()
        ))
        .size(24);

        let body: Element<'_, Message> = match self.current_page {
            // The Scrollables page contains its own scrollables; avoid wrapping it in
            // another scrollable to prevent awkward nested scrolling.
            Page::Scrolling => page_content,
            _ => scrollable(page_content).height(Fill).into(),
        };

        container(column![header, rule::horizontal(1), space().height(10), body]
            .spacing(10)
            .padding(20))
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn view_status_bar(&self) -> Element<'_, Message> {
        container(
            row![
                text(&self.status_message).size(12),
                space().width(Fill),
                text(format!(
                    "Theme: {} | Page: {}",
                    if self.dark_mode { "Dark" } else { "Light" },
                    self.current_page.name()
                ))
                .size(12),
            ]
            .padding([4, 8]),
        )
        .width(Fill)
        .into()
    }
}
