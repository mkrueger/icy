//! Demo App - A comprehensive showcase of icy widgets
//!
//! This application demonstrates all the major widgets available in icy,
//! organized into pages accessible via a sidebar navigation.

mod pages;

use std::path::PathBuf;

use icy_ui::dnd::DropResult;
use icy_ui::keyboard::Key;
use icy_ui::widget::{
    button, column, container, date_picker, pane_grid, row, rule, scrollable, space, text,
    text_editor, toaster,
};
use icy_ui::window;
use icy_ui::{Element, Fill, Point, Subscription, Task, Theme};

use icy_ui::menu;

use pages::{
    AnchorPosition, ButtonsState, CanvasPageState, ComponentChoice, ContainerChoice,
    ContextMenuState, DndPageState, EventLogState, ListsState, MarkdownPageState,
    PaneGridPageState, PickersState, QrCodeState, ScrollDirection, ScrollablesTab, ScrollingState,
    ShaderState, SlidersState, TextInputsState, ThemePage, ThemePageState, ToastsState,
    TogglesState,
};

pub fn main() -> icy_ui::Result {
    icy_ui::application(DemoApp::default, DemoApp::update, DemoApp::view)
        .subscription(DemoApp::subscription)
        .theme(DemoApp::theme)
        .application_menu(DemoApp::application_menu)
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
    Dnd,
    QrCode,
    Shader,
    Canvas,
    PaneGrid,
    Markdown,
    EventLog,
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
        Page::Dnd,
        Page::QrCode,
        Page::Shader,
        Page::Canvas,
        Page::PaneGrid,
        Page::Markdown,
        Page::EventLog,
    ];

    fn name(&self) -> &'static str {
        match self {
            Page::Overview => "Overview",
            Page::Buttons => "Buttons",
            Page::TextInputs => "Text Inputs",
            Page::Sliders => "Progressbar",
            Page::Toggles => "Toggles",
            Page::Pickers => "Pickers",
            Page::Lists => "Pick Lists",
            Page::Scrolling => "Scrollables",
            Page::Theme => "Theme",
            Page::Toasts => "Toasts",
            Page::ContextMenu => "Context Menu",
            Page::Dnd => "Drag && Drop",
            Page::QrCode => "QR Code",
            Page::Shader => "Shader",
            Page::Canvas => "Canvas",
            Page::PaneGrid => "Pane Grid",
            Page::Markdown => "Markdown",
            Page::EventLog => "Event Log",
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
            Page::Dnd => "📦",
            Page::QrCode => "📱",
            Page::Shader => "✨",
            Page::Canvas => "🎨",
            Page::PaneGrid => "📰",
            Page::Markdown => "📝",
            Page::EventLog => "📊",
        }
    }

    fn source_file(&self) -> Option<&'static str> {
        match self {
            Page::Overview => None,
            Page::Buttons => Some("examples/demo_app/src/pages/buttons.rs"),
            Page::TextInputs => Some("examples/demo_app/src/pages/text_inputs.rs"),
            Page::Sliders => Some("examples/demo_app/src/pages/sliders.rs"),
            Page::Toggles => Some("examples/demo_app/src/pages/toggles.rs"),
            Page::Pickers => Some("examples/demo_app/src/pages/pickers.rs"),
            Page::Lists => Some("examples/demo_app/src/pages/lists.rs"),
            Page::Scrolling => Some("examples/demo_app/src/pages/scrolling.rs"),
            Page::Theme => Some("examples/demo_app/src/pages/theme_page.rs"),
            Page::Toasts => Some("examples/demo_app/src/pages/toasts.rs"),
            Page::ContextMenu => Some("examples/demo_app/src/pages/context_menu.rs"),
            Page::Dnd => Some("examples/demo_app/src/pages/dnd_page.rs"),
            Page::QrCode => Some("examples/demo_app/src/pages/qr_code.rs"),
            Page::Shader => Some("examples/demo_app/src/pages/shader_page.rs"),
            Page::Canvas => Some("examples/demo_app/src/pages/canvas_page.rs"),
            Page::PaneGrid => Some("examples/demo_app/src/pages/pane_grid_page.rs"),
            Page::Markdown => Some("examples/demo_app/src/pages/markdown_page.rs"),
            Page::EventLog => Some("examples/demo_app/src/pages/event_log.rs"),
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
    recent_files: Vec<String>,

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
    dnd: DndPageState,
    qr_code: QrCodeState,
    shader: ShaderState,
    canvas: CanvasPageState,
    pane_grid: PaneGridPageState,
    markdown: MarkdownPageState,
    event_log: EventLogState,
}

