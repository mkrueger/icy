//! Accessibility support via AccessKit.
//!
//! This module provides types for building an accessibility tree that can be
//! consumed by screen readers like NVDA (Windows), VoiceOver (macOS), and
//! Orca (Linux).

pub use accesskit::{Action, ActionRequest, Node, NodeId, Role, Tree, TreeUpdate};

use crate::{Point, Rectangle, Size};

/// Information about a widget for accessibility purposes.
#[derive(Debug, Clone)]
pub struct WidgetInfo {
    /// The accessibility role of the widget.
    pub role: Role,
    /// Whether the widget is enabled (interactive).
    pub enabled: bool,
    /// The accessible label/name of the widget.
    pub label: Option<String>,
    /// The current text value (for text inputs).
    pub value: Option<String>,
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
}

impl Default for WidgetInfo {
    fn default() -> Self {
        Self {
            role: Role::Unknown,
            enabled: true,
            label: None,
            value: None,
            numeric_value: None,
            min_value: None,
            max_value: None,
            step: None,
            toggled: None,
            text_selection_start: None,
            text_selection_end: None,
            expanded: None,
            placeholder: None,
            bounds: Rectangle::new(Point::ORIGIN, Size::ZERO),
            focusable: false,
            actions: Vec::new(),
            labelled_by: Vec::new(),
            children: Vec::new(),
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

    /// Creates widget info for a text input.
    pub fn text_input(value: impl Into<String>) -> Self {
        let value = value.into();
        Self {
            role: Role::TextInput,
            value: Some(value),
            focusable: true,
            actions: vec![Action::Focus, Action::SetValue],
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

    /// Sets text selection range (for text inputs).
    pub fn with_selection(mut self, start: usize, end: usize) -> Self {
        self.text_selection_start = Some(start);
        self.text_selection_end = Some(end);
        self
    }

    /// Sets placeholder text.
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
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

    /// Converts this info into an AccessKit `Node`.
    pub fn build(self) -> Node {
        let mut node = Node::new(self.role);

        if !self.enabled {
            node.set_disabled();
        }

        if let Some(label) = self.label {
            node.set_label(label);
        }

        if let Some(value) = self.value {
            node.set_value(value);
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
                    node: NodeId(0), // Will be set properly by the tree builder
                    character_index: start,
                },
                focus: accesskit::TextPosition {
                    node: NodeId(0),
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
        node.set_children(self.children);

        node
    }
}

/// Generates a stable [`NodeId`] from an arbitrary ID value.
pub fn node_id(id: u64) -> NodeId {
    NodeId(id)
}

/// The state of the accessibility tree.
#[derive(Debug, Default)]
pub struct AccessibilityState {
    /// The current focused node ID.
    pub focus: Option<NodeId>,
    /// Whether the tree has been modified and needs to be sent.
    pub dirty: bool,
    /// Whether accessibility is enabled.
    pub enabled: bool,
}

impl AccessibilityState {
    /// Creates a new accessibility state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables accessibility.
    pub fn enable(&mut self) {
        self.enabled = true;
        self.dirty = true;
    }

    /// Disables accessibility.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Marks the tree as dirty (needs update).
    pub fn mark_dirty(&mut self) {
        if self.enabled {
            self.dirty = true;
        }
    }

    /// Clears the dirty flag.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Sets the focused node.
    pub fn set_focus(&mut self, id: Option<NodeId>) {
        if self.focus != id {
            self.focus = id;
            self.mark_dirty();
        }
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
