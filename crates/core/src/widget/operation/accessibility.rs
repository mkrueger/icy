//! Operate on widgets that provide accessibility information.

use crate::Rectangle;
use crate::Vector;
use crate::accessibility::{
    Node, NodeId, TextSelectionTarget, WidgetInfo, derived_node_id, node_id_from_widget_id,
};
use crate::widget::Id;
use crate::widget::operation::{Operation, Outcome, Scrollable};

use std::collections::BTreeMap;
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A collected accessibility tree fragment.
///
/// This is intended to be consumed by shells that integrate with an accessibility
/// backend (e.g. AccessKit).
#[derive(Debug, Clone, Default)]
pub struct AccessibilityTree {
    /// All collected nodes (both top-level widget nodes and generated child nodes).
    pub nodes: Vec<(NodeId, Node)>,

    /// NodeIds of top-level widget nodes that should be direct children of the root.
    /// Extra generated children (e.g. TextRun) are NOT included here.
    pub top_level_ids: Vec<NodeId>,

    /// A mapping from `NodeId` to widget `Id`, when available.
    pub widgets: BTreeMap<NodeId, Id>,
}

/// Produces an [`Operation`] that collects accessibility information from the widget tree.
///
/// This operation tracks the widget tree path and generates stable NodeIds based on
/// the path when no explicit widget Id is provided. This ensures that accessibility
/// nodes have stable identities across frames as long as the widget tree structure
/// remains the same.
pub fn collect() -> impl Operation<AccessibilityTree> {
    struct Collect {
        /// Current path in the widget tree (indices at each level).
        path: Vec<usize>,
        /// Counter for children at the current level (used to track sibling index).
        child_counter: Vec<usize>,
        /// The collected accessibility tree.
        tree: AccessibilityTree,

        /// NodeIds referenced as children by any collected node.
        ///
        /// These should NOT appear as direct children of the root.
        referenced_children: HashSet<NodeId>,
        /// Stack of accumulated scroll translations.
        /// Each entry is the cumulative translation up to that scrollable container.
        translation_stack: Vec<Vector>,

        /// Number of scrollable translations to pop after the current `traverse`.
        ///
        /// `Scrollable::operate` calls `operation.scrollable(...)` and then immediately
        /// calls `operation.traverse(...)` to operate on its content. We use this counter
        /// to scope the scroll translation to that content traversal.
        pending_scroll_pops: usize,
    }

    impl Collect {
        /// Base value for path-based IDs to avoid collision with other ID schemes.
        /// Uses a different bit pattern than the old generated IDs.
        const PATH_BASED_ID_BASE: u64 = 1 << 61;

        /// Returns the current accumulated translation from all scrollable containers.
        fn current_translation(&self) -> Vector {
            self.translation_stack
                .last()
                .copied()
                .unwrap_or(Vector::ZERO)
        }

        /// Generates a stable NodeId based on the current path in the widget tree.
        ///
        /// The path is hashed to produce a stable ID that remains the same
        /// as long as the widget tree structure doesn't change.
        fn path_based_id(&self) -> NodeId {
            let mut hasher = DefaultHasher::new();
            // Hash both the path AND the current child counter to differentiate
            // siblings at the same level
            self.path.hash(&mut hasher);
            // Include the current sibling index at this level
            if let Some(&current_index) = self.child_counter.last() {
                current_index.hash(&mut hasher);
            }
            let hash = hasher.finish();

            // Combine with base to avoid collision with reserved IDs (0, 1)
            // and widget-id-based IDs
            let id = Self::PATH_BASED_ID_BASE | (hash & ((1 << 61) - 1));

            // Ensure we don't produce 0 or 1 (reserved for root and announcer)
            if id <= 1 { NodeId(id + 2) } else { NodeId(id) }
        }

        /// Called when entering a container to track the path.
        fn enter_container(&mut self) {
            // Get the current child index at this level
            let child_index = self.child_counter.last().copied().unwrap_or(0);
            self.path.push(child_index);
            // Start a new counter for children at this new level
            self.child_counter.push(0);
        }

        /// Called when leaving a container.
        fn leave_container_impl(&mut self) {
            let _ = self.path.pop();
            let _ = self.child_counter.pop();
            // Increment the counter at the parent level for the next sibling
            if let Some(counter) = self.child_counter.last_mut() {
                *counter += 1;
            }
        }

        /// Called when we see a leaf widget (accessibility node without children).
        fn count_sibling(&mut self) {
            if let Some(counter) = self.child_counter.last_mut() {
                *counter += 1;
            }
        }
    }

    impl Operation<AccessibilityTree> for Collect {
        fn container(&mut self, _id: Option<&Id>, _bounds: Rectangle) {
            self.enter_container();
        }

        fn leave_container(&mut self) {
            self.leave_container_impl();
        }

        fn scrollable(
            &mut self,
            _id: Option<&Id>,
            _bounds: Rectangle,
            _content_bounds: Rectangle,
            translation: Vector,
            _state: &mut dyn Scrollable,
        ) {
            // `Scrollable` renders content with `-translation`.
            // Therefore, to map content-local widget bounds to screen/window coordinates,
            // we need to apply `-translation` to all descendants.
            let current = self.current_translation();
            let scroll_delta = Vector::new(-translation.x, -translation.y);
            self.translation_stack.push(current + scroll_delta);

            // This translation applies only to the immediately-following `traverse`
            // that operates on the scrollable content.
            self.pending_scroll_pops += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<AccessibilityTree>)) {
            let pending_pops = self.pending_scroll_pops;
            self.pending_scroll_pops = 0;

            operate(self);

            // Pop any scrollable translations that were installed right before
            // this traversal (i.e. for the scrollable whose content we're traversing).
            for _ in 0..pending_pops {
                let _ = self.translation_stack.pop();
            }
        }

        fn accessibility(&mut self, id: Option<&Id>, bounds: Rectangle, info: WidgetInfo) {
            // Apply the current scroll translation to the bounds
            let translation = self.current_translation();
            let translated_bounds = Rectangle {
                x: bounds.x + translation.x,
                y: bounds.y + translation.y,
                width: bounds.width,
                height: bounds.height,
            };

            // Update the info with translated bounds
            let mut info = info;
            info.bounds = translated_bounds;

            // Also translate extra_children bounds
            for child in &mut info.extra_children {
                child.bounds = Rectangle {
                    x: child.bounds.x + translation.x,
                    y: child.bounds.y + translation.y,
                    width: child.bounds.width,
                    height: child.bounds.height,
                };
            }

            // Use explicit widget ID if available, otherwise generate from path
            let node_id = id
                .map(node_id_from_widget_id)
                .unwrap_or_else(|| self.path_based_id());

            if let Some(id) = id {
                let _ = self.tree.widgets.insert(node_id, id.clone());
            }

            // Build any extra generated children first (e.g. TextRun).
            let mut extra_child_ids: Vec<NodeId> = Vec::new();
            let mut extra_nodes: Vec<(NodeId, Node)> = Vec::new();

            for (index, child_info) in info.extra_children.iter().cloned().enumerate() {
                let child_id = derived_node_id(node_id, index as u64);
                extra_child_ids.push(child_id);

                // Extra children currently do not need their own selection remapping.
                let child_children = child_info.children.clone();
                let child_node = child_info.build_with_context(child_id, child_children);
                extra_nodes.push((child_id, child_node));
            }

            // Determine which node `TextSelection` should point to.
            let selection_node = match info.text_selection_target {
                TextSelectionTarget::ThisNode => node_id,
                TextSelectionTarget::ExtraChild(i) => {
                    extra_child_ids.get(i).copied().unwrap_or(node_id)
                }
            };

            // Combine widget-provided children with generated extra children.
            let mut combined_children = info.children.clone();
            combined_children.extend(extra_child_ids.iter().copied());

            // Track all referenced children so we can compute a valid root children list.
            for child in &combined_children {
                let _ = self.referenced_children.insert(*child);
            }

            let main_node = info.build_with_context(selection_node, combined_children);
            self.tree.nodes.push((node_id, main_node));

            // Track this as a top-level node (direct child of root).
            self.tree.top_level_ids.push(node_id);

            // Add generated child nodes to the tree (but NOT to top_level_ids).
            self.tree.nodes.extend(extra_nodes);

            // Count this as a sibling for path tracking
            self.count_sibling();
        }

        fn finish(&self) -> Outcome<AccessibilityTree> {
            let mut tree = self.tree.clone();

            // Only keep nodes that are not referenced as children.
            // This produces a structurally valid root children list.
            tree.top_level_ids = tree
                .top_level_ids
                .iter()
                .copied()
                .filter(|id| !self.referenced_children.contains(id))
                .collect();

            Outcome::Some(tree)
        }
    }

    Collect {
        path: Vec::new(),
        child_counter: vec![0], // Start with a counter for the root level
        tree: AccessibilityTree::default(),
        referenced_children: HashSet::new(),
        translation_stack: Vec::new(),
        pending_scroll_pops: 0,
    }
}

