use luma_core::id::{ConnectionId, NodeId};
use bevy::math::Rect;
use std::collections::HashSet;

#[derive(Debug, Default, Clone)]
pub struct SelectionState {
    pub selected_nodes: HashSet<NodeId>,
    pub selected_connections: HashSet<ConnectionId>,
    pub box_selection: Option<Rect>,
}

impl SelectionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn select_node(&mut self, id: NodeId, additive: bool) {
        if !additive {
            self.clear();
        }
        self.selected_nodes.insert(id);
    }

    pub fn deselect_node(&mut self, id: NodeId) {
        self.selected_nodes.remove(&id);
    }

    pub fn toggle_node(&mut self, id: NodeId) {
        if self.selected_nodes.contains(&id) {
            self.selected_nodes.remove(&id);
        } else {
            self.selected_nodes.insert(id);
        }
    }

    pub fn select_connection(&mut self, id: ConnectionId, additive: bool) {
        if !additive {
            self.clear();
        }
        self.selected_connections.insert(id);
    }

    pub fn clear(&mut self) {
        self.selected_nodes.clear();
        self.selected_connections.clear();
        self.box_selection = None;
    }

    pub fn is_node_selected(&self, id: NodeId) -> bool {
        self.selected_nodes.contains(&id)
    }

    pub fn is_connection_selected(&self, id: ConnectionId) -> bool {
        self.selected_connections.contains(&id)
    }

    pub fn is_empty(&self) -> bool {
        self.selected_nodes.is_empty() && self.selected_connections.is_empty()
    }

    pub fn selected_nodes_count(&self) -> usize {
        self.selected_nodes.len()
    }
}
