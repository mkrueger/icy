# Theme System

The theme system provides a modern, libcosmic-compatible theming architecture for iced applications. It replaces the previous enum-based theme with a flexible struct-based approach that supports custom color palettes, container layers, and component states.

## Overview

The theme system is organized into several key concepts:

- **Theme** - The main theme struct containing all styling information
- **Palette** - Raw color values (neutrals, accents, semantic colors)
- **Container** - Background layer colors (background, primary, secondary)
- **Component** - Widget state colors (base, hover, pressed, disabled, etc.)
- **Spacing** - Standardized spacing values
- **CornerRadii** - Standardized border radius values

## Quick Start

```rust
use icy_ui::Theme;

// Use built-in themes
let dark = Theme::dark();
let light = Theme::light();

// Create a custom theme
use icy_ui::theme::Palette;
let custom = Theme::custom("My Theme", Palette::dark());

// Get all available themes
let themes = Theme::all();
```

## Theme Structure

### The Theme Struct

```rust
pub struct Theme {
    /// The name of the theme.
    pub name: String,

    /// The background container (outermost layer).
    pub background: Container,

    /// The primary container (main content areas).
    pub primary: Container,

    /// The secondary container (dialogs, popovers).
    pub secondary: Container,

    /// Accent component colors.
    pub accent: Component,

    /// Success/positive component colors.
    pub success: Component,

    /// Destructive/danger component colors.
    pub destructive: Component,

    /// Warning component colors.
    pub warning: Component,

    /// Various button style components...
    pub accent_button: Component,
    pub success_button: Component,
    pub destructive_button: Component,
    pub warning_button: Component,
    pub icon_button: Component,
    pub link_button: Component,
    pub text_button: Component,
    pub button: Component,

    /// The underlying color palette.
    pub palette: Palette,

    /// Spacing values.
    pub spacing: Spacing,

    /// Corner radii values.
    pub corner_radii: CornerRadii,

    /// Whether this is a dark theme.
    pub is_dark: bool,

    /// Whether this is a high contrast theme.
    pub is_high_contrast: bool,

    /// Window shade/overlay color.
    pub shade: Color,
}
```

## Container Layers

The UI has three depth layers, each represented by a `Container`:

| Layer | Use Case |
|-------|----------|
| `background` | The outermost window background |
| `primary` | Main content areas, cards, panels |
| `secondary` | Dialogs, popovers, tooltips |

### Container Properties

```rust
pub struct Container {
    /// The base background color of the container.
    pub base: Color,

    /// Component colors for widgets within this container.
    pub component: Component,

    /// Divider/separator color.
    pub divider: Color,

    /// Text color on this container.
    pub on: Color,

    /// Background for small widgets (checkboxes, toggles, etc.).
    pub small_widget: Color,
}
```

### Accessing Containers

```rust
// Direct field access
let bg_color = theme.background.base;
let text_color = theme.background.on;

// Or by layer enum
use icy_ui::theme::Layer;
let container = theme.container(Layer::Primary);
```

## Component States

A `Component` defines colors for all interactive states of a widget:

```rust
pub struct Component {
    /// Base/normal state background color.
    pub base: Color,

    /// Hovered state background color.
    pub hover: Color,

    /// Pressed/active state background color.
    pub pressed: Color,

    /// Selected state background color.
    pub selected: Color,

    /// Text color when selected.
    pub selected_text: Color,

    /// Focus indicator color.
    pub focus: Color,

    /// Divider/separator color within the component.
    pub divider: Color,

    /// Text/icon color on the component.
    pub on: Color,

    /// Disabled state background color.
    pub disabled: Color,

    /// Text color when disabled.
    pub on_disabled: Color,

    /// Border color.
    pub border: Color,

    /// Border color when disabled.
    pub disabled_border: Color,
}
```

### Using Components in Widgets

```rust
fn style_button(theme: &Theme, status: button::Status) -> button::Style {
    let component = &theme.button;
    
    let (background, text_color) = match status {
        Status::Active => (component.base, component.on),
        Status::Hovered => (component.hover, component.on),
        Status::Pressed => (component.pressed, component.on),
        Status::Disabled => (component.disabled, component.on_disabled),
    };
    
    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        ..Default::default()
    }
}
```

## Palette

The `Palette` contains all raw color values used to build the theme:

### Neutral Colors

11 neutral grays from 0 (darkest) to 10 (lightest):

```rust
// Access by index
let mid_gray = palette.neutral(5);

// Or directly
let dark = palette.neutral_2;
let light = palette.neutral_8;
```

### Semantic Colors

```rust
palette.bright_red     // Errors, destructive actions
palette.bright_green   // Success states
palette.bright_orange  // Warnings
```

### Accent Colors

```rust
palette.accent_blue
palette.accent_indigo
palette.accent_purple
palette.accent_pink
palette.accent_red
palette.accent_orange
palette.accent_yellow
palette.accent_green
palette.accent_warm_grey

// Get the current accent (defaults to blue)
let accent = palette.accent();
```

## Spacing

Consistent spacing values throughout the UI:

```rust
pub struct Spacing {
    pub none: u16,   // 0
    pub xxxs: u16,   // 2
    pub xxs: u16,    // 4
    pub xs: u16,     // 8
    pub s: u16,      // 12
    pub m: u16,      // 16 (default)
    pub l: u16,      // 24
    pub xl: u16,     // 32
    pub xxl: u16,    // 48
    pub xxxl: u16,   // 64
}
```

### Spacing Presets

