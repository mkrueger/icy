//! Overview page

use icy_ui::widget::{column, space, text};
use icy_ui::Element;

use crate::Message;

pub fn view_overview() -> Element<'static, Message> {
    column![
        text("Welcome to the Demo App!").size(20),
        space().height(20),
        text("This application showcases the various widgets available in icy."),
        text("Use the sidebar on the right to navigate between different widget demonstrations."),
        space().height(20),
        text("Features:").size(16),
        text("  • Main menu with keyboard shortcuts"),
        text("  • Dark/Light theme toggle (Ctrl+D)"),
        text("  • Multiple widget categories"),
        text("  • Interactive demos"),
        space().height(20),
        text("Pages:").size(16),
        text("  🔘 Buttons - Various button styles and states"),
        text("  📝 Text Inputs - Text fields and password inputs"),
        text("  🎚️ Sliders & Progress - Sliders and progress bars"),
        text("  ✅ Toggles & Checkboxes - Boolean controls"),
        text("  🎨 Color & Date Pickers - Advanced picker widgets"),
        text("  📋 Pick Lists & Combos - Selection widgets"),
        text("  📜 Scrollables - Scrollable content areas"),
        text("  🔔 Toasts - Notification toasts"),
        text("  📌 Context Menu - Right-click menus"),
    ]
    .spacing(4)
    .into()
}
