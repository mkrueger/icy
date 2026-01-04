//! RTL (Right-to-Left) Layout Example
//!
//! This example demonstrates the layout direction system in icy_ui.
//! It shows various widgets that should adapt their layout based on
//! the text direction (LTR vs RTL).
//!
//! In RTL mode:
//! - Labels should appear on the right side of inputs
//! - Sliders should fill from right to left
//! - Progress bars should fill from right to left
//! - Checkboxes should have their box on the right
//! - Menus should open to the left

use icy_ui::keyboard::Key;
use icy_ui::menu::{self, MenuShortcut};
use icy_ui::widget::{
    button, checkbox, column, container, pick_list, progress_bar, row, rule, scrollable, slider,
    text, text_input, toggler, Space,
};
use icy_ui::{window, Center, Element, Fill, Task};
use icy_ui_core::LayoutDirection;

pub fn main() -> icy_ui::Result {
    icy_ui::application(RtlDemo::default, RtlDemo::update, RtlDemo::view)
        .application_menu(RtlDemo::application_menu)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    // Layout direction toggle
    SetDirection(LayoutDirection),

    // Widget interactions
    TextInputChanged(String),
    SearchInputChanged(String),
    SliderChanged(f32),
    ProgressChanged(f32),
    CheckboxToggled(bool),
    TogglerToggled(bool),
    PickListSelected(Language),

    // Menu actions
    MenuNew,
    MenuOpen,
    MenuSave,
    MenuUndo,
    MenuRedo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    English,
    Arabic,
    Hebrew,
    German,
    Japanese,
}

impl Language {
    const ALL: &'static [Language] = &[
        Language::English,
        Language::Arabic,
        Language::Hebrew,
        Language::German,
        Language::Japanese,
    ];

    fn is_rtl(&self) -> bool {
        matches!(self, Language::Arabic | Language::Hebrew)
    }

    fn hello(&self) -> &'static str {
        match self {
            Language::English => "Hello, World!",
            Language::Arabic => "مرحبا بالعالم!",
            Language::Hebrew => "שלום עולם!",
            Language::German => "Hallo Welt!",
            Language::Japanese => "こんにちは世界!",
        }
    }

    fn placeholder(&self) -> &'static str {
        match self {
            Language::English => "Enter text here...",
            Language::Arabic => "أدخل النص هنا...",
            Language::Hebrew => "הזן טקסט כאן...",
            Language::German => "Text hier eingeben...",
            Language::Japanese => "ここにテキストを入力...",
        }
    }

    fn search(&self) -> &'static str {
        match self {
            Language::English => "Search...",
            Language::Arabic => "بحث...",
            Language::Hebrew => "חיפוש...",
            Language::German => "Suchen...",
            Language::Japanese => "検索...",
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Language::English => "Name",
            Language::Arabic => "الاسم",
            Language::Hebrew => "שם",
            Language::German => "Name",
            Language::Japanese => "名前",
        }
    }

    fn volume(&self) -> &'static str {
        match self {
            Language::English => "Volume",
            Language::Arabic => "الصوت",
            Language::Hebrew => "עוצמה",
            Language::German => "Lautstärke",
            Language::Japanese => "音量",
        }
    }

    fn progress(&self) -> &'static str {
        match self {
            Language::English => "Progress",
            Language::Arabic => "التقدم",
            Language::Hebrew => "התקדמות",
            Language::German => "Fortschritt",
            Language::Japanese => "進捗",
        }
    }

    fn enable_notifications(&self) -> &'static str {
        match self {
            Language::English => "Enable notifications",
            Language::Arabic => "تفعيل الإشعارات",
            Language::Hebrew => "אפשר התראות",
            Language::German => "Benachrichtigungen aktivieren",
            Language::Japanese => "通知を有効にする",
        }
    }

    fn dark_mode(&self) -> &'static str {
        match self {
            Language::English => "Dark mode",
            Language::Arabic => "الوضع الداكن",
            Language::Hebrew => "מצב כהה",
            Language::German => "Dunkelmodus",
            Language::Japanese => "ダークモード",
        }
    }

    fn toggle_rtl(&self) -> &'static str {
        match self {
            Language::English => "Toggle RTL",
            Language::Arabic => "تبديل RTL",
            Language::Hebrew => "החלף RTL",
            Language::German => "RTL umschalten",
            Language::Japanese => "RTL切替",
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::English => write!(f, "English"),
            Language::Arabic => write!(f, "العربية"),
            Language::Hebrew => write!(f, "עברית"),
            Language::German => write!(f, "Deutsch"),
            Language::Japanese => write!(f, "日本語"),
        }
    }
}

#[derive(Default)]
struct RtlDemo {
    // Current direction (for UI display)
    direction: LayoutDirection,

    // Widget state
    text_input: String,
    search_input: String,
    slider_value: f32,
    progress_value: f32,
    checkbox_checked: bool,
    toggler_enabled: bool,
    selected_language: Language,
}

impl RtlDemo {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SetDirection(direction) => {
                self.direction = direction;
                return window::set_layout_direction(direction);
            }
            Message::TextInputChanged(value) => {
                self.text_input = value;
            }
            Message::SearchInputChanged(value) => {
                self.search_input = value;
            }
            Message::SliderChanged(value) => {
                self.slider_value = value;
            }
            Message::ProgressChanged(value) => {
                self.progress_value = value;
            }
            Message::CheckboxToggled(checked) => {
                self.checkbox_checked = checked;
            }
            Message::TogglerToggled(enabled) => {
                self.toggler_enabled = enabled;
            }
            Message::PickListSelected(lang) => {
                self.selected_language = lang;
                // Auto-switch direction based on language
                let direction = if lang.is_rtl() {
                    LayoutDirection::Rtl
                } else {
                    LayoutDirection::Ltr
                };
                self.direction = direction;
                return window::set_layout_direction(direction);
            }
            Message::MenuNew | Message::MenuOpen | Message::MenuSave => {}
            Message::MenuUndo | Message::MenuRedo => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let lang = self.selected_language;
        let is_rtl = self.direction.is_rtl();

