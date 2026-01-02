# Menu System

icy_ui has **two layers** of menu APIs:

1. **High-level (recommended):** `icy_ui::menu` — a platform-agnostic menu model with macros, shortcuts, and native integration.
   - **Application menu**: native menu bar on macOS; widget-based fallback on Windows/Linux.
   - **Context menu**: native on macOS; widget overlay on other platforms.
2. **Low-level widget menus:** `icy_ui::widget::menu` — a fully custom in-window menu system (MenuBar/MenuTree/etc.).

In most apps you want `icy_ui::menu` as the **default entry**.

## Recommended: `icy_ui::menu`

### Application Menu (native menu bar on macOS)

Define an `AppMenu<Message>` using the `menu::*` macros. IDs are stable by default (based on source location), and you can pin them explicitly via `id = ...`.

```rust
use icy_ui::menu::{self, AppMenu, MenuContext, MenuId, MenuShortcut, MenuNode};
use icy_ui::keyboard::Key;

fn application_menu(context: &MenuContext) -> Option<AppMenu<Message>> {
    let file_menu = menu::submenu!("File", [
        menu::item!("New", Message::New, MenuShortcut::cmd(Key::Character("n".into()))),
        menu::item!("Open…", Message::Open, MenuShortcut::cmd(Key::Character("o".into()))),
        menu::separator!(),
        // macOS will relocate this to the Application menu with ⌘Q
        menu::quit!(Message::Quit),
    ]);

    let window_menu = menu::submenu!("Window", [
        // Example dynamic menu: pin IDs deterministically
        // (instead of relying on file!/line! which changes when you edit code)
        MenuNode::submenu("Windows", context
            .windows
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let id = MenuId::from_str("window").child(i as u64);
                MenuNode::item_with_id(id, &w.title, Message::FocusWindow(w.id))
            })
            .collect()),
    ]);

    Some(AppMenu::new(vec![file_menu, window_menu]))
}
```

Enable the application menu in the builder:

```rust
fn main() -> icy_ui::Result {
    icy_ui::application(MyApp::default, MyApp::update, MyApp::view)
        .application_menu(MyApp::application_menu)
        .run()
}
```

### Context Menus (native on macOS)

Use the `context_menu` widget with `Vec<MenuNode<Message>>`. The widget handles native menu display and routes selection back into your `Message` automatically.

```rust
use icy_ui::menu;
use icy_ui::widget::menu::context_menu;

let nodes = vec![
    menu::item!("Cut", Message::Cut),
    menu::item!("Copy", Message::Copy),
    menu::separator!(),
    menu::item!("Paste", Message::Paste),
];

let content = my_content();
let with_menu = context_menu(content, &nodes);
```

### Keyboard Shortcuts

Shortcuts are part of the menu model (`MenuShortcut`). On macOS they show up as native key equivalents.

```rust
use icy_ui::menu::{self, MenuShortcut};
use icy_ui::keyboard::Key;

menu::item!("Save", Message::Save, MenuShortcut::cmd(Key::Character("s".into())));
```

#### Parsing Shortcuts from Strings

For user-configurable shortcuts (e.g., from config files), use `FromStr`:

```rust
use icy_ui::menu::MenuShortcut;

// Parse from string
let shortcut: MenuShortcut = "cmd+s".parse().unwrap();
let shortcut2: MenuShortcut = "Ctrl+Shift+N".parse().unwrap();
let shortcut3: MenuShortcut = "F5".parse().unwrap();

// macOS symbols also work
let shortcut4: MenuShortcut = "⌘⇧S".parse().unwrap();
```

**Supported modifiers:**
- `cmd`, `command`, `⌘` — Command/Ctrl (platform-dependent)
- `ctrl`, `control`, `⌃` — Control key
- `alt`, `option`, `opt`, `⌥` — Alt/Option key
- `shift`, `⇧` — Shift key
- `super`, `logo`, `win`, `meta` — Logo/Super/Windows key

