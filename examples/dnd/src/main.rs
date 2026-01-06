//! Drag and Drop Demo
//!
//! This demonstrates:
//! 1. **Drop Target**: Receiving drops from other applications
//!    - `DragEntered` - when a drag enters the window
//!    - `DragMoved` - as the cursor moves during drag
//!    - `DragDropped` - when data is dropped
//!    - `DragLeft` - when drag leaves without dropping
//!
//! 2. **Drag Source**: Starting a drag to other applications
//!    - Click and drag from the "Drag me!" box to start a drag

use icy_ui::dnd::{self, DndAction, DragData, DropResult};
use icy_ui::event::{self, Event};
use icy_ui::keyboard::Modifiers;
use icy_ui::mouse;
use icy_ui::widget::{column, container, row, text, text_input, Space};
use icy_ui::window;
use icy_ui::{Center, Element, Fill, Length, Point, Subscription, Task, Theme};

use std::borrow::Cow;

pub fn main() -> icy_ui::Result {
    icy_ui::application(DndDemo::default, DndDemo::update, DndDemo::view)
        .subscription(DndDemo::subscription)
        .title("Drag and Drop Demo")
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    // Text input for drag source
    TextChanged(String),

    // Start dragging (triggered by mouse press on drag source)
    StartDrag,

    // Drag completed
    DragCompleted(DropResult),

    // External drag events (from outside the app via Wayland DnD)
    DragEntered {
        window: window::Id,
        position: Point,
        formats: Vec<String>,
    },
    DragMoved {
        window: window::Id,
        position: Point,
        modifiers: Modifiers,
    },
    DragLeft(window::Id),
    DragDropped {
        window: window::Id,
        position: Point,
        data: Vec<u8>,
        format: String,
        action: DndAction,
    },
}

#[derive(Debug, Clone)]
struct DragState {
    window: window::Id,
    /// Current position of the drag cursor
    position: Point,
    /// Formats offered by the drag source
    formats: Vec<String>,
    /// Last observed modifiers during DragMoved (best-effort per platform)
    modifiers: Modifiers,
    /// Currently advertised action (what we last told the backend we will do)
    advertised_action: DndAction,
    /// Dropped data (from DragDropped event)
    dropped_data: Option<DroppedData>,
}

#[derive(Debug, Clone)]
struct DroppedData {
    data: Vec<u8>,
    format: String,
    action: DndAction,
}

struct DndDemo {
    /// Text to drag from this app
    drag_text: String,
    /// Whether we're currently dragging
    is_dragging: bool,
    /// Current incoming drag state (Some if a drag is in progress)
    incoming_drag: Option<DragState>,
    /// History of dropped items
    drop_history: Vec<String>,
}

impl Default for DndDemo {
    fn default() -> Self {
        Self {
            drag_text: "Hello from iced! 🦀".to_string(),
            is_dragging: false,
            incoming_drag: None,
            drop_history: Vec::new(),
        }
    }
}

