//! Accessibility support via AccessKit.
//!
//! This module provides types for building an accessibility tree that can be
//! consumed by screen readers like NVDA (Windows), VoiceOver (macOS), and
//! Orca (Linux).
//!
//! # Accessibility Mode
//!
//! When a screen reader connects (e.g., VoiceOver, NVDA), the application enters
//! **accessibility mode**. This mode changes several behaviors:
//!
//! - **Focus handling**: The accessibility focus (VoiceOver cursor) is separate
//!   from the widget focus. The a11y focus can point to non-focusable widgets
//!   like static text.
//! - **Tab navigation**: In accessibility mode, Tab/Shift+Tab traverses ALL
//!   interactive controls, not just text inputs.
//! - **Announcements**: Focus changes are announced to the screen reader.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Application                               │
//! │  ┌─────────────┐     ┌─────────────┐     ┌──────────────┐  │
//! │  │  Widget     │────▶│ WidgetInfo  │────▶│  AccessKit   │  │
//! │  │  Tree       │     │  (per node) │     │  TreeUpdate  │  │
//! │  └─────────────┘     └─────────────┘     └──────────────┘  │
//! │                                                 │           │
//! │                                                 ▼           │
//! │                                         ┌──────────────┐   │
//! │                                         │ Screen Reader│   │
//! │                                         └──────────────┘   │
//! │                                                 │           │
//! │                                                 ▼           │
//! │  ┌─────────────┐     ┌─────────────┐     ┌──────────────┐  │
//! │  │  Event      │◀────│ A11y Event  │◀────│ ActionRequest│  │
//! │  │  Handler    │     │             │     │              │  │
//! │  └─────────────┘     └─────────────┘     └──────────────┘  │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub use accesskit::{Action, ActionRequest, Node, NodeId, Role, Tree, TreeUpdate};

use crate::widget;
use crate::{Point, Rectangle, Size};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};

// ============================================================================
// Global Accessibility State
// ============================================================================

/// Global flag indicating whether accessibility mode is active.
///
/// This is set when a screen reader connects (e.g., VoiceOver, NVDA) and
/// can be queried by widgets to adjust their rendering (e.g., skip drawing
/// focus rings since the screen reader provides its own highlighting).
static ACCESSIBILITY_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Returns `true` if accessibility mode is globally active.
///
/// Widgets can use this to skip rendering focus indicators, since screen
/// readers like VoiceOver provide their own focus highlighting.
pub fn is_accessibility_active() -> bool {
    ACCESSIBILITY_ACTIVE.load(Ordering::Relaxed)
}

/// Sets the global accessibility active state.
///
/// This should be called by the shell when a screen reader connects or
/// disconnects.
pub fn set_accessibility_active(active: bool) {
    ACCESSIBILITY_ACTIVE.store(active, Ordering::Relaxed);
}

// ============================================================================
// Accessibility Mode
// ============================================================================

/// The current accessibility mode of the application.
///
/// When a screen reader connects, the application enters `Active` mode which
/// changes focus handling and navigation behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// No screen reader is connected. Normal focus and navigation.
    #[default]
    Inactive,
    /// A screen reader is connected. Accessibility-specific behavior is active.
    Active,
}

impl Mode {
    /// Returns `true` if accessibility is active (screen reader connected).
    pub fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }
}

// ============================================================================
// Accessibility Focus
// ============================================================================

/// Represents the accessibility focus, which is separate from widget focus.
///
/// The accessibility focus (also called "VoiceOver cursor" on macOS or
/// "virtual cursor" on other platforms) can point to any element in the
/// accessibility tree, including non-interactive elements like static text.
///
/// This is different from widget focus, which only applies to focusable
/// interactive widgets.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Focus {
    /// The currently focused node ID.
    pub node: Option<NodeId>,
    /// Whether the focus was set programmatically (vs. by user navigation).
    pub programmatic: bool,
}

impl Focus {
    /// Creates a new accessibility focus.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the focus to a specific node.
    pub fn set(&mut self, id: NodeId) {
        self.node = Some(id);
        self.programmatic = false;
    }

    /// Sets the focus programmatically (won't trigger announcement).
    pub fn set_programmatic(&mut self, id: NodeId) {
        self.node = Some(id);
        self.programmatic = true;
    }

