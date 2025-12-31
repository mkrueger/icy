//! Accessibility example
//!
//! Run with:
//! - `cargo run -p accessibility`
//!
//! On macOS, turn VoiceOver on (Cmd+F5) to test announcements.

use iced::accessibility::{self, Priority, WidgetInfo};
use iced::advanced::Renderer as _;
use iced::widget::{Column, button, checkbox, column, container, row, text, text_input, toggler};
use iced::{Color, Element, Event, Fill, Length, Rectangle, Renderer, Size, Task, Theme};

pub fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .subscription(App::subscription)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    CheckboxToggled(bool),
    TogglerToggled(bool),
    AnnouncePolite,
    AnnounceAssertive,
    CustomPressed,
    AxEvent(iced::accessibility::Event),
}

struct App {
    value: String,
    checked: bool,
    enabled: bool,
    last_ax: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            value: String::new(),
            checked: false,
            enabled: false,
            last_ax: None,
        }
    }
}

impl App {
    fn subscription(&self) -> iced::Subscription<Message> {
        iced::event::listen_with(|event, _status, _id| match event {
            Event::Accessibility(ax) => Some(Message::AxEvent(ax)),
            _ => None,
        })
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InputChanged(v) => {
                self.value = v;
                Task::none()
            }
            Message::CheckboxToggled(v) => {
                self.checked = v;
                Task::none()
            }
            Message::TogglerToggled(v) => {
                self.enabled = v;
                Task::none()
            }
            Message::AnnouncePolite => accessibility::announce(
                format!("Polite announcement: value is '{}'", self.value),
                Priority::Polite,
            )
            .discard(),
            Message::AnnounceAssertive => accessibility::announce(
                "Assertive announcement: something happened",
                Priority::Assertive,
            )
            .discard(),
            Message::CustomPressed => {
                self.last_ax = Some("Custom widget pressed".to_string());
                accessibility::announce("Custom widget pressed", Priority::Polite).discard()
            }
            Message::AxEvent(ax) => {
                self.last_ax = Some(format!("AX event: {:?} target={:?}", ax.action, ax.target));
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let input = text_input("Type here…", &self.value)
            .on_input(Message::InputChanged)
            .width(Length::Fill);

        let custom =
            CustomAccessibleButton::new("Custom accessible control", Message::CustomPressed)
                .width(Length::Fill);

        let content: Column<Message> = column![
            text("Accessibility example (AccessKit)").size(24),
            text("Use a screen reader to interact. The custom control responds to AX click/focus."),
            row![
                button("Announce (polite)").on_press(Message::AnnouncePolite),
                button("Announce (assertive)").on_press(Message::AnnounceAssertive),
            ]
            .spacing(12),
            input,
            row![
                checkbox(self.checked)
                    .label("Checkbox")
                    .on_toggle(Message::CheckboxToggled),
                toggler(self.enabled)
                    .label("Toggler")
                    .on_toggle(Message::TogglerToggled),
            ]
            .spacing(16),
            custom,
            text(self.last_ax.as_deref().unwrap_or("Last AX event: (none)")).style(
                |_theme: &Theme| iced::widget::text::Style {
                    color: Some(Color::from_rgb8(80, 80, 80)),
                }
            ),
        ]
        .spacing(16)
        .max_width(720);

        container(content)
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .padding(24)
            .into()
    }
}

/// A minimal custom widget that:
/// - draws a simple rectangle
/// - reports its own accessibility node
/// - handles `Event::Accessibility` click/focus/blur
struct CustomAccessibleButton<Message> {
    id: iced::widget::Id,
    label: String,
    on_press: Message,
    width: Length,
}

impl<Message: Clone> CustomAccessibleButton<Message> {
    fn new(label: impl Into<String>, on_press: Message) -> Self {
        Self {
            id: iced::widget::Id::new("custom_accessible_button"),
            label: label.into(),
            on_press,
            width: Length::Shrink,
        }
    }

    fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }
}

impl<Message, Theme_> iced::advanced::Widget<Message, Theme_, Renderer>
    for CustomAccessibleButton<Message>
where
    Message: Clone + 'static,
    Theme_: 'static,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Fixed(44.0),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let size = limits.resolve(self.width, Length::Fixed(44.0), Size::ZERO);
        iced::advanced::layout::Node::new(size)
    }

    fn draw(
        &self,
        _tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme_,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        renderer.fill_quad(
            iced::advanced::renderer::Quad {
                bounds,
                border: iced::Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(120, 120, 120),
                },
                ..iced::advanced::renderer::Quad::default()
            },
            Color::from_rgb8(245, 245, 245),
        );

        // Draw label using regular Text widget would require composition; keep draw minimal.
    }

    fn operate(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        operation.container(None, layout.bounds());

        let info = WidgetInfo::button(self.label.clone())
            .with_bounds(layout.bounds())
            .with_enabled(true);

        operation.accessibility(Some(&self.id), layout.bounds(), info);

        // We don’t participate in keyboard focus here; this is purely an AX demo.
    }

    fn update(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        event: &Event,
        _layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        if let Event::Accessibility(ax) = event {
            if ax.target != iced::accessibility::node_id_from_widget_id(&self.id) {
                return;
            }

            if ax.is_click() {
                shell.publish(self.on_press.clone());
                shell.capture_event();
            }
        }
    }
}

impl<'a, Message> From<CustomAccessibleButton<Message>> for Element<'a, Message>
where
    Message: Clone + 'static,
{
    fn from(widget: CustomAccessibleButton<Message>) -> Self {
        iced::Element::new(widget)
    }
}