impl DndDemo {
    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, id| match event {
            Event::Window(window_event) => match window_event {
                window::Event::DragEntered { position, formats } => Some(Message::DragEntered {
                    window: id,
                    position,
                    formats,
                }),
                window::Event::DragMoved {
                    position,
                    modifiers,
                } => Some(Message::DragMoved {
                    window: id,
                    position,
                    modifiers,
                }),
                window::Event::DragLeft => Some(Message::DragLeft(id)),
                window::Event::DragDropped {
                    position,
                    data,
                    format,
                    action,
                } => Some(Message::DragDropped {
                    window: id,
                    position,
                    data,
                    format,
                    action,
                }),
                _ => None,
            },
            _ => None,
        })
    }

    fn action_from_modifiers(modifiers: Modifiers) -> DndAction {
        if modifiers.shift() {
            DndAction::Move
        } else if modifiers.alt() {
            DndAction::Link
        } else {
            // Default + CTRL (common on Windows/Linux) both map to Copy.
            DndAction::Copy
        }
    }

    fn formats_to_cow(formats: &[String]) -> Vec<Cow<'static, str>> {
        formats.iter().cloned().map(Cow::Owned).collect()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        println!("Message: {:?}", message);
        match message {
            Message::TextChanged(new_text) => {
                self.drag_text = new_text;
            }

            Message::StartDrag => {
                if !self.drag_text.is_empty() {
                    self.is_dragging = true;
                    let data = DragData::from_text(&self.drag_text);
                    return dnd::start_drag(data, None).map(Message::DragCompleted);
                }
            }

            Message::DragCompleted(result) => {
                self.is_dragging = false;
                match result {
                    DropResult::Dropped(action) => {
                        self.drop_history
                            .push(format!("Outgoing drag completed: {:?}", action));
                    }
                    DropResult::Cancelled => {
                        self.drop_history
                            .push("Outgoing drag cancelled".to_string());
                    }
                }
            }

            Message::DragEntered {
                window,
                position,
                formats,
            } => {
                // Until we receive `DragMoved` (with modifiers), default to Copy.
                let action = DndAction::Copy;

                let formats_for_task = Self::formats_to_cow(&formats);

                self.incoming_drag = Some(DragState {
                    window,
                    position,
                    formats,
                    modifiers: Modifiers::empty(),
                    advertised_action: action,
                    dropped_data: None,
                });

                return dnd::accept_drag(window, formats_for_task, action);
            }

            Message::DragMoved {
                window,
                position,
                modifiers,
            } => {
                if self.incoming_drag.is_none() {
                    // Some Wayland compositor/source combinations can deliver motion
                    // without a prior Enter (or Enter can be missed). Keep the UI
                    // responsive by creating a minimal drag state.
                    self.incoming_drag = Some(DragState {
                        window,
                        position,
                        formats: Vec::new(),
                        modifiers,
                        advertised_action: Self::action_from_modifiers(modifiers),
                        dropped_data: None,
                    });
                }

                if let Some(ref mut drag) = self.incoming_drag {
                    if drag.window != window {
                        return Task::none();
                    }

                    drag.position = position;
                    drag.modifiers = modifiers;

                    let desired_action = Self::action_from_modifiers(modifiers);
                    if desired_action != drag.advertised_action {
                        drag.advertised_action = desired_action;
                        // Only (re)advertise acceptance if we have formats from DragEntered.
                        if !drag.formats.is_empty() {
                            let formats = Self::formats_to_cow(&drag.formats);
                            return dnd::accept_drag(window, formats, desired_action);
                        }
                    }
                }
            }

            Message::DragLeft(window) => {
                // Ensure we stop advertising acceptance when leaving.
                let had_drag = match self.incoming_drag.as_ref() {
                    Some(drag) => drag.window == window,
                    None => false,
                };

                // Keep the "Dropped!" UI visible even if the platform sends DragLeft
                // immediately after the drop.
                let keep_dropped = self
                    .incoming_drag
                    .as_ref()
                    .is_some_and(|drag| drag.window == window && drag.dropped_data.is_some());

                if !keep_dropped {
                    self.incoming_drag = None;
                }

                if had_drag && !keep_dropped {
                    return dnd::reject_drag(window);
                }
            }

            Message::DragDropped {
                window,
                position,
                data,
                format,
                action,
            } => {
                // Add to history
                let preview = Self::preview_data(&data, &format);
                self.drop_history.push(format!(
                    "Received at ({:.0}, {:.0}) as {:?}: {} - {}",
                    position.x, position.y, action, format, preview
                ));

                // Update drag state to show drop result.
                // If we didn't see DragEntered/DragMoved (or state was cleared), still
                // create the UI state so the drop feedback is visible.
                match self.incoming_drag.as_mut() {
                    Some(drag) if drag.window == window => {
                        drag.position = position;
                        if drag.formats.is_empty() {
                            drag.formats = vec![format.clone()];
                        }
                        drag.dropped_data = Some(DroppedData {
                            data,
                            format,
                            action,
                        });
                    }
                    _ => {
                        self.incoming_drag = Some(DragState {
                            window,
                            position,
                            formats: vec![format.clone()],
                            modifiers: Modifiers::empty(),
                            advertised_action: DndAction::Copy,
                            dropped_data: Some(DroppedData {
                                data,
                                format,
                                action,
                            }),
                        });
                    }
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let title = text("Drag and Drop Demo").size(28);

        // Left side: Drag source
        let drag_source = self.view_drag_source();

        // Right side: Drop target
        let drop_target: Element<'_, Message> = if let Some(ref drag) = self.incoming_drag {
            self.view_drag_active(drag)
        } else {
            self.view_drop_zone()
        };

        let main_row = row![drag_source, Space::new().width(40), drop_target,].align_y(Center);

        // Drop history
        let history: Element<'_, Message> = if self.drop_history.is_empty() {
            Space::new().height(0).into()
        } else {
            let history_items: Vec<Element<'_, Message>> = self
                .drop_history
                .iter()
                .rev()
                .take(5)
                .map(|s| text(s).size(12).into())
                .collect();

            column![text("History:").size(14), column(history_items).spacing(2),]
                .spacing(5)
                .into()
        };

        let content = column![
            title,
            Space::new().height(30),
            main_row,
            Space::new().height(30),
            history,
        ]
        .spacing(10)
        .padding(40)
        .align_x(Center);

        container(content)
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .into()
    }

    fn view_drag_source(&self) -> Element<'_, Message> {
        let title = text("Drag Source").size(18);

        let input = text_input("Enter text to drag...", &self.drag_text)
            .on_input(Message::TextChanged)
            .padding(10)
            .width(Length::Fixed(250.0));

        let drag_hint = if self.is_dragging {
            text("Dragging...").size(12)
        } else {
            text("Click and drag the box below ↓").size(12)
        };

        // The draggable box
        let drag_box_content = column![text("📤").size(24), text("Drag me!").size(14),]
            .spacing(5)
            .align_x(Center);

        let drag_box = container(drag_box_content)
            .width(Length::Fixed(120.0))
            .height(Length::Fixed(80.0))
            .center_x(Fill)
            .center_y(Fill)
            .style(move |theme: &Theme| {
                let bg = if self.is_dragging {
                    theme.accent.base
                } else {
                    theme.success.base
                };
                container::Style {
                    background: Some(bg.into()),
                    border: icy_ui::Border {
                        color: theme.success.on,
                        width: 2.0,
                        radius: 8.0.into(),
                    },
                    text_color: Some(theme.success.on),
                    ..Default::default()
                }
            });

        // Wrap in mouse_area to detect press
        let draggable = icy_ui::widget::mouse_area(drag_box)
            .on_press(Message::StartDrag)
            .interaction(if self.is_dragging {
                mouse::Interaction::Grabbing
            } else {
                mouse::Interaction::Grab
            });

        let content = column![
            title,
            Space::new().height(10),
            input,
            Space::new().height(10),
            drag_hint,
            Space::new().height(5),
            draggable,
        ]
        .spacing(5)
        .align_x(Center);

        container(content)
            .width(Length::Fixed(300.0))
            .height(Length::Fixed(280.0))
            .padding(20)
            .center_x(Fill)
            .style(|theme: &Theme| container::Style {
                background: Some(theme.primary.component.base.into()),
                border: icy_ui::Border {
                    color: theme.primary.base,
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    fn view_drop_zone(&self) -> Element<'_, Message> {
        let title = text("Drop Target").size(18);
        let icon = text("📥").size(48);
        let label = text("Drop here").size(16);
        let hint = text("From other apps").size(12);

        let content = column![title, Space::new().height(20), icon, label, hint]
            .spacing(10)
            .align_x(Center);

        container(content)
            .width(Length::Fixed(300.0))
            .height(Length::Fixed(280.0))
            .padding(20)
            .center_x(Fill)
            .center_y(Fill)
            .style(|theme: &Theme| container::Style {
                background: Some(theme.primary.base.into()),
                border: icy_ui::Border {
                    color: theme.primary.on,
                    width: 2.0,
                    radius: 12.0.into(),
                },
                text_color: Some(theme.primary.on),
                ..Default::default()
            })
            .into()
    }

    fn view_drag_active(&self, drag: &DragState) -> Element<'_, Message> {
        let title = text("Drop Target").size(18);

        let icon = if drag.dropped_data.is_some() {
            text("✅").size(48)
        } else {
            text("📥").size(48)
        };

        // Content type description
        let content_type = Self::describe_formats(&drag.formats);
        let type_label = text(format!("Content: {}", content_type)).size(14);

        // Position
        let position_text = if drag.position != Point::ORIGIN {
            text(format!("({:.0}, {:.0})", drag.position.x, drag.position.y)).size(12)
        } else {
            text("").size(12)
        };

        let action_text = text(format!("Action: {:?}", drag.advertised_action)).size(12);

        // Dropped data info
        let drop_info: Element<'_, Message> = if let Some(ref dropped) = drag.dropped_data {
            let preview = Self::preview_data(&dropped.data, &dropped.format);
            column![
                text("✓ Dropped!").size(16),
                text(format!("Action: {:?}", dropped.action)).size(12),
                text(format!("{}", dropped.format)).size(10),
                text(preview).size(11),
            ]
            .spacing(3)
            .into()
        } else {
            text("Release to drop...").size(12).into()
        };

        let content = column![
            title,
            icon,
            type_label,
            position_text,
            action_text,
            drop_info,
        ]
        .spacing(5)
        .align_x(Center);

        let has_drop = drag.dropped_data.is_some();

        container(content)
            .width(Length::Fixed(300.0))
            .height(Length::Fixed(280.0))
            .padding(20)
            .center_x(Fill)
            .center_y(Fill)
            .style(move |theme: &Theme| {
                let bg = if has_drop {
                    theme.success.base
                } else {
                    theme.accent.base
                };
                container::Style {
                    background: Some(bg.into()),
                    border: icy_ui::Border {
                        color: theme.accent.on,
                        width: 3.0,
                        radius: 12.0.into(),
                    },
                    text_color: Some(theme.accent.on),
                    ..Default::default()
                }
            })
            .into()
    }

    fn describe_formats(formats: &[String]) -> String {
        if formats.is_empty() {
            return "Unknown".into();
        }

        let has_files = formats
            .iter()
            .any(|m| m.contains("uri-list") || m.contains("file"));
        let has_text = formats.iter().any(|m| m.starts_with("text/plain"));
        let has_image = formats.iter().any(|m| m.starts_with("image/"));
        let has_html = formats.iter().any(|m| m.contains("html"));

        if has_files {
            "📁 File(s)".into()
        } else if has_image {
            "🖼️ Image".into()
        } else if has_html {
            "🌐 HTML".into()
        } else if has_text {
            "📝 Text".into()
        } else {
            format!("📦 {}", formats[0])
        }
    }

    fn preview_data(data: &[u8], format: &str) -> String {
        if format.starts_with("text/") || format.contains("uri-list") || format.contains("UTF8") {
            match std::str::from_utf8(data) {
                Ok(s) => {
                    let clean: String = s
                        .chars()
                        .filter(|c| !c.is_control() || *c == ' ')
                        .take(60)
                        .collect();
                    if s.len() > 60 {
                        format!("{}...", clean)
                    } else {
                        clean
                    }
                }
                Err(_) => "(binary)".into(),
            }
        } else if format.starts_with("image/") {
            format!("{} image", format.replace("image/", "").to_uppercase())
        } else {
            "(binary data)".into()
        }
    }
}
