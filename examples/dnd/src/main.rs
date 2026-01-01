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

use icy_ui::dnd::{self, DragData, DropResult};
use icy_ui::event::{self, Event};
use icy_ui::mouse;
use icy_ui::widget::{column, container, row, text, text_input, Space};
use icy_ui::window;
use icy_ui::{Center, Element, Fill, Length, Point, Subscription, Task, Theme};

use std::path::PathBuf;

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
        position: Point,
        mime_types: Vec<String>,
    },
    DragMoved(Point),
    DragLeft,
    DragDropped {
        position: Point,
        data: Vec<u8>,
        mime_type: String,
    },

    // File drag events (winit native - works on X11/Windows/macOS)
    FileHovered(PathBuf),
    FileDropped(PathBuf),
    FilesHoveredLeft,
}

#[derive(Debug, Clone, Default)]
struct DragState {
    /// Current position of the drag cursor
    position: Point,
    /// MIME types offered by the drag source
    mime_types: Vec<String>,
    /// Files being hovered (from winit FileHovered events)
    hovered_files: Vec<PathBuf>,
    /// Dropped data (from DragDropped event)
    dropped_data: Option<DroppedData>,
    /// Dropped files (from winit FileDropped events)
    dropped_files: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
struct DroppedData {
    data: Vec<u8>,
    mime_type: String,
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
        event::listen_with(|event, _status, _id| match event {
            Event::Window(window_event) => match window_event {
                window::Event::DragEntered {
                    position,
                    mime_types,
                } => Some(Message::DragEntered {
                    position,
                    mime_types,
                }),
                window::Event::DragMoved { position } => Some(Message::DragMoved(position)),
                window::Event::DragLeft => Some(Message::DragLeft),
                window::Event::DragDropped {
                    position,
                    data,
                    mime_type,
                    ..
                } => Some(Message::DragDropped {
                    position,
                    data,
                    mime_type,
                }),
                window::Event::FileHovered(path) => Some(Message::FileHovered(path)),
                window::Event::FileDropped(path) => Some(Message::FileDropped(path)),
                window::Event::FilesHoveredLeft => Some(Message::FilesHoveredLeft),
                _ => None,
            },
            _ => None,
        })
    }

    fn update(&mut self, message: Message) -> Task<Message> {
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
                position,
                mime_types,
            } => {
                self.incoming_drag = Some(DragState {
                    position,
                    mime_types,
                    ..Default::default()
                });
            }

            Message::DragMoved(position) => {
                if let Some(ref mut drag) = self.incoming_drag {
                    drag.position = position;
                }
            }

            Message::DragLeft => {
                self.incoming_drag = None;
            }

            Message::DragDropped {
                position,
                data,
                mime_type,
            } => {
                // Add to history
                let preview = Self::preview_data(&data, &mime_type);
                self.drop_history.push(format!(
                    "Received at ({:.0}, {:.0}): {} - {}",
                    position.x, position.y, mime_type, preview
                ));

                // Update drag state to show drop result
                if let Some(ref mut drag) = self.incoming_drag {
                    drag.position = position;
                    drag.dropped_data = Some(DroppedData { data, mime_type });
                }
            }

            Message::FileHovered(path) => {
                if let Some(ref mut drag) = self.incoming_drag {
                    drag.hovered_files.push(path);
                } else {
                    self.incoming_drag = Some(DragState {
                        hovered_files: vec![path],
                        ..Default::default()
                    });
                }
            }

            Message::FileDropped(path) => {
                // Add to history
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                self.drop_history
                    .push(format!("File dropped: {}", filename));

                if let Some(ref mut drag) = self.incoming_drag {
                    drag.dropped_files.push(path);
                } else {
                    self.incoming_drag = Some(DragState {
                        dropped_files: vec![path],
                        ..Default::default()
                    });
                }
            }

            Message::FilesHoveredLeft => {
                self.incoming_drag = None;
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

        let icon = if drag.dropped_data.is_some() || !drag.dropped_files.is_empty() {
            text("✅").size(48)
        } else {
            text("📥").size(48)
        };

        // Content type description
        let content_type = Self::describe_mime_types(&drag.mime_types);
        let type_label = text(format!("Content: {}", content_type)).size(14);

        // Position
        let position_text = if drag.position != Point::ORIGIN {
            text(format!("({:.0}, {:.0})", drag.position.x, drag.position.y)).size(12)
        } else {
            text("").size(12)
        };

        // Files being hovered
        let files_info: Element<'_, Message> = if !drag.hovered_files.is_empty() {
            let file_names: Vec<String> = drag
                .hovered_files
                .iter()
                .filter_map(|p| p.file_name())
                .filter_map(|n| n.to_str())
                .map(|s| s.to_string())
                .collect();
            column![
                text(format!("📁 {} file(s)", drag.hovered_files.len())).size(12),
                text(file_names.join(", ")).size(10),
            ]
            .spacing(3)
            .into()
        } else {
            Space::new().height(0).into()
        };

        // Dropped data info
        let drop_info: Element<'_, Message> = if let Some(ref dropped) = drag.dropped_data {
            let preview = Self::preview_data(&dropped.data, &dropped.mime_type);
            column![
                text("✓ Dropped!").size(16),
                text(format!("{}", dropped.mime_type)).size(10),
                text(preview).size(11),
            ]
            .spacing(3)
            .into()
        } else if !drag.dropped_files.is_empty() {
            let file_names: Vec<String> = drag
                .dropped_files
                .iter()
                .filter_map(|p| p.file_name())
                .filter_map(|n| n.to_str())
                .map(|s| s.to_string())
                .collect();
            column![
                text("✓ Files Dropped!").size(16),
                text(file_names.join(", ")).size(10),
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
            files_info,
            drop_info,
        ]
        .spacing(5)
        .align_x(Center);

        let has_drop = drag.dropped_data.is_some() || !drag.dropped_files.is_empty();

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

    fn describe_mime_types(mime_types: &[String]) -> String {
        if mime_types.is_empty() {
            return "File(s)".into();
        }

        let has_files = mime_types
            .iter()
            .any(|m| m.contains("uri-list") || m.contains("file"));
        let has_text = mime_types.iter().any(|m| m.starts_with("text/plain"));
        let has_image = mime_types.iter().any(|m| m.starts_with("image/"));
        let has_html = mime_types.iter().any(|m| m.contains("html"));

        if has_files {
            "📁 File(s)".into()
        } else if has_image {
            "🖼️ Image".into()
        } else if has_html {
            "🌐 HTML".into()
        } else if has_text {
            "📝 Text".into()
        } else {
            format!("📦 {}", mime_types[0])
        }
    }

    fn preview_data(data: &[u8], mime_type: &str) -> String {
        if mime_type.starts_with("text/")
            || mime_type.contains("uri-list")
            || mime_type.contains("UTF8")
        {
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
        } else if mime_type.starts_with("image/") {
            format!("{} image", mime_type.replace("image/", "").to_uppercase())
        } else {
            "(binary data)".into()
        }
    }
}
