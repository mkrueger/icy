//! Theme page

use icy_ui::widget::{button, column, container, pick_list, row, rule, space, text};
use icy_ui::{Color, Element, Theme};

use crate::Message;

// Theme page enums
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemePage {
    #[default]
    Info,
    Container,
    Component,
    Palette,
}

impl std::fmt::Display for ThemePage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemePage::Info => write!(f, "Info"),
            ThemePage::Container => write!(f, "Container"),
            ThemePage::Component => write!(f, "Component"),
            ThemePage::Palette => write!(f, "Palette"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContainerChoice {
    #[default]
    Background,
    Primary,
    Secondary,
}

impl std::fmt::Display for ContainerChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerChoice::Background => write!(f, "Background"),
            ContainerChoice::Primary => write!(f, "Primary"),
            ContainerChoice::Secondary => write!(f, "Secondary"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComponentChoice {
    #[default]
    Accent,
    Success,
    Destructive,
    Warning,
    AccentButton,
    Button,
}

impl std::fmt::Display for ComponentChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentChoice::Accent => write!(f, "Accent"),
            ComponentChoice::Success => write!(f, "Success"),
            ComponentChoice::Destructive => write!(f, "Destructive"),
            ComponentChoice::Warning => write!(f, "Warning"),
            ComponentChoice::AccentButton => write!(f, "Accent Button"),
            ComponentChoice::Button => write!(f, "Button"),
        }
    }
}

#[derive(Default, Clone)]
pub struct ThemePageState {
    pub theme_page: ThemePage,
    pub container_choice: ContainerChoice,
    pub component_choice: ComponentChoice,
}

pub fn update_theme_page(state: &mut ThemePageState, message: &Message) -> bool {
    match message {
        Message::ThemePageChanged(page) => {
            state.theme_page = *page;
            true
        }
        Message::ContainerChoiceChanged(choice) => {
            state.container_choice = *choice;
            true
        }
        Message::ComponentChoiceChanged(choice) => {
            state.component_choice = *choice;
            true
        }
        _ => false,
    }
}

pub fn view_theme(theme: Theme, state: &ThemePageState) -> Element<'static, Message> {
    let is_dark = theme.is_dark;

    // Tab buttons
    let tabs = row![
        theme_tab_button("Info", ThemePage::Info, state.theme_page),
        theme_tab_button("Container", ThemePage::Container, state.theme_page),
        theme_tab_button("Component", ThemePage::Component, state.theme_page),
        theme_tab_button("Palette", ThemePage::Palette, state.theme_page),
    ]
    .spacing(4);

    let page_content: Element<'_, Message> = match state.theme_page {
        ThemePage::Info => view_theme_info(&theme),
        ThemePage::Container => view_theme_container(&theme, state.container_choice),
        ThemePage::Component => view_theme_component(&theme, state.component_choice),
        ThemePage::Palette => view_theme_palette(&theme),
    };

    column![
        text(format!(
            "Current theme: {} ({})",
            theme.name,
            if is_dark { "dark" } else { "light" }
        ))
        .size(14),
        space().height(10),
        tabs,
        rule::horizontal(1),
        page_content,
    ]
    .spacing(8)
    .into()
}

fn view_theme_info(theme: &Theme) -> Element<'static, Message> {
    let spacing = theme.spacing;
    let radii = theme.corner_radii;

    column![
        text("Mode").size(18),
        theme_info_row("Is Dark", theme.is_dark.to_string()),
        theme_info_row("Is High Contrast", theme.is_high_contrast.to_string()),
        theme_info_row("Name", theme.name.clone()),
        rule::horizontal(1),
        text("Spacing").size(18),
        theme_info_row("XXXS", format!("{}", spacing.xxxs)),
        theme_info_row("XXS", format!("{}", spacing.xxs)),
        theme_info_row("XS", format!("{}", spacing.xs)),
        theme_info_row("S", format!("{}", spacing.s)),
        theme_info_row("M", format!("{}", spacing.m)),
        theme_info_row("L", format!("{}", spacing.l)),
        theme_info_row("XL", format!("{}", spacing.xl)),
        theme_info_row("XXL", format!("{}", spacing.xxl)),
        theme_info_row("XXXL", format!("{}", spacing.xxxl)),
        rule::horizontal(1),
        text("Corner Radii").size(18),
        theme_info_row("0", format!("{:?}", radii.radius_0)),
        theme_info_row("XS", format!("{:?}", radii.radius_xs)),
        theme_info_row("S", format!("{:?}", radii.radius_s)),
        theme_info_row("M", format!("{:?}", radii.radius_m)),
        theme_info_row("L", format!("{:?}", radii.radius_l)),
        theme_info_row("XL", format!("{:?}", radii.radius_xl)),
    ]
    .spacing(4)
    .into()
}

