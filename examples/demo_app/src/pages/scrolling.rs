//! Scrollables page (tab-based), based on the `examples/scrollable` example.

use icy_ui::widget::{
    button, canvas, column, container, pick_list, progress_bar, radio_group, row, rule,
    scroll_area, scrollable, slider, space, text,
};
use icy_ui::{
    Border, Center, Color, Element, Fill, Font, Length, Pixels, Point, Rectangle, Size, Theme,
};

use crate::Message;

/// Virtual scrolling constants
pub const TOTAL_ROWS: usize = 100_000;
pub const CANVAS_SIZE: f32 = 100_000.0;

const SCROLL_AREA_HEIGHT: f32 = 420.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollablesTab {
    #[default]
    LongList,
    LargeCanvas,
    StyleOptions,
}

impl ScrollablesTab {
    pub const ALL: [ScrollablesTab; 3] = [
        ScrollablesTab::LongList,
        ScrollablesTab::LargeCanvas,
        ScrollablesTab::StyleOptions,
    ];

    fn title(&self) -> &'static str {
        match self {
            ScrollablesTab::LongList => "📜 Virtual List (100k rows)",
            ScrollablesTab::LargeCanvas => "🎨 Large Canvas (100k×100k)",
            ScrollablesTab::StyleOptions => "⚙️ Style Options",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollDirection {
    #[default]
    Vertical,
    Horizontal,
    Both,
}

impl std::fmt::Display for ScrollDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrollDirection::Vertical => write!(f, "Vertical"),
            ScrollDirection::Horizontal => write!(f, "Horizontal"),
            ScrollDirection::Both => write!(f, "Both"),
        }
    }
}

/// Anchor position for scrollable content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnchorPosition {
    #[default]
    Start,
    End,
}

impl std::fmt::Display for AnchorPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnchorPosition::Start => write!(f, "Start"),
            AnchorPosition::End => write!(f, "End"),
        }
    }
}

impl From<AnchorPosition> for scrollable::Anchor {
    fn from(pos: AnchorPosition) -> Self {
        match pos {
            AnchorPosition::Start => scrollable::Anchor::Start,
            AnchorPosition::End => scrollable::Anchor::End,
        }
    }
}

#[derive(Clone)]
pub struct ScrollingState {
    pub active_tab: ScrollablesTab,
    pub row_height: f32,
    pub style_preset: scrollable::Preset,
    pub direction: ScrollDirection,
    pub scrollbar_width: u32,
    pub scrollbar_margin: u32,
    pub scroller_width: u32,
    pub anchor: AnchorPosition,
    pub scroll_offset: scrollable::RelativeOffset,
}

impl Default for ScrollingState {
    fn default() -> Self {
        Self {
            active_tab: ScrollablesTab::default(),
            row_height: 30.0,
            style_preset: scrollable::Preset::default(),
            direction: ScrollDirection::default(),
            scrollbar_width: 10,
            scrollbar_margin: 0,
            scroller_width: 10,
            anchor: AnchorPosition::default(),
            scroll_offset: scrollable::RelativeOffset::START,
        }
    }
}

pub fn update_scrolling(state: &mut ScrollingState, message: &Message) -> bool {
    match message {
        Message::ScrollablesTabSelected(tab) => {
            state.active_tab = *tab;
            state.scroll_offset = scrollable::RelativeOffset::START;
            true
        }
        Message::Scrolled(viewport) => {
            state.scroll_offset = viewport.relative_offset();
            true
        }
        Message::RowHeightChanged(height) => {
            state.row_height = *height;
            true
        }
        Message::ScrollStylePresetChanged(preset) => {
            state.style_preset = *preset;
            true
        }
        Message::ScrollDirectionChanged(direction) => {
            state.direction = *direction;
            state.scroll_offset = scrollable::RelativeOffset::START;
            true
        }
        Message::ScrollbarWidthChanged(width) => {
            state.scrollbar_width = *width;
            true
        }
        Message::ScrollbarMarginChanged(margin) => {
            state.scrollbar_margin = *margin;
            true
        }
        Message::ScrollerWidthChanged(width) => {
            state.scroller_width = *width;
            true
        }
        Message::ScrollAnchorChanged(anchor) => {
            state.anchor = *anchor;
            state.scroll_offset = scrollable::RelativeOffset::START;
            true
        }
        _ => false,
    }
}