impl Default for DemoApp {
    fn default() -> Self {
        Self {
            current_page: Page::default(),
            dark_mode: true,
            status_message: "Welcome to Demo App!".into(),
            recent_files: vec![
                "README.md".to_string(),
                "doc/THEME.md".to_string(),
                "examples/demo_app/src/main.rs".to_string(),
            ],
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
            dnd: DndPageState::default(),
            qr_code: QrCodeState::default(),
            shader: ShaderState::default(),
            canvas: CanvasPageState::default(),
            event_log: EventLogState::default(),
            pane_grid: PaneGridPageState::default(),
            markdown: MarkdownPageState::default(),
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
    EmailChanged(String),

    // Sliders
    SliderChanged(f32),
    VerticalSliderChanged(f32),
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
    ScrollStylePresetChanged(scrollable::Preset),
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

    // Drag and Drop
    DndTextChanged(String),
    DndStartDrag,
    DndDragCompleted(DropResult),
    DndDragEntered {
        position: Point,
        mime_types: Vec<String>,
    },
    DndDragMoved(Point),
    DndDragLeft,
    DndDragDropped {
        position: Point,
        data: Vec<u8>,
        mime_type: String,
    },
    DndFileHovered(PathBuf),
    DndFileDropped(PathBuf),
    DndFilesHoveredLeft,

    // QR Code
    QrCodeInputChanged(String),

    // Shader
    ShaderCRealChanged(f32),
    ShaderCImagChanged(f32),
    ShaderZoomChanged(f32),

    // Canvas
    CanvasStartLine(icy_ui::Point),
    CanvasAddPoint(icy_ui::Point),
    CanvasEndLine,
    CanvasStrokeWidthChanged(f32),
    CanvasColorChanged(pages::StrokeColor),
    CanvasClear,

    // Pane Grid
    PaneGridSplitHorizontal(pane_grid::Pane),
    PaneGridSplitVertical(pane_grid::Pane),
    PaneGridClose(pane_grid::Pane),
    PaneGridDragged(pane_grid::DragEvent),
    PaneGridResized(pane_grid::ResizeEvent),
    PaneGridClicked(pane_grid::Pane),

    // Markdown
    MarkdownEditorAction(text_editor::Action),
    MarkdownLinkClicked(String),

    // Event Log
    EventLogReceived {
        event_type: String,
        details: String,
    },
    EventLogClear,

    // Global
    OpenUrl(String),

    // Application menu
    MenuOpenRecent(String),
    MenuActivateWindow(window::Id),
}

// =============================================================================
// Update Logic
// =============================================================================

impl DemoApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        // Handle global messages first
        match &message {
            Message::OpenUrl(url) => {
                let _ = open::that(url);
                return Task::none();
            }
            Message::MenuOpenRecent(path) => {
                self.status_message = format!("Open recent: {path}");
                return Task::none();
            }
            Message::MenuActivateWindow(id) => {
                self.status_message = format!("Activate window {id}");
                return Task::none();
            }
            Message::GoToPage(page) => {
                self.current_page = *page;
                self.status_message = format!("Switched to {} page", page.name());
                return Task::none();
            }
            Message::MenuAction(action) => {
                match action {
                    MenuAction::ToggleDarkMode => {
                        self.dark_mode = !self.dark_mode;
                        self.status_message =
                            format!("Theme: {}", if self.dark_mode { "Dark" } else { "Light" });
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

        if let Some(task) = pages::update_dnd(&mut self.dnd, &message) {
            return task;
        }

        if pages::update_qr_code(&mut self.qr_code, &message) {
            return Task::none();
        }

        if pages::update_shader(&mut self.shader, &message) {
            return Task::none();
        }

        if pages::update_canvas(&mut self.canvas, &message) {
            return Task::none();
        }

        if pages::update_pane_grid(&mut self.pane_grid, &message) {
            return Task::none();
        }

        if pages::update_markdown(&mut self.markdown, &message) {
            return Task::none();
        }

        if let Some(status) = pages::update_event_log(&mut self.event_log, &message) {
            self.status_message = status;
            return Task::none();
        }

        Task::none()
    }

    fn application_menu(
        state: &DemoApp,
        context: &menu::MenuContext,
    ) -> Option<menu::AppMenu<Message>> {
        // Use role-based quit item - on macOS this moves to the app menu with ⌘Q.
        let file_exit = menu::quit!(Message::MenuAction(MenuAction::Exit));

        // Base IDs for dynamic menu items (hashed from descriptive strings)
        const NO_RECENT_ID: menu::MenuId = menu::MenuId::from_str("demo.no_recent");
        const RECENT_BASE_ID: menu::MenuId = menu::MenuId::from_str("demo.recent");
        const RECENT_SUBMENU_ID: menu::MenuId = menu::MenuId::from_str("demo.recent_submenu");

        let recent_children: Vec<menu::MenuNode<Message>> = if state.recent_files.is_empty() {
            vec![menu::MenuNode::new_with_id(
                NO_RECENT_ID,
                menu::MenuKind::Item {
                    label: "(No Recent Files)".to_string(),
                    enabled: false,
                    shortcut: None,
                    on_activate: Message::NoOp,
                },
            )]
        } else {
            state
                .recent_files
                .iter()
                .enumerate()
                .map(|(idx, path)| {
                    // Dynamic items: base ID + index
                    menu::MenuNode::item_with_id(
                        menu::MenuId::from_u64(RECENT_BASE_ID.as_u64().wrapping_add(idx as u64)),
                        path.clone(),
                        Message::MenuOpenRecent(path.clone()),
                    )
                })
                .collect()
        };

        let file_menu = menu::submenu!(
            "File",
            [
                menu::MenuNode::submenu_with_id(RECENT_SUBMENU_ID, "Recent Files", recent_children),
                menu::separator!(),
                file_exit,
            ]
        );

        let view_menu = menu::submenu!(
            "View",
            [menu::check_item!(
                "Dark Mode",
                state.dark_mode,
                Message::MenuAction(MenuAction::ToggleDarkMode)
            ),]
        );

        // Dynamic page items: base ID + index
        const PAGE_BASE_ID: menu::MenuId = menu::MenuId::from_str("demo.page");
        const PAGES_SUBMENU_ID: menu::MenuId = menu::MenuId::from_str("demo.pages");

        let pages_children: Vec<menu::MenuNode<Message>> = Page::ALL
            .iter()
            .enumerate()
            .map(|(idx, page)| {
                menu::MenuNode::item_with_id(
                    menu::MenuId::from_u64(PAGE_BASE_ID.as_u64().wrapping_add(idx as u64)),
                    page.name(),
                    Message::GoToPage(*page),
                )
            })
            .collect();

        let pages_menu = menu::MenuNode::submenu_with_id(PAGES_SUBMENU_ID, "Pages", pages_children);

        // Use role-based about item - on macOS this moves to the app menu.
        let help_menu = menu::submenu!(
            "Help",
            [menu::about!(
                "About Demo App",
                Message::MenuAction(MenuAction::About)
            ),]
        );

        // Dynamic window items: base ID + index
        const WINDOW_BASE_ID: menu::MenuId = menu::MenuId::from_str("demo.window");
        const NO_WINDOWS_ID: menu::MenuId = menu::MenuId::from_str("demo.no_windows");
        const WINDOW_SUBMENU_ID: menu::MenuId = menu::MenuId::from_str("demo.window_menu");

        let mut window_children: Vec<menu::MenuNode<Message>> = context
            .windows
            .iter()
            .enumerate()
            .map(|(idx, info)| {
                let label = if info.title.is_empty() {
                    format!("Window {}", info.id)
                } else {
                    info.title.clone()
                };

                menu::MenuNode::check_item_with_id(
                    menu::MenuId::from_u64(WINDOW_BASE_ID.as_u64().wrapping_add(idx as u64)),
                    label,
                    info.focused,
                    Message::MenuActivateWindow(info.id),
                )
            })
            .collect();

        if window_children.is_empty() {
            window_children.push(menu::MenuNode::new_with_id(
                NO_WINDOWS_ID,
                menu::MenuKind::Item {
                    label: "(No Windows)".to_string(),
                    enabled: false,
                    shortcut: None,
                    on_activate: Message::NoOp,
                },
            ));
        }

        let window_menu =
            menu::MenuNode::submenu_with_id(WINDOW_SUBMENU_ID, "Window", window_children);

        Some(menu::AppMenu::new(vec![
            file_menu,
            view_menu,
            pages_menu,
            window_menu,
            help_menu,
        ]))
    }

    fn subscription(&self) -> Subscription<Message> {
        use icy_ui::keyboard;
        use icy_ui::time;
        use std::time::Duration;

        let keyboard_sub = keyboard::listen().filter_map(|event| {
            if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                let cmd = modifiers.command();
                match key.as_ref() {
                    Key::Character("q") if cmd => Some(Message::MenuAction(MenuAction::Exit)),
                    Key::Character("d") if cmd => {
                        Some(Message::MenuAction(MenuAction::ToggleDarkMode))
                    }
                    _ => None,
                }
            } else {
                None
            }
        });

        let mut subs: Vec<Subscription<Message>> = vec![keyboard_sub];

        if self.current_page == Page::Sliders {
            subs.push(time::every(Duration::from_millis(50)).map(|_| Message::ProgressTick));
        }

        if self.current_page == Page::Dnd {
            subs.push(pages::subscription_dnd());
        }

        if self.current_page == Page::EventLog {
            subs.push(pages::subscription_event_log());
        }

        Subscription::batch(subs)
    }

    fn theme(&self) -> Theme {
        if self.dark_mode {
            Theme::dark()
        } else {
            Theme::light()
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let sidebar = self.view_sidebar();
        let content = self.view_content();
        let status_bar = self.view_status_bar();

        let main_area = row![content, rule::vertical(1), sidebar,].height(Fill);

        column![main_area, rule::horizontal(1), status_bar].into()
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
            Page::Dnd => pages::view_dnd(&self.dnd),
            Page::QrCode => pages::view_qr_code(&self.qr_code),
            Page::Shader => pages::view_shader(&self.shader),
            Page::Canvas => pages::canvas_page_view(&self.canvas),
            Page::PaneGrid => pages::view_pane_grid(&self.pane_grid),
            Page::Markdown => pages::view_markdown(&self.markdown),
            Page::EventLog => pages::view_event_log(&self.event_log),
        };

        let header = text(format!(
            "{} {}",
            self.current_page.icon(),
            self.current_page.name()
        ))
        .size(24);

        let header_row: Element<'_, Message> =
            if let Some(source_file) = self.current_page.source_file() {
                let github_url = format!(
                    "https://github.com/mkrueger/icy/blob/master/{}",
                    source_file
                );
                row![
                    header,
                    space().width(Fill),
                    button::hyperlink("💻 Source Code", github_url)
                ]
                .align_y(icy_ui::Center)
                .into()
            } else {
                header.into()
            };

        let body: Element<'_, Message> = match self.current_page {
            // The Scrollables page contains its own scrollables; avoid wrapping it in
            // another scrollable to prevent awkward nested scrolling.
            // Canvas, PaneGrid, and Markdown pages need full control of their layout.
            Page::Scrolling | Page::Canvas | Page::PaneGrid | Page::Markdown => page_content,
            _ => scrollable(page_content).height(Fill).into(),
        };

        container(
            column![header_row, rule::horizontal(1), space().height(10), body]
                .spacing(10)
                .padding(20),
        )
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
