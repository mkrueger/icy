//! Drag and Drop Demo using the new Drag events.
//!
//! This demonstrates the new `Event::Drag` input events:
//! - `Drag::Begin` - when drag threshold is exceeded
//! - `Drag::Update` - as cursor moves during drag  
//! - `Drag::End` - when drag completes or is cancelled

use iced::event::{self, Event};
use iced::mouse;
use iced::widget::{column, container, row, text, Space};
use iced::window;
use iced::{color, Center, Element, Fill, Length, Point, Subscription, Task, Theme};

use std::path::PathBuf;

pub fn main() -> iced::Result {
    iced::application(DndDemo::default, DndDemo::update, DndDemo::view)
        .subscription(DndDemo::subscription)
        .title("Drag and Drop Demo")
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    // New drag events from the system
    DragBegan(Point),
    DragMoved(Point),
    DragEnded(Option<Point>),
    
    // For item selection
    ItemPressed(usize, usize),
    
    // Zone hover (for non-drag hover)
    EnteredZone(usize),
    LeftZone,
    
    // External drag events (from outside the app)
    ExternalDragEntered { position: Point, mime_types: Vec<String> },
    ExternalDragMoved(Point),
    ExternalDragLeft,
    ExternalDragDropped { data: Vec<u8>, mime_type: String },
    
    // File drag events (simpler winit-native events)
    FileHovered(PathBuf),
    FileDropped(PathBuf),
    FilesHoveredLeft,
}

#[derive(Debug, Clone)]
struct DraggableItem {
    label: String,
    color: iced::Color,
}

struct DndDemo {
    zones: Vec<Vec<DraggableItem>>,
    zone_names: Vec<String>,
    
    // The item being dragged (set on press, becomes drag on threshold)
    selected_item: Option<(usize, usize)>, // (zone, item)
    
    // Active drag state
    is_dragging: bool,
    drag_position: Point,
    
    // Hover state for zones
    hover_zone: Option<usize>,
    
    // External drag state
    external_drag: Option<ExternalDrag>,
}

#[derive(Debug, Clone)]
struct ExternalDrag {
    position: Point,
    mime_types: Vec<String>,
    dropped_data: Option<(Vec<u8>, String)>,
    // For file drops (simpler path-based)
    hovered_files: Vec<PathBuf>,
    dropped_files: Vec<PathBuf>,
}

impl Default for DndDemo {
    fn default() -> Self {
        Self {
            zones: vec![
                vec![
                    DraggableItem { label: "Document".into(), color: color!(0x5B8DEE) },
                    DraggableItem { label: "Image".into(), color: color!(0x50C878) },
                    DraggableItem { label: "Video".into(), color: color!(0xE85D75) },
                ],
                vec![
                    DraggableItem { label: "Music".into(), color: color!(0xAA7DCE) },
                ],
                vec![],
            ],
            zone_names: vec!["Inbox".into(), "Archive".into(), "Trash".into()],
            selected_item: None,
            is_dragging: false,
            drag_position: Point::ORIGIN,
            hover_zone: None,
            external_drag: None,
        }
    }
}

