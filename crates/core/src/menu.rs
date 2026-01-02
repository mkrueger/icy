//! Application menu model.
//!
//! This module defines a platform-agnostic representation of an application's
//! main menu. Backends can render it as a native menu bar (e.g. macOS) or as an
//! in-window widget menu (e.g. Windows/Linux).
//!
//! # Macros for Stable IDs
//!
//! Use the provided macros to create menu items with automatically stable IDs
//! based on a compile-time hash of the source location:
//!
//! ```ignore
//! use icy_ui_core::menu;
//!
//! let file_menu = menu::submenu!("File", [
//!     menu::item!("New", Message::New),
//!     menu::item!("Open", Message::Open),
//!     menu::separator!(),
//!     menu::item!("Save", Message::Save),
//! ]);
//! ```
//!
//! For dynamic menus (e.g., window lists), use the `*_with_id` functions directly:
//!
//! ```ignore
//! for (i, window) in windows.iter().enumerate() {
//!     items.push(MenuNode::item_with_id(
//!         MenuId::from_u64(base_id.as_u64().wrapping_add(i as u64)),
//!         &window.title,
//!         Message::FocusWindow(window.id),
//!     ));
//! }
//! ```

use crate::keyboard;
use crate::window;

/// Platform-specific menu item role.
///
/// Items with a role may be relocated by the platform backend.
/// For example, macOS moves `Quit`, `About`, and `Preferences` items
/// into the application menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum MenuRole {
    /// No special handling; item stays where defined.
    #[default]
    None,

    /// "About <AppName>" – macOS: app menu; others: as defined.
    About,

    /// "Settings…" / "Preferences…" – macOS: app menu (⌘,); others: as defined.
    Preferences,

    /// "Quit <AppName>" – macOS: app menu (⌘Q); others: as defined.
    Quit,

    /// Application-specific item to be relocated to the app menu on macOS.
    ///
    /// Use this for custom items that should appear in the application menu
    /// but don't fit the standard About/Preferences/Quit roles.
    /// On macOS these appear between Preferences and the separator before Quit.
    ApplicationSpecific,
}

/// Stable identifier for a menu item.
///
/// This is a 64-bit hash computed at compile time from the source location.
/// Apps are responsible for keeping these stable across updates so
/// platform backends can patch native menus efficiently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MenuId(pub u64);

impl MenuId {
    /// Creates a new [`MenuId`] from a raw u64 value.
    #[must_use]
    pub const fn from_u64(id: u64) -> Self {
        Self(id)
    }

    /// Returns the raw u64 value.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Creates a [`MenuId`] by hashing a string at compile time.
    #[must_use]
    pub const fn from_str(s: &str) -> Self {
        Self(fnv1a_hash_str(s))
    }

    /// Derives a deterministic child [`MenuId`] from this ID and a numeric value.
    ///
    /// Useful for dynamic menus (e.g. window lists) where you want stable IDs
    /// without relying on source location.
    ///
    /// # Example
    /// ```ignore
    /// use icy_ui_core::menu::MenuId;
    ///
    /// let base = MenuId::from_str("window");
    /// let first = base.child(0);
    /// let second = base.child(1);
    /// assert_ne!(first, second);
    /// ```
    #[must_use]
    pub const fn child(self, value: u64) -> Self {
        Self(fnv1a_hash_u64_pair(self.0, value))
    }

    /// Derives a deterministic child [`MenuId`] from this ID and a string.
    ///
    /// This is a convenience wrapper around hashing the string and combining it.
    #[must_use]
    pub const fn child_str(self, value: &str) -> Self {
        Self(fnv1a_hash_u64_pair(self.0, fnv1a_hash_str(value)))
    }

    /// Creates a [`MenuId`] from file and line number.
    #[must_use]
    pub const fn from_location(file: &str, line: u32) -> Self {
        Self(fnv1a_hash_location(file, line))
    }
}

// ============================================================================
// FNV-1a Hash (compile-time capable)
// ============================================================================

/// FNV-1a 64-bit offset basis.
const FNV1A_OFFSET: u64 = 0xcbf29ce484222325;

/// FNV-1a 64-bit prime.
const FNV1A_PRIME: u64 = 0x00000100000001B3;

/// Computes FNV-1a hash of a string at compile time.
#[must_use]
pub const fn fnv1a_hash_str(s: &str) -> u64 {
    let bytes = s.as_bytes();
    let mut hash = FNV1A_OFFSET;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV1A_PRIME);
        i += 1;
    }
    hash
}

