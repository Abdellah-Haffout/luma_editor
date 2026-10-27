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
        Self::new()
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