    /// Clears the focus.
    pub fn clear(&mut self) {
        self.node = None;
        self.programmatic = false;
    }
}

// ============================================================================
// Widget Info
// ============================================================================

/// Information about a widget for accessibility purposes.
#[derive(Debug, Clone)]
pub struct WidgetInfo {
    /// The accessibility role of the widget.
    pub role: Role,
    /// Whether the widget is enabled (interactive).
    pub enabled: bool,
    /// The accessible label/name of the widget.
    pub label: Option<String>,
    /// The accessible description/help text.
    ///
    /// This is announced by VoiceOver when pressing VO+Shift+H (hear hint).
    /// Use this for additional context like "Enter your email address" or
    /// "Password must be at least 8 characters".
    pub description: Option<String>,
    /// Whether this field is required.
    ///
    /// VoiceOver announces "required" for required fields.
    pub required: bool,
    /// The current text value (for text inputs).
    pub value: Option<String>,

    /// For text runs, the length (non-inclusive) of each character in UTF-8 code units (bytes).
    ///
    /// See `accesskit::Node::set_character_lengths`.
    pub character_lengths: Option<Vec<u8>>,

    /// For text runs, the length of each word in characters, as defined by `character_lengths`.
    ///
    /// See `accesskit::Node::set_word_lengths`.
    pub word_lengths: Option<Vec<u8>>,
    /// The numeric value (for sliders, spinboxes).
    pub numeric_value: Option<f64>,
    /// The minimum numeric value.
    pub min_value: Option<f64>,
    /// The maximum numeric value.
    pub max_value: Option<f64>,
    /// The numeric value step.
    pub step: Option<f64>,
    /// Whether the widget is toggled/checked (for checkboxes, toggles).
    pub toggled: Option<bool>,
    /// Text selection start (for text inputs).
    pub text_selection_start: Option<usize>,
    /// Text selection end (for text inputs).
    pub text_selection_end: Option<usize>,

    /// Where `text_selection` positions should point.
    pub text_selection_target: TextSelectionTarget,
    /// Whether the widget is expanded (for combo boxes, menus).
    pub expanded: Option<bool>,
    /// Placeholder/hint text.
    pub placeholder: Option<String>,
    /// The widget bounds in window coordinates.
    pub bounds: Rectangle,
    /// Whether the widget is focusable.
    pub focusable: bool,
    /// Available actions on this widget.
    pub actions: Vec<Action>,
    /// IDs of widgets that label this widget.
    pub labelled_by: Vec<NodeId>,
    /// IDs of child widgets.
    pub children: Vec<NodeId>,

    /// Extra accessibility children generated by this widget (e.g. a `Role::TextRun`).
    ///
    /// These nodes do not correspond to separate widgets in the widget tree.
    pub extra_children: Vec<WidgetInfo>,
}

/// Where `TextSelection.anchor.node` / `TextSelection.focus.node` should point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextSelectionTarget {
    /// Point to the node represented by this `WidgetInfo`.
    ThisNode,
    /// Point to an extra child node by index in `extra_children`.
    ExtraChild(usize),
}

impl Default for WidgetInfo {
    fn default() -> Self {
        Self {
            role: Role::Unknown,
            enabled: true,
            label: None,
            description: None,
            required: false,
            value: None,
            character_lengths: None,
            word_lengths: None,
            numeric_value: None,
            min_value: None,
            max_value: None,
            step: None,
            toggled: None,
            text_selection_start: None,
            text_selection_end: None,
            text_selection_target: TextSelectionTarget::ThisNode,
            expanded: None,
            placeholder: None,
            bounds: Rectangle::new(Point::ORIGIN, Size::ZERO),
            focusable: false,
            actions: Vec::new(),
            labelled_by: Vec::new(),
            children: Vec::new(),
            extra_children: Vec::new(),
        }
    }
}

impl WidgetInfo {
    /// Creates a new `WidgetInfo` with the given role.
    pub fn new(role: Role) -> Self {
        Self {
            role,
            ..Default::default()
        }
    }

    /// Creates widget info for a button.
    pub fn button(label: impl Into<String>) -> Self {
        Self {
            role: Role::Button,
            label: Some(label.into()),
            focusable: true,
            actions: vec![Action::Click, Action::Focus],
            ..Default::default()
        }
    }