**Supported keys:**
- Single characters: `a`-`z`, `0`-`9`, punctuation
- Function keys: `f1`-`f35`
- Named keys: `enter`, `return`, `tab`, `space`, `escape`, `esc`, `backspace`, `delete`, `del`, `insert`, `home`, `end`, `pageup`, `pagedown`, `up`, `down`, `left`, `right`
- Aliases: `plus`, `minus`, `comma`, `period`, `slash`, `backslash`, `equal`, `grave`

### Stable IDs (and `id = ...`)

The macros default to a stable ID derived from `file!/line!`. For items that must remain stable across refactors, pin the ID explicitly:

```rust
use icy_ui::menu::{self, MenuId};

menu::item!("Open", Message::Open, id = MenuId::from_str("file.open"));
menu::separator!(id = MenuId::from_str("file.sep"));
menu::submenu!("File", [/* ... */], id = MenuId::from_str("menu.file"));
```

For dynamic lists, derive IDs from a base:

```rust
use icy_ui::menu::MenuId;

let base = MenuId::from_str("recent");
let id0 = base.child(0);
let id_docs = base.child_str("docs");
```

## Low-level: `icy_ui::widget::menu`

The widget menu system (`MenuBar`, `MenuTree`, mnemonics, etc.) is the **low-level API** for building fully custom, in-window menus.
Use it when you want complete control over layout/styling/behavior beyond the standard application/context menu model.

### Basic Usage

```rust
use icy_ui::widget::button;
use icy_ui::widget::menu::{MenuTree, MenuBar};

let sub_menu = MenuTree::with_children(
    button("Sub Menu"),
    vec![
        MenuTree::new(button("Item 1")),
        MenuTree::new(button("Item 2")),
        MenuTree::new(button("Item 3")),
    ],
);

let file_menu = MenuTree::with_children(
    button("File"),
    vec![
        MenuTree::new(button("New")),
        MenuTree::new(button("Open")),
        sub_menu,
        MenuTree::new(button("Save")),
    ],
);

let edit_menu = MenuTree::with_children(
    button("Edit"),
    vec![
        MenuTree::new(button("Cut")),
        MenuTree::new(button("Copy")),
        MenuTree::new(button("Paste")),
    ],
);

let menu_bar = MenuBar::new(vec![file_menu, edit_menu]);
```

### MenuTree

`MenuTree` represents a node in a widget menu hierarchy. It can be either a leaf item or a folder containing children.

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

## ContextMenu (Widget)

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

## Application Menus: `MenuRole` and `MenuContext`

When you build your application menu via `icy_ui::menu`, you can assign roles to let the macOS backend place items in the standard Application menu.

### MenuRole

On **macOS**, items with roles are moved to the Application menu (named after your app). On other platforms, roles are ignored and items stay where defined.

| Role | macOS behavior |
|------|----------------|
| `MenuRole::About` | moved to app menu |
| `MenuRole::Preferences` | moved to app menu (typically ⌘,) |
| `MenuRole::Quit` | moved to app menu (typically ⌘Q) |
| `MenuRole::ApplicationSpecific` | moved to app menu |

Convenience macros/builders:

```rust
use icy_ui::menu::{self, MenuRole, MenuNode};

let about = menu::about!("About My App", Message::ShowAbout);
let prefs = menu::preferences!("Settings…", Message::OpenSettings);
let quit = menu::quit!(Message::Quit);

let license = MenuNode::item("License Info", Message::ShowLicense)
    .with_role(MenuRole::ApplicationSpecific);
```

### MenuContext (dynamic menus)

Use `MenuContext` to build dynamic menus (e.g. window switcher). For stable IDs, prefer `MenuId::child(...)` over relying on source location.

```rust
use icy_ui::menu::{AppMenu, MenuContext, MenuId, MenuNode};

fn application_menu(context: &MenuContext) -> Option<AppMenu<Message>> {
    let windows: Vec<_> = context
        .windows
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let id = MenuId::from_str("window").child(i as u64);
            MenuNode::item_with_id(id, &w.title, Message::FocusWindow(w.id))
        })
        .collect();

    Some(AppMenu::new(vec![MenuNode::submenu("Window", windows)]))
}
```

