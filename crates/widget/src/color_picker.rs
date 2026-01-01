//! A widget for selecting colors.
//!
//! Inspired by libcosmic's color picker design.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use icy_ui_widget::*; } pub use icy_ui_widget::core::*; }
//! # pub type Element<'a, Message> = icy_ui_widget::core::Element<'a, Message, icy_ui_widget::Theme, icy_ui_widget::Renderer>;
//! use icy_ui::widget::color_picker::{self, ColorPicker};
//! use icy_ui::Color;
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     ColorChanged(Color),
//! }
//!
//! struct State {
//!     color: Color,
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     color_picker::color_picker(state.color, Message::ColorChanged).into()
//! }
//! ```

use crate::button;
use crate::container;
use crate::core;
use crate::core::Renderer as _;
use crate::core::gradient::Linear;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::renderer::{self, Quad};
use crate::core::widget::tree::{self, Tree};
use crate::core::widget::{Operation, Widget};
use crate::core::{
    Background, Border, Clipboard, Color, Element, Event, Gradient, Length, Padding, Point,
    Radians, Rectangle, Shadow, Shell, Size, Theme,
};
use crate::text_input;

// ============================================================================
// HSV Color Space
// ============================================================================

/// A color in HSV (Hue, Saturation, Value) color space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsv {
    /// Hue in degrees (0-360).
    pub hue: f32,
    /// Saturation (0-1).
    pub saturation: f32,
    /// Value/Brightness (0-1).
    pub value: f32,
}

impl Hsv {
    /// Create a new HSV color.
    pub fn new(hue: f32, saturation: f32, value: f32) -> Self {
        Self {
            hue: hue.rem_euclid(360.0),
            saturation: saturation.clamp(0.0, 1.0),
            value: value.clamp(0.0, 1.0),
        }
    }

    /// Convert to RGB Color.
    pub fn to_color(&self) -> Color {
        let h = self.hue / 60.0;
        let c = self.value * self.saturation;
        let x = c * (1.0 - (h.rem_euclid(2.0) - 1.0).abs());
        let m = self.value - c;

        let (r, g, b) = match h as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Color::from_rgb(r + m, g + m, b + m)
    }

    /// Create from RGB Color.
    pub fn from_color(color: Color) -> Self {
        let r = color.r;
        let g = color.g;
        let b = color.b;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let hue = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta).rem_euclid(6.0))
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        let saturation = if max == 0.0 { 0.0 } else { delta / max };
        let value = max;

        Self::new(hue, saturation, value)
    }
}

impl Default for Hsv {
    fn default() -> Self {
        Self::new(0.0, 1.0, 1.0)
    }
}

// ============================================================================
// Color Picker Widget
// ============================================================================

/// A color picker widget.
pub struct ColorPicker<'a, Message> {
    color: Color,
    on_change: Box<dyn Fn(Color) -> Message + 'a>,
    width: Length,
    height: f32,
}

/// Create a new color picker widget.
pub fn color_picker<'a, Message: Clone + 'static>(
    color: Color,
    on_change: impl Fn(Color) -> Message + 'a,
) -> ColorPicker<'a, Message> {
    ColorPicker {
        color,
        on_change: Box::new(on_change),
        width: Length::Fixed(280.0),
        height: 180.0,
    }
}

impl<'a, Message: Clone + 'static> ColorPicker<'a, Message> {
    /// Set the width of the color picker.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Set the height of the saturation-value area.
    #[must_use]
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl<'a, Message: Clone + 'static> From<ColorPicker<'a, Message>>
    for Element<'a, Message, crate::Theme, crate::Renderer>
{
    fn from(picker: ColorPicker<'a, Message>) -> Self {
        ColorPickerInner::new(picker.color, picker.on_change, picker.width, picker.height).into()
    }
}

// ============================================================================
// Internal Widget Implementation
// ============================================================================

struct ColorPickerInner<'a, Message> {
    hsv: Hsv,
    on_change: Box<dyn Fn(Color) -> Message + 'a>,
    width: Length,
    sv_height: f32,
    text_input: text_input::TextInput<'a, HexInputEvent>,
}

impl<'a, Message: Clone + 'static> ColorPickerInner<'a, Message> {
    fn new(
        color: Color,
        on_change: Box<dyn Fn(Color) -> Message + 'a>,
        width: Length,
        sv_height: f32,
    ) -> Self {
        Self {
            hsv: Hsv::from_color(color),
            on_change,
            width,
            sv_height,
            text_input: text_input::TextInput::new("#RRGGBB", "")
                .on_input(HexInputEvent::TextChanged)
                .on_paste(HexInputEvent::TextChanged)
                .on_submit(HexInputEvent::Submitted)
                .padding(Padding::new(8.0)),
        }
    }
}

