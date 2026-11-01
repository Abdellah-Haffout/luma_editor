use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextualTool {
    pub id: &'static str,
    pub title: String,
    pub category: String,
    pub icon: String,
}

impl ContextualTool {
    pub fn new(
        id: &'static str,
        title: impl Into<String>,
        category: impl Into<String>,
        icon: impl Into<String>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            category: category.into(),
            icon: icon.into(),
        }
    }
}

#[derive(Debug, Default)]
pub struct ContextualToolRegistry {
    tools_by_kind: HashMap<String, Vec<ContextualTool>>,
}

impl ContextualToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_default_toolsets() -> Self {
        let mut reg = Self::new();
        reg.register_tools(
            "MeshNode",
            vec![
                ContextualTool::new("mesh.translate", "Translate", "Transform", "move"),
                ContextualTool::new("mesh.rotate", "Rotate", "Transform", "rotate-3d"),
                ContextualTool::new("mesh.scale", "Scale", "Transform", "maximize-2"),
                ContextualTool::new("mesh.subdivide", "Subdivide", "Geometry", "grid"),
                ContextualTool::new("mesh.wireframe", "Wireframe", "Display", "eye"),
            ],
        );
        reg.register_tools(
            "MaterialNode",
            vec![
                ContextualTool::new("mat.color_picker", "Color Picker", "Shader", "palette"),
                ContextualTool::new("mat.roughness", "Roughness", "Shader", "sliders"),
                ContextualTool::new("mat.metallic", "Metallic", "Shader", "disc"),
                ContextualTool::new("mat.emission", "Emission", "Shader", "sun"),
            ],
        );
        reg.register_tools(
            "PhysicsNode",
            vec![
                ContextualTool::new("physics.collider", "Collider Shape", "Physics", "box"),
                ContextualTool::new("physics.mass", "Mass & Inertia", "Physics", "weight"),
                ContextualTool::new("physics.gravity", "Gravity & Forces", "Physics", "arrow-down"),
            ],
        );
        reg
    }

    pub fn register_tools(&mut self, kind: impl Into<String>, tools: Vec<ContextualTool>) {
        self.tools_by_kind.insert(kind.into(), tools);
    }

    /// Retrieve the tools strictly registered for this node kind.
    /// Guarantees Contextual Isolation: returns empty if kind has no tools.
    pub fn get_tools_for_kind(&self, kind: &str) -> &[ContextualTool] {
        self.tools_by_kind
            .get(kind)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}