/// Computes FNV-1a hash of file path and line number at compile time.
#[must_use]
pub const fn fnv1a_hash_location(file: &str, line: u32) -> u64 {
    let bytes = file.as_bytes();
    let mut hash = FNV1A_OFFSET;

    // Hash file path
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV1A_PRIME);
        i += 1;
    }

    // Add separator
    hash ^= b':' as u64;
    hash = hash.wrapping_mul(FNV1A_PRIME);

    // Hash line number (as bytes)
    let line_bytes = line.to_be_bytes();
    let mut j = 0;
    while j < 4 {
        hash ^= line_bytes[j] as u64;
        hash = hash.wrapping_mul(FNV1A_PRIME);
        j += 1;
    }

    hash
}

/// Computes FNV-1a hash of two u64 values (big-endian byte order) at compile time.
#[must_use]
const fn fnv1a_hash_u64_pair(a: u64, b: u64) -> u64 {
    let mut hash = FNV1A_OFFSET;

    let a_bytes = a.to_be_bytes();
    let mut i = 0;
    while i < 8 {
        hash ^= a_bytes[i] as u64;
        hash = hash.wrapping_mul(FNV1A_PRIME);
        i += 1;
    }

    let b_bytes = b.to_be_bytes();
    let mut j = 0;
    while j < 8 {
        hash ^= b_bytes[j] as u64;
        hash = hash.wrapping_mul(FNV1A_PRIME);
        j += 1;
    }

    hash
}

/// A keyboard shortcut displayed in menus and/or bound by platform backends.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MenuShortcut {
    /// Modifier keys.
    pub modifiers: keyboard::Modifiers,
    /// Trigger key.
    pub key: keyboard::Key,
}

impl MenuShortcut {
    /// Creates a new [`MenuShortcut`].
    #[must_use]
    pub fn new(modifiers: keyboard::Modifiers, key: keyboard::Key) -> Self {
        Self { modifiers, key }
    }

    /// Creates a shortcut with Command/Ctrl + the given key.
    ///
    /// On macOS this uses ⌘, on other platforms Ctrl.
    #[must_use]
    pub fn cmd(key: keyboard::Key) -> Self {
        Self {
            modifiers: keyboard::Modifiers::COMMAND,
            key,
        }
    }

    /// Creates a shortcut with Command/Ctrl + Shift + the given key.
    #[must_use]
    pub fn cmd_shift(key: keyboard::Key) -> Self {
        Self {
            modifiers: keyboard::Modifiers::COMMAND.union(keyboard::Modifiers::SHIFT),
            key,
        }
    }

    /// Creates a shortcut with Command/Ctrl + Alt + the given key.
    #[must_use]
    pub fn cmd_alt(key: keyboard::Key) -> Self {
        Self {
            modifiers: keyboard::Modifiers::COMMAND.union(keyboard::Modifiers::ALT),
            key,
        }
    }

    /// Creates a shortcut with Shift + the given key.
    #[must_use]
    pub fn shift(key: keyboard::Key) -> Self {
        Self {
            modifiers: keyboard::Modifiers::SHIFT,
            key,
        }
    }

    /// Creates a shortcut with Alt + the given key.
    #[must_use]
    pub fn alt(key: keyboard::Key) -> Self {
        Self {
            modifiers: keyboard::Modifiers::ALT,
            key,
        }
    }

    /// Creates a shortcut with just the given key (no modifiers).
    #[must_use]
    pub fn key_only(key: keyboard::Key) -> Self {
        Self {
            modifiers: keyboard::Modifiers::empty(),
            key,
        }
    }
}

/// Current window information provided to `Program::application_menu`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowInfo {
    /// Window identifier.
    pub id: window::Id,
    /// Current window title.
    pub title: String,
    /// Whether the window is currently focused.
    pub focused: bool,
    /// Whether the window is currently minimized.
    pub minimized: bool,
}

/// Context provided to `Program::application_menu`.
#[derive(Debug, Default, Clone)]
pub struct MenuContext {
    /// Snapshot of currently known windows.
    pub windows: Vec<WindowInfo>,
}

/// A full application menu model.
#[derive(Debug, Clone)]
pub struct AppMenu<Message> {
    /// Top-level menu roots (e.g. File/Edit/View/Window/Help).
    pub roots: Vec<MenuNode<Message>>,
}

impl<Message> AppMenu<Message> {
    /// Creates a new [`AppMenu`].
    #[must_use]
    pub fn new(roots: Vec<MenuNode<Message>>) -> Self {
        Self { roots }
    }
}

