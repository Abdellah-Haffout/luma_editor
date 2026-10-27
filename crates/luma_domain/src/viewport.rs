use luma_core::id::NodeId;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NodeViewportTarget {
    pub node_id: NodeId,
    pub resolution: (u32, u32),
    pub is_dirty: bool,
    pub is_visible: bool,
    pub last_render_time: f64,
    pub fps_limit: u32,
}

impl NodeViewportTarget {
    pub fn new(node_id: NodeId, resolution: (u32, u32)) -> Self {
        Self {
            node_id,
            resolution,
            is_dirty: true,
            is_visible: true,
            last_render_time: 0.0,
            fps_limit: 15, // Default throttled frame rate for mini-viewports
        }
    }

    /// Determines if this mini-viewport needs a GPU render pass on this frame.
    /// Employs dirty-checking, frustum culling, and fps throttling.
    pub fn should_render(&self, current_time: f64) -> bool {
        if !self.is_visible || !self.is_dirty {
            return false;
        }

        let min_interval = 1.0 / (self.fps_limit as f64);
        (current_time - self.last_render_time) >= min_interval
    }

    pub fn mark_rendered(&mut self, current_time: f64) {
        self.last_render_time = current_time;
        self.is_dirty = false;
    }
}

#[derive(Debug, Default)]
pub struct ViewportManager {
    targets: HashMap<NodeId, NodeViewportTarget>,
}

impl ViewportManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, node_id: NodeId, resolution: (u32, u32)) {
        self.targets
            .insert(node_id, NodeViewportTarget::new(node_id, resolution));
    }

    pub fn unregister(&mut self, node_id: NodeId) -> Option<NodeViewportTarget> {
        self.targets.remove(&node_id)
    }

    pub fn mark_dirty(&mut self, node_id: NodeId) {
        if let Some(target) = self.targets.get_mut(&node_id) {
            target.is_dirty = true;
        }
    }

    pub fn set_visibility(&mut self, node_id: NodeId, visible: bool) {
        if let Some(target) = self.targets.get_mut(&node_id) {
            target.is_visible = visible;
        }
    }

    pub fn get_target(&self, node_id: NodeId) -> Option<&NodeViewportTarget> {
        self.targets.get(&node_id)
    }

    pub fn get_target_mut(&mut self, node_id: NodeId) -> Option<&mut NodeViewportTarget> {
        self.targets.get_mut(&node_id)
    }

    pub fn active_count(&self) -> usize {
        self.targets.len()
    }
}