pub fn view_scrolling(state: &ScrollingState) -> Element<'static, Message> {
    let tabs = row(ScrollablesTab::ALL.iter().map(|tab| {
        let is_active = *tab == state.active_tab;
        let style = if is_active {
            button::primary
        } else {
            button::secondary
        };

        button(text(tab.title()).size(14))
            .style(style)
            .padding([8, 16])
            .on_press(Message::ScrollablesTabSelected(*tab))
            .into()
    }))
    .spacing(4);

    let content = match state.active_tab {
        ScrollablesTab::LongList => view_long_list(
            state.row_height,
            state.style_preset,
            state.scrollbar_width,
            state.scrollbar_margin,
            state.scroller_width,
            state.scroll_offset,
        ),
        ScrollablesTab::LargeCanvas => view_large_canvas(
            state.style_preset,
            state.scrollbar_width,
            state.scrollbar_margin,
            state.scroller_width,
            state.scroll_offset,
        ),
        ScrollablesTab::StyleOptions => view_style_options(
            state.style_preset,
            state.direction,
            state.scrollbar_width,
            state.scrollbar_margin,
            state.scroller_width,
            state.anchor,
        ),
    };

    let progress = view_progress(state.active_tab, state.direction, state.scroll_offset);

    column![
        tabs,
        rule::horizontal(1),
        content,
        rule::horizontal(1),
        progress
    ]
    .spacing(10)
    .into()
}

fn view_long_list(
    row_height: f32,
    style_preset: scrollable::Preset,
    scrollbar_width: u32,
    scrollbar_margin: u32,
    scroller_width: u32,
    current_scroll_offset: scrollable::RelativeOffset,
) -> Element<'static, Message> {
    let controls = row![
        text(format!("Virtual list: {} rows", TOTAL_ROWS))
            .size(14)
            .color(Color::from_rgb(0.7, 0.7, 0.7)),
        space::horizontal(),
        text("Row Height:"),
        slider(20.0..=60.0, row_height, Message::RowHeightChanged).width(100),
        text(format!("{:.0}px", row_height)),
        space::horizontal(),
        style_preset_picker(style_preset),
    ]
    .spacing(10)
    .align_y(Center);

    let virtual_list = scroll_area()
        .auto_scroll(true)
        .width(Fill)
        .height(Length::Fixed(SCROLL_AREA_HEIGHT))
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(scrollbar_width)
                .margin(scrollbar_margin)
                .scroller_width(scroller_width)
                .preset(style_preset),
        ))
        .style(move |theme, status| scrollable_style(theme, status, style_preset))
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

    let total_height = TOTAL_ROWS as f32 * row_height;
    let current_row = (current_scroll_offset.y * TOTAL_ROWS as f32) as usize;

    let info = row![
        text(format!("Total height: {:.0}px", total_height))
            .size(12)
            .color(Color::from_rgb(0.6, 0.6, 0.6)),
        space::horizontal(),
        text(format!("Current row: ~{}", current_row))
            .size(12)
            .color(Color::from_rgb(0.6, 0.6, 0.6)),
    ];

    column![controls, virtual_list, info].spacing(10).into()
}