/// A menu node.
#[derive(Debug, Clone)]
pub struct MenuNode<Message> {
    /// Stable identifier of this node.
    pub id: MenuId,
    /// Platform-specific role for this node.
    ///
    /// Nodes with a role may be relocated by the platform backend
    /// (e.g. macOS moves `Quit` to the application menu).
    pub role: MenuRole,
    /// Node contents.
    pub kind: MenuKind<Message>,
}

impl<Message> MenuNode<Message> {
    /// Creates a new [`MenuNode`] with the given ID.
    ///
    /// For automatic stable IDs, prefer using the macros like [`item!`], [`submenu!`], etc.
    #[must_use]
    pub fn new_with_id(id: MenuId, kind: MenuKind<Message>) -> Self {
        Self {
            id,
            role: MenuRole::None,
            kind,
        }
    }

    /// Sets the role of this node.
    #[must_use]
    pub fn with_role(mut self, role: MenuRole) -> Self {
        self.role = role;
        self
    }

    /// Creates a separator [`MenuNode`] with the given ID.
    ///
    /// For automatic stable IDs, prefer using [`separator!`].
    #[must_use]
    pub fn separator_with_id(id: MenuId) -> Self {
        Self::new_with_id(id, MenuKind::Separator)
    }

    /// Creates a submenu [`MenuNode`] with the given ID.
    ///
    /// For automatic stable IDs, prefer using [`submenu!`].
    #[must_use]
    pub fn submenu_with_id(
        id: MenuId,
        label: impl Into<String>,
        children: Vec<MenuNode<Message>>,
    ) -> Self {
        Self::new_with_id(
            id,
            MenuKind::Submenu {
                label: label.into(),
                enabled: true,
                children,
            },
        )
    }

    /// Creates a clickable menu item [`MenuNode`] with the given ID.
    ///
    /// For automatic stable IDs, prefer using [`item!`].
    #[must_use]
    pub fn item_with_id(id: MenuId, label: impl Into<String>, on_activate: Message) -> Self {
        Self::new_with_id(
            id,
            MenuKind::Item {
                label: label.into(),
                enabled: true,
                shortcut: None,
                on_activate,
            },
        )
    }

    /// Creates a checkbox menu item [`MenuNode`] with the given ID.
    ///
    /// For automatic stable IDs, prefer using [`check_item!`].
    #[must_use]
    pub fn check_item_with_id(
        id: MenuId,
        label: impl Into<String>,
        checked: bool,
        on_activate: Message,
    ) -> Self {
        Self::new_with_id(
            id,
            MenuKind::CheckItem {
                label: label.into(),
                enabled: true,
                checked,
                shortcut: None,
                on_activate,
            },
        )
    }

    /// Creates a "Quit" menu item with [`MenuRole::Quit`].
    ///
    /// On macOS this will be relocated to the application menu with ⌘Q.
    #[must_use]
    pub fn quit(on_activate: Message) -> Self {
        Self {
            id: MenuId::from_str("app.quit"),
            role: MenuRole::Quit,
            kind: MenuKind::Item {
                label: "Quit".into(),
                enabled: true,
                shortcut: None,
                on_activate,
            },
        }
    }

    /// Creates an "About" menu item with [`MenuRole::About`].
    ///
    /// On macOS this will be relocated to the application menu.
    #[must_use]
    pub fn about(label: impl Into<String>, on_activate: Message) -> Self {
        Self {
            id: MenuId::from_str("app.about"),
            role: MenuRole::About,
            kind: MenuKind::Item {
                label: label.into(),
                enabled: true,
                shortcut: None,
                on_activate,
            },
        }
    }

    /// Creates a "Preferences" / "Settings" menu item with [`MenuRole::Preferences`].
    ///
    /// On macOS this will be relocated to the application menu with ⌘,.
    #[must_use]
    pub fn preferences(label: impl Into<String>, on_activate: Message) -> Self {
        Self {
            id: MenuId::from_str("app.preferences"),
            role: MenuRole::Preferences,
            kind: MenuKind::Item {
                label: label.into(),
                enabled: true,
                shortcut: None,
                on_activate,
            },
        }
    }

