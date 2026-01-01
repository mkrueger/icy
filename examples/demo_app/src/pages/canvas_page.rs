//! Canvas painting page - a simple drawing application
//!
//! Demonstrates the canvas widget with interactive mouse drawing.

use icy_ui::widget::{button, canvas, column, container, pick_list, row, slider, space, text};
use icy_ui::{mouse, Color, Element, Fill, Point, Rectangle};

// =============================================================================
// Canvas Page State
// =============================================================================

/// State for the canvas painting page
#[derive(Debug, Clone)]
pub struct CanvasPageState {
    /// All lines drawn on the canvas
    pub lines: Vec<Line>,
    /// Currently drawing line (while mouse is pressed)
    pub current_line: Option<Line>,
    /// Stroke width for drawing
    pub stroke_width: f32,
    /// Current stroke color
    pub stroke_color: StrokeColor,
}

impl Default for CanvasPageState {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            current_line: None,
            stroke_width: 3.0,
            stroke_color: StrokeColor::Red,
        }
    }
}

/// A line consisting of multiple points
#[derive(Debug, Clone)]
pub struct Line {
    pub points: Vec<Point>,
    pub color: Color,
    pub width: f32,
}

/// Available stroke colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StrokeColor {
    #[default]
    Red,
    Green,
    Blue,
    Yellow,
    White,
    Black,
}

impl StrokeColor {
    pub const ALL: &'static [StrokeColor] = &[
        StrokeColor::Red,
        StrokeColor::Green,
        StrokeColor::Blue,
        StrokeColor::Yellow,
        StrokeColor::White,
        StrokeColor::Black,
    ];

    pub fn to_color(self) -> Color {
        match self {
            StrokeColor::Red => Color::from_rgb(1.0, 0.2, 0.2),
            StrokeColor::Green => Color::from_rgb(0.2, 0.9, 0.2),
            StrokeColor::Blue => Color::from_rgb(0.3, 0.5, 1.0),
            StrokeColor::Yellow => Color::from_rgb(1.0, 0.9, 0.2),
            StrokeColor::White => Color::WHITE,
            StrokeColor::Black => Color::BLACK,
        }
    }
}

impl std::fmt::Display for StrokeColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrokeColor::Red => write!(f, "🔴 Red"),
            StrokeColor::Green => write!(f, "🟢 Green"),
            StrokeColor::Blue => write!(f, "🔵 Blue"),
            StrokeColor::Yellow => write!(f, "🟡 Yellow"),
            StrokeColor::White => write!(f, "⚪ White"),
            StrokeColor::Black => write!(f, "⚫ Black"),
        }
    }
}

// =============================================================================
// Canvas Program Implementation
// =============================================================================

/// The painting program that handles canvas events and drawing
pub struct PaintingCanvas<'a> {
    state: &'a CanvasPageState,
}

impl<'a> PaintingCanvas<'a> {
    pub fn new(state: &'a CanvasPageState) -> Self {
        Self { state }
    }
}

/// Internal state for the canvas widget
#[derive(Debug, Default)]
pub struct PaintingState {
    /// Whether the mouse is currently pressed
    is_drawing: bool,
    /// Last known cursor position
    last_position: Option<Point>,
}

