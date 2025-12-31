//! Operate on widgets that provide accessibility information.

use crate::Rectangle;
use crate::accessibility::{Node, NodeId, WidgetInfo, node_id_from_widget_id};
use crate::widget::Id;
use crate::widget::operation::{Operation, Outcome};

use std::collections::BTreeMap;

/// A collected accessibility tree fragment.
///
/// This is intended to be consumed by shells that integrate with an accessibility
/// backend (e.g. AccessKit).
#[derive(Debug, Clone, Default)]
pub struct AccessibilityTree {
    /// All collected nodes.
    pub nodes: Vec<(NodeId, Node)>,

    /// A mapping from `NodeId` to widget `Id`, when available.
    pub widgets: BTreeMap<NodeId, Id>,
}

/// Produces an [`Operation`] that collects accessibility information from the widget tree.
pub fn collect() -> impl Operation<AccessibilityTree> {
    struct Collect {
        next_generated: u64,
        tree: AccessibilityTree,
    }

    impl Collect {
        const GENERATED_BASE: u64 = 1 << 62;

        fn generated_id(&mut self) -> NodeId {
            let id = NodeId(Self::GENERATED_BASE | self.next_generated);
            self.next_generated = self.next_generated.wrapping_add(1);
            id
        }
    }

    impl Operation<AccessibilityTree> for Collect {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<AccessibilityTree>)) {
            operate(self);
        }

        fn accessibility(&mut self, id: Option<&Id>, _bounds: Rectangle, info: WidgetInfo) {
            let node_id = id
                .map(node_id_from_widget_id)
                .unwrap_or_else(|| self.generated_id());

            if let Some(id) = id {
                let _ = self.tree.widgets.insert(node_id, id.clone());
            }

            self.tree.nodes.push((node_id, info.build()));
        }

        fn finish(&self) -> Outcome<AccessibilityTree> {
            Outcome::Some(self.tree.clone())
        }
    }

    Collect {
        next_generated: 0,
        tree: AccessibilityTree::default(),
    }
}