    /// Creates widget info for a checkbox.
    pub fn checkbox(label: impl Into<String>, checked: bool) -> Self {
        Self {
            role: Role::CheckBox,
            label: Some(label.into()),
            toggled: Some(checked),
            focusable: true,
            actions: vec![Action::Click, Action::Focus],
            ..Default::default()
        }
    }

    /// Creates widget info for a toggle/switch control.
    ///
    /// Uses `Role::Switch` which VoiceOver announces as "activated/deactivated"
    /// rather than "checked/unchecked" like a checkbox.
    pub fn toggle(label: impl Into<String>, is_toggled: bool) -> Self {
        Self {
            role: Role::Switch,
            label: Some(label.into()),
            toggled: Some(is_toggled),
            focusable: true,
            actions: vec![Action::Click, Action::Focus],
            ..Default::default()
        }
    }

    /// Creates widget info for a text input.
    ///
    /// For secure/password inputs, use [`text_input_secure`](Self::text_input_secure)
    /// which masks the value for screen readers.
    pub fn text_input(value: impl Into<String>) -> Self {
        let value = value.into();
        Self {
            role: Role::TextInput,
            value: Some(value),
            focusable: true,
            actions: vec![
                Action::Click,
                Action::Focus,
                Action::SetValue,
                Action::SetTextSelection,
            ],
            ..Default::default()
        }
    }

    /// Creates widget info for a secure/password text input.
    ///
    /// The value is masked (shown as dots) to prevent screen readers from
    /// announcing the actual password characters. Uses `Role::PasswordInput`
    /// which VoiceOver announces as "secure text field".
    pub fn text_input_secure(value_length: usize) -> Self {
        // Create masked value with dots for each character
        let masked_value: String = std::iter::repeat('•').take(value_length).collect();
        Self {
            role: Role::PasswordInput,
            value: Some(masked_value),
            focusable: true,
            actions: vec![
                Action::Click,
                Action::Focus,
                Action::SetValue,
                Action::SetTextSelection,
            ],
            ..Default::default()
        }
    }

    /// Creates widget info for a slider.
    pub fn slider(value: f64, min: f64, max: f64) -> Self {
        Self {
            role: Role::Slider,
            numeric_value: Some(value),
            min_value: Some(min),
            max_value: Some(max),
            focusable: true,
            actions: vec![
                Action::Focus,
                Action::SetValue,
                Action::Increment,
                Action::Decrement,
            ],
            ..Default::default()
        }
    }

    /// Creates widget info for a static text label.
    pub fn label(text: impl Into<String>) -> Self {
        Self {
            role: Role::Label,
            label: Some(text.into()),
            ..Default::default()
        }
    }

    /// Creates widget info for a radio button.
    ///
    /// Radio buttons are used to select a single option from a group.
    /// VoiceOver announces them as "radio button, 1 of 3" etc.
    pub fn radio_button(label: impl Into<String>, is_selected: bool) -> Self {
        Self {
            role: Role::RadioButton,
            label: Some(label.into()),
            toggled: Some(is_selected),
            focusable: true,
            actions: vec![Action::Click, Action::Focus],
            ..Default::default()
        }
    }

    /// Creates widget info for a radio button group container.
    ///
    /// This groups multiple radio buttons together for accessibility.
    /// The group itself is focusable to allow VoiceOver cursor navigation.
    pub fn radio_group(label: Option<impl Into<String>>) -> Self {
        Self {
            role: Role::RadioGroup,
            label: label.map(|l| l.into()),
            focusable: true,
            actions: vec![Action::Focus],
            ..Default::default()
        }
    }

    /// Creates widget info for a pick list (combo box / dropdown).
    ///
    /// VoiceOver announces this as "pop-up button" with the current value.
    pub fn pick_list(value: impl Into<String>) -> Self {
        Self {
            role: Role::ComboBox,
            value: Some(value.into()),
            focusable: true,
            actions: vec![Action::Click, Action::Focus],
            ..Default::default()
        }
    }

    /// Creates widget info for a menu (popup/dropdown menu).
    ///
    /// VoiceOver announces this as "menu" and navigates menu items.
    pub fn menu() -> Self {
        Self {
            role: Role::Menu,
            focusable: true,
            actions: vec![Action::Focus],
            ..Default::default()
        }
    }

