//! A widget that displays toast notifications.
//!
//! Toast notifications are temporary messages that appear at the bottom of the screen
//! and automatically disappear after a configurable duration.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use icy_ui_widget::*; } pub use icy_ui_widget::core::*; pub type Task<T> = icy_ui_runtime::Task<T>; }
//! # pub type Element<'a, Message> = icy_ui_widget::core::Element<'a, Message, icy_ui_widget::Theme, icy_ui_widget::Renderer>;
//! use icy_ui::widget::toaster::{self, Toast, Toasts};
//! use icy_ui::Task;
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     AddToast,
//!     CloseToast(toaster::Id),
//! }
//!
//! struct State {
//!     toasts: Toasts<Message>,
//! }
//!
//! impl State {
//!     fn new() -> Self {
//!         Self {
//!             toasts: Toasts::new(Message::CloseToast),
//!         }
//!     }
//!
//!     fn update(&mut self, message: Message) -> Task<Message> {
//!         match message {
//!             Message::AddToast => {
//!                 // With the `tokio` feature, push returns a Task that will
//!                 // automatically close the toast after its duration expires.
//!                 self.toasts.push(Toast::new("Hello, world!"))
//!             }
//!             Message::CloseToast(id) => {
//!                 self.toasts.remove(id);
//!                 Task::none()
//!             }
//!         }
//!     }
//!
//!     fn view(&self) -> Element<'_, Message> {
//!         toaster::toaster(&self.toasts, "Your app content here").into()
//!     }
//! }
//! ```

use crate::container;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::tree::{self, Tree};
use crate::core::widget::{Operation, Widget};
use crate::core::{
    Alignment, Border, Clipboard, Element, Event, Length, Padding, Point, Rectangle, Shadow, Shell,
    Size, Vector,
};
use crate::{Column, Row, button, text};

use slotmap::{SlotMap, new_key_type};
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration as StdDuration;

// ============================================================================
// Toast ID (SlotMap-based)
// ============================================================================

new_key_type! {
    /// A unique identifier for a toast notification.
    pub struct Id;
}

// ============================================================================
// Duration
// ============================================================================

/// The duration a toast will be displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Duration {
    /// Short duration (5 seconds).
    #[default]
    Short,
    /// Long duration (15 seconds).
    Long,
    /// Custom duration.
    Custom(StdDuration),
}

impl Duration {
    /// Get the duration as a `std::time::Duration`.
    #[cfg(feature = "tokio")]
    fn as_std(&self) -> StdDuration {
        match self {
            Duration::Short => StdDuration::from_millis(5000),
            Duration::Long => StdDuration::from_millis(15000),
            Duration::Custom(d) => *d,
        }
    }
}

impl From<StdDuration> for Duration {
    fn from(duration: StdDuration) -> Self {
        Duration::Custom(duration)
    }
}

// ============================================================================
// Action
// ============================================================================

/// An action that can be triggered by clicking a button on a toast.
#[derive(Clone)]
pub struct Action<Message> {
    /// The text to display on the action button.
    pub label: String,
    /// A function that produces the message when the action is triggered.
    pub on_press: Rc<dyn Fn(Id) -> Message>,
}

impl<Message> std::fmt::Debug for Action<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Action")
            .field("label", &self.label)
            .finish()
    }
}

// ============================================================================
// Toast Style
// ============================================================================

/// A style function for a toast.
pub type StyleFn = fn(&crate::Theme) -> container::Style;

// ============================================================================
// Toast
// ============================================================================

/// A toast notification.
pub struct Toast<Message> {
    /// The message to display.
    pub message: String,
    /// An optional action button.
    pub action: Option<Action<Message>>,
    /// How long the toast should be displayed.
    pub duration: Duration,
    /// Custom style function for this toast.
    pub style: StyleFn,
}

impl<Message> std::fmt::Debug for Toast<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Toast")
            .field("message", &self.message)
            .field("action", &self.action)
            .field("duration", &self.duration)
            .finish()
    }
}

impl<Message: Clone> Clone for Toast<Message> {
    fn clone(&self) -> Self {
        Self {
            message: self.message.clone(),
            action: self.action.clone(),
            duration: self.duration,
            style: self.style,
        }
    }
}

