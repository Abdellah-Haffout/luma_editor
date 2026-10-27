pub mod math;

pub use math::register_math_nodes;

use luma_dataflow::registry::NodeRegistry;
use std::sync::OnceLock;

/// Returns a shared reference to the standard library node registry.
pub fn stdlib_registry() -> &'static NodeRegistry {
    static REGISTRY: OnceLock<NodeRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut reg = NodeRegistry::new();
        register_stdlib(&mut reg);
        reg
    })
}

/// Populates a registry with the standard library of mathematical nodes.
pub fn register_stdlib(registry: &mut NodeRegistry) {
    register_math_nodes(registry);
}