    /// Returns `true` if this node is a submenu with no visible children.
    ///
    /// This is useful for hiding menus that became empty after role-based
    /// items were relocated (e.g., a Help menu with only an About item on macOS).
    #[must_use]
    pub fn is_empty_submenu(&self) -> bool {
        match &self.kind {
            MenuKind::Submenu { children, .. } => children
                .iter()
                .all(|c| c.role != MenuRole::None || c.is_empty_submenu()),
            _ => false,
        }
    }

    /// Sets a keyboard shortcut for this menu item.
    ///
    /// This only has an effect on `Item` and `CheckItem` nodes.
    ///
    /// # Example
    /// ```ignore
    /// use icy_ui_core::menu::{self, MenuShortcut};
    /// use icy_ui_core::keyboard::Key;
    ///
    /// menu::item!("Save", Message::Save)
    ///     .shortcut(MenuShortcut::cmd(Key::Character("s".into())))
    /// ```
    #[must_use]
    pub fn shortcut(mut self, shortcut: MenuShortcut) -> Self {
        match &mut self.kind {
            MenuKind::Item { shortcut: s, .. } => *s = Some(shortcut),
            MenuKind::CheckItem { shortcut: s, .. } => *s = Some(shortcut),
            _ => {}
        }
        self
    }

    /// Sets whether this menu item is enabled.
    ///
    /// This only has an effect on `Item`, `CheckItem`, and `Submenu` nodes.
    #[must_use]
    pub fn enabled(mut self, enabled_val: bool) -> Self {
        match &mut self.kind {
            MenuKind::Item { enabled: e, .. } => *e = enabled_val,
            MenuKind::CheckItem { enabled: e, .. } => *e = enabled_val,
            MenuKind::Submenu { enabled: e, .. } => *e = enabled_val,
            _ => {}
        }
        self
    }
}

/// The concrete type of a menu node.
#[derive(Debug, Clone)]
pub enum MenuKind<Message> {
    /// A clickable menu item.
    Item {
        /// Text label.
        label: String,
        /// Whether the item is enabled.
        enabled: bool,
        /// Optional keyboard shortcut.
        shortcut: Option<MenuShortcut>,
        /// Message produced when the item is activated.
        on_activate: Message,
    },

    /// A clickable menu item with a checkmark.
    CheckItem {
        /// Text label.
        label: String,
        /// Whether the item is enabled.
        enabled: bool,
        /// Whether the item is checked.
        checked: bool,
        /// Optional keyboard shortcut.
        shortcut: Option<MenuShortcut>,
        /// Message produced when the item is activated.
        on_activate: Message,
    },

    /// A submenu.
    Submenu {
        /// Text label.
        label: String,
        /// Whether the submenu can be opened.
        enabled: bool,
        /// Child menu nodes.
        children: Vec<MenuNode<Message>>,
    },

    /// A separator/divider.
    Separator,
}

// ============================================================================
// Macros for creating menu items with stable source-location-based IDs
// ============================================================================

/// Creates a menu separator with a stable ID based on source location hash.
///
/// # Example
/// ```ignore
/// use icy_ui_core::menu;
///
/// let sep = menu::separator!();
/// ```
#[macro_export]
macro_rules! menu_separator {
    () => {
        $crate::menu::MenuNode::separator_with_id($crate::menu::MenuId::from_location(
            file!(),
            line!(),
        ))
    };
    (id = $id:expr $(,)?) => {
        $crate::menu::MenuNode::separator_with_id($id)
    };
}

/// Creates a menu item with a stable ID based on source location hash.
///
/// # Examples
/// ```ignore
/// use icy_ui_core::menu::{self, MenuShortcut};
/// use icy_ui_core::keyboard::Key;
///
/// // Without shortcut
/// let item = menu::item!("Open", Message::Open);
///
/// // With shortcut
/// let item = menu::item!("Save", Message::Save, MenuShortcut::cmd(Key::Character("s".into())));
///
/// // Or use the builder pattern
/// let item = menu::item!("Save", Message::Save)
///     .shortcut(MenuShortcut::cmd(Key::Character("s".into())));
/// ```
#[macro_export]
macro_rules! menu_item {
    ($label:expr, $on_activate:expr $(,)?) => {
        $crate::menu::MenuNode::item_with_id(
            $crate::menu::MenuId::from_location(file!(), line!()),
            $label,
            $on_activate,
        )
    };
    ($label:expr, $on_activate:expr, $shortcut:expr $(,)?) => {
        $crate::menu::MenuNode::item_with_id(
            $crate::menu::MenuId::from_location(file!(), line!()),
            $label,
            $on_activate,
        )
        .shortcut($shortcut)
    };
    ($label:expr, $on_activate:expr, id = $id:expr $(,)?) => {
        $crate::menu::MenuNode::item_with_id($id, $label, $on_activate)
    };
    ($label:expr, $on_activate:expr, $shortcut:expr, id = $id:expr $(,)?) => {
        $crate::menu::MenuNode::item_with_id($id, $label, $on_activate).shortcut($shortcut)
    };
}