impl<Message> Toast<Message> {
    /// Create a new toast with the given message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            action: None,
            duration: Duration::default(),
            style: default_style,
        }
    }

    /// Set an action button on the toast.
    #[must_use]
    pub fn action(
        mut self,
        label: impl Into<String>,
        on_press: impl Fn(Id) -> Message + 'static,
    ) -> Self {
        self.action = Some(Action {
            label: label.into(),
            on_press: Rc::new(on_press),
        });
        self
    }

    /// Set the duration for this toast.
    #[must_use]
    pub fn duration(mut self, duration: impl Into<Duration>) -> Self {
        self.duration = duration.into();
        self
    }

    /// Set a custom style for this toast.
    ///
    /// # Example
    /// ```ignore
    /// Toast::new("Error!")
    ///     .style(|theme| container::Style {
    ///         background: Some(theme.palette.danger.into()),
    ///         text_color: Some(iced::Color::WHITE),
    ///         ..Default::default()
    ///     })
    /// ```
    #[must_use]
    pub fn style(mut self, style: StyleFn) -> Self {
        self.style = style;
        self
    }
}

// ============================================================================
// Toasts Collection
// ============================================================================

/// A collection of toast notifications.
#[derive(Debug)]
pub struct Toasts<Message> {
    toasts: SlotMap<Id, Toast<Message>>,
    queue: VecDeque<Id>,
    on_close: fn(Id) -> Message,
    limit: usize,
}

impl<Message> Toasts<Message> {
    /// Create a new toast collection.
    ///
    /// The `on_close` function is called when a toast should be closed
    /// (either by timeout or by clicking the close button).
    pub fn new(on_close: fn(Id) -> Message) -> Self {
        let limit = 5;
        Self {
            toasts: SlotMap::with_capacity_and_key(limit),
            queue: VecDeque::new(),
            on_close,
            limit,
        }
    }

    /// Set the maximum number of toasts that can be displayed at once.
    #[must_use]
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Add a new toast notification.
    ///
    /// With the `tokio` feature enabled, this returns a [`Task`] that will
    /// automatically send the close message after the toast's duration expires.
    ///
    /// Without `tokio`, returns the [`Id`] and you must handle timeouts manually.
    ///
    /// [`Task`]: icy_ui_runtime::Task
    #[cfg(feature = "tokio")]
    pub fn push(&mut self, toast: Toast<Message>) -> icy_ui_runtime::Task<Message>
    where
        Message: Clone + Send + 'static,
    {
        // Remove oldest toasts if we're at the limit
        while self.toasts.len() >= self.limit {
            if let Some(oldest_id) = self.queue.pop_front() {
                let _ = self.toasts.remove(oldest_id);
            }
        }

        let duration = toast.duration.as_std();
        let id = self.toasts.insert(toast);
        self.queue.push_back(id);

        let on_close = self.on_close;
        icy_ui_runtime::Task::future(async move {
            tokio::time::sleep(duration).await;
            on_close(id)
        })
    }

    /// Add a new toast notification.
    ///
    /// Returns the [`Id`] of the toast, which can be used to remove it manually.
    /// You must handle timeouts manually (e.g., using `time::every`).
    #[cfg(not(feature = "tokio"))]
    pub fn push(&mut self, toast: Toast<Message>) -> Id {
        // Remove oldest toasts if we're at the limit
        while self.toasts.len() >= self.limit {
            if let Some(oldest_id) = self.queue.pop_front() {
                let _ = self.toasts.remove(oldest_id);
            }
        }

        let id = self.toasts.insert(toast);
        self.queue.push_back(id);
        id
    }

    /// Remove a toast by its ID.
    pub fn remove(&mut self, id: Id) {
        let _ = self.toasts.remove(id);
        if let Some(pos) = self.queue.iter().position(|key| *key == id) {
            let _ = self.queue.remove(pos);
        }
    }

    /// Check if there are any toasts.
    pub fn is_empty(&self) -> bool {
        self.toasts.is_empty()
    }

    /// Get the number of toasts.
    pub fn len(&self) -> usize {
        self.toasts.len()
    }

    /// Iterate over the toasts in display order (newest first).
    fn iter(&self) -> impl Iterator<Item = (Id, &Toast<Message>)> {
        self.queue
            .iter()
            .rev()
            .filter_map(|id| self.toasts.get(*id).map(|toast| (*id, toast)))
    }
}

// ============================================================================
// Toaster Widget
// ============================================================================

