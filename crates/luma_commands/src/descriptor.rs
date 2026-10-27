use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandId(pub String);

impl CommandId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl From<&str> for CommandId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for CommandId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandCategory {
    Graph,
    Edit,
    View,
    Tools,
    File,
}

impl CommandCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Graph => "Graph",
            Self::Edit => "Edit",
            Self::View => "View",
            Self::Tools => "Tools",
            Self::File => "File",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDescriptor {
    pub id: CommandId,
    pub title: String,
    pub category: CommandCategory,
    pub description: String,
    pub default_shortcut: Option<String>,
    pub is_undoable: bool,
}

impl CommandDescriptor {
    pub fn new(
        id: impl Into<CommandId>,
        title: impl Into<String>,
        category: CommandCategory,
        description: impl Into<String>,
        is_undoable: bool,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            category,
            description: description.into(),
            default_shortcut: None,
            is_undoable,
        }
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.default_shortcut = Some(shortcut.into());
        self
    }
}

#[derive(Debug, Default)]
pub struct CommandRegistry {
    descriptors: HashMap<CommandId, CommandDescriptor>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, descriptor: CommandDescriptor) {
        self.descriptors.insert(descriptor.id.clone(), descriptor);
    }

    pub fn get(&self, id: &CommandId) -> Option<&CommandDescriptor> {
        self.descriptors.get(id)
    }

    pub fn all(&self) -> impl Iterator<Item = &CommandDescriptor> {
        self.descriptors.values()
    }

    pub fn search(&self, query: &str) -> Vec<&CommandDescriptor> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return self.descriptors.values().collect();
        }

        let mut results: Vec<&CommandDescriptor> = self
            .descriptors
            .values()
            .filter(|desc| {
                desc.title.to_lowercase().contains(&q)
                    || desc.id.0.to_lowercase().contains(&q)
                    || desc.category.as_str().to_lowercase().contains(&q)
                    || desc.description.to_lowercase().contains(&q)
            })
            .collect();

        results.sort_by(|a, b| a.title.cmp(&b.title));
        results
    }
}