fn view_theme_container(theme: &Theme, container_choice: ContainerChoice) -> Element<'static, Message> {
    let c = match container_choice {
        ContainerChoice::Background => &theme.background,
        ContainerChoice::Primary => &theme.primary,
        ContainerChoice::Secondary => &theme.secondary,
    };

    let picker = pick_list(
        vec![
            ContainerChoice::Background,
            ContainerChoice::Primary,
            ContainerChoice::Secondary,
        ],
        Some(container_choice),
        Message::ContainerChoiceChanged,
    )
    .width(200);

    column![
        row![text("Select Container:").size(14), picker]
            .spacing(16)
            .align_y(icy_ui::Alignment::Center),
        rule::horizontal(1),
        text("Container Colors").size(18),
        labeled_color_swatch("Base", c.base),
        labeled_color_swatch("On (Text)", c.on),
        labeled_color_swatch("Divider", c.divider),
        labeled_color_swatch("Small Widget", c.small_widget),
        rule::horizontal(1),
        text("Embedded Component").size(18),
        labeled_color_swatch("Component Base", c.component.base),
        labeled_color_swatch("Component Hover", c.component.hover),
        labeled_color_swatch("Component Pressed", c.component.pressed),
        labeled_color_swatch("Component On", c.component.on),
    ]
    .spacing(4)
    .into()
}

fn view_theme_component(theme: &Theme, component_choice: ComponentChoice) -> Element<'static, Message> {
    let c = match component_choice {
        ComponentChoice::Accent => &theme.accent,
        ComponentChoice::Success => &theme.success,
        ComponentChoice::Destructive => &theme.destructive,
        ComponentChoice::Warning => &theme.warning,
        ComponentChoice::AccentButton => &theme.accent_button,
        ComponentChoice::Button => &theme.button,
    };

    let picker = pick_list(
        vec![
            ComponentChoice::Accent,
            ComponentChoice::Success,
            ComponentChoice::Destructive,
            ComponentChoice::Warning,
            ComponentChoice::AccentButton,
            ComponentChoice::Button,
        ],
        Some(component_choice),
        Message::ComponentChoiceChanged,
    )
    .width(200);

    column![
        row![text("Select Component:").size(14), picker]
            .spacing(16)
            .align_y(icy_ui::Alignment::Center),
        rule::horizontal(1),
        text("State Colors").size(18),
        labeled_color_swatch("Base", c.base),
        labeled_color_swatch("Hover", c.hover),
        labeled_color_swatch("Pressed", c.pressed),
        labeled_color_swatch("Selected", c.selected),
        labeled_color_swatch("Focus", c.focus),
        labeled_color_swatch("Disabled", c.disabled),
        rule::horizontal(1),
        text("Text/Icon Colors").size(18),
        labeled_color_swatch("On (Text)", c.on),
        labeled_color_swatch("Selected Text", c.selected_text),
        labeled_color_swatch("On Disabled", c.on_disabled),
        rule::horizontal(1),
        text("Border & Divider").size(18),
        labeled_color_swatch("Border", c.border),
        labeled_color_swatch("Disabled Border", c.disabled_border),
        labeled_color_swatch("Divider", c.divider),
    ]
    .spacing(4)
    .into()
}

