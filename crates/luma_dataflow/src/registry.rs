use std::collections::HashMap;
use std::sync::Arc;

use crate::descriptor::NodeDescriptor;

/// Central catalog containing declarative descriptors and compute kernels for all registered node types.
/// The core editor never hardcodes any node: any plugin or crate registers its nodes here.
#[derive(Default, Clone)]
pub struct NodeRegistry {
    descriptors: HashMap<String, Arc<NodeDescriptor>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new node descriptor in the catalog.
    pub fn register(&mut self, descriptor: NodeDescriptor) {
        let type_id = descriptor.type_id.to_string();
        self.descriptors.insert(type_id, Arc::new(descriptor));
    }

    /// Retrieves a node descriptor by its unique type string ID.
    pub fn get(&self, type_id: &str) -> Option<&Arc<NodeDescriptor>> {
        self.descriptors.get(type_id)
    }

    /// Checks if a type ID is registered in the catalog.
    pub fn contains(&self, type_id: &str) -> bool {
        self.descriptors.contains_key(type_id)
    }

    /// Returns an iterator over all registered descriptors for menus and search palettes.
    pub fn descriptors(&self) -> impl Iterator<Item = &Arc<NodeDescriptor>> {
        self.descriptors.values()
    }

    /// Returns the total count of registered node types.
    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }
}