/// Produces an [`Operation`] that finds the currently focused accessibility node.
///
/// Unlike `operation::focusable::find_focused`, this does not require widgets to
/// have an explicit [`Id`]. It relies on the same NodeId scheme as [`collect`]:
/// widget IDs when present, otherwise a path-based fallback.
#[cfg(feature = "accessibility")]
pub fn find_focused_node() -> impl Operation<NodeId> {
    use crate::widget::operation::Focusable;

    struct FindFocused {
        path: Vec<usize>,
        child_counter: Vec<usize>,
        pending_node_id: Option<NodeId>,
        focused: Option<NodeId>,
    }

    impl FindFocused {
        const PATH_BASED_ID_BASE: u64 = 1 << 61;

        fn path_based_id(&self) -> NodeId {
            let mut hasher = DefaultHasher::new();
            self.path.hash(&mut hasher);
            if let Some(&current_index) = self.child_counter.last() {
                current_index.hash(&mut hasher);
            }
            let hash = hasher.finish();

            let id = Self::PATH_BASED_ID_BASE | (hash & ((1 << 61) - 1));

            if id <= 1 { NodeId(id + 2) } else { NodeId(id) }
        }

        fn enter_container(&mut self) {
            // Clear any pending node id when we enter a new widget container.
            self.pending_node_id = None;

            let child_index = self.child_counter.last().copied().unwrap_or(0);
            self.path.push(child_index);
            self.child_counter.push(0);
        }

        fn leave_container_impl(&mut self) {
            let _ = self.path.pop();
            let _ = self.child_counter.pop();

            if let Some(counter) = self.child_counter.last_mut() {
                *counter += 1;
            }
        }

        fn count_sibling(&mut self) {
            if let Some(counter) = self.child_counter.last_mut() {
                *counter += 1;
            }
        }
    }

    impl Operation<NodeId> for FindFocused {
        fn container(&mut self, _id: Option<&Id>, _bounds: Rectangle) {
            self.enter_container();
        }

        fn leave_container(&mut self) {
            self.leave_container_impl();
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<NodeId>)) {
            if self.focused.is_some() {
                return;
            }

            operate(self);
        }

        fn accessibility(&mut self, id: Option<&Id>, _bounds: Rectangle, _info: WidgetInfo) {
            let node_id = id
                .map(node_id_from_widget_id)
                .unwrap_or_else(|| self.path_based_id());

            self.pending_node_id = Some(node_id);
            self.count_sibling();
        }

        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if self.focused.is_some() || !state.is_focused() {
                // Count sibling only if we didn't get it from accessibility
                if self.pending_node_id.is_none() {
                    self.count_sibling();
                } else {
                    self.pending_node_id = None;
                }
                return;
            }

            // Check if we got the NodeId from a preceding accessibility() call
            let from_accessibility = self.pending_node_id.is_some();

            self.focused = Some(
                id.map(node_id_from_widget_id)
                    .or(self.pending_node_id.take())
                    .unwrap_or_else(|| self.path_based_id()),
            );

            // Count sibling only if we didn't get it from accessibility
            if !from_accessibility {
                self.count_sibling();
            }
        }

        fn finish(&self) -> Outcome<NodeId> {
            self.focused.map_or(Outcome::None, Outcome::Some)
        }
    }

    FindFocused {
        path: Vec::new(),
        child_counter: vec![0],
        pending_node_id: None,
        focused: None,
    }
}