        // Header with language selector and direction toggle
        let header = {
            let lang_picker = pick_list(
                Language::ALL,
                Some(self.selected_language),
                Message::PickListSelected,
            )
            .placeholder("Select language");

            let direction_label = text(format!(
                "Direction: {}",
                if is_rtl { "RTL ←" } else { "LTR →" }
            ))
            .size(16);

            let toggle_btn = button(lang.toggle_rtl()).on_press(Message::SetDirection(self.direction.flip()));

            row![lang_picker, Space::new().width(Fill), direction_label, toggle_btn,]
                .spacing(15)
                .align_y(Center)
        };

        // Welcome text
        let welcome = text(lang.hello()).size(28);

        // Label + Text Input
        let text_input_row = {
            let label = text(lang.name()).size(16);
            let input = text_input(lang.placeholder(), &self.text_input)
                .on_input(Message::TextInputChanged)
                .width(300);

            // `Row` mirrors automatically in RTL
            row![label, input].spacing(10).align_y(Center)
        };

        // Search input
        let search_row = {
            let search_icon = text("🔍").size(16);
            let input = text_input(lang.search(), &self.search_input)
                .on_input(Message::SearchInputChanged)
                .width(300);

            // `Row` mirrors automatically in RTL
            row![search_icon, input].spacing(10).align_y(Center)
        };

        // Slider with label
        let slider_row = {
            let label = text(lang.volume()).size(16);
            let slider_widget = slider(0.0..=100.0, self.slider_value, Message::SliderChanged)
                .width(250);
            let value_text = text(format!("{:.0}%", self.slider_value)).size(14).width(50);

            // `Row` mirrors automatically in RTL
            row![label, slider_widget, value_text]
                .spacing(10)
                .align_y(Center)
        };

        // Progress bar with label
        let progress_row = {
            let label = text(lang.progress()).size(16);
            let progress = progress_bar(0.0..=100.0, self.progress_value)
                .length(250);
            let value_text = text(format!("{:.0}%", self.progress_value)).size(14).width(50);

            // `Row` mirrors automatically in RTL
            row![label, progress, value_text]
                .spacing(10)
                .align_y(Center)
        };

        // Progress control slider
        let progress_control = {
            let label = text("Adjust progress").size(14);
            let slider_widget =
                slider(0.0..=100.0, self.progress_value, Message::ProgressChanged).width(200);

            row![label, slider_widget].spacing(10).align_y(Center)
        };

        // Checkbox (in RTL: box on right side)
        let checkbox_row = checkbox(self.checkbox_checked)
            .label(lang.enable_notifications())
            .on_toggle(Message::CheckboxToggled);

        // Toggler (in RTL: toggle on right side)
        let toggler_row = toggler(self.toggler_enabled)
            .label(lang.dark_mode())
            .on_toggle(Message::TogglerToggled);

        // Info section
        let info = column![
            rule::horizontal(1),
            text("RTL Layout Considerations").size(18),
            text("• Labels appear on the opposite side of inputs"),
            text("• Sliders and progress bars fill in the opposite direction"),
            text("• Checkboxes have their indicator on the opposite side"),
            text("• Menus open in the opposite direction"),
            text("• Tab order follows reading direction"),
        ]
        .spacing(5);

        // Main layout
        let content = column![
            header,
            rule::horizontal(1),
            welcome,
            Space::new().height(20),
            text("Form Elements").size(20),
            text_input_row,
            search_row,
            Space::new().height(10),
            text("Sliders & Progress").size(20),
            slider_row,
            progress_row,
            progress_control,
            Space::new().height(10),
            text("Toggles").size(20),
            checkbox_row,
            toggler_row,
            Space::new().height(20),
            info,
        ]
        .spacing(15)
        .padding(20);

        container(scrollable(content))
            .width(Fill)
            .height(Fill)
            .into()
    }

    fn application_menu(
        _state: &RtlDemo,
        _context: &menu::MenuContext,
    ) -> Option<menu::AppMenu<Message>> {
        let file_menu = menu::submenu!(
            "&File",
            [
                menu::item!(
                    "&New",
                    Message::MenuNew,
                    MenuShortcut::cmd(Key::Character("n".into()))
                ),
                menu::item!(
                    "&Open",
                    Message::MenuOpen,
                    MenuShortcut::cmd(Key::Character("o".into()))
                ),
                menu::item!(
                    "&Save",
                    Message::MenuSave,
                    MenuShortcut::cmd(Key::Character("s".into()))
                ),
            ]
        );

        let edit_menu = menu::submenu!(
            "&Edit",
            [
                menu::item!(
                    "&Undo",
                    Message::MenuUndo,
                    MenuShortcut::cmd(Key::Character("z".into()))
                ),
                menu::item!(
                    "&Redo",
                    Message::MenuRedo,
                    MenuShortcut::cmd_shift(Key::Character("z".into()))
                ),
            ]
        );

        let view_menu = menu::submenu!(
            "&View",
            [
                menu::item!("Set &LTR", Message::SetDirection(LayoutDirection::Ltr)),
                menu::item!("Set &RTL", Message::SetDirection(LayoutDirection::Rtl)),
            ]
        );

        Some(menu::AppMenu::new(vec![file_menu, edit_menu, view_menu]))
    }
}