/// Creates a checkbox menu item with a stable ID based on source location hash.
///
/// # Examples
/// ```ignore
/// use icy_ui_core::menu::{self, MenuShortcut};
/// use icy_ui_core::keyboard::Key;
///
/// // Without shortcut
/// let item = menu::check_item!("Dark Mode", state.dark_mode, Message::ToggleDarkMode);
///
/// // With shortcut
/// let item = menu::check_item!("Dark Mode", state.dark_mode, Message::ToggleDarkMode,
///     MenuShortcut::cmd(Key::Character("d".into())));
/// ```
#[macro_export]
macro_rules! menu_check_item {
    ($label:expr, $checked:expr, $on_activate:expr $(,)?) => {
        $crate::menu::MenuNode::check_item_with_id(
            $crate::menu::MenuId::from_location(file!(), line!()),
            $label,
            $checked,
            $on_activate,
        )
    };
    ($label:expr, $checked:expr, $on_activate:expr, $shortcut:expr $(,)?) => {
        $crate::menu::MenuNode::check_item_with_id(
            $crate::menu::MenuId::from_location(file!(), line!()),
            $label,
            $checked,
            $on_activate,
        )
        .shortcut($shortcut)
    };
    ($label:expr, $checked:expr, $on_activate:expr, id = $id:expr $(,)?) => {
        $crate::menu::MenuNode::check_item_with_id($id, $label, $checked, $on_activate)
    };
    ($label:expr, $checked:expr, $on_activate:expr, $shortcut:expr, id = $id:expr $(,)?) => {
        $crate::menu::MenuNode::check_item_with_id($id, $label, $checked, $on_activate)
            .shortcut($shortcut)
    };
}

/// Creates a submenu with a stable ID based on source location hash.
///
/// # Example
/// ```ignore
/// use icy_ui_core::menu;
///
/// let file_menu = menu::submenu!("File", [
///     menu::item!("New", Message::New),
///     menu::item!("Open", Message::Open),
///     menu::separator!(),
///     menu::item!("Save", Message::Save),
/// ]);
/// ```
#[macro_export]
macro_rules! menu_submenu {
    ($label:expr, [$($child:expr),* $(,)?] $(,)?) => {
        $crate::menu::MenuNode::submenu_with_id(
            $crate::menu::MenuId::from_location(file!(), line!()),
            $label,
            vec![$($child),*],
        )
    };
    ($label:expr, [$($child:expr),* $(,)?], id = $id:expr $(,)?) => {
        $crate::menu::MenuNode::submenu_with_id($id, $label, vec![$($child),*])
    };
}

/// Creates a "Quit" menu item with [`MenuRole::Quit`].
///
/// On macOS this will be relocated to the application menu with ⌘Q.
///
/// # Example
/// ```ignore
/// use icy_ui_core::menu;
///
/// let quit = menu::quit!(Message::Quit);
/// ```
#[macro_export]
macro_rules! menu_quit {
    ($on_activate:expr $(,)?) => {
        $crate::menu::MenuNode::quit($on_activate)
    };
    ($on_activate:expr, id = $id:expr $(,)?) => {{
        let mut node = $crate::menu::MenuNode::quit($on_activate);
        node.id = $id;
        node
    }};
}

/// Creates an "About" menu item with [`MenuRole::About`].
///
/// On macOS this will be relocated to the application menu.
///
/// # Example
/// ```ignore
/// use icy_ui_core::menu;
///
/// let about = menu::about!("About My App", Message::About);
/// ```
#[macro_export]
macro_rules! menu_about {
    ($label:expr, $on_activate:expr $(,)?) => {
        $crate::menu::MenuNode::about($label, $on_activate)
    };
    ($label:expr, $on_activate:expr, id = $id:expr $(,)?) => {{
        let mut node = $crate::menu::MenuNode::about($label, $on_activate);
        node.id = $id;
        node
    }};
}