fn view_theme_palette(theme: &Theme) -> Element<'static, Message> {
    let p = theme.palette.clone();

    // Neutral colors (0-10)
    let neutrals = column![
        text("Neutrals (0-10)").size(16),
        row![
            color_swatch("0", p.neutral_0),
            color_swatch("1", p.neutral_1),
            color_swatch("2", p.neutral_2),
            color_swatch("3", p.neutral_3),
            color_swatch("4", p.neutral_4),
            color_swatch("5", p.neutral_5),
        ]
        .spacing(4),
        row![
            color_swatch("6", p.neutral_6),
            color_swatch("7", p.neutral_7),
            color_swatch("8", p.neutral_8),
            color_swatch("9", p.neutral_9),
            color_swatch("10", p.neutral_10),
        ]
        .spacing(4),
    ]
    .spacing(4);

    // Accent colors
    let accents = column![
        text("Accent Colors").size(16),
        row![
            color_swatch("Blue", p.accent_blue),
            color_swatch("Indigo", p.accent_indigo),
            color_swatch("Purple", p.accent_purple),
            color_swatch("Pink", p.accent_pink),
            color_swatch("Red", p.accent_red),
        ]
        .spacing(4),
        row![
            color_swatch("Orange", p.accent_orange),
            color_swatch("Yellow", p.accent_yellow),
            color_swatch("Green", p.accent_green),
            color_swatch("Warm Gray", p.accent_warm_grey),
        ]
        .spacing(4),
    ]
    .spacing(4);

    // Semantic colors
    let semantic = column![
        text("Semantic Colors").size(16),
        row![
            color_swatch("Bright Red", p.bright_red),
            color_swatch("Bright Green", p.bright_green),
            color_swatch("Bright Orange", p.bright_orange),
        ]
        .spacing(4),
    ]
    .spacing(4);

    // Surface grays
    let grays = column![
        text("Surface Grays").size(16),
        row![
            color_swatch("Gray 1", p.gray_1),
            color_swatch("Gray 2", p.gray_2),
        ]
        .spacing(4),
    ]
    .spacing(4);

    column![
        neutrals,
        rule::horizontal(1),
        accents,
        rule::horizontal(1),
        semantic,
        rule::horizontal(1),
        grays,
    ]
    .spacing(8)
    .into()
}

// =============================================================================
// Helper Functions
// =============================================================================

fn theme_tab_button<'a>(label: &'a str, page: ThemePage, current: ThemePage) -> Element<'a, Message> {
    let is_active = page == current;

    if is_active {
        button(text(label).size(14)).style(button::primary).into()
    } else {
        button(text(label).size(14))
            .style(button::secondary)
            .on_press(Message::ThemePageChanged(page))
            .into()
    }
}

fn theme_info_row(label: &str, value: String) -> Element<'static, Message> {
    row![
        text(format!("{}:", label)).size(14).width(180),
        text(value).size(14),
    ]
    .spacing(16)
    .align_y(icy_ui::Alignment::Center)
    .into()
}

fn labeled_color_swatch(label: &str, bg: Color) -> Element<'static, Message> {
    row![
        text(format!("{}:", label)).size(14).width(150),
        container(text(""))
            .width(40)
            .height(30)
            .style(move |_| container::Style {
                background: Some(bg.into()),
                border: icy_ui::Border {
                    color: Color::from_rgb(0.5, 0.5, 0.5),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }),
        text(format_color(bg)).size(12).width(100),
        text(format!("alpha: {:.2}", bg.a)).size(11),
    ]
    .spacing(12)
    .align_y(icy_ui::Alignment::Center)
    .into()
}

fn color_swatch(label: &str, bg: Color) -> Element<'static, Message> {
    let text_color = best_contrast_text_color(bg);

    container(
        column![
            text(label.to_string()).size(11),
            text(format!(
                "#{:02X}{:02X}{:02X}",
                (bg.r * 255.0) as u8,
                (bg.g * 255.0) as u8,
                (bg.b * 255.0) as u8
            ))
            .size(9),
        ]
        .align_x(icy_ui::Alignment::Center),
    )
    .padding(8)
    .width(80)
    .height(50)
    .align_x(icy_ui::alignment::Horizontal::Center)
    .align_y(icy_ui::alignment::Vertical::Center)
    .style(move |_| container::Style {
        background: Some(bg.into()),
        text_color: Some(text_color),
        ..Default::default()
    })
    .into()
}

fn best_contrast_text_color(bg: Color) -> Color {
    let l_bg = relative_luminance(bg);
    let contrast_black = (l_bg + 0.05) / 0.05;
    let contrast_white = 1.05 / (l_bg + 0.05);

    if contrast_black >= contrast_white {
        Color::BLACK
    } else {
        Color::WHITE
    }
}

fn relative_luminance(c: Color) -> f32 {
    fn srgb_to_linear(x: f32) -> f32 {
        if x <= 0.04045 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    }

    let r = srgb_to_linear(c.r);
    let g = srgb_to_linear(c.g);
    let b = srgb_to_linear(c.b);

    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn format_color(c: Color) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8
    )
}
