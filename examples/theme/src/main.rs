//! Example demonstrating the libcosmic-compatible theme system.
//!
//! This example shows:
//! - Theme info (spacing, radii, mode)
//! - Container colors with selection
//! - Component colors with selection
//! - Full palette colors
//! - Loading themes from RON files

use iced::theme::{load_theme_from_file, CornerRadii, Spacing, Theme};
use iced::widget::{button, column, container, pick_list, row, rule, scrollable, text};
use iced::{Color, Element, Length, Task};

pub fn main() -> iced::Result {
    iced::application(State::default, State::update, State::view)
        .title("Theme Explorer")
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Page {
    #[default]
    Info,
    Container,
    Component,
    Palette,
}

impl std::fmt::Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Page::Info => write!(f, "Info"),
            Page::Container => write!(f, "Container"),
            Page::Component => write!(f, "Component"),
            Page::Palette => write!(f, "Palette"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ContainerChoice {
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
enum ComponentChoice {
    #[default]
    Accent,
    Success,
    Destructive,
    Warning,
    AccentButton,
    SuccessButton,
    DestructiveButton,
    WarningButton,
    IconButton,
    LinkButton,
    TextButton,
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
            ComponentChoice::SuccessButton => write!(f, "Success Button"),
            ComponentChoice::DestructiveButton => write!(f, "Destructive Button"),
            ComponentChoice::WarningButton => write!(f, "Warning Button"),
            ComponentChoice::IconButton => write!(f, "Icon Button"),
            ComponentChoice::LinkButton => write!(f, "Link Button"),
            ComponentChoice::TextButton => write!(f, "Text Button"),
            ComponentChoice::Button => write!(f, "Button"),
        }
    }
}

struct State {
    theme: Theme,
    page: Page,
    container_choice: ContainerChoice,
    component_choice: ComponentChoice,
    load_error: Option<String>,
}

impl State {
    fn default() -> (Self, Task<Message>) {
        (
            Self {
                theme: Theme::dark(),
                page: Page::Info,
                container_choice: ContainerChoice::Background,
                component_choice: ComponentChoice::Accent,
                load_error: None,
            },
            Task::none(),
        )
    }
}

#[derive(Debug, Clone)]
enum Message {
    ToggleTheme,
    LoadTheme,
    ThemeLoaded(Result<Theme, String>),
    PageChanged(Page),
    ContainerChanged(ContainerChoice),
    ComponentChanged(ComponentChoice),
}

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleTheme => {
                self.theme = if self.theme.is_dark {
                    Theme::light()
                } else {
                    Theme::dark()
                };
                self.load_error = None;
                Task::none()
            }
            Message::LoadTheme => Task::perform(
                async {
                    let file = rfd::AsyncFileDialog::new()
                        .add_filter("RON Theme", &["ron"])
                        .set_title("Load Theme File")
                        .pick_file()
                        .await;

                    match file {
                        Some(handle) => {
                            let path = handle.path();
                            match load_theme_from_file(path) {
                                Ok(theme) => Ok(theme),
                                Err(e) => Err(e.to_string()),
                            }
                        }
                        None => Err("No file selected".to_string()),
                    }
                },
                Message::ThemeLoaded,
            ),
            Message::ThemeLoaded(result) => {
                match result {
                    Ok(theme) => {
                        self.theme = theme;
                        self.load_error = None;
                    }
                    Err(e) => {
                        if e != "No file selected" {
                            self.load_error = Some(e);
                        }
                    }
                }
                Task::none()
            }
            Message::PageChanged(page) => {
                self.page = page;
                Task::none()
            }
            Message::ContainerChanged(choice) => {
                self.container_choice = choice;
                Task::none()
            }
            Message::ComponentChanged(choice) => {
                self.component_choice = choice;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let theme = &self.theme;

        let spacing_m = theme.spacing.m as f32;
        let spacing_s = theme.spacing.s as f32;
        let is_dark = theme.is_dark;
        let name = theme.name.clone();

        let bg_color = theme.background.base;
        let on_bg = theme.on_background();

        // Header with theme toggle, load button, and page tabs
        let mut header: iced::widget::Column<'_, Message> = column![
            text("Theme Explorer").size(28),
            text(format!(
                "Current theme: {} ({})",
                name,
                if is_dark { "dark" } else { "light" }
            ))
            .size(14),
            row![
                button(text(if is_dark {
                    "Switch to Light"
                } else {
                    "Switch to Dark"
                }))
                .on_press(Message::ToggleTheme),
                button(text("Load Theme…")).on_press(Message::LoadTheme),
            ]
            .spacing(spacing_s),
            row![
                tab_button("Info", Page::Info, self.page),
                tab_button("Container", Page::Container, self.page),
                tab_button("Component", Page::Component, self.page),
                tab_button("Palette", Page::Palette, self.page),
            ]
            .spacing(spacing_s),
        ]
        .spacing(spacing_s);

        // Show error if any
        if let Some(ref err) = self.load_error {
            header = header.push(
                text(format!("Error: {}", err))
                    .size(12)
                    .color(Color::from_rgb(0.9, 0.2, 0.2)),
            );
        }

        // Page content - build inline to avoid lifetime issues
        let page_content: Element<'_, Message> = match self.page {
            Page::Info => build_info_page(
                is_dark,
                theme.is_high_contrast,
                name.clone(),
                theme.spacing,
                theme.corner_radii,
                theme.shade,
            ),
            Page::Container => {
                let c = match self.container_choice {
                    ContainerChoice::Background => &theme.background,
                    ContainerChoice::Primary => &theme.primary,
                    ContainerChoice::Secondary => &theme.secondary,
                };
                build_container_page(
                    self.container_choice,
                    c.base,
                    c.on,
                    c.divider,
                    c.small_widget,
                    c.component.base,
                    c.component.hover,
                    c.component.pressed,
                    c.component.on,
                )
            }
            Page::Component => {
                let c = match self.component_choice {
                    ComponentChoice::Accent => &theme.accent,
                    ComponentChoice::Success => &theme.success,
                    ComponentChoice::Destructive => &theme.destructive,
                    ComponentChoice::Warning => &theme.warning,
                    ComponentChoice::AccentButton => &theme.accent_button,
                    ComponentChoice::SuccessButton => &theme.success_button,
                    ComponentChoice::DestructiveButton => &theme.destructive_button,
                    ComponentChoice::WarningButton => &theme.warning_button,
                    ComponentChoice::IconButton => &theme.icon_button,
                    ComponentChoice::LinkButton => &theme.link_button,
                    ComponentChoice::TextButton => &theme.text_button,
                    ComponentChoice::Button => &theme.button,
                };
                build_component_page(
                    self.component_choice,
                    c.base,
                    c.hover,
                    c.pressed,
                    c.selected,
                    c.selected_text,
                    c.focus,
                    c.disabled,
                    c.on,
                    c.on_disabled,
                    c.border,
                    c.disabled_border,
                    c.divider,
                )
            }
            Page::Palette => build_palette_page(theme.palette.clone(), is_dark),
        };

        let content = column![header, rule::horizontal(1), page_content,]
            .spacing(spacing_m)
            .padding(spacing_m);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(bg_color.into()),
                text_color: Some(on_bg),
                ..Default::default()
            })
            .into()
    }
}

