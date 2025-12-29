use iced::widget::{
    button, column, container, pick_list, progress_bar, radio, row, rule, scroll_area, scrollable,
    slider, space, text,
};
use iced::{Border, Center, Color, Element, Fill, Length, Rectangle, Size, Task, Theme};

pub fn main() -> iced::Result {
    iced::application(
        ScrollableDemo::default,
        ScrollableDemo::update,
        ScrollableDemo::view,
    )
    .theme(ScrollableDemo::theme)
    .run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Tab {
    #[default]
    LongList,
    LargeCanvas,
    StyleOptions,
}

impl Tab {
    const ALL: [Tab; 3] = [Tab::LongList, Tab::LargeCanvas, Tab::StyleOptions];

    fn title(&self) -> &'static str {
        match self {
            Tab::LongList => "📜 Virtual List (100k rows)",
            Tab::LargeCanvas => "🎨 Large Canvas (100k×100k)",
            Tab::StyleOptions => "⚙️ Style Options",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ScrollStylePreset {
    #[default]
    Floating,
    Thin,
    Solid,
}

impl ScrollStylePreset {
    const ALL: [ScrollStylePreset; 3] = [
        ScrollStylePreset::Floating,
        ScrollStylePreset::Thin,
        ScrollStylePreset::Solid,
    ];
}

impl std::fmt::Display for ScrollStylePreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrollStylePreset::Floating => write!(f, "Floating"),
            ScrollStylePreset::Thin => write!(f, "Thin"),
            ScrollStylePreset::Solid => write!(f, "Solid"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Vertical,
    Horizontal,
    Both,
}

struct ScrollableDemo {
    active_tab: Tab,
    // Long list settings
    row_height: f32,
    // Style settings
    style_preset: ScrollStylePreset,
    direction: Direction,
    scrollbar_width: u32,
    scrollbar_margin: u32,
    scroller_width: u32,
    anchor: scrollable::Anchor,
    // Scroll state
    current_scroll_offset: scrollable::RelativeOffset,
}

// Constants for virtual content
const TOTAL_ROWS: usize = 100_000;
const CANVAS_SIZE: f32 = 100_000.0;

#[derive(Debug, Clone)]
enum Message {
    TabSelected(Tab),
    RowHeightChanged(f32),
    StylePresetChanged(ScrollStylePreset),
    DirectionChanged(Direction),
    ScrollbarWidthChanged(u32),
    ScrollbarMarginChanged(u32),
    ScrollerWidthChanged(u32),
    AnchorChanged(scrollable::Anchor),
    Scrolled(scrollable::Viewport),
}

impl Default for ScrollableDemo {
    fn default() -> Self {
        Self {
            active_tab: Tab::default(),
            row_height: 30.0,
            style_preset: ScrollStylePreset::default(),
            direction: Direction::Vertical,
            scrollbar_width: 10,
            scrollbar_margin: 0,
            scroller_width: 10,
            anchor: scrollable::Anchor::Start,
            current_scroll_offset: scrollable::RelativeOffset::START,
        }
    }
}

impl ScrollableDemo {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabSelected(tab) => {
                self.active_tab = tab;
                self.current_scroll_offset = scrollable::RelativeOffset::START;
                Task::none()
            }
            Message::RowHeightChanged(height) => {
                self.row_height = height;
                Task::none()
            }
            Message::StylePresetChanged(preset) => {
                self.style_preset = preset;
                Task::none()
            }
            Message::DirectionChanged(direction) => {
                self.direction = direction;
                self.current_scroll_offset = scrollable::RelativeOffset::START;
                Task::none()
            }
            Message::ScrollbarWidthChanged(width) => {
                self.scrollbar_width = width;
                Task::none()
            }
            Message::ScrollbarMarginChanged(margin) => {
                self.scrollbar_margin = margin;
                Task::none()
            }
            Message::ScrollerWidthChanged(width) => {
                self.scroller_width = width;
                Task::none()
            }
            Message::AnchorChanged(anchor) => {
                self.anchor = anchor;
                self.current_scroll_offset = scrollable::RelativeOffset::START;
                Task::none()
            }
            Message::Scrolled(viewport) => {
                self.current_scroll_offset = viewport.relative_offset();
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let tabs = row(Tab::ALL.iter().map(|tab| {
            let is_active = *tab == self.active_tab;
            let style = if is_active {
                button::primary
            } else {
                button::secondary
            };
            button(text(tab.title()).size(14))
                .style(style)
                .padding([8, 16])
                .on_press(Message::TabSelected(*tab))
                .into()
        }))
        .spacing(4);

        let content = match self.active_tab {
            Tab::LongList => self.view_long_list(),
            Tab::LargeCanvas => self.view_large_canvas(),
            Tab::StyleOptions => self.view_style_options(),
        };

        let progress = self.view_progress();

        column![
            tabs,
            rule::horizontal(1),
            content,
            rule::horizontal(1),
            progress
        ]
        .spacing(10)
        .padding(20)
        .into()
    }

    fn view_long_list(&self) -> Element<'_, Message> {
        let row_height = self.row_height;

        let controls = row![
            text(format!("Virtual list: {} rows", TOTAL_ROWS))
                .size(14)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            space::horizontal(),
            text("Row Height:"),
            slider(20.0..=60.0, self.row_height, Message::RowHeightChanged).width(100),
            text(format!("{:.0}px", self.row_height)),
            space::horizontal(),
            self.style_preset_picker(),
        ]
        .spacing(10)
        .align_y(Center);

        // Virtual scrolling for 100k rows
        let virtual_list = scroll_area()
            .width(Fill)
            .height(Fill)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(self.scrollbar_width)
                    .margin(self.scrollbar_margin)
                    .scroller_width(self.scroller_width),
            ))
            .style(|theme, status| self.scrollable_style(theme, status))
            .show_rows(row_height, TOTAL_ROWS, move |range| {
                column(range.map(|i| {
                    let bg = if i % 2 == 0 {
                        Color::from_rgba(1.0, 1.0, 1.0, 0.03)
                    } else {
                        Color::TRANSPARENT
                    };
                    container(
                        row![
                            text(format!("{:>6}", i + 1))
                                .size(14)
                                .color(Color::from_rgb(0.5, 0.5, 0.5)),
                            rule::vertical(1),
                            text(format!("Row {} - Lorem ipsum dolor sit amet", i + 1)).size(14),
                        ]
                        .spacing(10)
                        .align_y(Center),
                    )
                    .padding([4, 10])
                    .style(move |_theme| container::Style {
                        background: Some(bg.into()),
                        ..Default::default()
                    })
                    .height(row_height)
                    .width(Fill)
                    .into()
                }))
                .into()
            })
            .on_scroll(Message::Scrolled);

        let total_height = TOTAL_ROWS as f32 * self.row_height;
        let current_row = (self.current_scroll_offset.y * TOTAL_ROWS as f32) as usize;

        let info = row![
            text(format!("Total height: {:.0}px", total_height))
                .size(12)
                .color(Color::from_rgb(0.6, 0.6, 0.6)),
            space::horizontal(),
            text(format!("Current row: ~{}", current_row))
                .size(12)
                .color(Color::from_rgb(0.6, 0.6, 0.6)),
        ];

        column![controls, virtual_list, info]
            .spacing(10)
            .height(Fill)
            .into()
    }

    fn view_large_canvas(&self) -> Element<'_, Message> {
        let controls = row![
            text(format!(
                "Virtual canvas: {:.0}×{:.0} pixels",
                CANVAS_SIZE, CANVAS_SIZE
            ))
            .size(14)
            .color(Color::from_rgb(0.7, 0.7, 0.7)),
            space::horizontal(),
            self.style_preset_picker(),
        ]
        .spacing(10)
        .align_y(Center);

        // Virtual canvas with tiles
        let tile_size = 200.0;

        let virtual_canvas = scroll_area()
            .width(Fill)
            .height(Fill)
            .direction(scrollable::Direction::Both {
                vertical: scrollable::Scrollbar::new()
                    .width(self.scrollbar_width)
                    .margin(self.scrollbar_margin)
                    .scroller_width(self.scroller_width),
                horizontal: scrollable::Scrollbar::new()
                    .width(self.scrollbar_width)
                    .margin(self.scrollbar_margin)
                    .scroller_width(self.scroller_width),
            })
            .style(|theme, status| self.scrollable_style(theme, status))
            .show_viewport(Size::new(CANVAS_SIZE, CANVAS_SIZE), move |viewport| {
                Self::render_tiles(viewport, tile_size)
            })
            .with_cell_size(Size::new(tile_size, tile_size))
            .on_scroll(Message::Scrolled);

        let pos_x = self.current_scroll_offset.x * CANVAS_SIZE;
        let pos_y = self.current_scroll_offset.y * CANVAS_SIZE;

        let info = row![
            text(format!("Position: ({:.0}, {:.0})", pos_x, pos_y))
                .size(12)
                .color(Color::from_rgb(0.6, 0.6, 0.6)),
            space::horizontal(),
            text(format!(
                "Progress: {:.2}% x {:.2}%",
                self.current_scroll_offset.x * 100.0,
                self.current_scroll_offset.y * 100.0
            ))
            .size(12)
            .color(Color::from_rgb(0.6, 0.6, 0.6)),
            space::horizontal(),
            text("Only visible tiles are rendered!")
                .size(12)
                .color(Color::from_rgb(0.5, 0.7, 0.5)),
        ];

        column![controls, virtual_canvas, info]
            .spacing(10)
            .height(Fill)
            .into()
    }

    fn render_tiles(viewport: Rectangle, tile_size: f32) -> Element<'static, Message> {
        // Calculate visible tile range
        let first_col = (viewport.x / tile_size).floor() as i32;
        let last_col = ((viewport.x + viewport.width) / tile_size).ceil() as i32;
        let first_row = (viewport.y / tile_size).floor() as i32;
        let last_row = ((viewport.y + viewport.height) / tile_size).ceil() as i32;

        // Build rows of tiles
        let rows: Vec<Element<'static, Message>> = (first_row..=last_row)
            .map(|r| {
                let cols: Vec<Element<'static, Message>> = (first_col..=last_col)
                    .map(|c| {
                        let tile_x = c as f32 * tile_size;
                        let tile_y = r as f32 * tile_size;

                        // Color based on position
                        let hue = ((c + r) as f32 * 0.05) % 1.0;
                        let color = hsv_to_rgb(hue, 0.4, 0.2);

                        container(
                            column![
                                text(format!("({}, {})", c, r))
                                    .size(16)
                                    .color(Color::from_rgb(0.8, 0.8, 0.8)),
                                text(format!("{:.0},{:.0}", tile_x, tile_y))
                                    .size(11)
                                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
                            ]
                            .spacing(4)
                            .align_x(Center),
                        )
                        .width(tile_size)
                        .height(tile_size)
                        .center(Fill)
                        .style(move |_theme| container::Style {
                            background: Some(color.into()),
                            border: Border {
                                color: Color::from_rgba(1.0, 1.0, 1.0, 0.15),
                                width: 1.0,
                                radius: 0.0.into(),
                            },
                            ..Default::default()
                        })
                        .into()
                    })
                    .collect();
                row(cols).into()
            })
            .collect();

        column(rows).into()
    }

    fn view_style_options(&self) -> Element<'_, Message> {
        let preset_section = column![
            text("Style Preset").size(16),
            row(ScrollStylePreset::ALL.iter().map(|preset| {
                radio(
                    format!("{}", preset),
                    *preset,
                    Some(self.style_preset),
                    Message::StylePresetChanged,
                )
                .into()
            }))
            .spacing(20),
            text(match self.style_preset {
                ScrollStylePreset::Floating => {
                    "Floating: Scrollbars fade in on hover, float over content"
                }
                ScrollStylePreset::Thin => {
                    "Thin: Thin bars that expand on hover, slightly transparent"
                }
                ScrollStylePreset::Solid => {
                    "Solid: Always visible scrollbars that allocate space"
                }
            })
            .size(12)
            .color(Color::from_rgb(0.6, 0.6, 0.6)),
        ]
        .spacing(10);

        let direction_section = column![
            text("Scroll Direction").size(16),
            row![
                radio(
                    "Vertical",
                    Direction::Vertical,
                    Some(self.direction),
                    Message::DirectionChanged,
                ),
                radio(
                    "Horizontal",
                    Direction::Horizontal,
                    Some(self.direction),
                    Message::DirectionChanged,
                ),
                radio(
                    "Both",
                    Direction::Both,
                    Some(self.direction),
                    Message::DirectionChanged,
                ),
            ]
            .spacing(20),
        ]
        .spacing(10);

        let dimensions_section = column![
            text("Scrollbar Dimensions").size(16),
            row![
                column![
                    text("Bar Width").size(12),
                    slider(2..=20, self.scrollbar_width, Message::ScrollbarWidthChanged).width(120),
                    text(format!("{}px", self.scrollbar_width))
                        .size(12)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                ]
                .spacing(4),
                column![
                    text("Margin").size(12),
                    slider(
                        0..=10,
                        self.scrollbar_margin,
                        Message::ScrollbarMarginChanged
                    )
                    .width(120),
                    text(format!("{}px", self.scrollbar_margin))
                        .size(12)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                ]
                .spacing(4),
                column![
                    text("Scroller Width").size(12),
                    slider(2..=20, self.scroller_width, Message::ScrollerWidthChanged).width(120),
                    text(format!("{}px", self.scroller_width))
                        .size(12)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                ]
                .spacing(4),
            ]
            .spacing(20),
        ]
        .spacing(10);

        let anchor_section = column![
            text("Anchor Position").size(16),
            row![
                radio(
                    "Start",
                    scrollable::Anchor::Start,
                    Some(self.anchor),
                    Message::AnchorChanged,
                ),
                radio(
                    "End",
                    scrollable::Anchor::End,
                    Some(self.anchor),
                    Message::AnchorChanged,
                ),
            ]
            .spacing(20),
        ]
        .spacing(10);

        let controls = column![
            preset_section,
            rule::horizontal(1),
            direction_section,
            rule::horizontal(1),
            dimensions_section,
            rule::horizontal(1),
            anchor_section,
        ]
        .spacing(15)
        .width(Length::FillPortion(1));

        // Demo content using regular scrollable
        let demo_content = self.create_style_demo_content();

        let demo_area = container(demo_content)
            .style(|_theme| container::Style {
                border: Border {
                    color: Color::from_rgb(0.3, 0.3, 0.3),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
            .width(Length::FillPortion(2))
            .height(Fill);

        row![controls, demo_area].spacing(20).into()
    }

    fn create_style_demo_content(&self) -> Element<'_, Message> {
        let scrollbar = scrollable::Scrollbar::new()
            .width(self.scrollbar_width)
            .margin(self.scrollbar_margin)
            .scroller_width(self.scroller_width)
            .anchor(self.anchor);

        let direction = match self.direction {
            Direction::Vertical => scrollable::Direction::Vertical(scrollbar),
            Direction::Horizontal => scrollable::Direction::Horizontal(scrollbar),
            Direction::Both => scrollable::Direction::Both {
                vertical: scrollbar,
                horizontal: scrollbar,
            },
        };

        let content: Element<'_, Message> = match self.direction {
            Direction::Vertical => {
                let items: Vec<Element<'_, Message>> = (1..=50)
                    .map(|i| {
                        container(text(format!("Item {}", i)).size(16))
                            .padding(15)
                            .width(Fill)
                            .style(move |_theme| {
                                let bg = if i % 2 == 0 {
                                    Color::from_rgba(1.0, 1.0, 1.0, 0.05)
                                } else {
                                    Color::TRANSPARENT
                                };
                                container::Style {
                                    background: Some(bg.into()),
                                    ..Default::default()
                                }
                            })
                            .into()
                    })
                    .collect();
                column(items).into()
            }
            Direction::Horizontal => {
                let items: Vec<Element<'_, Message>> = (1..=30)
                    .map(|i| {
                        container(text(format!("Col {}", i)).size(14))
                            .padding([20, 40])
                            .height(200)
                            .center_y(Fill)
                            .style(move |_theme| {
                                let bg = if i % 2 == 0 {
                                    Color::from_rgba(1.0, 1.0, 1.0, 0.05)
                                } else {
                                    Color::TRANSPARENT
                                };
                                container::Style {
                                    background: Some(bg.into()),
                                    ..Default::default()
                                }
                            })
                            .into()
                    })
                    .collect();
                row(items).into()
            }
            Direction::Both => {
                // Grid of items
                let rows: Vec<Element<'_, Message>> = (0..20)
                    .map(|r| {
                        let cols: Vec<Element<'_, Message>> = (0..20)
                            .map(|c| {
                                let hue = ((r + c) as f32 * 0.05) % 1.0;
                                let color = hsv_to_rgb(hue, 0.3, 0.15);
                                container(text(format!("({},{})", c, r)).size(12))
                                    .width(80)
                                    .height(60)
                                    .center(Fill)
                                    .style(move |_theme| container::Style {
                                        background: Some(color.into()),
                                        border: Border {
                                            color: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
                                            width: 1.0,
                                            radius: 0.0.into(),
                                        },
                                        ..Default::default()
                                    })
                                    .into()
                            })
                            .collect();
                        row(cols).into()
                    })
                    .collect();
                column(rows).into()
            }
        };

        scrollable(content)
            .direction(direction)
            .on_scroll(Message::Scrolled)
            .width(Fill)
            .height(Fill)
            .style(|theme, status| self.scrollable_style(theme, status))
            .into()
    }

    fn style_preset_picker(&self) -> Element<'_, Message> {
        row![
            text("Style:").size(14),
            pick_list(
                &ScrollStylePreset::ALL[..],
                Some(self.style_preset),
                Message::StylePresetChanged
            )
            .text_size(14)
            .padding([4, 8]),
        ]
        .spacing(8)
        .align_y(Center)
        .into()
    }

    fn scrollable_style(&self, theme: &Theme, status: scrollable::Status) -> scrollable::Style {
        let base = scrollable::default(theme, status);
        let palette = theme.extended_palette();

        let scroll = match self.style_preset {
            ScrollStylePreset::Floating => scrollable::ScrollStyle::floating(),
            ScrollStylePreset::Thin => scrollable::ScrollStyle::thin(),
            ScrollStylePreset::Solid => scrollable::ScrollStyle::solid(),
        };

        let hover_factor = match status {
            scrollable::Status::Active { hover_factor, .. } => hover_factor,
            scrollable::Status::Hovered { hover_factor, .. } => hover_factor,
            scrollable::Status::Dragged { hover_factor, .. } => hover_factor,
        };

        let (is_h_interacting, is_v_interacting) = match status {
            scrollable::Status::Active { .. } => (false, false),
            scrollable::Status::Hovered {
                is_horizontal_scrollbar_hovered,
                is_vertical_scrollbar_hovered,
                ..
            } => (
                is_horizontal_scrollbar_hovered,
                is_vertical_scrollbar_hovered,
            ),
            scrollable::Status::Dragged {
                is_horizontal_scrollbar_dragged,
                is_vertical_scrollbar_dragged,
                ..
            } => (
                is_horizontal_scrollbar_dragged,
                is_vertical_scrollbar_dragged,
            ),
        };
        let is_interacting = is_h_interacting || is_v_interacting;

        let handle_opacity = scroll.handle_opacity(hover_factor, is_interacting);
        let bg_opacity = scroll.background_opacity(hover_factor, is_interacting);

        let handle_color = if is_interacting {
            if matches!(status, scrollable::Status::Dragged { .. }) {
                palette.primary.base.color
            } else {
                palette.primary.strong.color
            }
        } else {
            palette.background.strongest.color
        };

        scrollable::Style {
            scroll: scrollable::ScrollStyle {
                rail_background: Some(palette.background.weak.color.scale_alpha(bg_opacity)),
                handle_color: handle_color.scale_alpha(handle_opacity),
                handle_color_hovered: palette.primary.strong.color.scale_alpha(handle_opacity),
                handle_color_dragged: palette.primary.base.color.scale_alpha(handle_opacity),
                ..scroll
            },
            ..base
        }
    }

    fn view_progress(&self) -> Element<'_, Message> {
        let y_bar = progress_bar(0.0..=1.0, self.current_scroll_offset.y);
        let x_bar = progress_bar(0.0..=1.0, self.current_scroll_offset.x).style(|theme: &Theme| {
            progress_bar::Style {
                background: theme.extended_palette().background.strong.color.into(),
                bar: Color::from_rgb8(250, 85, 134).into(),
                border: Border::default(),
            }
        });

        match self.active_tab {
            Tab::LargeCanvas | Tab::StyleOptions if self.direction == Direction::Both => {
                row![text("Y:").size(12), y_bar, text("X:").size(12), x_bar,]
                    .spacing(10)
                    .align_y(Center)
                    .into()
            }
            _ if self.active_tab == Tab::StyleOptions
                && self.direction == Direction::Horizontal =>
            {
                row![text("X:").size(12), x_bar]
                    .spacing(10)
                    .align_y(Center)
                    .into()
            }
            _ => row![text("Y:").size(12), y_bar]
                .spacing(10)
                .align_y(Center)
                .into(),
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let c = v * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match (h * 6.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Color::from_rgb(r + m, g + m, b + m)
}