fn view_large_canvas(
    style_preset: scrollable::Preset,
    scrollbar_width: u32,
    scrollbar_margin: u32,
    scroller_width: u32,
    current_scroll_offset: scrollable::RelativeOffset,
) -> Element<'static, Message> {
    let controls = row![
        text(format!(
            "Virtual canvas: {:.0}×{:.0} pixels",
            CANVAS_SIZE, CANVAS_SIZE
        ))
        .size(14)
        .color(Color::from_rgb(0.7, 0.7, 0.7)),
        space::horizontal(),
        style_preset_picker(style_preset),
    ]
    .spacing(10)
    .align_y(Center);

    let tile_size = 200.0;
    let virtual_canvas = scroll_area()
        .auto_scroll(true)
        .width(Fill)
        .height(Length::Fixed(SCROLL_AREA_HEIGHT))
        .direction(scrollable::Direction::Both {
            vertical: scrollable::Scrollbar::new()
                .width(scrollbar_width)
                .margin(scrollbar_margin)
                .scroller_width(scroller_width)
                .preset(style_preset),
            horizontal: scrollable::Scrollbar::new()
                .width(scrollbar_width)
                .margin(scrollbar_margin)
                .scroller_width(scroller_width)
                .preset(style_preset),
        })
        .style(move |theme, status| scrollable_style(theme, status, style_preset))
        .show_viewport(Size::new(CANVAS_SIZE, CANVAS_SIZE), move |viewport| {
            render_tiles(viewport, tile_size)
        })
        .on_scroll(Message::Scrolled);

    let pos_x = current_scroll_offset.x * CANVAS_SIZE;
    let pos_y = current_scroll_offset.y * CANVAS_SIZE;

    let info = row![
        text(format!("Position: ({:.0}, {:.0})", pos_x, pos_y))
            .size(12)
            .color(Color::from_rgb(0.6, 0.6, 0.6)),
        space::horizontal(),
        text("Only visible tiles are rendered!")
            .size(12)
            .color(Color::from_rgb(0.5, 0.7, 0.5)),
    ];

    column![controls, virtual_canvas, info].spacing(10).into()
}

fn view_style_options(
    style_preset: scrollable::Preset,
    direction: ScrollDirection,
    scrollbar_width: u32,
    scrollbar_margin: u32,
    scroller_width: u32,
    anchor: AnchorPosition,
) -> Element<'static, Message> {
    let preset_section = column![
        text("Style Preset").size(16),
        radio_group(
            scrollable::Preset::ALL,
            Some(style_preset),
            Message::ScrollStylePresetChanged,
        ),
        text(match style_preset {
            scrollable::Preset::Floating => {
                "Floating: Scrollbars fade in on hover, float over content"
            }
            scrollable::Preset::Thin => {
                "Thin: Thin bars that expand on hover, slightly transparent"
            }
            scrollable::Preset::Solid => "Solid: Always visible scrollbars that allocate space",
        })
        .size(12)
        .color(Color::from_rgb(0.6, 0.6, 0.6)),
    ]
    .spacing(10);

    let direction_section = column![
        text("Scroll Direction").size(16),
        radio_group(
            [
                ScrollDirection::Vertical,
                ScrollDirection::Horizontal,
                ScrollDirection::Both
            ],
            Some(direction),
            Message::ScrollDirectionChanged,
        ),
    ]
    .spacing(10);

    let dimensions_section = column![
        text("Scrollbar Dimensions").size(16),
        row![
            column![
                text("Bar Width").size(12),
                slider(2..=20, scrollbar_width, Message::ScrollbarWidthChanged).width(120),
                text(format!("{}px", scrollbar_width))
                    .size(12)
                    .color(Color::from_rgb(0.6, 0.6, 0.6)),
            ]
            .spacing(4),
            column![
                text("Margin").size(12),
                slider(0..=10, scrollbar_margin, Message::ScrollbarMarginChanged).width(120),
                text(format!("{}px", scrollbar_margin))
                    .size(12)
                    .color(Color::from_rgb(0.6, 0.6, 0.6)),
            ]
            .spacing(4),
            column![
                text("Scroller Width").size(12),
                slider(2..=20, scroller_width, Message::ScrollerWidthChanged).width(120),
                text(format!("{}px", scroller_width))
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
        radio_group(
            [AnchorPosition::Start, AnchorPosition::End],
            Some(anchor),
            Message::ScrollAnchorChanged,
        ),
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

    let demo_content = create_style_demo_content(
        style_preset,
        direction,
        scrollbar_width,
        scrollbar_margin,
        scroller_width,
        anchor,
    );

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
        .height(Length::Fixed(SCROLL_AREA_HEIGHT));

    row![controls, demo_area].spacing(20).into()
}

fn create_style_demo_content(
    style_preset: scrollable::Preset,
    direction: ScrollDirection,
    scrollbar_width: u32,
    scrollbar_margin: u32,
    scroller_width: u32,
    anchor: AnchorPosition,
) -> Element<'static, Message> {
    let mut scrollbar = scrollable::Scrollbar::new()
        .width(scrollbar_width)
        .margin(scrollbar_margin)
        .scroller_width(scroller_width)
        .anchor(anchor.into());

    scrollbar = scrollbar.preset(style_preset);

    let direction = match direction {
        ScrollDirection::Vertical => scrollable::Direction::Vertical(scrollbar),
        ScrollDirection::Horizontal => scrollable::Direction::Horizontal(scrollbar),
        ScrollDirection::Both => scrollable::Direction::Both {
            vertical: scrollbar,
            horizontal: scrollbar,
        },
    };

    // Create demo content - a wide and tall text block
    let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.";

    let items: Vec<Element<'static, Message>> = (1..=30)
        .map(|i| {
            let bg = if i % 2 == 0 {
                Color::from_rgba(1.0, 1.0, 1.0, 0.05)
            } else {
                Color::TRANSPARENT
            };
            container(
                row![
                    text(format!("{:>3}.", i))
                        .size(14)
                        .color(Color::from_rgb(0.5, 0.5, 0.5)),
                    text(format!("{} — {} — {} — {}", lorem, lorem, lorem, lorem)).size(14),
                ]
                .spacing(10),
            )
            .padding([8, 12])
            .style(move |_theme| container::Style {
                background: Some(bg.into()),
                ..Default::default()
            })
            .into()
        })
        .collect();

    let content: Element<'static, Message> = column(items).into();

    scrollable(content)
        .direction(direction)
        .auto_scroll(true)
        .on_scroll(Message::Scrolled)
        .width(Fill)
        .height(Length::Fixed(SCROLL_AREA_HEIGHT))
        .style(move |theme, status| scrollable_style(theme, status, style_preset))
        .into()
}

fn style_preset_picker(style_preset: scrollable::Preset) -> Element<'static, Message> {
    row![
        text("Style:").size(14),
        pick_list(
            &scrollable::Preset::ALL[..],
            Some(style_preset),
            Message::ScrollStylePresetChanged
        )
        .text_size(14)
        .padding([4, 8]),
    ]
    .spacing(8)
    .align_y(Center)
    .into()
}