/// Result of navigating through accessibility nodes.
#[derive(Debug, Clone, Copy)]
pub struct A11yNavigation {
    /// The new focus node after navigation.
    pub new_focus: Option<NodeId>,
    /// Total number of focusable accessibility nodes.
    pub total_nodes: usize,
}

/// Produces an [`Operation`] that moves the accessibility focus to the next node.
///
/// This navigates through all accessibility nodes, wrapping from last to first.
/// The `current_focus` is the currently focused NodeId (if any).
#[cfg(feature = "accessibility")]
pub fn focus_next_a11y_node(current_focus: Option<NodeId>) -> impl Operation<A11yNavigation> {
    CollectAndNavigate {
        current_focus,
        direction: NavigateDirection::Next,
        path: Vec::new(),
        child_counter: vec![0],
        nodes: Vec::new(),
    }
}

/// Produces an [`Operation`] that moves the accessibility focus to the previous node.
///
/// This navigates through all accessibility nodes, wrapping from first to last.
/// The `current_focus` is the currently focused NodeId (if any).
#[cfg(feature = "accessibility")]
pub fn focus_previous_a11y_node(current_focus: Option<NodeId>) -> impl Operation<A11yNavigation> {
    CollectAndNavigate {
        current_focus,
        direction: NavigateDirection::Previous,
        path: Vec::new(),
        child_counter: vec![0],
        nodes: Vec::new(),
    }
}

