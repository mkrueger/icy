//! Drag gesture detection.
//!
//! This module provides a `DragDetector` that watches mouse events
//! and detects when a drag gesture should start (mouse moved past threshold
//! while button is held).

use crate::core::drag::{Actions, DragId, DragOffer, Event as DragEvent, Outcome};
use crate::core::keyboard::Modifiers;
use crate::core::mouse;
use crate::core::{Event, Point};
use std::sync::atomic::{AtomicU64, Ordering};

/// Global drag ID counter
static DRAG_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_drag_id() -> DragId {
    DragId(DRAG_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Configuration for drag detection.
#[derive(Debug, Clone)]
pub struct DragConfig {
    /// Minimum distance (in logical pixels) the cursor must move
    /// before a drag is recognized.
    pub threshold: f32,
    /// Which mouse button initiates a drag.
    pub button: mouse::Button,
}

impl Default for DragConfig {
    fn default() -> Self {
        Self {
            threshold: 5.0, // 5 pixels is a common default
            button: mouse::Button::Left,
        }
    }
}

/// State for an active drag operation.
#[derive(Debug, Clone)]
struct ActiveDrag {
    /// The drag ID.
    id: DragId,
    /// Starting position.
    start_position: Point,
    /// Whether Begin event has been sent.
    began: bool,
    /// Current keyboard modifiers.
    modifiers: Modifiers,
}

/// Detects drag gestures from mouse events.
///
/// Tracks mouse button press/release and cursor movement to determine
/// when a drag gesture starts (threshold exceeded).
#[derive(Debug, Default)]
pub struct DragDetector {
    /// Configuration for drag detection.
    config: DragConfig,
    /// Current mouse position.
    cursor_position: Point,
    /// Current keyboard modifiers.
    modifiers: Modifiers,
    /// State when mouse button is pressed (potential drag).
    pending: Option<PendingDrag>,
    /// Active drag operation (threshold exceeded).
    active: Option<ActiveDrag>,
}

/// A potential drag that hasn't exceeded the threshold yet.
#[derive(Debug, Clone)]
struct PendingDrag {
    /// Position where the button was pressed.
    start_position: Point,
    /// Keyboard modifiers when pressed.
    modifiers: Modifiers,
}

impl DragDetector {
    /// Create a new drag detector with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new drag detector with the given configuration.
    pub fn with_config(config: DragConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Set the drag threshold in logical pixels.
    pub fn set_threshold(&mut self, threshold: f32) {
        self.config.threshold = threshold;
    }

    /// Returns true if a drag is currently active.
    pub fn is_dragging(&self) -> bool {
        self.active.is_some()
    }

    /// Returns the current drag ID if dragging.
    pub fn drag_id(&self) -> Option<DragId> {
        self.active.as_ref().map(|d| d.id)
    }

    /// Process a core event and return any drag events that should be emitted.
    ///
    /// Returns `Some(DragEvent)` when a drag gesture is detected or updated.
    /// The caller should emit `Event::Drag(...)` for each returned event.
    pub fn process(&mut self, event: &Event) -> Option<DragEvent> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed { button, modifiers }) => {
                self.modifiers = *modifiers;
                if *button == self.config.button && self.active.is_none() {
                    // Start tracking potential drag
                    self.pending = Some(PendingDrag {
                        start_position: self.cursor_position,
                        modifiers: *modifiers,
                    });
                }
                None
            }

            Event::Mouse(mouse::Event::CursorMoved { position, modifiers }) => {
                self.cursor_position = *position;
                self.modifiers = *modifiers;

                if let Some(ref mut active) = self.active {
                    // Already dragging - emit Update
                    active.modifiers = *modifiers;
                    Some(DragEvent::Update {
                        position: *position,
                        modifiers: *modifiers,
                    })
                } else if let Some(ref pending) = self.pending {
                    // Check if we've moved past threshold
                    let dx = position.x - pending.start_position.x;
                    let dy = position.y - pending.start_position.y;
                    let distance = (dx * dx + dy * dy).sqrt();

                    if distance >= self.config.threshold {
                        // Threshold exceeded - start drag
                        let id = next_drag_id();
                        let offer = DragOffer {
                            id,
                            mime_types: Vec::new(), // Will be set by the widget
                            source_actions: Actions::COPY | Actions::MOVE,
                            internal: true,
                        };

                        self.active = Some(ActiveDrag {
                            id,
                            start_position: pending.start_position,
                            began: true,
                            modifiers: *modifiers,
                        });
                        self.pending = None;

                        Some(DragEvent::Begin {
                            position: *position,
                            modifiers: *modifiers,
                            offer,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }

            Event::Mouse(mouse::Event::ButtonReleased { button, modifiers }) => {
                self.modifiers = *modifiers;

                if *button == self.config.button {
                    // Clear pending (was just a click, not a drag)
                    self.pending = None;

                    // If dragging, end the drag
                    if let Some(active) = self.active.take() {
                        // Determine outcome - for internal drags, we'll say Accepted
                        // The actual outcome depends on whether a drop target accepted
                        return Some(DragEvent::End {
                            position: Some(self.cursor_position),
                            modifiers: *modifiers,
                            // Default to cancelled - the drop target will set the real outcome
                            outcome: Outcome::Cancelled,
                        });
                    }
                }
                None
            }

            // Modifiers are updated from mouse events which include them
            _ => None,
        }
    }

    /// Cancel the current drag operation.
    ///
    /// Returns a `DragEvent::End` with `Cancelled` outcome if a drag was active.
    pub fn cancel(&mut self) -> Option<DragEvent> {
        self.pending = None;

        if let Some(_active) = self.active.take() {
            Some(DragEvent::End {
                position: Some(self.cursor_position),
                modifiers: self.modifiers,
                outcome: Outcome::Cancelled,
            })
        } else {
            None
        }
    }

    /// Complete the drag with a successful drop.
    ///
    /// Returns a `DragEvent::End` with the given action if a drag was active.
    pub fn complete(&mut self, action: Actions) -> Option<DragEvent> {
        self.pending = None;

        if let Some(_active) = self.active.take() {
            Some(DragEvent::End {
                position: Some(self.cursor_position),
                modifiers: self.modifiers,
                outcome: Outcome::Accepted(action),
            })
        } else {
            None
        }
    }

    /// Reset all state.
    pub fn reset(&mut self) {
        self.pending = None;
        self.active = None;
    }
}