    /// Creates widget info for a menu item.
    ///
    /// VoiceOver announces this as "menu item" with the label.
    pub fn menu_item(label: impl Into<String>) -> Self {
        Self {
            role: Role::MenuItem,
            label: Some(label.into()),
            focusable: true,
            actions: vec![Action::Click, Action::Focus],
            ..Default::default()
        }
    }

    /// Creates widget info for a list box (scrollable list of options).
    ///
    /// VoiceOver announces this as "list" with item count.
    pub fn list_box() -> Self {
        Self {
            role: Role::ListBox,
            focusable: true,
            actions: vec![Action::Focus],
            ..Default::default()
        }
    }

    /// Creates widget info for a list box option.
    ///
    /// VoiceOver announces this as "option" with the label.
    pub fn list_box_option(label: impl Into<String>, is_selected: bool) -> Self {
        Self {
            role: Role::ListBoxOption,
            label: Some(label.into()),
            toggled: Some(is_selected),
            focusable: true,
            actions: vec![Action::Click, Action::Focus],
            ..Default::default()
        }
    }

    /// Creates widget info for a generic container.
    pub fn container() -> Self {
        Self {
            role: Role::GenericContainer,
            ..Default::default()
        }
    }

    /// Sets the bounds of the widget.
    pub fn with_bounds(mut self, bounds: Rectangle) -> Self {
        self.bounds = bounds;
        self
    }

    /// Sets the enabled state.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets the label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the value.
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Sets text selection range (for text inputs).
    pub fn with_selection(mut self, start: usize, end: usize) -> Self {
        self.text_selection_start = Some(start);
        self.text_selection_end = Some(end);
        self
    }

    /// Sets where text selection positions should point.
    pub fn with_text_selection_target(mut self, target: TextSelectionTarget) -> Self {
        self.text_selection_target = target;
        self
    }

    /// Sets text run character lengths (in UTF-8 bytes) for `Role::TextRun`.
    pub fn with_character_lengths(mut self, lengths: impl Into<Vec<u8>>) -> Self {
        self.character_lengths = Some(lengths.into());
        self
    }

    /// Sets text run word lengths (in characters) for `Role::TextRun`.
    pub fn with_word_lengths(mut self, lengths: impl Into<Vec<u8>>) -> Self {
        self.word_lengths = Some(lengths.into());
        self
    }

    /// Sets placeholder text.
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Sets the description/help text.
    ///
    /// This is announced by VoiceOver when pressing VO+Shift+H (hear hint).
    /// Use for additional context like "Enter your email address".
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Marks the widget as required.
    ///
    /// VoiceOver announces "required" for required fields.
    pub fn with_required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Sets whether the widget is required.
    pub fn set_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Sets the step for numeric values.
    pub fn with_step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    /// Adds a child node ID.
    pub fn with_child(mut self, child: NodeId) -> Self {
        self.children.push(child);
        self
    }

    /// Adds child node IDs.
    pub fn with_children(mut self, children: impl IntoIterator<Item = NodeId>) -> Self {
        self.children.extend(children);
        self
    }

    /// Adds an extra generated child accessibility node (e.g. a `Role::TextRun`).
    pub fn with_extra_child(mut self, child: WidgetInfo) -> Self {
        self.extra_children.push(child);
        self
    }