#[derive(Debug, Clone, Copy)]
enum NavigateDirection {
    Next,
    Previous,
}

struct CollectAndNavigate {
    current_focus: Option<NodeId>,
    direction: NavigateDirection,
    path: Vec<usize>,
    child_counter: Vec<usize>,
    nodes: Vec<NodeId>,
}

impl CollectAndNavigate {
    const PATH_BASED_ID_BASE: u64 = 1 << 61;

    fn path_based_id(&self) -> NodeId {
        let mut hasher = DefaultHasher::new();
        self.path.hash(&mut hasher);
        if let Some(&current_index) = self.child_counter.last() {
            current_index.hash(&mut hasher);
        }
        let hash = hasher.finish();

        let id = Self::PATH_BASED_ID_BASE | (hash & ((1 << 61) - 1));

        if id <= 1 { NodeId(id + 2) } else { NodeId(id) }
    }

    fn enter_container(&mut self) {
        let child_index = self.child_counter.last().copied().unwrap_or(0);
        self.path.push(child_index);
        self.child_counter.push(0);
    }

    fn leave_container_impl(&mut self) {
        let _ = self.path.pop();
        let _ = self.child_counter.pop();

        if let Some(counter) = self.child_counter.last_mut() {
            *counter += 1;
        }
    }

    fn count_sibling(&mut self) {
        if let Some(counter) = self.child_counter.last_mut() {
            *counter += 1;
        }
    }
}

impl Operation<A11yNavigation> for CollectAndNavigate {
    fn container(&mut self, _id: Option<&Id>, _bounds: Rectangle) {
        self.enter_container();
    }

    fn leave_container(&mut self) {
        self.leave_container_impl();
    }

    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<A11yNavigation>)) {
        operate(self);
    }

    fn accessibility(&mut self, id: Option<&Id>, _bounds: Rectangle, info: WidgetInfo) {
        // Only collect focusable nodes for navigation
        if info.focusable {
            let node_id = id
                .map(node_id_from_widget_id)
                .unwrap_or_else(|| self.path_based_id());

            self.nodes.push(node_id);
        }

        self.count_sibling();
    }

    fn finish(&self) -> Outcome<A11yNavigation> {
        let total_nodes = self.nodes.len();

        if total_nodes == 0 {
            return Outcome::Some(A11yNavigation {
                new_focus: None,
                total_nodes: 0,
            });
        }

        // Find current focus index
        let current_index = self
            .current_focus
            .and_then(|focus| self.nodes.iter().position(|&id| id == focus));

        let new_index = match (current_index, self.direction) {
            (None, NavigateDirection::Next) => 0,
            (None, NavigateDirection::Previous) => total_nodes - 1,
            (Some(idx), NavigateDirection::Next) => {
                if idx + 1 >= total_nodes {
                    0
                } else {
                    idx + 1
                }
            }
            (Some(idx), NavigateDirection::Previous) => {
                if idx == 0 {
                    total_nodes - 1
                } else {
                    idx - 1
                }
            }
        };

        Outcome::Some(A11yNavigation {
            new_focus: self.nodes.get(new_index).copied(),
            total_nodes,
        })
    }
}

/// Produces an [`Operation`] that finds the accessibility node at a given position.
///
/// This is used to update the VoiceOver focus when the user clicks on a widget.
/// Returns the NodeId of the focusable accessibility node at the given position,
/// or None if no focusable node is at that position.
#[cfg(feature = "accessibility")]
pub fn find_a11y_node_at_position(position: crate::Point) -> impl Operation<Option<NodeId>> {
    struct FindAtPosition {
        position: crate::Point,
        path: Vec<usize>,
        child_counter: Vec<usize>,
        found: Option<NodeId>,
    }

    impl FindAtPosition {
        const PATH_BASED_ID_BASE: u64 = 1 << 61;

        fn path_based_id(&self) -> NodeId {
            let mut hasher = DefaultHasher::new();
            self.path.hash(&mut hasher);
            if let Some(&current_index) = self.child_counter.last() {
                current_index.hash(&mut hasher);
            }
            let hash = hasher.finish();

            let id = Self::PATH_BASED_ID_BASE | (hash & ((1 << 61) - 1));

            if id <= 1 { NodeId(id + 2) } else { NodeId(id) }
        }

        fn enter_container(&mut self) {
            let child_index = self.child_counter.last().copied().unwrap_or(0);
            self.path.push(child_index);
            self.child_counter.push(0);
        }

        fn leave_container_impl(&mut self) {
            let _ = self.path.pop();
            let _ = self.child_counter.pop();

            if let Some(counter) = self.child_counter.last_mut() {
                *counter += 1;
            }
        }

        fn count_sibling(&mut self) {
            if let Some(counter) = self.child_counter.last_mut() {
                *counter += 1;
            }
        }
    }

    impl Operation<Option<NodeId>> for FindAtPosition {
        fn container(&mut self, _id: Option<&Id>, _bounds: Rectangle) {
            self.enter_container();
        }

        fn leave_container(&mut self) {
            self.leave_container_impl();
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Option<NodeId>>)) {
            operate(self);
        }

        fn accessibility(&mut self, id: Option<&Id>, bounds: Rectangle, info: WidgetInfo) {
            // Check if position is within bounds and node is focusable
            if info.focusable && bounds.contains(self.position) {
                let node_id = id
                    .map(node_id_from_widget_id)
                    .unwrap_or_else(|| self.path_based_id());

                // Keep the last (topmost) matching node
                self.found = Some(node_id);
            }

            self.count_sibling();
        }

        fn finish(&self) -> Outcome<Option<NodeId>> {
            Outcome::Some(self.found)
        }
    }

    FindAtPosition {
        position,
        path: Vec::new(),
        child_counter: vec![0],
        found: None,
    }
}