fn tab_button(label: &str, page: Page, current: Page) -> Element<'_, Message> {
    let is_active = page == current;

    if is_active {
        // Active tab: styled as selected with primary background
        button(text(label).size(14)).style(button::primary).into()
    } else {
        // Inactive tab: normal secondary style with click handler
        button(text(label).size(14))
            .style(button::secondary)
            .on_press(Message::PageChanged(page))
            .into()
    }
}

// ============================================================================
// Info Page
// ============================================================================

fn build_info_page(
    is_dark: bool,
    is_high_contrast: bool,
    name: String,
    spacing: Spacing,
    radii: CornerRadii,
    shade: Color,
) -> Element<'static, Message> {
    column![
        // Mode info
        text("Mode").size(18),
        info_row("Is Dark", is_dark.to_string()),
        info_row("Is High Contrast", is_high_contrast.to_string()),
        info_row("Name", name),
        rule::horizontal(1),
        // Spacing
        text("Spacing").size(18),
        info_row("XXXS (space_xxxs)", format!("{}", spacing.xxxs)),
        info_row("XXS (space_xxs)", format!("{}", spacing.xxs)),
        info_row("XS (space_xs)", format!("{}", spacing.xs)),
        info_row("S (space_s)", format!("{}", spacing.s)),
        info_row("M (space_m)", format!("{}", spacing.m)),
        info_row("L (space_l)", format!("{}", spacing.l)),
        info_row("XL (space_xl)", format!("{}", spacing.xl)),
        info_row("XXL (space_xxl)", format!("{}", spacing.xxl)),
        info_row("XXXL (space_xxxl)", format!("{}", spacing.xxxl)),
        rule::horizontal(1),
        // Corner Radii
        text("Corner Radii").size(18),
        info_row("0 (radius_0)", format!("{:?}", radii.radius_0)),
        info_row("XS (radius_xs)", format!("{:?}", radii.radius_xs)),
        info_row("S (radius_s)", format!("{:?}", radii.radius_s)),
        info_row("M (radius_m)", format!("{:?}", radii.radius_m)),
        info_row("L (radius_l)", format!("{:?}", radii.radius_l)),
        info_row("XL (radius_xl)", format!("{:?}", radii.radius_xl)),
        rule::horizontal(1),
        // Shade color
        text("Shade/Overlay").size(18),
        color_row("Shade", shade),
    ]
    .spacing(8)
    .into()
}

