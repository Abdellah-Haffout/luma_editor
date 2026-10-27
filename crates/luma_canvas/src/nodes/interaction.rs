use luma_core::graph::GraphStore;
use luma_core::history::{HistoryStack, MoveNodesCommand};
use luma_core::id::NodeId;
use bevy::math::Vec2;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct NodeDragState {
    drag_start_cursor: Option<Vec2>,
    start_positions: HashMap<NodeId, Vec2>,
}

impl NodeDragState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_dragging(&self) -> bool {
        self.drag_start_cursor.is_some()
    }

    pub fn begin_drag(
        &mut self,
        node_ids: &[NodeId],
        cursor_canvas_pos: Vec2,
        graph: &GraphStore,
    ) {
        self.drag_start_cursor = Some(cursor_canvas_pos);
        self.start_positions.clear();

        for &id in node_ids {
            if let Some(node) = graph.get_node(id) {
                self.start_positions.insert(id, node.position);
            }
        }
    }

    pub fn update_drag(&self, current_cursor_canvas_pos: Vec2, graph: &mut GraphStore) {
        let start_cursor = match self.drag_start_cursor {
            Some(c) => c,
            None => return,
        };

        let delta = current_cursor_canvas_pos - start_cursor;

        for (&id, &start_pos) in &self.start_positions {
            if let Some(node) = graph.get_node_mut(id) {
                node.position = start_pos + delta;
            }
        }
    }

    pub fn finish_drag(
        &mut self,
        graph: &mut GraphStore,
        history: &mut HistoryStack,
    ) -> Result<bool, luma_core::history::HistoryError> {
        if self.drag_start_cursor.take().is_none() {
            return Ok(false);
        }

        let mut moves = Vec::new();
        for (&id, &start_pos) in &self.start_positions {
            if let Some(node) = graph.get_node(id) {
                if (node.position - start_pos).length_squared() > 1.0 {
                    moves.push((id, start_pos, node.position));
                }
            }
        }

        self.start_positions.clear();

        if moves.is_empty() {
            return Ok(false);
        }

        let cmd = Box::new(MoveNodesCommand::new(moves));
        // We already mutated the node positions in update_drag, but we want the command in history.
        // HistoryStack::apply will execute the command, so we temporarily revert to old_pos before apply.
        history.apply(cmd, graph)?;

        Ok(true)
    }

    pub fn cancel_drag(&mut self, graph: &mut GraphStore) {
        self.drag_start_cursor = None;
        for (&id, &start_pos) in &self.start_positions {
            if let Some(node) = graph.get_node_mut(id) {
                node.position = start_pos;
            }
        }
        self.start_positions.clear();
    }
}
