//! Accessibility integration for winit using AccessKit.

use accesskit::{Node, NodeId, Role, Tree, TreeUpdate};
use accesskit_winit::Adapter;
use winit::event::WindowEvent as WinitWindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use crate::core::Rectangle;

use std::fmt;

// Re-export for convenience
pub use crate::core::accessibility::Event as AccessibilityEvent;

/// Accessibility adapter wrapper for a window.
pub struct AccessibilityAdapter {
    adapter: Adapter,
    receiver: std::sync::mpsc::Receiver<ProcessedEvent>,
    enabled: bool,
}

impl fmt::Debug for AccessibilityAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccessibilityAdapter")
            .field("enabled", &self.enabled)
            .finish_non_exhaustive()
    }
}

impl AccessibilityAdapter {
    /// The root node ID for the window.
    pub const ROOT_ID: NodeId = NodeId(0);

    /// Creates a new accessibility adapter for a window.
    ///
    /// This must be called before the window is shown.
    pub fn new(event_loop: &ActiveEventLoop, window: &Window) -> Self {
        use accesskit::{ActionHandler, ActivationHandler, DeactivationHandler, TreeUpdate};

        let (sender, receiver) = std::sync::mpsc::channel::<ProcessedEvent>();

        struct Activate {
            sender: std::sync::mpsc::Sender<ProcessedEvent>,
        }

        impl ActivationHandler for Activate {
            fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
                let _ = self.sender.send(ProcessedEvent::Activated);
                None
            }
        }

        struct DoAction {
            sender: std::sync::mpsc::Sender<ProcessedEvent>,
        }

        impl ActionHandler for DoAction {
            fn do_action(&mut self, request: accesskit::ActionRequest) {
                let _ = self.sender.send(ProcessedEvent::ActionRequested(
                    AccessibilityEvent::from_request(request),
                ));
            }
        }

        struct Deactivate {
            sender: std::sync::mpsc::Sender<ProcessedEvent>,
        }

        impl DeactivationHandler for Deactivate {
            fn deactivate_accessibility(&mut self) {
                let _ = self.sender.send(ProcessedEvent::Deactivated);
            }
        }

        let adapter = Adapter::with_direct_handlers(
            event_loop,
            window,
            Activate {
                sender: sender.clone(),
            },
            DoAction {
                sender: sender.clone(),
            },
            Deactivate { sender },
        );

        Self {
            adapter,
            receiver,
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

    /// Drains any pending accessibility events.
    pub fn drain_events(&mut self) -> Vec<ProcessedEvent> {
        let mut drained = Vec::new();

        while let Ok(event) = self.receiver.try_recv() {
            match event {
                ProcessedEvent::Activated => self.activate(),
                ProcessedEvent::Deactivated => self.deactivate(),
                _ => {}
            }

            drained.push(event);
        }

        drained
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
    /// Accessibility was activated (screen reader connected).
    Activated,
    /// Initial tree was requested - send the full tree.
    InitialTreeRequested,
    /// An action was requested on a node - convert to icy Event.
    ActionRequested(AccessibilityEvent),
    /// Accessibility was deactivated.
    Deactivated,
}

impl ProcessedEvent {
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
    ///
    /// Deprecated: the winit-proxy style adapter is no longer used.
    pub fn from_event(_event: &accesskit_winit::WindowEvent) -> Self {
        Self::Deactivated
    }
}