/// Create a toaster widget that wraps your content and displays toast notifications.
pub fn toaster<'a, Message: Clone + 'static>(
    toasts: &'a Toasts<Message>,
    content: impl Into<Element<'a, Message, crate::Theme, crate::Renderer>>,
) -> Toaster<'a, Message> {
    Toaster::new(toasts, content)
}

/// A widget that displays toast notifications as an overlay.
pub struct Toaster<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Theme: container::Catalog + button::Catalog + text::Catalog,
    Renderer: crate::core::Renderer + crate::core::text::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    toasts_element: Element<'a, Message, Theme, Renderer>,
    is_empty: bool,
}

impl<'a, Message: Clone + 'static> Toaster<'a, Message, crate::Theme, crate::Renderer> {
    /// Create a new toaster widget.
    pub fn new(
        toasts: &'a Toasts<Message>,
        content: impl Into<Element<'a, Message, crate::Theme, crate::Renderer>>,
    ) -> Self {
        let toasts_element = build_toasts(toasts);
        Self {
            content: content.into(),
            toasts_element,
            is_empty: toasts.is_empty(),
        }
    }
}

fn build_toasts<'a, Message: Clone + 'static>(
    toasts: &'a Toasts<Message>,
) -> Element<'a, Message, crate::Theme, crate::Renderer> {
    let on_close = toasts.on_close;

    let toast_views: Vec<Element<'a, Message, crate::Theme, crate::Renderer>> = toasts
        .iter()
        .map(|(id, toast)| {
            let message_text = text(&toast.message);

            let action_button: Option<Element<'a, Message, crate::Theme, crate::Renderer>> =
                toast.action.as_ref().map(|action| {
                    button::text_button(action.label.as_str())
                        .on_press((action.on_press)(id))
                        .into()
                });

            let close_button: Element<'a, Message, crate::Theme, crate::Renderer> =
                button::icon_button(text("✕").size(14))
                    .on_press(on_close(id))
                    .into();

            // Build row: message text + optional action + close button
            // Using theme spacing tokens (matching libcosmic)
            let mut actions_row = Row::new().align_y(Alignment::Center);

            if let Some(action_btn) = action_button {
                actions_row = actions_row.push(action_btn);
            }
            actions_row = actions_row.push(close_button);

            let content_row = Row::new()
                .push(message_text)
                .push(actions_row)
                .align_y(Alignment::Center);

            // Style matching libcosmic's toaster: Tooltip-like container
            // Padding: [xxs, s, xxs, m] = [4, 12, 4, 16] using COSMIC defaults
            let style_fn = toast.style;
            container::Container::new(content_row)
                .padding(Padding {
                    top: 4.0,
                    right: 12.0,
                    bottom: 4.0,
                    left: 16.0,
                })
                .style(style_fn)
                .into()
        })
        .collect();

    // Toast spacing: xxxs (2px in COSMIC default)
    Column::with_children(toast_views)
        .spacing(2)
        .width(Length::Shrink)
        .into()
}

/// Default toast container style (similar to libcosmic's Tooltip style).
pub fn default_style(theme: &crate::Theme) -> container::Style {
    let spacing = &theme.spacing;
    let corner_radii = &theme.corner_radii;

    container::Style {
        background: Some(theme.background.base.into()),
        text_color: Some(theme.background.on),
        border: Border {
            width: 1.0,
            radius: corner_radii.radius_m.into(),
            color: theme.background.divider,
        },
        shadow: Shadow {
            color: theme
                .background
                .on
                .scale_alpha(if theme.is_dark { 0.25 } else { 0.15 }),
            offset: Vector::new(0.0, 4.0),
            blur_radius: spacing.xs as f32,
        },
        ..container::Style::default()
    }
}

/// Success toast style (green tinted).
pub fn success_style(theme: &crate::Theme) -> container::Style {
    let corner_radii = &theme.corner_radii;
    let spacing = &theme.spacing;

    container::Style {
        background: Some(theme.palette.bright_green.scale_alpha(0.15).into()),
        text_color: Some(theme.background.on),
        border: Border {
            width: 1.0,
            radius: corner_radii.radius_m.into(),
            color: theme.palette.bright_green.scale_alpha(0.5),
        },
        shadow: Shadow {
            color: theme.palette.bright_green.scale_alpha(0.2),
            offset: Vector::new(0.0, 4.0),
            blur_radius: spacing.xs as f32,
        },
        ..container::Style::default()
    }
}