impl DndDemo {
    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _id| {
            match event {
                // Listen for the new Drag events
                Event::Drag(drag_event) => {
                    match drag_event {
                        iced::drag::Event::Begin { position, .. } => {
                            Some(Message::DragBegan(position))
                        }
                        iced::drag::Event::Update { position, .. } => {
                            Some(Message::DragMoved(position))
                        }
                        iced::drag::Event::End { position, .. } => {
                            Some(Message::DragEnded(position))
                        }
                    }
                }
                // Listen for external drag events (from outside the app)
                Event::Window(window_event) => {
                    match window_event {
                        window::Event::DragEntered { position, mime_types } => {
                            println!(">>> DragEntered at {:?}, mimes: {:?}", position, mime_types);
                            Some(Message::ExternalDragEntered { position, mime_types })
                        }
                        window::Event::DragMoved { position } => {
                            Some(Message::ExternalDragMoved(position))
                        }
                        window::Event::DragLeft => {
                            println!(">>> DragLeft");
                            Some(Message::ExternalDragLeft)
                        }
                        window::Event::DragDropped { data, mime_type, .. } => {
                            println!(">>> DragDropped: {} bytes, mime: {}", data.len(), mime_type);
                            Some(Message::ExternalDragDropped { data, mime_type })
                        }
                        // File drag events (winit native - works on all platforms)
                        window::Event::FileHovered(path) => {
                            println!(">>> FileHovered: {:?}", path);
                            Some(Message::FileHovered(path))
                        }
                        window::Event::FileDropped(path) => {
                            println!(">>> FileDropped: {:?}", path);
                            Some(Message::FileDropped(path))
                        }
                        window::Event::FilesHoveredLeft => {
                            println!(">>> FilesHoveredLeft");
                            Some(Message::FilesHoveredLeft)
                        }
                        _ => None,
                    }
                }
                _ => None,
            }
        })
    }

    fn can_drop(&self) -> bool {
        if self.is_dragging {
            if let Some((source_zone, _)) = self.selected_item {
                return self.hover_zone.map(|z| z != source_zone).unwrap_or(false);
            }
        }
        false
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ItemPressed(zone, item) => {
                // Remember which item was pressed
                self.selected_item = Some((zone, item));
            }
            
            Message::DragBegan(position) => {
                // Drag threshold exceeded - now we're really dragging
                if self.selected_item.is_some() {
                    self.is_dragging = true;
                    self.drag_position = position;
                }
            }
            
            Message::DragMoved(position) => {
                if self.is_dragging {
                    self.drag_position = position;
                }
            }
            
            Message::DragEnded(_position) => {
                if self.is_dragging {
                    if let Some((source_zone, item_idx)) = self.selected_item.take() {
                        if let Some(target_zone) = self.hover_zone {
                            if target_zone != source_zone && item_idx < self.zones[source_zone].len() {
                                let item = self.zones[source_zone].remove(item_idx);
                                self.zones[target_zone].push(item);
                            }
                        }
                    }
                }
                self.is_dragging = false;
                self.selected_item = None;
            }
            
            Message::EnteredZone(zone) => {
                self.hover_zone = Some(zone);
            }
            
            Message::LeftZone => {
                self.hover_zone = None;
            }
            
            // External drag events
            Message::ExternalDragEntered { position, mime_types } => {
                println!(">>> External drag entered at {:?}, mime_types: {:?}", position, mime_types);
                self.external_drag = Some(ExternalDrag {
                    position,
                    mime_types,
                    dropped_data: None,
                    hovered_files: Vec::new(),
                    dropped_files: Vec::new(),
                });
            }
            
            Message::ExternalDragMoved(position) => {
                println!(">>> External drag moved to {:?}", position);
                if let Some(ref mut drag) = self.external_drag {
                    drag.position = position;
                }
            }
            
            Message::ExternalDragLeft => {
                println!(">>> External drag left");
                self.external_drag = None;
            }
            
            Message::ExternalDragDropped { data, mime_type } => {
                println!(">>> External drag dropped: {} bytes, mime: {}", data.len(), mime_type);
                if let Some(ref mut drag) = self.external_drag {
                    drag.dropped_data = Some((data, mime_type));
                }
            }
            
            // File drag events (winit native)
            Message::FileHovered(path) => {
                if let Some(ref mut drag) = self.external_drag {
                    drag.hovered_files.push(path);
                } else {
                    // Start a new external drag for file
                    let mime = Self::mime_type_for_path(&path);
                    self.external_drag = Some(ExternalDrag {
                        position: Point::ORIGIN,
                        mime_types: vec![mime],
                        dropped_data: None,
                        hovered_files: vec![path],
                        dropped_files: Vec::new(),
                    });
                }
            }
            
            Message::FileDropped(path) => {
                if let Some(ref mut drag) = self.external_drag {
                    drag.dropped_files.push(path);
                } else {
                    let mime = Self::mime_type_for_path(&path);
                    self.external_drag = Some(ExternalDrag {
                        position: Point::ORIGIN,
                        mime_types: vec![mime],
                        dropped_data: None,
                        hovered_files: Vec::new(),
                        dropped_files: vec![path],
                    });
                }
            }
            
            Message::FilesHoveredLeft => {
                self.external_drag = None;
            }
        }
        Task::none()
    }
    
    fn mime_type_for_path(path: &std::path::Path) -> String {
        match path.extension().and_then(|e| e.to_str()) {
            Some("txt") => "text/plain".into(),
            Some("html" | "htm") => "text/html".into(),
            Some("png") => "image/png".into(),
            Some("jpg" | "jpeg") => "image/jpeg".into(),
            Some("gif") => "image/gif".into(),
            Some("svg") => "image/svg+xml".into(),
            Some("pdf") => "application/pdf".into(),
            Some("json") => "application/json".into(),
            Some("rs") => "text/x-rust".into(),
            Some("py") => "text/x-python".into(),
            Some("js") => "text/javascript".into(),
            Some(ext) => format!("application/x-{}", ext),
            None => "application/octet-stream".into(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let title = text("Drag and Drop Demo").size(24);
        let instructions = text("Uses new Event::Drag system events").size(14);

        // Check for external drag - show special view
        if let Some(ref external_drag) = self.external_drag {
            return self.view_external_drag(external_drag);
        }

        let zones_row: Element<'_, Message> = row(
            self.zones.iter().enumerate().map(|(zone_idx, items)| {
                self.view_zone(zone_idx, items)
            }).collect::<Vec<_>>()
        ).spacing(20).into();

        // Status
        let status = if self.is_dragging {
            if let Some((source_zone, item_idx)) = self.selected_item {
                let item_label = &self.zones[source_zone][item_idx].label;
                let target_info = match self.hover_zone {
                    Some(z) if z != source_zone => format!("→ {} ✓ Drop OK", self.zone_names[z]),
                    Some(z) => format!("→ {} ✗ Same zone", self.zone_names[z]),
                    None => "→ (outside) ✗".into(),
                };
                text(format!("Dragging: {} {} | pos: ({:.0}, {:.0})", 
                    item_label, target_info, 
                    self.drag_position.x, self.drag_position.y))
            } else {
                text("Dragging...")
            }
        } else if self.selected_item.is_some() {
            text("Mouse down - move to start drag...")
        } else {
            text("Click and drag an item to move it")
        };

        // Drag indicator
        let drag_indicator: Element<'_, Message> = if self.is_dragging {
            if let Some((source_zone, item_idx)) = self.selected_item {
                let item = &self.zones[source_zone][item_idx];
                let can = self.can_drop();
                container(
                    container(
                        row![
                            text(&item.label).size(14),
                            Space::new().width(10),
                            text(if can { "✓" } else { "✗" }).size(16),
                        ]
                    )
                    .padding(10)
                    .style(move |theme: &Theme| {
                        container::Style {
                            background: Some(item.color.into()),
                            border: iced::Border {
                                color: theme.primary.base,
                                width: 2.0,
                                radius: 4.0.into(),
                            },
                            text_color: Some(theme.background.on),
                            shadow: iced::Shadow {
                                color: theme.shade,
                                offset: iced::Vector::new(4.0, 4.0),
                                blur_radius: 8.0,
                            },
                            ..Default::default()
                        }
                    })
                )
                .width(Fill)
                .height(Length::Fixed(60.0))
                .center_x(Fill)
                .into()
            } else {
                Space::new().height(60).into()
            }
        } else {
            Space::new().height(60).into()
        };

        let content = column![
            title,
            instructions,
            Space::new().height(10),
            drag_indicator,
            zones_row,
            Space::new().height(20),
            status,
        ]
        .spacing(10)
        .padding(40)
        .align_x(Center);

        // Set cursor based on state
        let cursor = if self.is_dragging {
            if self.can_drop() {
                mouse::Interaction::Move
            } else {
                mouse::Interaction::NoDrop
            }
        } else {
            mouse::Interaction::default()
        };

        container(
            iced::widget::mouse_area(content).interaction(cursor)
        )
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .center_y(Fill)
        .into()
    }

    fn view_zone(&self, zone_idx: usize, items: &[DraggableItem]) -> Element<'_, Message> {
        let is_source = self.selected_item.map(|(z, _)| z == zone_idx).unwrap_or(false);
        let is_hover = self.hover_zone == Some(zone_idx);
        let is_valid_drop = is_hover && self.is_dragging && 
            self.selected_item.map(|(z, _)| z != zone_idx).unwrap_or(false);
        let is_invalid_hover = is_hover && is_source && self.is_dragging;

        let indicator = if is_valid_drop { " ✓" } else if is_invalid_hover { " ✗" } else { "" };
        let zone_title = text(format!("{}{}", self.zone_names[zone_idx], indicator)).size(16);

        let items_col: Vec<Element<'_, Message>> = items.iter().enumerate().map(|(item_idx, item)| {
            self.view_item(zone_idx, item_idx, item)
        }).collect();

        let items_view: Element<'_, Message> = if items_col.is_empty() {
            column![text("(empty)").size(12)].into()
        } else {
            column(items_col).spacing(8).into()
        };

        let zone_content = column![zone_title, Space::new().height(10), items_view]
            .spacing(5)
            .padding(15)
            .width(Length::Fixed(180.0))
            .height(Length::Fixed(280.0));

        let zone_container = container(zone_content)
            .style(move |theme: &Theme| {
                let (bg, text_color, border_color, border_width) = if is_valid_drop {
                    (theme.success.base, theme.success.on, theme.success.on, 3.0)
                } else if is_invalid_hover {
                    (theme.destructive.base, theme.destructive.on, theme.destructive.on, 3.0)
                } else {
                    (theme.primary.base, theme.primary.on, theme.primary.on, 1.0)
                };
                container::Style {
                    background: Some(bg.into()),
                    border: iced::Border {
                        color: border_color,
                        width: border_width,
                        radius: 8.0.into(),
                    },
                    text_color: Some(text_color),
                    ..Default::default()
                }
            });

        // Wrap zone in mouse_area to track hover
        iced::widget::mouse_area(zone_container)
            .on_enter(Message::EnteredZone(zone_idx))
            .on_exit(Message::LeftZone)
            .into()
    }

    fn view_item(&self, zone_idx: usize, item_idx: usize, item: &DraggableItem) -> Element<'_, Message> {
        let is_dragging = self.is_dragging && 
            self.selected_item.map(|(z, i)| z == zone_idx && i == item_idx).unwrap_or(false);

        let item_color = if is_dragging {
            iced::Color { a: 0.3, ..item.color }
        } else {
            item.color
        };

        let item_widget = container(text(item.label.clone()).size(14))
            .padding(10)
            .width(Fill)
            .style(move |theme: &Theme| {
                container::Style {
                    background: Some(item_color.into()),
                    border: iced::Border {
                        color: theme.primary.on,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    text_color: Some(theme.background.on),
                    ..Default::default()
                }
            });

        // Grab cursor when hovering items
        let item_cursor = if self.is_dragging {
            if self.can_drop() {
                mouse::Interaction::Move
            } else {
                mouse::Interaction::NoDrop
            }
        } else {
            mouse::Interaction::Grab
        };

        iced::widget::mouse_area(item_widget)
            .on_press(Message::ItemPressed(zone_idx, item_idx))
            .interaction(item_cursor)
            .into()
    }

    fn view_external_drag(&self, drag: &ExternalDrag) -> Element<'_, Message> {
        // Determine content type from MIME types
        let content_type = Self::describe_mime_types(&drag.mime_types);
        
        let title = text("External Drag").size(28);
        
        let type_label = text(format!("Type: {}", content_type)).size(18);
        
        let mime_list = text(format!("MIME types: {}", drag.mime_types.join(", "))).size(12);
        
        let position_text = if drag.position != Point::ORIGIN {
            text(format!("Position: ({:.0}, {:.0})", drag.position.x, drag.position.y)).size(14)
        } else {
            text("Position: (tracking via winit)").size(14)
        };
        
        // Show file info if we have hovered files
        let files_info: Element<'_, Message> = if !drag.hovered_files.is_empty() {
            let file_names: Vec<String> = drag.hovered_files.iter()
                .filter_map(|p| p.file_name())
                .filter_map(|n| n.to_str())
                .map(|s| s.to_string())
                .collect();
            column![
                text(format!("📁 {} file(s) hovering:", drag.hovered_files.len())).size(16),
                text(file_names.join(", ")).size(12),
            ].spacing(5).into()
        } else {
            Space::new().height(0).into()
        };
        
        // Show dropped files if we have them
        let dropped_files_info: Element<'_, Message> = if !drag.dropped_files.is_empty() {
            let file_names: Vec<String> = drag.dropped_files.iter()
                .filter_map(|p| p.file_name())
                .filter_map(|n| n.to_str())
                .map(|s| s.to_string())
                .collect();
            column![
                text("✓ Files Dropped!").size(20),
                text(format!("{} file(s):", drag.dropped_files.len())).size(14),
                text(file_names.join(", ")).size(12),
            ].spacing(5).into()
        } else {
            Space::new().height(0).into()
        };
        
        // Show dropped data info if available (from DragDropped event)
        let drop_info: Element<'_, Message> = if let Some((ref data, ref mime)) = drag.dropped_data {
            let preview = Self::preview_data(data, mime);
            column![
                text("✓ Data Dropped!").size(20),
                text(format!("MIME: {}", mime)).size(14),
                text(format!("Size: {} bytes", data.len())).size(14),
                text(format!("Preview: {}", preview)).size(12),
            ].spacing(5).into()
        } else if drag.dropped_files.is_empty() {
            text("Drop here to receive data...").size(16).into()
        } else {
            Space::new().height(0).into()
        };
        
        let content = column![
            title,
            Space::new().height(20),
            type_label,
            mime_list,
            Space::new().height(10),
            position_text,
            files_info,
            Space::new().height(20),
            dropped_files_info,
            drop_info,
        ]
        .spacing(10)
        .padding(40)
        .align_x(Center);

        let drop_area = container(content)
            .width(Length::Fixed(500.0))
            .height(Length::Fixed(400.0))
            .center_x(Fill)
            .center_y(Fill)
            .style(|theme: &Theme| {
                container::Style {
                    background: Some(theme.accent.base.into()),
                    border: iced::Border {
                        color: theme.accent.on,
                        width: 3.0,
                        radius: 12.0.into(),
                    },
                    text_color: Some(theme.accent.on),
                    ..Default::default()
                }
            });

        container(drop_area)
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .into()
    }
    
    fn describe_mime_types(mime_types: &[String]) -> String {
        // Check for common types
        let has_files = mime_types.iter().any(|m| m.contains("uri-list") || m.contains("file"));
        let has_text = mime_types.iter().any(|m| m.starts_with("text/"));
        let has_image = mime_types.iter().any(|m| m.starts_with("image/"));
        let has_html = mime_types.iter().any(|m| m.contains("html"));
        
        if has_files {
            "📁 File(s)".into()
        } else if has_image {
            let format = mime_types.iter()
                .find(|m| m.starts_with("image/"))
                .map(|m| m.replace("image/", "").to_uppercase())
                .unwrap_or_else(|| "Image".into());
            format!("🖼️ Image: {}", format)
        } else if has_html {
            "🌐 HTML".into()
        } else if has_text {
            "📝 Text".into()
        } else if !mime_types.is_empty() {
            format!("📦 {}", mime_types[0])
        } else {
            "❓ Unknown".into()
        }
    }
    
    fn preview_data(data: &[u8], mime_type: &str) -> String {
        if mime_type.starts_with("text/") || mime_type.contains("uri-list") {
            // Try to show text preview
            match std::str::from_utf8(data) {
                Ok(s) => {
                    let preview: String = s.chars().take(100).collect();
                    if s.len() > 100 {
                        format!("{}...", preview)
                    } else {
                        preview
                    }
                }
                Err(_) => "(binary data)".into(),
            }
        } else if mime_type.starts_with("image/") {
            format!("{} image data", mime_type.replace("image/", "").to_uppercase())
        } else {
            "(binary data)".into()
        }
    }
}