impl canvas::Program<crate::Message> for PaintingCanvas<'_> {
    type State = PaintingState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<crate::Message>> {
        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed {
                button: mouse::Button::Left,
                ..
            }) => {
                if let Some(position) = cursor.position_in(bounds) {
                    state.is_drawing = true;
                    state.last_position = Some(position);
                    // Start a new line
                    return Some(canvas::Action::publish(crate::Message::CanvasStartLine(
                        position,
                    )));
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased {
                button: mouse::Button::Left,
                ..
            }) => {
                if state.is_drawing {
                    state.is_drawing = false;
                    state.last_position = None;
                    // Finish the current line
                    return Some(canvas::Action::publish(crate::Message::CanvasEndLine));
                }
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.is_drawing {
                    if let Some(position) = cursor.position_in(bounds) {
                        state.last_position = Some(position);
                        // Add point to current line
                        return Some(canvas::Action::publish(crate::Message::CanvasAddPoint(
                            position,
                        )));
                    }
                }
            }
            _ => {}
        }
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &icy_ui::Renderer,
        _theme: &icy_ui::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<icy_ui::Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Draw background
        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            Color::from_rgb(0.1, 0.1, 0.12),
        );

        // Draw border
        frame.stroke(
            &canvas::Path::rectangle(Point::ORIGIN, bounds.size()),
            canvas::Stroke::default()
                .with_color(Color::from_rgb(0.3, 0.3, 0.35))
                .with_width(2.0),
        );

        // Draw all completed lines
        for line in &self.state.lines {
            if line.points.len() >= 2 {
                let path = canvas::Path::new(|builder| {
                    builder.move_to(line.points[0]);
                    for point in &line.points[1..] {
                        builder.line_to(*point);
                    }
                });
                frame.stroke(
                    &path,
                    canvas::Stroke::default()
                        .with_color(line.color)
                        .with_width(line.width)
                        .with_line_cap(canvas::LineCap::Round)
                        .with_line_join(canvas::LineJoin::Round),
                );
            }
        }

        // Draw current line being drawn
        if let Some(ref line) = self.state.current_line {
            if line.points.len() >= 2 {
                let path = canvas::Path::new(|builder| {
                    builder.move_to(line.points[0]);
                    for point in &line.points[1..] {
                        builder.line_to(*point);
                    }
                });
                frame.stroke(
                    &path,
                    canvas::Stroke::default()
                        .with_color(line.color)
                        .with_width(line.width)
                        .with_line_cap(canvas::LineCap::Round)
                        .with_line_join(canvas::LineJoin::Round),
                );
            }
        }

        // Draw instructions if canvas is empty
        if self.state.lines.is_empty() && self.state.current_line.is_none() {
            frame.fill_text(canvas::Text {
                content: "Click and drag to draw!".to_string(),
                position: Point::new(bounds.width / 2.0, bounds.height / 2.0),
                color: Color::from_rgb(0.5, 0.5, 0.5),
                size: icy_ui::Pixels::from(24.0),
                font: icy_ui::Font::DEFAULT,
                align_x: icy_ui::widget::text::Alignment::Center,
                align_y: icy_ui::alignment::Vertical::Center,
                ..Default::default()
            });
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            if state.is_drawing {
                mouse::Interaction::Crosshair
            } else {
                mouse::Interaction::Crosshair
            }
        } else {
            mouse::Interaction::default()
        }
    }
}

// =============================================================================
// View Function
// =============================================================================

/// Create the view for the canvas painting page
pub fn canvas_page_view(state: &CanvasPageState) -> Element<'_, crate::Message> {
    // Controls row
    let controls = row![
        text("Stroke Width:").size(14),
        slider(
            1.0..=20.0,
            state.stroke_width,
            crate::Message::CanvasStrokeWidthChanged
        )
        .width(150)
        .step(0.5),
        text(format!("{:.1}", state.stroke_width)).size(14),
        space().width(20),
        text("Color:").size(14),
        pick_list(
            StrokeColor::ALL,
            Some(state.stroke_color),
            crate::Message::CanvasColorChanged
        )
        .width(120),
        space().width(20),
        button(text("Clear Canvas").size(14))
            .on_press(crate::Message::CanvasClear)
            .style(button::danger),
    ]
    .spacing(10)
    .align_y(icy_ui::Alignment::Center);

    // The canvas widget
    let painting_canvas = canvas(PaintingCanvas::new(state)).width(Fill).height(Fill);

    // Stats row
    let stats = row![
        text(format!("Lines: {}", state.lines.len())).size(12),
        space().width(20),
        text(format!(
            "Points: {}",
            state.lines.iter().map(|l| l.points.len()).sum::<usize>()
                + state.current_line.as_ref().map_or(0, |l| l.points.len())
        ))
        .size(12),
    ]
    .spacing(10);

    let content = column![
        text("Canvas Painting").size(24),
        text("A simple drawing application demonstrating the canvas widget.")
            .size(14)
            .color(Color::from_rgb(0.6, 0.6, 0.6)),
        space().height(10),
        controls,
        space().height(10),
        painting_canvas,
        space().height(5),
        stats,
    ]
    .spacing(5)
    .padding(10);

    container(content).width(Fill).height(Fill).into()
}

// =============================================================================
// Update Function
// =============================================================================

/// Update the canvas page state based on messages
pub fn update_canvas(state: &mut CanvasPageState, message: &crate::Message) -> bool {
    match message {
        crate::Message::CanvasStartLine(point) => {
            // Start a new line with current settings
            state.current_line = Some(Line {
                points: vec![*point],
                color: state.stroke_color.to_color(),
                width: state.stroke_width,
            });
            true
        }
        crate::Message::CanvasAddPoint(point) => {
            // Add point to current line
            if let Some(ref mut line) = state.current_line {
                line.points.push(*point);
            }
            true
        }
        crate::Message::CanvasEndLine => {
            // Finish the current line and add it to completed lines
            if let Some(line) = state.current_line.take() {
                if line.points.len() >= 2 {
                    state.lines.push(line);
                }
            }
            true
        }
        crate::Message::CanvasStrokeWidthChanged(width) => {
            state.stroke_width = *width;
            true
        }
        crate::Message::CanvasColorChanged(color) => {
            state.stroke_color = *color;
            true
        }
        crate::Message::CanvasClear => {
            state.lines.clear();
            state.current_line = None;
            true
        }
        _ => false,
    }
}
