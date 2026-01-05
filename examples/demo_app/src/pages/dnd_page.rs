//! Drag and Drop page - based on the `examples/dnd` demo.

use icy_ui::dnd::{self, DragData, DropResult};
use icy_ui::event::{self, Event};
use icy_ui::mouse;
use icy_ui::widget::{column, container, row, text, text_input, DropTarget, Space};
use icy_ui::window;
use icy_ui::{Center, Element, Fill, Length, Point, Subscription, Task, Theme};

use std::path::PathBuf;

use crate::Message;

#[derive(Debug, Clone)]
pub struct DndPageState {
    /// Text to drag from this app
    pub drag_text: String,
    /// Whether we're currently dragging
    pub is_dragging: bool,
    /// Current incoming drag state (Some if a drag is in progress)
    pub incoming_drag: Option<DragState>,
    /// History of dropped items
    pub drop_history: Vec<String>,
}

impl Default for DndPageState {
    fn default() -> Self {
        Self {
            drag_text: "Hello from icy_ui! 🦀".to_string(),
            is_dragging: false,
            incoming_drag: None,
            drop_history: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DragState {
    /// Current position of the drag cursor
    position: Point,
    /// Formats offered by the drag source
    formats: Vec<String>,
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
    format: String,
}

pub fn subscription_dnd() -> Subscription<Message> {
    event::listen_with(|event, _status, _id| match event {
        Event::Window(window_event) => match window_event {
            window::Event::FileHovered(path) => Some(Message::DndFileHovered(path)),
            window::Event::FileDropped(path) => Some(Message::DndFileDropped(path)),
            window::Event::FilesHoveredLeft => Some(Message::DndFilesHoveredLeft),
            _ => None,
        },
        _ => None,
    })
}

pub fn update_dnd(state: &mut DndPageState, message: &Message) -> Option<Task<Message>> {
    match message {
        Message::DndTextChanged(new_text) => {
            state.drag_text = new_text.clone();
            None
        }
        Message::DndStartDrag => {
            if state.drag_text.is_empty() {
                return None;
            }

            state.is_dragging = true;
            let data = DragData::from_text(&state.drag_text);
            Some(dnd::start_drag(data, None).map(Message::DndDragCompleted))
        }
        Message::DndDragCompleted(result) => {
            state.is_dragging = false;
            match result {
                DropResult::Dropped(action) => {
                    state
                        .drop_history
                        .push(format!("Outgoing drag completed: {:?}", action));
                }
                DropResult::Cancelled => {
                    state
                        .drop_history
                        .push("Outgoing drag cancelled".to_string());
                }
            }
            None
        }
        Message::DndDragEntered {
            position,
            mime_types,
        } => {
            state.incoming_drag = Some(DragState {
                position: *position,
                formats: mime_types.clone(),
                ..Default::default()
            });
            None
        }
        Message::DndDragMoved(position) => {
            if let Some(ref mut drag) = state.incoming_drag {
                drag.position = *position;
            }
            None
        }
        Message::DndDragLeft => {
            state.incoming_drag = None;
            None
        }
        Message::DndDragDropped {
            position,
            data,
            mime_type,
        } => {
            let preview = preview_data(data, mime_type);
            state.drop_history.push(format!(
                "Received at ({:.0}, {:.0}): {} - {}",
                position.x, position.y, mime_type, preview
            ));

            if let Some(ref mut drag) = state.incoming_drag {
                drag.position = *position;
                drag.dropped_data = Some(DroppedData {
                    data: data.clone(),
                    format: mime_type.clone(),
                });
            }

            None
        }
        Message::DndFileHovered(path) => {
            if let Some(ref mut drag) = state.incoming_drag {
                drag.hovered_files.push(path.clone());
            } else {
                state.incoming_drag = Some(DragState {
                    hovered_files: vec![path.clone()],
                    ..Default::default()
                });
            }
            None
        }
        Message::DndFileDropped(path) => {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            state
                .drop_history
                .push(format!("File dropped: {}", filename));

            if let Some(ref mut drag) = state.incoming_drag {
                drag.dropped_files.push(path.clone());
            } else {
                state.incoming_drag = Some(DragState {
                    dropped_files: vec![path.clone()],
                    ..Default::default()
                });
            }

            None
        }
        Message::DndFilesHoveredLeft => {
            // On Windows, file drops are followed by `FilesHoveredLeft`.
            // Keep the last drop visible; only clear hover state if nothing was dropped.
            let should_clear = state
                .incoming_drag
                .as_ref()
                .is_none_or(|drag| drag.dropped_data.is_none() && drag.dropped_files.is_empty());

            if should_clear {
                state.incoming_drag = None;
            }
            None
        }
        _ => None,
    }
}

pub fn view_dnd(state: &DndPageState) -> Element<'_, Message> {
    let title = text("Drag and Drop Demo").size(28);

    let drag_source = view_drag_source(state);

    let drop_target_content: Element<'_, Message> = if let Some(ref drag) = state.incoming_drag {
        view_drag_active(drag)
    } else {
        view_drop_zone()
    };

    let drop_target: Element<'_, Message> = DropTarget::new(drop_target_content)
        .on_enter(|position, mime_types| Message::DndDragEntered {
            position,
            mime_types,
        })
        .on_move(Message::DndDragMoved)
        .on_leave(Message::DndDragLeft)
        .on_drop(|position, data, mime_type| Message::DndDragDropped {
            position,
            data,
            mime_type,
        })
        .into();

    let main_row = row![drag_source, Space::new().width(40), drop_target,].align_y(Center);

    let history: Element<'_, Message> = if state.drop_history.is_empty() {
        Space::new().height(0).into()
    } else {
        let history_items: Vec<Element<'_, Message>> = state
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

fn view_drag_source(state: &DndPageState) -> Element<'_, Message> {
    let title = text("Drag Source").size(18);

    let input = text_input("Enter text to drag...", &state.drag_text)
        .on_input(Message::DndTextChanged)
        .padding(10)
        .width(Length::Fixed(250.0));

    let drag_hint = if state.is_dragging {
        text("Dragging...").size(12)
    } else {
        text("Click and drag the box below ↓").size(12)
    };

    let drag_box_content = column![text("📤").size(24), text("Drag me!").size(14),]
        .spacing(5)
        .align_x(Center);

    let is_dragging = state.is_dragging;

    let drag_box = container(drag_box_content)
        .width(Length::Fixed(120.0))
        .height(Length::Fixed(80.0))
        .center_x(Fill)
        .center_y(Fill)
        .style(move |theme: &Theme| {
            let bg = if is_dragging {
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

    let draggable = icy_ui::widget::mouse_area(drag_box)
        .on_press(Message::DndStartDrag)
        .interaction(if state.is_dragging {
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

fn view_drop_zone() -> Element<'static, Message> {
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

fn view_drag_active(drag: &DragState) -> Element<'static, Message> {
    let title = text("Drop Target").size(18);

    let icon = if drag.dropped_data.is_some() || !drag.dropped_files.is_empty() {
        text("✅").size(48)
    } else {
        text("📥").size(48)
    };

    let content_type = describe_formats(&drag.formats);
    let type_label = text(format!("Content: {}", content_type)).size(14);

    let position_text = if drag.position != Point::ORIGIN {
        text(format!("({:.0}, {:.0})", drag.position.x, drag.position.y)).size(12)
    } else {
        text("").size(12)
    };

    let files_info: Element<'static, Message> = if !drag.hovered_files.is_empty() {
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

    let drop_info: Element<'static, Message> = if let Some(ref dropped) = drag.dropped_data {
        let preview = preview_data(&dropped.data, &dropped.format);
        column![
            text("✓ Dropped!").size(16),
            text(format!("{}", dropped.format)).size(10),
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
        drop_info
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

fn describe_formats(formats: &[String]) -> String {
    if formats.is_empty() {
        return "File(s)".into();
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