/// Produces an [`Operation`] that sets the widget focus to match the given NodeId.
///
/// This is used by VoiceOver navigation to keep the widget focus in sync with
/// the accessibility focus. When the user navigates via Tab in VO mode, we need
/// to also update the widget focus so that things like cursor rendering work.
#[cfg(feature = "accessibility")]
pub fn focus_widget_by_node_id<T>(target_node_id: NodeId) -> impl Operation<T> {
    use crate::widget::operation::Focusable;

    struct FocusByNodeId {
        target: NodeId,
        path: Vec<usize>,
        child_counter: Vec<usize>,
        /// The NodeId from the last accessibility() call, used by focusable().
        pending_node_id: Option<NodeId>,
    }

    impl FocusByNodeId {
        const PATH_BASED_ID_BASE: u64 = 1 << 61;

        fn path_based_id(&self) -> NodeId {
            let mut hasher = DefaultHasher::new();
            self.path.hash(&mut hasher);
            if let Some(&current_index) = self.child_counter.last() {
                current_index.hash(&mut hasher);
            }
            let hash = hasher.finish();

            let id = Self::PATH_BASED_ID_BASE | (hash & ((1 << 61) - 1));

            if id <= 1 { NodeId(id + 2) } else { NodeId(id) }
        }

        fn enter_container(&mut self) {
            // Clear pending when entering a new container
            self.pending_node_id = None;

            let child_index = self.child_counter.last().copied().unwrap_or(0);
            self.path.push(child_index);
            self.child_counter.push(0);
        }

        fn leave_container_impl(&mut self) {
            let _ = self.path.pop();
            let _ = self.child_counter.pop();

            if let Some(counter) = self.child_counter.last_mut() {
                *counter += 1;
            }
        }

        fn count_sibling(&mut self) {
            if let Some(counter) = self.child_counter.last_mut() {
                *counter += 1;
            }
        }

        fn node_id_for(&self, id: Option<&Id>) -> NodeId {
            id.map(node_id_from_widget_id)
                .unwrap_or_else(|| self.path_based_id())
        }
    }

    impl<T> Operation<T> for FocusByNodeId {
        fn container(&mut self, _id: Option<&Id>, _bounds: Rectangle) {
            self.enter_container();
        }

        fn leave_container(&mut self) {
            self.leave_container_impl();
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }

        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            // Check if we got the NodeId from a preceding accessibility() call
            let from_accessibility = self.pending_node_id.is_some();

            // Use the NodeId from accessibility() if available (called just before),
            // otherwise calculate it ourselves.
            let node_id = self
                .pending_node_id
                .take()
                .or_else(|| id.map(node_id_from_widget_id))
                .unwrap_or_else(|| self.path_based_id());

            if node_id == self.target {
                state.focus();
            } else {
                state.unfocus();
            }

            // Only count sibling if we didn't get it from accessibility
            // (accessibility already counted)
            if !from_accessibility {
                self.count_sibling();
            }
        }

        fn accessibility(&mut self, id: Option<&Id>, _bounds: Rectangle, _info: WidgetInfo) {
            // Store the NodeId for the following focusable() call
            let node_id = self.node_id_for(id);
            self.pending_node_id = Some(node_id);
            self.count_sibling();
        }
    }

    FocusByNodeId {
        target: target_node_id,
        path: Vec::new(),
        child_counter: vec![0],
        pending_node_id: None,
    }
}