fn info_row(label: &str, value: String) -> Element<'static, Message> {
    row![
        text(format!("{}:", label)).size(14).width(180),
        text(value).size(14),
    ]
    .spacing(16)
    .align_y(iced::Alignment::Center)
    .into()
}

fn color_row(label: &str, color: Color) -> Element<'static, Message> {
    row![
        text(format!("{}:", label)).size(14).width(180),
        color_swatch_small(color),
        text(format_color(color)).size(12),
    ]
    .spacing(16)
    .align_y(iced::Alignment::Center)
    .into()
}

// ============================================================================
// Container Page
// ============================================================================

fn build_container_page(
    choice: ContainerChoice,
    base: Color,
    on: Color,
    divider: Color,
    small_widget: Color,
    comp_base: Color,
    comp_hover: Color,
    comp_pressed: Color,
    comp_on: Color,
) -> Element<'static, Message> {
    let picker = pick_list(
        vec![
            ContainerChoice::Background,
            ContainerChoice::Primary,
            ContainerChoice::Secondary,
        ],
        Some(choice),
        Message::ContainerChanged,
    )
    .width(200);

    column![
        row![text("Select Container:").size(14), picker]
            .spacing(16)
            .align_y(iced::Alignment::Center),
        rule::horizontal(1),
        text("Container Colors").size(18),
        labeled_swatch("Base", base),
        labeled_swatch("On (Text)", on),
        labeled_swatch("Divider", divider),
        labeled_swatch("Small Widget", small_widget),
        rule::horizontal(1),
        text("Embedded Component").size(18),
        labeled_swatch("Component Base", comp_base),
        labeled_swatch("Component Hover", comp_hover),
        labeled_swatch("Component Pressed", comp_pressed),
        labeled_swatch("Component On", comp_on),
    ]
    .spacing(8)
    .into()
}

// ============================================================================
// Component Page
// ============================================================================

#[allow(clippy::too_many_arguments)]
fn build_component_page(
    choice: ComponentChoice,
    base: Color,
    hover: Color,
    pressed: Color,
    selected: Color,
    selected_text: Color,
    focus: Color,
    disabled: Color,
    on: Color,
    on_disabled: Color,
    border: Color,
    disabled_border: Color,
    divider: Color,
) -> Element<'static, Message> {
    let picker = pick_list(
        vec![
            ComponentChoice::Accent,
            ComponentChoice::Success,
            ComponentChoice::Destructive,
            ComponentChoice::Warning,
            ComponentChoice::AccentButton,
            ComponentChoice::SuccessButton,
            ComponentChoice::DestructiveButton,
            ComponentChoice::WarningButton,
            ComponentChoice::IconButton,
            ComponentChoice::LinkButton,
            ComponentChoice::TextButton,
            ComponentChoice::Button,
        ],
        Some(choice),
        Message::ComponentChanged,
    )
    .width(200);

    column![
        row![text("Select Component:").size(14), picker]
            .spacing(16)
            .align_y(iced::Alignment::Center),
        rule::horizontal(1),
        text("State Colors").size(18),
        labeled_swatch("Base", base),
        labeled_swatch("Hover", hover),
        labeled_swatch("Pressed", pressed),
        labeled_swatch("Selected", selected),
        labeled_swatch("Focus", focus),
        labeled_swatch("Disabled", disabled),
        rule::horizontal(1),
        text("Text/Icon Colors").size(18),
        labeled_swatch("On (Text)", on),
        labeled_swatch("Selected Text", selected_text),
        labeled_swatch("On Disabled", on_disabled),
        rule::horizontal(1),
        text("Border & Divider").size(18),
        labeled_swatch("Border", border),
        labeled_swatch("Disabled Border", disabled_border),
        labeled_swatch("Divider", divider),
    ]
    .spacing(8)
    .into()
}

