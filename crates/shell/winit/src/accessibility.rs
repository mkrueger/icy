//! Accessibility integration for winit using AccessKit.

use accesskit::{Node, NodeId, Role, Tree, TreeUpdate};
use accesskit_winit::Adapter;
use winit::event::WindowEvent as WinitWindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use crate::core::Rectangle;

// Re-export for convenience
pub use crate::core::accessibility::Event as AccessibilityEvent;

/// Accessibility adapter wrapper for a window.
pub struct AccessibilityAdapter {
    adapter: Adapter,
    enabled: bool,
}

impl AccessibilityAdapter {
    /// The root node ID for the window.
    pub const ROOT_ID: NodeId = NodeId(0);

    /// Creates a new accessibility adapter for a window.
    ///
    /// This must be called before the window is shown.
    pub fn new<T: From<accesskit_winit::Event> + Send + 'static>(
        event_loop: &ActiveEventLoop,
        window: &Window,
        proxy: winit::event_loop::EventLoopProxy<T>,
    ) -> Self {
        let adapter = Adapter::with_event_loop_proxy(event_loop, window, proxy);
        Self {
            adapter,
            enabled: false,
        }
    }

    /// Process a winit window event.
    ///
    /// This must be called for every window event before it is handled.
    pub fn process_event(&mut self, window: &Window, event: &WinitWindowEvent) {
        self.adapter.process_event(window, event);
    }

    /// Called when accessibility is activated (screen reader connected).
    pub fn activate(&mut self) {
        self.enabled = true;
    }

    /// Called when accessibility is deactivated.
    pub fn deactivate(&mut self) {
        self.enabled = false;
    }

    /// Returns whether accessibility is currently enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Updates the accessibility tree if accessibility is active.
    pub fn update(&mut self, tree_update: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(tree_update);
    }

    /// Sends a full tree update with the given nodes.
    pub fn update_tree(&mut self, nodes: Vec<(NodeId, Node)>, focus: Option<NodeId>) {
        if !self.enabled {
            return;
        }

        self.adapter.update_if_active(|| TreeUpdate {
            nodes,
            tree: Some(Tree::new(Self::ROOT_ID)),
            focus: focus.unwrap_or(Self::ROOT_ID),
        });
    }
}

/// Creates a basic root node for the window.
pub fn create_window_node(title: &str, bounds: Rectangle) -> Node {
    let mut node = Node::new(Role::Window);
    node.set_label(title.to_string());
    node.set_bounds(accesskit::Rect {
        x0: bounds.x as f64,
        y0: bounds.y as f64,
        x1: (bounds.x + bounds.width) as f64,
        y1: (bounds.y + bounds.height) as f64,
    });
    node
}

/// The result of processing an AccessKit window event.
#[derive(Debug, Clone)]
pub enum ProcessedEvent {
    /// Initial tree was requested - send the full tree.
    InitialTreeRequested,
    /// An action was requested on a node - convert to icy Event.
    ActionRequested(AccessibilityEvent),
    /// Accessibility was deactivated.
    Deactivated,
}

impl ProcessedEvent {
    /// Converts an AccessKit winit window event to a processed event.
    pub fn from_accesskit_event(event: &accesskit_winit::WindowEvent) -> Self {
        match event {
            accesskit_winit::WindowEvent::InitialTreeRequested => Self::InitialTreeRequested,
            accesskit_winit::WindowEvent::ActionRequested(request) => {
                Self::ActionRequested(AccessibilityEvent::from_request(request.clone()))
            }
            accesskit_winit::WindowEvent::AccessibilityDeactivated => Self::Deactivated,
        }
    }

    /// Returns the accessibility event if this is an action request.
    pub fn into_event(self) -> Option<AccessibilityEvent> {
        match self {
            Self::ActionRequested(event) => Some(event),
            _ => None,
        }
    }
}

/// Handles an AccessKit event and returns what action to take.
#[derive(Debug, Clone)]
#[deprecated(note = "Use ProcessedEvent instead")]
pub enum AccessibilityAction {
    /// Initial tree was requested - send the full tree.
    InitialTreeRequested,
    /// An action was requested on a node.
    ActionRequested(accesskit::ActionRequest),
    /// Accessibility was deactivated.
    Deactivated,
}

#[allow(deprecated)]
impl AccessibilityAction {
    /// Converts an AccessKit winit event to an action.
    pub fn from_event(event: &accesskit_winit::WindowEvent) -> Self {
        match event {
            accesskit_winit::WindowEvent::InitialTreeRequested => Self::InitialTreeRequested,
            accesskit_winit::WindowEvent::ActionRequested(request) => {
                Self::ActionRequested(request.clone())
            }
            accesskit_winit::WindowEvent::AccessibilityDeactivated => Self::Deactivated,
        }
    }
}
