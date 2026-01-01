//! Event Log page - displays all incoming events for debugging
//!
//! This page is useful for testing URL handling and other platform events.

use icy_ui::event::{self, Event};
use icy_ui::widget::{button, column, container, row, scrollable, space, text};
use icy_ui::{Element, Fill, Subscription};

use crate::Message;

/// Maximum number of events to keep in history
const MAX_EVENTS: usize = 100;

#[derive(Debug, Clone, Default)]
pub struct EventLogState {
    /// History of events
    pub events: Vec<EventEntry>,
    /// Counter for event IDs
    next_id: usize,
}

#[derive(Debug, Clone)]
pub struct EventEntry {
    pub id: usize,
    pub event_type: String,
    pub details: String,
}

impl EventLogState {
    pub fn add_event(&mut self, event_type: &str, details: &str) {
        self.events.push(EventEntry {
            id: self.next_id,
            event_type: event_type.to_string(),
            details: details.to_string(),
        });

        self.next_id += 1;

        // Keep only the last MAX_EVENTS events
        if self.events.len() > MAX_EVENTS {
            self.events.remove(0);
        }
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

/// Subscribe to all events for logging
pub fn subscription_event_log() -> Subscription<Message> {
    // Combine regular events with URL events
    let events_sub = event::listen_with(|event, _status, _id| {
        // Convert event to a loggable message
        let (event_type, details) = match &event {
            Event::Keyboard(kbd_event) => ("Keyboard".to_string(), format!("{:?}", kbd_event)),
            Event::Mouse(mouse_event) => {
                // Filter out frequent mouse move events to reduce noise
                match mouse_event {
                    icy_ui::mouse::Event::CursorMoved { .. } => {
                        // Skip cursor move events to reduce spam
                        return None;
                    }
                    _ => ("Mouse".to_string(), format!("{:?}", mouse_event)),
                }
            }
            Event::Touch(touch_event) => ("Touch".to_string(), format!("{:?}", touch_event)),
            Event::Window(window_event) => ("Window".to_string(), format!("{:?}", window_event)),
            Event::InputMethod(im_event) => ("InputMethod".to_string(), format!("{:?}", im_event)),
            // Catch any other events (e.g., Accessibility when feature is enabled)
            #[allow(unreachable_patterns)]
            _ => ("Other".to_string(), format!("{:?}", event)),
        };

        Some(Message::EventLogReceived {
            event_type,
            details,
        })
    });

    // Subscribe to URL events (macOS only, requires bundled app)
    let url_sub = event::listen_url().map(|url| Message::EventLogReceived {
        event_type: "URL".to_string(),
        details: url,
    });

    Subscription::batch([events_sub, url_sub])
}

pub fn update_event_log(state: &mut EventLogState, message: &Message) -> Option<String> {
    match message {
        Message::EventLogReceived {
            event_type,
            details,
        } => {
            state.add_event(event_type, details);
            Some(format!("Event received: {}", event_type))
        }
        Message::EventLogClear => {
            state.clear();
            Some("Event log cleared".to_string())
        }
        _ => None,
    }
}

pub fn view_event_log(state: &EventLogState) -> Element<'_, Message> {
    let title = text("Event Log").size(20);

    let description = text(
        "This page displays all incoming events. Useful for testing URL handling, \
         keyboard events, window events, and platform-specific events.",
    );

    let clear_button = button(text("Clear Log")).on_press(Message::EventLogClear);

    let event_count = text(format!("Events: {}", state.events.len())).size(14);

    let header = row![clear_button, space().width(20), event_count,].spacing(10);

    // Create the event list
    let event_list: Element<'_, Message> = if state.events.is_empty() {
        container(text("No events yet. Interact with the application to see events...").size(14))
            .padding(20)
            .into()
    } else {
        let events_column = state
            .events
            .iter()
            .rev()
            .fold(column![].spacing(4), |col, entry| {
                let event_row = row![
                    text(format!("#{}", entry.id)).size(12),
                    space().width(10),
                    text(&entry.event_type).size(12),
                    space().width(10),
                    text(&entry.details).size(11),
                ]
                .spacing(5);

                col.push(event_row)
            });

        scrollable(events_column).height(Fill).into()
    };

    let url_test_section = column![
        text("URL Handler Test").size(16),
        text("To test URL handling on macOS:").size(12),
        text("1. Build the app as a proper .app bundle").size(11),
        text("2. Add CFBundleURLTypes to Info.plist").size(11),
        text("3. Open a URL like: demoapp://test/action").size(11),
        text("URL events will appear in the log above.").size(11),
    ]
    .spacing(4);

    column![
        title,
        space().height(10),
        description,
        space().height(20),
        header,
        space().height(10),
        event_list,
        space().height(20),
        url_test_section,
    ]
    .spacing(5)
    .padding(10)
    .into()
}