// ============================================================================
// Palette Page
// ============================================================================

fn build_palette_page(p: iced::theme::Palette, is_dark: bool) -> Element<'static, Message> {
    let on_light = p.neutral_10;
    let on_dark = p.neutral_0;

    // Neutral colors (0-10)
    let neutrals = column![
        text("Neutrals (0-10)").size(18),
        row![
            color_swatch("0", p.neutral_0, on_light),
            color_swatch("1", p.neutral_1, on_light),
            color_swatch("2", p.neutral_2, on_light),
            color_swatch("3", p.neutral_3, on_light),
            color_swatch("4", p.neutral_4, on_light),
            color_swatch("5", p.neutral_5, on_light),
            color_swatch("6", p.neutral_6, on_dark),
            color_swatch("7", p.neutral_7, on_dark),
            color_swatch("8", p.neutral_8, on_dark),
            color_swatch("9", p.neutral_9, on_dark),
            color_swatch("10", p.neutral_10, on_dark),
        ]
        .spacing(4),
    ]
    .spacing(8);

    // Accent colors
    let accents = column![
        text("Accent Colors").size(18),
        row![
            color_swatch("Blue", p.accent_blue, on_dark),
            color_swatch("Indigo", p.accent_indigo, on_dark),
            color_swatch("Purple", p.accent_purple, on_dark),
            color_swatch("Pink", p.accent_pink, on_dark),
            color_swatch("Red", p.accent_red, on_dark),
            color_swatch("Orange", p.accent_orange, on_dark),
            color_swatch("Yellow", p.accent_yellow, on_dark),
            color_swatch("Green", p.accent_green, on_dark),
            color_swatch("Warm Gray", p.accent_warm_grey, on_dark),
        ]
        .spacing(4),
    ]
    .spacing(8);

    // Semantic colors
    let semantic = column![
        text("Semantic Colors").size(18),
        row![
            color_swatch("Bright Red", p.bright_red, on_dark),
            color_swatch("Bright Green", p.bright_green, on_dark),
            color_swatch("Bright Orange", p.bright_orange, on_dark),
        ]
        .spacing(4),
    ]
    .spacing(8);

    // Surface grays
    let grays = column![
        text("Surface Grays").size(18),
        row![
            color_swatch("Gray 1", p.gray_1, if is_dark { on_light } else { on_dark }),
            color_swatch("Gray 2", p.gray_2, if is_dark { on_light } else { on_dark }),
        ]
        .spacing(4),
    ]
    .spacing(8);

    // Extended palette
    let extended = column![
        text("Extended Palette").size(18),
        row![
            color_swatch("Ext Blue", p.ext_blue, on_dark),
            color_swatch("Ext Indigo", p.ext_indigo, on_dark),
            color_swatch("Ext Purple", p.ext_purple, on_dark),
            color_swatch("Ext Pink", p.ext_pink, on_dark),
            color_swatch("Ext Orange", p.ext_orange, on_dark),
            color_swatch("Ext Yellow", p.ext_yellow, on_dark),
            color_swatch("Ext Warm Gray", p.ext_warm_grey, on_dark),
        ]
        .spacing(4),
    ]
    .spacing(8);

    column![
        neutrals,
        rule::horizontal(1),
        accents,
        rule::horizontal(1),
        semantic,
        rule::horizontal(1),
        grays,
        rule::horizontal(1),
        extended,
    ]
    .spacing(12)
    .into()
}

// ============================================================================
// Helper functions
// ============================================================================

fn color_swatch(label: &str, bg: Color, text_color: Color) -> Element<'static, Message> {
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
        .align_x(iced::Alignment::Center),
    )
    .padding(8)
    .width(80)
    .height(50)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .style(move |_| container::Style {
        background: Some(bg.into()),
        text_color: Some(text_color),
        ..Default::default()
    })
    .into()
}

fn color_swatch_small(bg: Color) -> Element<'static, Message> {
    container(text(""))
        .width(24)
        .height(24)
        .style(move |_| container::Style {
            background: Some(bg.into()),
            border: iced::Border {
                color: Color::from_rgb(0.5, 0.5, 0.5),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn labeled_swatch(label: &str, bg: Color) -> Element<'static, Message> {
    row![
        text(format!("{}:", label)).size(14).width(150),
        container(text(""))
            .width(40)
            .height(30)
            .style(move |_| container::Style {
                background: Some(bg.into()),
                border: iced::Border {
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
    .align_y(iced::Alignment::Center)
    .into()
}

fn format_color(c: Color) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8
    )
}
