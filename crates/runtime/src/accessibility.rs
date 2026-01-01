//! Accessibility runtime support.
//!
//! This module provides tasks for accessibility features like announcements
//! that can be consumed by screen readers.
//!
//! # Example
//!
//! ```no_run
//! use icy_ui::accessibility::{self, Priority};
//!
//! // Announce a message to the screen reader
//! let task = accessibility::announce("File saved successfully", Priority::Polite);
//!
//! // Interrupt with an important message
//! let task = accessibility::announce("Error: Connection lost", Priority::Assertive);
//! ```

use crate::task::{self, Task};

// ============================================================================
// Public API
// ============================================================================

/// The priority/urgency of an accessibility announcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Priority {
    /// The announcement will wait until the screen reader is idle.
    /// Use this for non-urgent status updates.
    #[default]
    Polite,
    /// The announcement will interrupt whatever the screen reader is saying.
    /// Use this for urgent messages like errors.
    Assertive,
}

/// Announce a message to screen readers.
///
/// This creates an invisible "live region" that screen readers will detect
/// and read aloud. The priority determines whether the announcement waits
/// for the screen reader to finish (Polite) or interrupts immediately (Assertive).
///
/// # Arguments
///
/// * `message` - The text to be announced
/// * `priority` - How urgently the message should be announced
///
/// # Returns
///
/// A task that completes when the announcement has been queued.
///
/// # Example
///
/// ```no_run
/// use icy_ui::accessibility::{self, Priority};
///
/// fn update(&mut self, message: Message) -> Task<Message> {
///     match message {
///         Message::FileSaved => {
///             // Give feedback to screen reader users
///             accessibility::announce("File saved", Priority::Polite)
///                 .discard()
///         }
///         Message::Error(e) => {
///             // Urgent - interrupt immediately
///             accessibility::announce(
///                 format!("Error: {e}"),
///                 Priority::Assertive
///             ).discard()
///         }
///     }
/// }
/// ```
pub fn announce(message: impl Into<String>, priority: Priority) -> Task<()> {
    task::effect(crate::Action::Accessibility(Action::Announce {
        message: message.into(),
        priority,
    }))
}

/// Request focus on a specific accessible element.
///
/// This tells the screen reader to move focus to the element with the given ID.
pub fn focus(target: icy_ui_core::accessibility::NodeId) -> Task<()> {
    task::effect(crate::Action::Accessibility(Action::Focus { target }))
}

// ============================================================================
// Action types (internal)
// ============================================================================

/// An accessibility action that can be performed by the runtime.
#[derive(Debug, Clone)]
pub enum Action {
    /// Announce a message to screen readers.
    Announce {
        /// The message to announce.
        message: String,
        /// The priority of the announcement.
        priority: Priority,
    },
    /// Request focus on an accessible element.
    Focus {
        /// The target node ID to focus.
        target: icy_ui_core::accessibility::NodeId,
    },
}