```rust
Spacing::default()     // Standard spacing
Spacing::compact()     // Tighter spacing for dense UIs
Spacing::comfortable() // Looser spacing for touch/accessibility
```

## Corner Radii

```rust
pub struct CornerRadii {
    pub none: [f32; 4],   // [0, 0, 0, 0]
    pub xs: [f32; 4],     // [2, 2, 2, 2]
    pub s: [f32; 4],      // [4, 4, 4, 4]
    pub m: [f32; 4],      // [8, 8, 8, 8]
    pub l: [f32; 4],      // [12, 12, 12, 12]
    pub xl: [f32; 4],     // [16, 16, 16, 16]
    pub full: [f32; 4],   // [999, 999, 999, 999] (pill shape)
}
```

## Creating Custom Themes

### From a Palette

```rust
use icy_ui::theme::{Theme, Palette};

// Start with a base palette
let mut palette = Palette::dark();

// Customize colors
palette.accent_blue = Color::from_rgb(0.2, 0.6, 1.0);
palette.name = "My Dark Theme".into();

// Create the theme
let theme = Theme::custom("My Dark Theme", palette);
```

### Loading from Files

The theme system supports loading from RON files (requires `serde` feature):

```rust
use icy_ui::theme::load_theme_from_file;

let theme = load_theme_from_file("my-theme.ron")?;
```

#### RON Theme Format

```ron
// my-theme.ron
(
    name: "Custom Dark",
    bright_red: (r: 1.0, g: 0.3, b: 0.3, a: 1.0),
    bright_green: (r: 0.3, g: 0.8, b: 0.4, a: 1.0),
    // ... other palette fields
)
```

### System Theme (Pop!_OS / COSMIC)

On Pop!_OS, you can load the system theme:

```rust
use icy_ui::theme::load_system_theme;

let theme = load_system_theme(true)?; // prefer_dark = true
```

## The Base Trait

For theme polymorphism (e.g., in generic widgets), the `Base` trait provides a common interface:

```rust
pub trait Base: Sized {
    /// Returns the mode (light or dark) of the theme.
    fn mode(&self) -> Mode;

    /// Returns the name of the theme.
    fn name(&self) -> &str;

    /// Returns the base application Style of the theme.
    fn base(&self) -> Style;

    /// Returns the color Palette of the theme (for debugging).
    fn palette(&self) -> Option<Palette>;

    /// Returns a default theme for the given mode.
    fn default(mode: Mode) -> Self;
}
```

### Mode Enum

```rust
pub enum Mode {
    None,   // No specific mode (default)
    Light,  // Light mode
    Dark,   // Dark mode
}
```

### Style Struct

The base application style:

```rust
pub struct Style {
    pub background_color: Color,
    pub text_color: Color,
}
```

## Widget Styling

Widgets use the theme's components for consistent styling:

### Button Example

```rust
// Use the theme's button component
let style = button::Style {
    background: Some(Background::Color(theme.button.base)),
    text_color: theme.button.on,
    border: Border {
        color: theme.button.border,
        width: 1.0,
        radius: theme.corner_radii.s.into(),
    },
    ..Default::default()
};
```

### Semantic Buttons

```rust
// Accent (primary action)
let accent_bg = theme.accent_button.base;

// Success (confirm, save)
let success_bg = theme.success_button.base;

// Destructive (delete, remove)  
let destructive_bg = theme.destructive_button.base;

// Warning (caution)
let warning_bg = theme.warning_button.base;
```

### Text Colors

```rust
// Text on background layer
let text = theme.background.on;

// Text on primary content area
let text = theme.primary.on;

// Convenience methods
let text = theme.on_background();
let text = theme.on_primary();
let text = theme.on_secondary();
```

## Migration from Old Theme

If migrating from the previous enum-based theme:

### Before (Old Theme)

```rust
// Old style
let theme = Theme::Dark;
let palette = theme.extended_palette();
let bg = palette.background.base.color;
```

### After (New Theme)

```rust
// New style
let theme = Theme::dark();
let bg = theme.background.base;
```

### Key Differences

| Old | New |
|-----|-----|
| `Theme::Dark` (enum variant) | `Theme::dark()` (constructor) |
| `theme.extended_palette()` | Direct field access on theme |
| `palette.background.base.color` | `theme.background.base` |
| `theme.palette()` | `theme.palette.clone()` |
| Static `Theme::ALL` | `Theme::all()` returns `Vec` |

## Examples

### Theme Selector

```rust
#[derive(Default)]
struct App {
    theme: Theme,
}

enum Message {
    ThemeChanged(Theme),
}

fn update(&mut self, message: Message) {
    match message {
        Message::ThemeChanged(theme) => {
            self.theme = theme;
        }
    }
}

fn view(&self) -> Element<Message> {
    let themes = Theme::all();
    
    pick_list(
        themes.clone(),
        Some(self.theme.clone()),
        Message::ThemeChanged,
    )
    .into()
}

fn theme(&self) -> Theme {
    self.theme.clone()
}
```

### Custom Accent Color

```rust
let mut palette = Palette::dark();
palette.accent_blue = Color::from_rgb(0.0, 0.8, 0.6); // Teal accent

let theme = Theme::from_palette(palette, true);
```

## Platform Integration

### Pop!_OS / COSMIC Desktop

The theme system is designed for compatibility with libcosmic themes:

- Loads system themes from `cosmic-config` directories
- Uses the same color structure as COSMIC apps
- Supports RON palette files

### Other Platforms

On non-COSMIC systems:
- Light and dark themes are always available
- Custom themes can be loaded from RON files
- System theme loading falls back to specified preference