#[derive(Debug, Clone)]
enum HexInputEvent {
    TextChanged(String),
    Submitted,
}

#[derive(Debug)]
struct PickerState {
    dragging_sv: bool,
    dragging_hue: bool,
    input: String,
}

impl PickerState {
    fn new(initial_color: Color) -> Self {
        Self {
            dragging_sv: false,
            dragging_hue: false,
            input: color_to_hex_string(initial_color),
        }
    }
}

impl<Message> Widget<Message, Theme, crate::Renderer>
    for ColorPickerInner<'_, Message>
where
    Message: Clone + 'static,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<PickerState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(PickerState::new(self.hsv.to_color()))
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.text_input as &dyn Widget<_, _, _>)]
    }

    fn diff(&self, _tree: &mut Tree) {
        // Do nothing so the child `TextInput` keeps its internal state.
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &crate::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let picker_state = tree.state.downcast_mut::<PickerState>();

        let width = match self.width {
            Length::Fixed(w) => w,
            Length::Fill | Length::FillPortion(_) => limits.max().width,
            Length::Shrink => 280.0,
        };

        // SV area + spacing + hue slider + spacing + color preview with hex
        let hue_slider_height = 16.0;
        let preview_height = 40.0;
        let spacing = 12.0;
        let total_height =
            self.sv_height + spacing + hue_slider_height + spacing + preview_height;

        // Keep the input in sync with the current color when it is not focused.
        let is_focused = tree.children[0]
            .state
            .downcast_ref::<text_input::State<<crate::Renderer as core::text::Renderer>::Paragraph>>()
            .is_focused();

        if !is_focused {
            picker_state.input = color_to_hex_string(self.hsv.to_color());

            // Rebuild the text input so it reflects the current input.
            self.text_input = text_input::TextInput::new("#RRGGBB", &picker_state.input)
                .on_input(HexInputEvent::TextChanged)
                .on_paste(HexInputEvent::TextChanged)
                .on_submit(HexInputEvent::Submitted)
                .padding(Padding::new(8.0));
        }

        let preview_size = preview_height;
        let input_width = (width - preview_size - spacing).max(0.0);
        let input_limits = layout::Limits::new(Size::ZERO, Size::new(input_width, preview_height))
            .width(Length::Fixed(input_width))
            .height(Length::Fixed(preview_height));

        let preview_y = self.sv_height + spacing + hue_slider_height + spacing;
        let input_node = self.text_input.layout(
            &mut tree.children[0],
            renderer,
            &input_limits,
            None,
        );

        layout::Node::with_children(
            Size::new(width, total_height),
            vec![input_node.move_to(Point::new(preview_size + spacing, preview_y))],
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state: &PickerState = tree.state.downcast_ref();

        let spacing = 12.0;
        let hue_slider_height = 16.0;
        let preview_height = 40.0;
        let border_radius = 8.0;

        // SV area bounds
        let sv_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: self.sv_height,
        };

        // Draw SV gradient using proper gradients like libcosmic
        // First: draw base hue color
        let base_color = Hsv::new(self.hsv.hue, 1.0, 1.0).to_color();
        renderer.fill_quad(
            Quad {
                bounds: sv_bounds,
                border: Border {
                    radius: border_radius.into(),
                    ..Default::default()
                },
                ..Quad::default()
            },
            Background::Color(base_color),
        );

        // Overlay horizontal white-to-transparent gradient (saturation)
        // NOTE: In `core::gradient::Linear`, 0 rad is vertical; use PI/2 for horizontal.
        let saturation_gradient = Linear::new(Radians(std::f32::consts::FRAC_PI_2))
            .add_stop(0.0, Color::WHITE)
            .add_stop(1.0, Color::TRANSPARENT);
        renderer.fill_quad(
            Quad {
                bounds: sv_bounds,
                border: Border {
                    radius: border_radius.into(),
                    ..Default::default()
                },
                ..Quad::default()
            },
            Background::Gradient(Gradient::Linear(saturation_gradient)),
        );

        // Overlay vertical transparent-to-black gradient (value)
        // We want top -> bottom, so use PI.
        let value_gradient = Linear::new(Radians(std::f32::consts::PI))
            .add_stop(0.0, Color::TRANSPARENT)
            .add_stop(1.0, Color::BLACK);
        renderer.fill_quad(
            Quad {
                bounds: sv_bounds,
                border: Border {
                    radius: border_radius.into(),
                    ..Default::default()
                },
                ..Quad::default()
            },
            Background::Gradient(Gradient::Linear(value_gradient)),
        );

        // Draw SV border
        renderer.fill_quad(
            Quad {
                bounds: sv_bounds,
                border: Border {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                    width: 1.0,
                    radius: border_radius.into(),
                },
                ..Quad::default()
            },
            Background::Color(Color::TRANSPARENT),
        );

        // Draw SV handle (cosmic style - circle with border)
        let handle_x = sv_bounds.x + self.hsv.saturation * sv_bounds.width;
        let handle_y = sv_bounds.y + (1.0 - self.hsv.value) * sv_bounds.height;
        let is_sv_hover = cursor.position().map(|p| sv_bounds.contains(p)).unwrap_or(false);
        draw_cosmic_handle(renderer, handle_x, handle_y, state.dragging_sv || is_sv_hover);

        // Hue slider bounds
        let hue_y = sv_bounds.y + sv_bounds.height + spacing;
        let hue_bounds = Rectangle {
            x: bounds.x,
            y: hue_y,
            width: bounds.width,
            height: hue_slider_height,
        };

        // Draw hue gradient (rainbow) using proper gradient
        // Hue bar should run left -> right.
        let hue_gradient = Linear::new(Radians(std::f32::consts::FRAC_PI_2))
            .add_stop(0.0, Hsv::new(0.0, 1.0, 1.0).to_color())
            .add_stop(0.166, Hsv::new(60.0, 1.0, 1.0).to_color())
            .add_stop(0.333, Hsv::new(120.0, 1.0, 1.0).to_color())
            .add_stop(0.5, Hsv::new(180.0, 1.0, 1.0).to_color())
            .add_stop(0.666, Hsv::new(240.0, 1.0, 1.0).to_color())
            .add_stop(0.833, Hsv::new(300.0, 1.0, 1.0).to_color())
            .add_stop(1.0, Hsv::new(360.0, 1.0, 1.0).to_color());

        renderer.fill_quad(
            Quad {
                bounds: hue_bounds,
                border: Border {
                    radius: (hue_slider_height / 2.0).into(),
                    ..Default::default()
                },
                ..Quad::default()
            },
            Background::Gradient(Gradient::Linear(hue_gradient)),
        );

        // Draw hue border
        renderer.fill_quad(
            Quad {
                bounds: hue_bounds,
                border: Border {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                    width: 1.0,
                    radius: (hue_slider_height / 2.0).into(),
                },
                ..Quad::default()
            },
            Background::Color(Color::TRANSPARENT),
        );

        // Draw hue handle
        let hue_handle_x = hue_bounds.x + (self.hsv.hue / 360.0) * hue_bounds.width;
        let hue_handle_y = hue_bounds.y + hue_bounds.height / 2.0;
        let is_hue_hover = cursor.position().map(|p| hue_bounds.contains(p)).unwrap_or(false);
        draw_cosmic_handle(renderer, hue_handle_x, hue_handle_y, state.dragging_hue || is_hue_hover);

        // Color preview with hex - combined into one row
        let preview_y = hue_y + hue_slider_height + spacing;
        let current_color = self.hsv.to_color();

        // Color preview square
        let preview_size = preview_height;
        let preview_bounds = Rectangle {
            x: bounds.x,
            y: preview_y,
            width: preview_size,
            height: preview_size,
        };

        // Draw checkerboard background for transparency indication
        draw_checkerboard(renderer, preview_bounds, border_radius);

        renderer.fill_quad(
            Quad {
                bounds: preview_bounds,
                border: Border {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                    width: 1.0,
                    radius: border_radius.into(),
                },
                ..Quad::default()
            },
            Background::Color(current_color),
        );

        // Draw the actual cosmic-styled input box (TextInput widget)
        if let Some(input_layout) = layout.children().next() {
            Widget::<HexInputEvent, Theme, crate::Renderer>::draw(
                &self.text_input,
                &tree.children[0],
                renderer,
                theme,
                style,
                input_layout,
                cursor,
                _viewport,
            );
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state: &mut PickerState = tree.state.downcast_mut();
        let bounds = layout.bounds();
        let spacing = 12.0;
        let hue_slider_height = 16.0;

        // SV area bounds
        let sv_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: self.sv_height,
        };

        // Hue slider bounds
        let hue_y = sv_bounds.y + sv_bounds.height + spacing;
        let hue_bounds = Rectangle {
            x: bounds.x,
            y: hue_y,
            width: bounds.width,
            height: hue_slider_height,
        };

        // First, allow the input box to handle the event.
        let mut input_captured_event = false;
        if let Some(input_layout) = layout.children().next() {
            let mut local_messages = Vec::new();
            let mut local_shell = Shell::new(&mut local_messages);

            self.text_input.update(
                &mut tree.children[0],
                event,
                input_layout,
                cursor,
                renderer,
                clipboard,
                &mut local_shell,
                viewport,
            );

            if local_shell.is_event_captured() {
                input_captured_event = true;
                shell.capture_event();
            }

            shell.request_redraw_at(local_shell.redraw_request());
            shell.request_input_method(local_shell.input_method());

            for message in local_messages {
                match message {
                    HexInputEvent::TextChanged(new_value) => {
                        state.input = new_value;
                        if let Some(color) = parse_hex_color(&state.input) {
                            self.hsv = Hsv::from_color(color);
                            shell.publish((self.on_change)(color));
                            shell.invalidate_layout();
                            shell.request_redraw();
                        }
                    }
                    HexInputEvent::Submitted => {
                        if let Some(color) = parse_hex_color(&state.input) {
                            self.hsv = Hsv::from_color(color);
                            shell.publish((self.on_change)(color));
                            shell.invalidate_layout();
                            shell.request_redraw();
                        }
                    }
                }
            }
        }

        if input_captured_event {
            return;
        }

        // Then, handle SV/Hue dragging.
        let is_input_focused = tree.children[0]
            .state
            .downcast_ref::<text_input::State<<crate::Renderer as core::text::Renderer>::Paragraph>>()
            .is_focused();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed { button: mouse::Button::Left, .. }) => {
                if let Some(pos) = cursor.position() {
                    if sv_bounds.contains(pos) {
                        state.dragging_sv = true;
                        self.update_sv_from_position(pos, sv_bounds, shell);
                        if !is_input_focused {
                            state.input = color_to_hex_string(self.hsv.to_color());
                        }
                    } else if hue_bounds.contains(pos) {
                        state.dragging_hue = true;
                        self.update_hue_from_position(pos, hue_bounds, shell);
                        if !is_input_focused {
                            state.input = color_to_hex_string(self.hsv.to_color());
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased { button: mouse::Button::Left, .. }) => {
                state.dragging_sv = false;
                state.dragging_hue = false;
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(pos) = cursor.position() {
                    if state.dragging_sv {
                        self.update_sv_from_position(pos, sv_bounds, shell);
                        if !is_input_focused {
                            state.input = color_to_hex_string(self.hsv.to_color());
                        }
                    } else if state.dragging_hue {
                        self.update_hue_from_position(pos, hue_bounds, shell);
                        if !is_input_focused {
                            state.input = color_to_hex_string(self.hsv.to_color());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        // Let the input box win if the cursor is over it.
        if let Some(input_layout) = layout.children().next() {
            let input_interaction =
                Widget::<HexInputEvent, Theme, crate::Renderer>::mouse_interaction(
                &self.text_input,
                &tree.children[0],
                input_layout,
                cursor,
                viewport,
                renderer,
            );

            if input_interaction != mouse::Interaction::default() {
                return input_interaction;
            }
        }

        let state: &PickerState = tree.state.downcast_ref();
        let bounds = layout.bounds();
        let spacing = 12.0;
        let hue_slider_height = 16.0;

        let sv_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: self.sv_height,
        };

        let hue_y = sv_bounds.y + sv_bounds.height + spacing;
        let hue_bounds = Rectangle {
            x: bounds.x,
            y: hue_y,
            width: bounds.width,
            height: hue_slider_height,
        };

        if state.dragging_sv || state.dragging_hue {
            mouse::Interaction::Grabbing
        } else if cursor
            .position()
            .map(|p| sv_bounds.contains(p) || hue_bounds.contains(p))
            .unwrap_or(false)
        {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }

    fn operate(
        &mut self,
        state: &mut Tree,
        _layout: Layout<'_>,
        _renderer: &crate::Renderer,
        _operation: &mut dyn Operation,
    ) {
        // Forward operations to children when needed.
        let _ = state;
    }
}

fn draw_cosmic_handle<Renderer: core::Renderer>(renderer: &mut Renderer, x: f32, y: f32, active: bool) {
    let radius = if active { 8.0 } else { 6.0 };
    let border_width = if active { 3.0 } else { 2.0 };

    // Outer border (dark)
    renderer.fill_quad(
        Quad {
            bounds: Rectangle {
                x: x - radius - 1.0,
                y: y - radius - 1.0,
                width: (radius + 1.0) * 2.0,
                height: (radius + 1.0) * 2.0,
            },
            border: Border {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                width: 1.0,
                radius: (radius + 1.0).into(),
            },
            shadow: Shadow::default(),
            ..Quad::default()
        },
        Background::Color(Color::TRANSPARENT),
    );

    // Main handle (white border, transparent fill)
    renderer.fill_quad(
        Quad {
            bounds: Rectangle {
                x: x - radius,
                y: y - radius,
                width: radius * 2.0,
                height: radius * 2.0,
            },
            border: Border {
                color: Color::WHITE,
                width: border_width,
                radius: radius.into(),
            },
            shadow: Shadow::default(),
            ..Quad::default()
        },
        Background::Color(Color::TRANSPARENT),
    );
}

fn draw_checkerboard<Renderer: core::Renderer>(renderer: &mut Renderer, bounds: Rectangle, _radius: f32) {
    let check_size = 6.0;
    let cols = (bounds.width / check_size).ceil() as i32;
    let rows = (bounds.height / check_size).ceil() as i32;

    for row in 0..rows {
        for col in 0..cols {
            let is_light = (row + col) % 2 == 0;
            let color = if is_light {
                Color::from_rgb(0.9, 0.9, 0.9)
            } else {
                Color::from_rgb(0.7, 0.7, 0.7)
            };

            let cell_bounds = Rectangle {
                x: bounds.x + col as f32 * check_size,
                y: bounds.y + row as f32 * check_size,
                width: (bounds.x + bounds.width - (bounds.x + col as f32 * check_size))
                    .min(check_size)
                    .max(0.0),
                height: (bounds.y + bounds.height - (bounds.y + row as f32 * check_size))
                    .min(check_size)
                    .max(0.0),
            };

            renderer.fill_quad(
                Quad {
                    bounds: cell_bounds,
                    border: Border::default(),
                    ..Quad::default()
                },
                Background::Color(color),
            );
        }
    }
}

fn color_to_hex_string(color: Color) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8
    )
}

fn parse_hex_color(input: &str) -> Option<Color> {
    let s = input.trim();
    let s = s.strip_prefix('#').unwrap_or(s);

    match s.len() {
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;

            Some(Color::from_rgb8(r, g, b))
        }
        3 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()?;
            let g = u8::from_str_radix(&s[1..2], 16).ok()?;
            let b = u8::from_str_radix(&s[2..3], 16).ok()?;

            // Expand #RGB -> #RRGGBB
            Some(Color::from_rgb8(r * 17, g * 17, b * 17))
        }
        _ => None,
    }
}

impl<Message: Clone + 'static> ColorPickerInner<'_, Message> {
    fn update_sv_from_position(
        &mut self,
        pos: Point,
        bounds: Rectangle,
        shell: &mut Shell<'_, Message>,
    ) {
        let s = ((pos.x - bounds.x) / bounds.width).clamp(0.0, 1.0);
        let v = 1.0 - ((pos.y - bounds.y) / bounds.height).clamp(0.0, 1.0);

        self.hsv.saturation = s;
        self.hsv.value = v;

        shell.publish((self.on_change)(self.hsv.to_color()));
    }

    fn update_hue_from_position(
        &mut self,
        pos: Point,
        bounds: Rectangle,
        shell: &mut Shell<'_, Message>,
    ) {
        let h = ((pos.x - bounds.x) / bounds.width).clamp(0.0, 1.0) * 360.0;
        self.hsv.hue = h;

        shell.publish((self.on_change)(self.hsv.to_color()));
    }
}

impl<'a, Message> From<ColorPickerInner<'a, Message>>
    for Element<'a, Message, Theme, crate::Renderer>
where
    Message: Clone + 'static,
{
    fn from(picker: ColorPickerInner<'a, Message>) -> Self {
        Element::new(picker)
    }
}

// ============================================================================
// Color Button (shows current color and can trigger picker)
// ============================================================================

/// Create a button that displays a color.
///
/// Useful for showing the currently selected color and opening a color picker.
pub fn color_button<'a, Message: Clone + 'static>(
    color: Color,
    on_press: Option<Message>,
) -> button::Button<'a, Message, crate::Theme, crate::Renderer> {
    let content = container::Container::new(crate::Space::new().width(Length::Fill).height(Length::Fill))
        .width(Length::Fixed(24.0))
        .height(Length::Fixed(24.0))
        .style(move |_theme| container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                color: Color::from_rgb(0.3, 0.3, 0.3),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

    let mut btn = button::Button::new(content).padding(4);
    if let Some(msg) = on_press {
        btn = btn.on_press(msg);
    }
    btn
}