/// Creates a "Preferences" menu item with [`MenuRole::Preferences`].
///
/// On macOS this will be relocated to the application menu with ⌘,.
///
/// # Example
/// ```ignore
/// use icy_ui_core::menu;
///
/// let prefs = menu::preferences!("Preferences…", Message::Preferences);
/// ```
#[macro_export]
macro_rules! menu_preferences {
    ($label:expr, $on_activate:expr $(,)?) => {
        $crate::menu::MenuNode::preferences($label, $on_activate)
    };
    ($label:expr, $on_activate:expr, id = $id:expr $(,)?) => {{
        let mut node = $crate::menu::MenuNode::preferences($label, $on_activate);
        node.id = $id;
        node
    }};
}

// Re-export macros under the menu module for nicer syntax: menu::item!(), menu::submenu!(), etc.
#[doc(inline)]
pub use crate::menu_about as about;
#[doc(inline)]
pub use crate::menu_check_item as check_item;
#[doc(inline)]
pub use crate::menu_item as item;
#[doc(inline)]
pub use crate::menu_preferences as preferences;
#[doc(inline)]
pub use crate::menu_quit as quit;
#[doc(inline)]
pub use crate::menu_separator as separator;
#[doc(inline)]
pub use crate::menu_submenu as submenu;

// ============================================================================
// Context Menu Items (for native platform menus)
// ============================================================================

/// A simplified menu item for native context menus.
///
/// Unlike [`MenuNode`], this type has no generic `Message` parameter
/// and no callbacks. It's designed for sending to platform-specific
/// menu implementations. When an item is selected, the [`MenuId`] is
/// returned and the widget that initiated the menu handles the callback.
#[derive(Debug, Clone)]
pub struct ContextMenuItem {
    /// Unique identifier for this menu item.
    pub id: MenuId,
    /// The kind of menu item.
    pub kind: ContextMenuItemKind,
}

/// The kind of a context menu item.
#[derive(Debug, Clone)]
pub enum ContextMenuItemKind {
    /// A regular menu item that can be activated.
    Item {
        /// The label text.
        label: String,
        /// Whether the item is enabled.
        enabled: bool,
    },
    /// A separator line.
    Separator,
    /// A submenu with children.
    Submenu {
        /// The label text.
        label: String,
        /// The child items.
        children: Vec<ContextMenuItem>,
    },
    /// A checkable menu item.
    CheckItem {
        /// The label text.
        label: String,
        /// Whether the item is enabled.
        enabled: bool,
        /// Whether the item is currently checked.
        checked: bool,
    },
}

impl ContextMenuItem {
    /// Creates a new regular menu item.
    pub fn item(id: MenuId, label: impl Into<String>, enabled: bool) -> Self {
        Self {
            id,
            kind: ContextMenuItemKind::Item {
                label: label.into(),
                enabled,
            },
        }
    }

    /// Creates a separator.
    pub fn separator(id: MenuId) -> Self {
        Self {
            id,
            kind: ContextMenuItemKind::Separator,
        }
    }

    /// Creates a submenu.
    pub fn submenu(id: MenuId, label: impl Into<String>, children: Vec<ContextMenuItem>) -> Self {
        Self {
            id,
            kind: ContextMenuItemKind::Submenu {
                label: label.into(),
                children,
            },
        }
    }

    /// Creates a checkable item.
    pub fn check_item(id: MenuId, label: impl Into<String>, enabled: bool, checked: bool) -> Self {
        Self {
            id,
            kind: ContextMenuItemKind::CheckItem {
                label: label.into(),
                enabled,
                checked,
            },
        }
    }

    /// Converts a slice of [`MenuNode`]s to a vector of [`ContextMenuItem`]s.
    ///
    /// This extracts the label, enabled state, and structure from the nodes
    /// while discarding the callbacks (which are handled by the widget).
    pub fn from_menu_nodes<Message>(nodes: &[MenuNode<Message>]) -> Vec<ContextMenuItem> {
        nodes
            .iter()
            .map(|node| {
                let id = node.id.clone();
                match &node.kind {
                    MenuKind::Item { label, enabled, .. } => {
                        ContextMenuItem::item(id, label.clone(), *enabled)
                    }
                    MenuKind::Separator => ContextMenuItem::separator(id),
                    MenuKind::Submenu {
                        label, children, ..
                    } => ContextMenuItem::submenu(
                        id,
                        label.clone(),
                        ContextMenuItem::from_menu_nodes(children),
                    ),
                    MenuKind::CheckItem {
                        label,
                        enabled,
                        checked,
                        ..
                    } => ContextMenuItem::check_item(id, label.clone(), *enabled, *checked),
                }
            })
            .collect()
    }
}