/// Warning toast style (orange tinted).
pub fn warning_style(theme: &crate::Theme) -> container::Style {
    let corner_radii = &theme.corner_radii;
    let spacing = &theme.spacing;

    container::Style {
        background: Some(theme.palette.bright_orange.scale_alpha(0.15).into()),
        text_color: Some(theme.background.on),
        border: Border {
            width: 1.0,
            radius: corner_radii.radius_m.into(),
            color: theme.palette.bright_orange.scale_alpha(0.5),
        },
        shadow: Shadow {
            color: theme.palette.bright_orange.scale_alpha(0.2),
            offset: Vector::new(0.0, 4.0),
            blur_radius: spacing.xs as f32,
        },
        ..container::Style::default()
    }
}

/// Danger/error toast style (red tinted).
pub fn danger_style(theme: &crate::Theme) -> container::Style {
    let corner_radii = &theme.corner_radii;
    let spacing = &theme.spacing;

    container::Style {
        background: Some(theme.palette.bright_red.scale_alpha(0.15).into()),
        text_color: Some(theme.background.on),
        border: Border {
            width: 1.0,
            radius: corner_radii.radius_m.into(),
            color: theme.palette.bright_red.scale_alpha(0.5),
        },
        shadow: Shadow {
            color: theme.palette.bright_red.scale_alpha(0.2),
            offset: Vector::new(0.0, 4.0),
            blur_radius: spacing.xs as f32,
        },
        ..container::Style::default()
    }
}

/// Info toast style (blue tinted).
pub fn info_style(theme: &crate::Theme) -> container::Style {
    let corner_radii = &theme.corner_radii;
    let spacing = &theme.spacing;

    container::Style {
        background: Some(theme.palette.accent_blue.scale_alpha(0.15).into()),
        text_color: Some(theme.background.on),
        border: Border {
            width: 1.0,
            radius: corner_radii.radius_m.into(),
            color: theme.palette.accent_blue.scale_alpha(0.5),
        },
        shadow: Shadow {
            color: theme.palette.accent_blue.scale_alpha(0.2),
            offset: Vector::new(0.0, 4.0),
            blur_radius: spacing.xs as f32,
        },
        ..container::Style::default()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Toaster<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog + button::Catalog + text::Catalog,
    Renderer: crate::core::Renderer + crate::core::text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<()>()
    }

    fn state(&self) -> tree::State {
        tree::State::None
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content), Tree::new(&self.toasts_element)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.content, &self.toasts_element]);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &mut self,
        state: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut state.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        state: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut state.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &state.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        if self.is_empty {
            // Pass through to content's overlay
            self.content.as_widget_mut().overlay(
                &mut state.children[0],
                layout,
                renderer,
                viewport,
                translation,
            )
        } else {
            // Show our toast overlay
            Some(overlay::Element::new(Box::new(ToasterOverlay {
                state: &mut state.children[1],
                element: &mut self.toasts_element,
            })))
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Toaster<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: container::Catalog + button::Catalog + text::Catalog + 'a,
    Renderer: crate::core::Renderer + crate::core::text::Renderer + 'a,
{
    fn from(toaster: Toaster<'a, Message, Theme, Renderer>) -> Self {
        Element::new(toaster)
    }
}

// ============================================================================
// Toaster Overlay
// ============================================================================

struct ToasterOverlay<'a, 'b, Message, Theme, Renderer> {
    state: &'b mut Tree,
    element: &'b mut Element<'a, Message, Theme, Renderer>,
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for ToasterOverlay<'_, '_, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds);

        let node = self
            .element
            .as_widget_mut()
            .layout(self.state, renderer, &limits);

        // Position at the bottom center with offset (15px like libcosmic)
        let offset = 15.0;
        let position = Point::new(
            (bounds.width / 2.0) - (node.size().width / 2.0),
            bounds.height - node.size().height - offset,
        );

        node.move_to(position)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();
        self.element
            .as_widget()
            .draw(self.state, renderer, theme, style, layout, cursor, &bounds);
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        self.element.as_widget_mut().update(
            self.state,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        );
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.element.as_widget().mouse_interaction(
            self.state,
            layout,
            cursor,
            &layout.bounds(),
            renderer,
        )
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'c>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
        self.element.as_widget_mut().overlay(
            self.state,
            layout,
            renderer,
            &layout.bounds(),
            Vector::ZERO,
        )
    }
}