    /// Builds an AccessKit `Node`, using a resolved NodeId for any `TextSelection` positions.
    pub fn build_with_context(self, selection_node: NodeId, children: Vec<NodeId>) -> Node {
        let mut node = Node::new(self.role);

        if !self.enabled {
            node.set_disabled();
        }

        if let Some(label) = self.label {
            node.set_label(label);
        }

        // Description is announced by VoiceOver as "hint" (VO+Shift+H)
        if let Some(description) = self.description {
            node.set_description(description);
        }

        // Required fields are announced as "required" by VoiceOver
        if self.required {
            node.set_required();
        }

        if let Some(value) = self.value {
            node.set_value(value);
        }

        if let Some(character_lengths) = self.character_lengths {
            node.set_character_lengths(character_lengths);
        }

        if let Some(word_lengths) = self.word_lengths {
            node.set_word_lengths(word_lengths);
        }

        if let Some(numeric_value) = self.numeric_value {
            node.set_numeric_value(numeric_value);
        }

        if let Some(min) = self.min_value {
            node.set_min_numeric_value(min);
        }

        if let Some(max) = self.max_value {
            node.set_max_numeric_value(max);
        }

        if let Some(step) = self.step {
            node.set_numeric_value_step(step);
        }

        if let Some(toggled) = self.toggled {
            node.set_toggled(if toggled {
                accesskit::Toggled::True
            } else {
                accesskit::Toggled::False
            });
        }

        if let (Some(start), Some(end)) = (self.text_selection_start, self.text_selection_end) {
            node.set_text_selection(Box::new(accesskit::TextSelection {
                anchor: accesskit::TextPosition {
                    node: selection_node,
                    character_index: start,
                },
                focus: accesskit::TextPosition {
                    node: selection_node,
                    character_index: end,
                },
            }));
        }

        if let Some(expanded) = self.expanded {
            node.set_expanded(expanded);
        }

        if let Some(placeholder) = self.placeholder {
            node.set_placeholder(placeholder);
        }

        // Set bounds
        node.set_bounds(accesskit::Rect {
            x0: self.bounds.x as f64,
            y0: self.bounds.y as f64,
            x1: (self.bounds.x + self.bounds.width) as f64,
            y1: (self.bounds.y + self.bounds.height) as f64,
        });

        // Add actions
        for action in self.actions {
            node.add_action(action);
        }

        // Add labelled_by references
        for id in self.labelled_by {
            node.push_labelled_by(id);
        }

        // Set children
        node.set_children(children);

        node
    }

    /// Converts this info into an AccessKit `Node`.
    ///
    /// This is a convenience wrapper for widgets that don't need extra children
    /// or text selection node remapping.
    pub fn build(self) -> Node {
        let selection_node = NodeId(0);
        let children = self.children.clone();
        self.build_with_context(selection_node, children)
    }
}

/// Derives a stable child `NodeId` from a parent node id.
pub fn derived_node_id(parent: NodeId, child_index: u64) -> NodeId {
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();
    parent.0.hash(&mut hasher);
    child_index.hash(&mut hasher);

    let mut value = hasher.finish();
    if value <= 1 {
        value = value.wrapping_add(2);
    }
    NodeId(value)
}

/// Generates a stable [`NodeId`] from an arbitrary ID value.
pub fn node_id(id: u64) -> NodeId {
    NodeId(id)
}

/// Generates a stable [`NodeId`] from a widget [`widget::Id`].
///
/// This is useful to associate accessibility action targets back to widgets
/// that already have stable identifiers.
pub fn node_id_from_widget_id(id: &widget::Id) -> NodeId {
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);

    // Avoid clashing with reserved/special IDs commonly used by shells.
    let mut value = hasher.finish();
    if value <= 1 {
        value = value.wrapping_add(2);
    }

    NodeId(value)
}

// ============================================================================
// Accessibility State
// ============================================================================

/// The state of the accessibility system for a window.
///
/// This tracks whether accessibility is active, the current focus,
/// and pending updates to the accessibility tree.
#[derive(Debug, Default)]
pub struct AccessibilityState {
    /// The current accessibility mode.
    pub mode: Mode,
    /// The accessibility focus (separate from widget focus).
    pub a11y_focus: Focus,
    /// The widget focus node (synced from widget tree).
    pub widget_focus: Option<NodeId>,
    /// Whether the tree has been modified and needs to be sent.
    pub dirty: bool,
    /// Serial number for announcements (to ensure unique values).
    pub announcement_serial: u64,
    /// Pending announcement to be read by screen reader.
    pub pending_announcement: Option<(String, AnnouncementPriority)>,
}

/// Priority for screen reader announcements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnnouncementPriority {
    /// Wait for screen reader to finish current speech.
    #[default]
    Polite,
    /// Interrupt current speech immediately.
    Assertive,
}

impl AccessibilityState {
    /// Creates a new accessibility state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether accessibility is currently active.
    pub fn is_active(&self) -> bool {
        self.mode.is_active()
    }

    /// Activates accessibility mode (screen reader connected).
    pub fn activate(&mut self) {
        if self.mode != Mode::Active {
            self.mode = Mode::Active;
            self.dirty = true;
        }
    }

    /// Deactivates accessibility mode (screen reader disconnected).
    pub fn deactivate(&mut self) {
        self.mode = Mode::Inactive;
        self.a11y_focus.clear();
    }

    /// Marks the tree as dirty (needs update).
    pub fn mark_dirty(&mut self) {
        if self.is_active() {
            self.dirty = true;
        }
    }

