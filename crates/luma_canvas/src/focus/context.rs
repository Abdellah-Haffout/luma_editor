use luma_core::id::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkspaceContext {
    RootGraph,
    FocusedNode {
        node_id: NodeId,
        node_kind: String,
        node_title: String,
        #[serde(default)]
        saved_cam_x: f32,
        #[serde(default)]
        saved_cam_y: f32,
        #[serde(default = "default_scale")]
        saved_cam_scale: f32,
    },
}

fn default_scale() -> f32 {
    1.0
}

#[derive(Debug, Clone, PartialEq)]
pub struct NavigationState {
    stack: Vec<WorkspaceContext>,
}

impl Default for NavigationState {
    fn default() -> Self {
        Self {
            stack: vec![WorkspaceContext::RootGraph],
        }
    }
}

impl NavigationState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current(&self) -> &WorkspaceContext {
        self.stack.last().unwrap_or(&WorkspaceContext::RootGraph)
    }

    pub fn is_in_focus(&self) -> bool {
        matches!(self.current(), WorkspaceContext::FocusedNode { .. })
    }

    pub fn focused_node_id(&self) -> Option<NodeId> {
        match self.current() {
            WorkspaceContext::FocusedNode { node_id, .. } => Some(*node_id),
            WorkspaceContext::RootGraph => None,
        }
    }

    pub fn focused_node_kind(&self) -> Option<&str> {
        match self.current() {
            WorkspaceContext::FocusedNode { node_kind, .. } => Some(node_kind),
            WorkspaceContext::RootGraph => None,
        }
    }

    pub fn dive_into(
        &mut self,
        node_id: NodeId,
        node_kind: impl Into<String>,
        node_title: impl Into<String>,
        saved_cam_x: f32,
        saved_cam_y: f32,
        saved_cam_scale: f32,
    ) {
        self.stack.push(WorkspaceContext::FocusedNode {
            node_id,
            node_kind: node_kind.into(),
            node_title: node_title.into(),
            saved_cam_x,
            saved_cam_y,
            saved_cam_scale,
        });
    }

    pub fn exit_focus(&mut self) -> Option<WorkspaceContext> {
        if self.stack.len() > 1 {
            self.stack.pop()
        } else {
            None
        }
    }

    pub fn reset_to_root(&mut self) {
        self.stack.truncate(1);
    }

    pub fn breadcrumbs(&self) -> Vec<String> {
        self.stack
            .iter()
            .map(|ctx| match ctx {
                WorkspaceContext::RootGraph => "Root Graph".to_string(),
                WorkspaceContext::FocusedNode { node_title, .. } => node_title.clone(),
            })
            .collect()
    }

    pub fn depth(&self) -> usize {
        self.stack.len()
    }
}