fn scrollable_style(
    theme: &Theme,
    status: scrollable::Status,
    style_preset: scrollable::Preset,
) -> scrollable::Style {
    let base = scrollable::default(theme, status);

    let scroll = style_preset.scroll_style();

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
            theme.accent.base
        } else {
            theme.accent.hover
        }
    } else {
        theme.background.on
    };

    scrollable::Style {
        scroll: scrollable::ScrollStyle {
            rail_background: Some(theme.background.base.scale_alpha(bg_opacity)),
            handle_color: handle_color.scale_alpha(handle_opacity),
            handle_color_hovered: theme.accent.hover.scale_alpha(handle_opacity),
            handle_color_dragged: theme.accent.base.scale_alpha(handle_opacity),
            ..scroll
        },
        ..base
    }
}

fn view_progress(
    active_tab: ScrollablesTab,
    direction: ScrollDirection,
    current_scroll_offset: scrollable::RelativeOffset,
) -> Element<'static, Message> {
    let y_bar = progress_bar(0.0..=1.0, current_scroll_offset.y);
    let x_bar = progress_bar(0.0..=1.0, current_scroll_offset.x).style(|_theme: &Theme| {
        progress_bar::Style {
            background: Color::from_rgb(0.3, 0.3, 0.3).into(),
            bar: Color::from_rgb8(250, 85, 134).into(),
            border: Border::default(),
        }
    });

    match active_tab {
        ScrollablesTab::LargeCanvas | ScrollablesTab::StyleOptions
            if direction == ScrollDirection::Both =>
        {
            row![text("Y:").size(12), y_bar, text("X:").size(12), x_bar]
                .spacing(10)
                .align_y(Center)
                .into()
        }
        _ if active_tab == ScrollablesTab::StyleOptions
            && direction == ScrollDirection::Horizontal =>
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

fn render_tiles(viewport: Rectangle, tile_size: f32) -> Element<'static, Message> {
    // Use canvas for custom drawing - this is the correct approach for show_viewport
    // The viewport gives us the visible area in content coordinates
    struct TileCanvas {
        viewport: Rectangle,
        tile_size: f32,
    }

    impl canvas::Program<Message> for TileCanvas {
        type State = ();

        fn draw(
            &self,
            _state: &Self::State,
            renderer: &icy_ui::Renderer,
            _theme: &Theme,
            bounds: Rectangle,
            _cursor: icy_ui::mouse::Cursor,
        ) -> Vec<canvas::Geometry<icy_ui::Renderer>> {
            // Calculate visible tile range based on viewport
            let first_col = (self.viewport.x / self.tile_size).floor() as i32;
            let last_col = ((self.viewport.x + self.viewport.width) / self.tile_size).ceil() as i32;
            let first_row = (self.viewport.y / self.tile_size).floor() as i32;
            let last_row =
                ((self.viewport.y + self.viewport.height) / self.tile_size).ceil() as i32;

            // Create a frame and draw tiles
            let mut frame = canvas::Frame::new(renderer, bounds.size());

            for r in first_row..=last_row {
                for c in first_col..=last_col {
                    // Calculate tile position relative to viewport (screen coordinates)
                    let tile_x = c as f32 * self.tile_size - self.viewport.x;
                    let tile_y = r as f32 * self.tile_size - self.viewport.y;

                    // Skip if outside visible bounds
                    if tile_x + self.tile_size < 0.0
                        || tile_y + self.tile_size < 0.0
                        || tile_x > bounds.width
                        || tile_y > bounds.height
                    {
                        continue;
                    }

                    // Color based on position
                    let hue = ((c + r) as f32 * 0.05) % 1.0;
                    let color = hsv_to_rgb(hue, 0.4, 0.2);

                    // Draw tile background
                    frame.fill_rectangle(
                        Point::new(tile_x, tile_y),
                        Size::new(self.tile_size, self.tile_size),
                        color,
                    );

                    // Draw tile border
                    frame.stroke(
                        &canvas::Path::rectangle(
                            Point::new(tile_x, tile_y),
                            Size::new(self.tile_size, self.tile_size),
                        ),
                        canvas::Stroke::default()
                            .with_color(Color::from_rgba(1.0, 1.0, 1.0, 0.1))
                            .with_width(1.0),
                    );

                    // Draw tile label
                    let label = format!("({}, {})", c, r);
                    frame.fill_text(canvas::Text {
                        content: label,
                        position: Point::new(
                            tile_x + self.tile_size / 2.0,
                            tile_y + self.tile_size / 2.0 - 10.0,
                        ),
                        color: Color::from_rgb(0.8, 0.8, 0.8),
                        size: Pixels::from(16.0),
                        font: Font::DEFAULT,
                        align_x: icy_ui::widget::text::Alignment::Center,
                        align_y: icy_ui::alignment::Vertical::Center,
                        ..Default::default()
                    });

                    // Draw coordinate info
                    let coord_x = format!("x:{:.0}", c as f32 * self.tile_size);
                    frame.fill_text(canvas::Text {
                        content: coord_x,
                        position: Point::new(
                            tile_x + self.tile_size / 2.0,
                            tile_y + self.tile_size / 2.0 + 5.0,
                        ),
                        color: Color::from_rgb(0.6, 0.6, 0.6),
                        size: Pixels::from(10.0),
                        font: Font::DEFAULT,
                        align_x: icy_ui::widget::text::Alignment::Center,
                        align_y: icy_ui::alignment::Vertical::Center,
                        ..Default::default()
                    });

                    let coord_y = format!("y:{:.0}", r as f32 * self.tile_size);
                    frame.fill_text(canvas::Text {
                        content: coord_y,
                        position: Point::new(
                            tile_x + self.tile_size / 2.0,
                            tile_y + self.tile_size / 2.0 + 18.0,
                        ),
                        color: Color::from_rgb(0.6, 0.6, 0.6),
                        size: Pixels::from(10.0),
                        font: Font::DEFAULT,
                        align_x: icy_ui::widget::text::Alignment::Center,
                        align_y: icy_ui::alignment::Vertical::Center,
                        ..Default::default()
                    });
                }
            }

            vec![frame.into_geometry()]
        }
    }

    canvas(TileCanvas {
        viewport,
        tile_size,
    })
    .width(Fill)
    .height(Fill)
    .into()
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