    /// Clears the dirty flag.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Sets the accessibility focus.
    ///
    /// This is the focus controlled by the screen reader (VoiceOver cursor).
    /// It is separate from the widget focus.
    pub fn set_a11y_focus(&mut self, id: NodeId) {
        if self.a11y_focus.node != Some(id) {
            self.a11y_focus.set(id);
            self.mark_dirty();
        }
    }

    /// Sets the widget focus (synced from the widget tree).
    ///
    /// When in accessibility mode, changes to widget focus will update
    /// the accessibility focus as well.
    pub fn set_widget_focus(&mut self, id: Option<NodeId>) {
        if self.widget_focus != id {
            self.widget_focus = id;
            // Sync a11y focus with widget focus
            if let Some(id) = id {
                self.a11y_focus.set_programmatic(id);
            }
            self.mark_dirty();
        }
    }

    /// Returns the current focus for the accessibility tree.
    ///
    /// This returns the a11y focus if set, otherwise falls back to widget focus.
    pub fn effective_focus(&self) -> Option<NodeId> {
        self.a11y_focus.node.or(self.widget_focus)
    }

    /// Queues an announcement for the screen reader.
    pub fn announce(&mut self, message: impl Into<String>, priority: AnnouncementPriority) {
        self.pending_announcement = Some((message.into(), priority));
        self.mark_dirty();
    }

    /// Takes the pending announcement (clearing it).
    pub fn take_announcement(&mut self) -> Option<(String, AnnouncementPriority)> {
        self.pending_announcement.take()
    }

    /// Increments the announcement serial and returns the new value.
    pub fn next_announcement_serial(&mut self) -> u64 {
        self.announcement_serial = self.announcement_serial.wrapping_add(1);
        self.announcement_serial
    }

    // Legacy compatibility
    /// Returns the focused node (legacy API).
    #[deprecated(note = "Use effective_focus() instead")]
    pub fn focus(&self) -> Option<NodeId> {
        self.effective_focus()
    }

    /// Returns whether accessibility is enabled (legacy API).
    #[deprecated(note = "Use is_active() instead")]
    pub fn enabled(&self) -> bool {
        self.is_active()
    }
}

/// An accessibility event from a screen reader or assistive technology.
#[derive(Debug, Clone)]
pub struct Event {
    /// The action requested by the screen reader.
    pub action: Action,
    /// The target widget ID.
    pub target: NodeId,
    /// Additional data for the action.
    pub data: Option<ActionData>,
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        // Only compare action and target, not data (ActionData doesn't impl PartialEq)
        self.action == other.action && self.target == other.target
    }
}

impl Event {
    /// Creates a new accessibility event from an AccessKit action request.
    pub fn from_request(request: ActionRequest) -> Self {
        Self {
            action: request.action,
            target: request.target,
            data: request.data,
        }
    }

    /// Returns true if this is a click action.
    pub fn is_click(&self) -> bool {
        self.action == Action::Click
    }

    /// Returns true if this is a focus action.
    pub fn is_focus(&self) -> bool {
        self.action == Action::Focus
    }

    /// Returns true if this is a blur (unfocus) action.
    pub fn is_blur(&self) -> bool {
        self.action == Action::Blur
    }

    /// Returns true if this is an increment action (for sliders).
    pub fn is_increment(&self) -> bool {
        self.action == Action::Increment
    }

    /// Returns true if this is a decrement action (for sliders).
    pub fn is_decrement(&self) -> bool {
        self.action == Action::Decrement
    }

    /// Returns the value if this is a SetValue action.
    pub fn set_value_data(&self) -> Option<&str> {
        match &self.data {
            Some(ActionData::Value(value)) => Some(value),
            _ => None,
        }
    }

    /// Returns the numeric value if this is a SetValue action with numeric data.
    pub fn set_numeric_value_data(&self) -> Option<f64> {
        match &self.data {
            Some(ActionData::NumericValue(value)) => Some(*value),
            _ => None,
        }
    }

    /// Returns the text selection if this is a SetTextSelection action.
    pub fn text_selection_data(&self) -> Option<&accesskit::TextSelection> {
        match &self.data {
            Some(ActionData::SetTextSelection(selection)) => Some(selection),
            _ => None,
        }
    }
}

/// Re-export ActionData for convenience.
pub use accesskit::ActionData;
