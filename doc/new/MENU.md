# Menu System

The menu system provides a flexible, hierarchical menu implementation for desktop applications. It is ported from [libcosmic](https://github.com/pop-os/libcosmic) and [iced_aw](https://github.com/iced-rs/iced_aw), licensed under MIT/MPL-2.0.

## Overview

The menu system consists of several components:

- **MenuBar** - A horizontal bar containing top-level menu buttons
- **MenuTree** - A tree structure representing menu items and submenus
- **ContextMenu** - Right-click context menus
- **MenuAction** - A trait for defining actionable menu items
- **KeyBind** - Keyboard shortcut definitions
- **Mnemonics** - Keyboard navigation via `Alt+letter` underlines

## Basic Usage

```rust
use icy_ui::widget::button;
use icy_ui::widget::menu::{MenuTree, MenuBar};

// Create a submenu
let sub_menu = MenuTree::with_children(
    button("Sub Menu"),
    vec![
        MenuTree::new(button("Item 1")),
        MenuTree::new(button("Item 2")),
        MenuTree::new(button("Item 3")),
    ]
);

// Create a root menu with nested items
let file_menu = MenuTree::with_children(
    button("File"),
    vec![
        MenuTree::new(button("New")),
        MenuTree::new(button("Open")),
        sub_menu,
        MenuTree::new(button("Save")),
    ]
);

let edit_menu = MenuTree::with_children(
    button("Edit"),
    vec![
        MenuTree::new(button("Cut")),
        MenuTree::new(button("Copy")),
        MenuTree::new(button("Paste")),
    ]
);

// Create the menu bar
let menu_bar = MenuBar::new(vec![file_menu, edit_menu]);
```

## MenuTree

`MenuTree` represents a node in the menu hierarchy. It can be either a leaf item or a folder containing children.

### Creating Items

```rust
// Leaf item (no submenu)
let item = MenuTree::new(button("Click me"));

// Item with children (creates a submenu)
let folder = MenuTree::with_children(
    button("Submenu"),
    vec![
        MenuTree::new(button("Child 1")),
        MenuTree::new(button("Child 2")),
    ]
);
```

### Builder Methods

```rust
MenuTree::new(button("Item"))
    .width(200)              // Set item width in pixels
    .height(32)              // Set item height in pixels
    .mnemonic('i')           // Set Alt+I keyboard shortcut
```

### MenuItem Enum

Menu items can also be separators for visual grouping:

```rust
use icy_ui::widget::menu::Item;

let items = vec![
    Item::Item(button("Cut"), vec![]),
    Item::Item(button("Copy"), vec![]),
    Item::Separator,  // Visual divider
    Item::Item(button("Paste"), vec![]),
];
```

## MenuBar

`MenuBar` is the main widget that displays and manages the menu tree.

```rust
let menu_bar = MenuBar::new(vec![file_menu, edit_menu, help_menu])
    .spacing(8.0)           // Spacing between top-level items
    .padding([4, 8, 4, 8])  // Padding around the bar
    .item_width(ItemWidth::Static(150))
    .item_height(ItemHeight::Static(28))
    .close_condition(CloseCondition::default())
    .path_highlight(PathHighlight::Full);
```

### Configuration Options

#### ItemWidth

Controls menu item widths:
- `ItemWidth::Static(u16)` - Fixed width for all items
- `ItemWidth::Uniform` - All items take the width of the widest item

#### ItemHeight

Controls menu item heights:
- `ItemHeight::Static(u16)` - Fixed height for all items
- `ItemHeight::Uniform` - All items take the height of the tallest item
- `ItemHeight::Dynamic` - Each item has its natural height

#### CloseCondition

Controls when menus close:
- `close_when_cursor_leaves` - Close menu when mouse exits
- `close_when_clicked_outside` - Close menu when clicking outside

#### PathHighlight

Controls how the selected path is highlighted:
- `PathHighlight::Full` - Highlight the entire path from root
- `PathHighlight::Single` - Highlight only the hovered item

## ContextMenu

Right-click context menus for elements:

```rust
use icy_ui::widget::menu::{ContextMenu, context_menu};

let content = button("Right-click me");

let menu_tree = MenuTree::with_children(
    button("Context"),  // Not displayed, just for structure
    vec![
        MenuTree::new(button("Action 1")),
        MenuTree::new(button("Action 2")),
    ]
);

let widget = context_menu(content, menu_tree);
```

## MenuAction Trait

For applications that want structured menu actions with keyboard bindings:

```rust
use icy_ui::widget::menu::{Action, KeyBind, Modifier};
use icy_ui::keyboard::Key;

#[derive(Clone, Debug)]
enum MenuAction {
    New,
    Open,
    Save,
    Quit,
}

impl Action for MenuAction {
    type Message = AppMessage;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::New => AppMessage::New,
            MenuAction::Open => AppMessage::Open,
            MenuAction::Save => AppMessage::Save,
            MenuAction::Quit => AppMessage::Quit,
        }
    }
}
```

## KeyBind (Keyboard Shortcuts)

Define keyboard shortcuts for menu items:

```rust
use icy_ui::widget::menu::{KeyBind, Modifier};
use icy_ui::keyboard::Key;

let save_shortcut = KeyBind {
    modifiers: vec![Modifier::Ctrl],
    key: Key::Character("s".into()),
};

// Check if a key event matches
if save_shortcut.matches(event_modifiers, &event_key) {
    // Handle save
}

// Display format: "Ctrl + S"
println!("{}", save_shortcut);
```

### Modifier Keys

- `Modifier::Super` - Windows/Command key
- `Modifier::Ctrl` - Control key
- `Modifier::Alt` - Alt/Option key
- `Modifier::Shift` - Shift key

## Mnemonics

Mnemonics provide keyboard navigation using `Alt+letter`. Mark a mnemonic character with `&` in the label:

```rust
use icy_ui::widget::menu::{parse_mnemonic, mnemonic_text, MnemonicDisplay};

// Parse a label with mnemonic marker
let label = "&File";  // 'F' is the mnemonic
let parsed = parse_mnemonic(label);
// parsed.text = "File"
// parsed.mnemonic = Some('f')

// Create text widget with mnemonic display
let text_widget = mnemonic_text(
    "File",
    Some('f'),
    MnemonicDisplay::Underline  // Underlines the 'F'
);
```

### MnemonicDisplay Modes

- `MnemonicDisplay::Underline` - Underline the mnemonic character
- `MnemonicDisplay::Parentheses` - Show as "File (F)"
- `MnemonicDisplay::None` - Don't show mnemonic indicator

### Platform Support

Mnemonics are enabled on Windows and Linux but disabled on macOS (which uses Cmd-based shortcuts).

## Styling

The menu system uses the `StyleSheet` trait for theming:

```rust
use icy_ui::widget::menu::{StyleSheet, Appearance, Style};

// Default styles are provided
let menu_bar = MenuBar::new(menus).style(Style::Default);
```

### Appearance Properties

```rust
pub struct Appearance {
    pub background: Color,              // Menu background
    pub border_width: f32,              // Border thickness
    pub bar_border_radius: [f32; 4],    // Menu bar corner radius
    pub menu_border_radius: [f32; 4],   // Dropdown menu corner radius
    pub border_color: Color,            // Border color
    pub background_expand: [u16; 4],    // Background padding
    pub path: Color,                    // Highlighted item color (popups)
    pub bar_path: Color,                // Highlighted item color (bar)
    pub path_border_radius: [f32; 4],   // Highlight corner radius
    pub menu_content_padding: [f32; 4], // Bar item padding
    pub menu_inner_content_padding: [f32; 4], // Popup item padding
}
```

### Button Styles for Menu Items

Helper functions for styling menu buttons:

```rust
use icy_ui::widget::menu::{menu_item, menu_folder, menu_root_style};

// For regular menu items
let item_button = button("Item").style(|theme, status| menu_item(theme, status));

// For submenu folders
let folder_button = button("Submenu →").style(|theme, status| menu_folder(theme, status));

// For top-level menu bar items
let root_button = button("File").style(|theme, status| menu_root_style(theme, status));
```

## Helper Functions

The module provides convenience functions for common patterns:

```rust
use icy_ui::widget::menu::{bar, root, items};

// Create a menu bar
let menu_bar = bar(vec![file_menu, edit_menu]);

// Create a root menu
let file = root(button("File"), children);

// Create menu items
let children = items(vec![
    button("New"),
    button("Open"),
    button("Save"),
]);
```

## Complete Example

```rust
use icy_ui::widget::{button, row, text};
use icy_ui::widget::menu::{MenuTree, MenuBar, menu_item};

fn view(&self) -> Element<Message> {
    let file_menu = MenuTree::with_children(
        button("File").style(menu_root_style),
        vec![
            MenuTree::new(
                button("New")
                    .on_press(Message::New)
                    .style(menu_item)
            ).mnemonic('n'),
            MenuTree::new(
                button("Open")
                    .on_press(Message::Open)
                    .style(menu_item)
            ).mnemonic('o'),
            MenuTree::new(
                button("Save")
                    .on_press(Message::Save)
                    .style(menu_item)
            ).mnemonic('s'),
        ]
    ).mnemonic('f');

    let menu_bar = MenuBar::new(vec![file_menu])
        .spacing(4.0)
        .padding([2, 8]);

    column![
        menu_bar,
        // ... rest of your UI
    ].into()
}
```

## Platform Notes

- On macOS, consider using the native menu bar for better integration
- The `Super` modifier maps to Command on macOS and Windows key on other platforms
- Mnemonic underlines are typically shown only when Alt is pressed

## Application Menu (Native Menu Bar)

For a native platform experience, icy_ui supports an **Application Menu** model that renders as the system menu bar on macOS and as an in-window widget menu bar on Windows/Linux.

### Overview

The application menu is defined via the `Program::application_menu` method, which returns an `AppMenu<Message>` structure. The platform backend then renders it appropriately:

- **macOS**: Native `NSMenu` in the system menu bar
- **Windows/Linux**: Widget-based `MenuBar` rendered in the window

### Basic Usage

```rust
use icy_ui_core::menu::{AppMenu, MenuNode, MenuKind, MenuContext};

impl Program for MyApp {
    // ...

    fn application_menu(
        &self,
        context: &MenuContext,
    ) -> Option<AppMenu<Message>> {
        let file_menu = MenuNode::submenu(
            "File",
            vec![
                MenuNode::item("New", Message::New),
                MenuNode::item("Open", Message::Open),
                MenuNode::separator(),
                MenuNode::quit(Message::Quit),
            ],
        );

        let help_menu = MenuNode::submenu(
            "Help",
            vec![
                MenuNode::about("About My App", Message::ShowAbout),
            ],
        );

        Some(AppMenu::new(vec![file_menu, help_menu]))
    }
}
```

### Enable in Application Builder

```rust
fn main() -> icy_ui::Result {
    icy_ui::application(MyApp::default, MyApp::update, MyApp::view)
        .application_menu(MyApp::application_menu)  // Enable app menu
        .run()
}
```

### MenuNode Types

```rust
// Simple clickable item
MenuNode::item("Label", Message::Action)

// Submenu with children
MenuNode::submenu("Label", vec![...children...])

// Visual separator
MenuNode::separator()

// Checkbox item
MenuNode::new(MenuKind::CheckItem {
    label: "Dark Mode".into(),
    enabled: true,
    checked: state.dark_mode,
    shortcut: None,
    on_activate: Message::ToggleDarkMode,
})
```

### MenuRole - Cross-Platform Item Relocation

`MenuRole` allows certain menu items to be automatically relocated to platform-specific locations. On **macOS**, items with roles are moved to the **Application menu** (the menu named after the app). On **Windows/Linux**, roles are ignored and items stay where defined.

#### Available Roles

| Role | Description | macOS Behavior |
|------|-------------|----------------|
| `MenuRole::None` | No special handling (default) | Stays in place |
| `MenuRole::About` | "About" dialog | Moved to app menu |
| `MenuRole::Preferences` | Settings/Preferences | Moved to app menu with ⌘, |
| `MenuRole::Quit` | Quit/Exit application | Moved to app menu with ⌘Q |
| `MenuRole::ApplicationSpecific` | Custom app-specific items | Moved to app menu |

#### Role-Based Convenience Constructors

```rust
// Creates a Quit item with MenuRole::Quit
// On macOS: appears in app menu with ⌘Q shortcut
MenuNode::quit(Message::Quit)

// Creates an About item with MenuRole::About
// On macOS: appears at top of app menu
MenuNode::about("About My App", Message::ShowAbout)

// Creates a Preferences item with MenuRole::Preferences
// On macOS: appears in app menu with ⌘, shortcut
MenuNode::preferences("Settings…", Message::OpenSettings)
```

#### Custom Role Assignment

```rust
// Use with_role() for any item
MenuNode::item("License Info", Message::ShowLicense)
    .with_role(MenuRole::ApplicationSpecific)
```

#### macOS Application Menu Order

Items are arranged in standard macOS order:

1. **About** (MenuRole::About)
2. Separator
3. **Preferences** (MenuRole::Preferences) with ⌘,
4. Separator
5. **Application-Specific Items** (MenuRole::ApplicationSpecific)
6. Separator
7. **Quit** (MenuRole::Quit) with ⌘Q

### MenuContext

The `MenuContext` provides runtime information useful for building dynamic menus:

```rust
fn application_menu(
    &self,
    context: &MenuContext,
) -> Option<AppMenu<Message>> {
    // Access list of open windows
    for window in &context.windows {
        println!("Window: {} (id: {:?})", window.title, window.id);
    }

    // Build window switcher menu
    let window_items: Vec<_> = context.windows
        .iter()
        .map(|w| MenuNode::item(
            &w.title,
            Message::FocusWindow(w.id),
        ))
        .collect();

    // ...
}
```

### Complete Example with Roles

```rust
fn application_menu(
    state: &MyApp,
    context: &MenuContext,
) -> Option<AppMenu<Message>> {
    let file_menu = MenuNode::submenu("File", vec![
        MenuNode::item("New", Message::New),
        MenuNode::item("Open…", Message::Open),
        MenuNode::separator(),
        // Quit will move to app menu on macOS
        MenuNode::quit(Message::Quit),
    ]);

    let edit_menu = MenuNode::submenu("Edit", vec![
        MenuNode::item("Undo", Message::Undo),
        MenuNode::item("Redo", Message::Redo),
        MenuNode::separator(),
        // Preferences will move to app menu on macOS
        MenuNode::preferences("Settings…", Message::OpenSettings),
    ]);

    let help_menu = MenuNode::submenu("Help", vec![
        // About will move to app menu on macOS
        MenuNode::about("About My App", Message::ShowAbout),
        MenuNode::item("Documentation", Message::OpenDocs),
    ]);

    Some(AppMenu::new(vec![file_menu, edit_menu, help_menu]))
}
```

On **macOS**, the resulting menus look like:

**Application Menu (e.g., "My App"):**
- About My App
- ---
- Settings… (⌘,)
- ---
- Quit My App (⌘Q)

**File Menu:**
- New
- Open…

**Edit Menu:**
- Undo
- Redo

**Help Menu:**
- Documentation

