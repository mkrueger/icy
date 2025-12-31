# Drag and Drop Support

icy provides drag and drop (DnD) support via a fork of [smithay-clipboard](https://github.com/mkrueger/smithay-clipboard), enabling file drops and data transfer between applications.

## Overview

DnD support uses the native platform mechanisms:
- **Linux**: Wayland data-device protocol / X11 XDND
- **Windows**: OLE Drag and Drop
- **macOS**: NSPasteboard / NSDraggingInfo

## Receiving Drops (Drop Target)

### Basic File Drop

Handle files dropped onto your application:

```rust
use iced::{Event, Task};
use iced::dnd::{DndEvent, DropEvent};

fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::Event(Event::Dnd(dnd_event)) => {
            match dnd_event {
                DndEvent::Drop(drop) => {
                    // Files were dropped
                    if let Some(paths) = drop.paths {
                        for path in paths {
                            println!("Dropped file: {}", path.display());
                        }
                    }
                }
                DndEvent::Enter { position, .. } => {
                    // Drag entered the window
                    self.drop_target_active = true;
                }
                DndEvent::Motion { position, .. } => {
                    // Drag is moving over the window
                    self.drop_position = position;
                }
                DndEvent::Leave => {
                    // Drag left the window
                    self.drop_target_active = false;
                }
            }
            Task::none()
        }
        _ => Task::none()
    }
}
```

### Drop Zones

Define specific areas that accept drops:

```rust
use iced::dnd::{set_drop_zones, DropZone};
use iced::Rectangle;

fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::SetupDropZones => {
            // Define drop zones with their bounds
            set_drop_zones(vec![
                DropZone {
                    id: "file-list".into(),
                    bounds: Rectangle::new(Point::new(0.0, 0.0), Size::new(400.0, 300.0)),
                    mime_types: vec!["text/uri-list".into()],
                },
                DropZone {
                    id: "image-area".into(),
                    bounds: Rectangle::new(Point::new(0.0, 300.0), Size::new(400.0, 200.0)),
                    mime_types: vec!["image/png".into(), "image/jpeg".into()],
                },
            ])
        }
        _ => Task::none()
    }
}
```

### Accepting or Rejecting Drops

Control whether a drop is accepted:

```rust
use iced::dnd::{accept_drag, reject_drag};

fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::Event(Event::Dnd(DndEvent::Enter { mime_types, .. })) => {
            // Check if we can handle the offered data
            if mime_types.contains(&"text/uri-list".to_string()) {
                accept_drag()
            } else {
                reject_drag()
            }
        }
        _ => Task::none()
    }
}
```

### Requesting Specific Data

Request data in a specific format when a drop occurs:

```rust
use iced::dnd::request_data;

fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::Event(Event::Dnd(DndEvent::Drop(_))) => {
            // Request the dropped data as text
            request_data("text/plain").map(Message::DataReceived)
        }
        Message::DataReceived(data) => {
            if let Some(bytes) = data {
                let text = String::from_utf8_lossy(&bytes);
                println!("Received: {}", text);
            }
            Task::none()
        }
        _ => Task::none()
    }
}
```

## Initiating Drags (Drag Source)

### Starting a Drag Operation

```rust
use iced::dnd::{start_drag, DragData};

fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::StartDrag(item_id) => {
            let item = &self.items[item_id];
            
            start_drag(DragData {
                // Offer data in multiple formats
                data: vec![
                    ("text/plain".into(), item.name.as_bytes().to_vec()),
                    ("text/uri-list".into(), item.path.to_string_lossy().as_bytes().to_vec()),
                ],
                // Optional: drag icon
                icon: None,
            })
        }
        _ => Task::none()
    }
}
```

### Drag with Actions

Specify allowed drag actions (copy, move, link):

```rust
use iced::dnd::{start_drag_with_actions, DragData, DndAction};

fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::StartDrag(item_id) => {
            start_drag_with_actions(
                DragData {
                    data: vec![("text/uri-list".into(), self.get_uri(item_id))],
                    icon: None,
                },
                DndAction::COPY | DndAction::MOVE,  // Allowed actions
            )
        }
        _ => Task::none()
    }
}
```

## DnD Events

| Event | Description |
|-------|-------------|
| `DndEvent::Enter` | Drag entered the window/drop zone |
| `DndEvent::Motion` | Drag is moving over the window |
| `DndEvent::Leave` | Drag left the window |
| `DndEvent::Drop` | Data was dropped |

### DropEvent Fields

```rust
pub struct DropEvent {
    /// Position where the drop occurred
    pub position: Point,
    /// Dropped file paths (if available)
    pub paths: Option<Vec<PathBuf>>,
    /// Raw data by MIME type
    pub data: HashMap<String, Vec<u8>>,
    /// The action that was performed (copy, move, etc.)
    pub action: DndAction,
}
```

## MIME Types

Common MIME types for drag and drop:

| MIME Type | Description |
|-----------|-------------|
| `text/plain` | Plain text |
| `text/uri-list` | List of file URIs |
| `text/html` | HTML content |
| `image/png` | PNG image data |
| `image/jpeg` | JPEG image data |
| `application/json` | JSON data |

### Checking Available Formats

```rust
DndEvent::Enter { mime_types, .. } => {
    for mime in &mime_types {
        println!("Offered format: {}", mime);
    }
    
    // Accept if any image format is offered
    let accepts_images = mime_types.iter().any(|m| m.starts_with("image/"));
    if accepts_images {
        accept_drag()
    } else {
        reject_drag()
    }
}
```

## Example: File Drop Zone Widget

```rust
use iced::widget::{container, text, Column};
use iced::{Element, Length};

fn view(&self) -> Element<Message> {
    let drop_zone = container(
        if self.drop_active {
            text("Drop files here!")
        } else {
            text("Drag files to upload")
        }
    )
    .width(Length::Fill)
    .height(200)
    .center_x()
    .center_y()
    .style(if self.drop_active {
        style::drop_zone_active
    } else {
        style::drop_zone
    });

    let file_list = Column::with_children(
        self.dropped_files.iter().map(|path| {
            text(path.file_name().unwrap_or_default().to_string_lossy()).into()
        })
    );

    Column::new()
        .push(drop_zone)
        .push(file_list)
        .into()
}

fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::Event(Event::Dnd(event)) => {
            match event {
                DndEvent::Enter { .. } => {
                    self.drop_active = true;
                }
                DndEvent::Leave => {
                    self.drop_active = false;
                }
                DndEvent::Drop(drop) => {
                    self.drop_active = false;
                    if let Some(paths) = drop.paths {
                        self.dropped_files.extend(paths);
                    }
                }
                _ => {}
            }
            Task::none()
        }
        _ => Task::none()
    }
}
```

## API Reference

### Types (from `iced::dnd`)

| Type | Description |
|------|-------------|
| `DndEvent` | Drag and drop event enum |
| `DropEvent` | Data from a completed drop |
| `DragData` | Data to be dragged |
| `DropZone` | Definition of a drop target area |
| `DndAction` | Drag action flags (Copy, Move, Link) |

### Functions (from `iced::dnd`)

| Function | Description |
|----------|-------------|
| `start_drag(data)` | Start a drag operation |
| `start_drag_with_actions(data, actions)` | Start drag with specific actions |
| `accept_drag()` | Accept the current drag |
| `reject_drag()` | Reject the current drag |
| `request_data(mime_type)` | Request dropped data in specific format |
| `set_drop_zones(zones)` | Define drop target areas |

## Platform Notes

### Linux (Wayland)
- Uses `smithay-clipboard` for Wayland data-device protocol
- Full support for MIME type negotiation
- Drag icons supported

### Linux (X11)
- Uses XDND protocol
- File drops work via `text/uri-list`

### Windows
- Uses OLE Drag and Drop
- File paths available in `DropEvent.paths`

### macOS
- Uses NSPasteboard
- File URLs converted to paths automatically
