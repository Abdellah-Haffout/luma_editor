use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use luma_core::graph::GraphStore;
use luma_core::id::{NodeId, PinId};
use crate::cache::{EngineCacheStore, NodeExecutionStatus};
use crate::descriptor::{ExecutionError, NodeKernel, SlotId};
use crate::registry::NodeRegistry;
use crate::types::coerce;
use crate::value::{PrimitiveValue, Value};

/// The central dataflow execution engine.
/// Coordinates Dirty-Push propagation, Demand-Pull evaluation, memoization, and early exit.
pub struct DataflowEngine {
    pub registry: NodeRegistry,
    pub cache: EngineCacheStore,
    /// Unconnected pin parameter overrides edited directly by user widgets (sliders, textboxes, etc.).
    pub pin_values: HashMap<PinId, PrimitiveValue>,
}

impl Default for DataflowEngine {
    fn default() -> Self {
        Self {
            registry: NodeRegistry::new(),
            cache: EngineCacheStore::default(),
            pin_values: HashMap::new(),
        }
    }
}

impl DataflowEngine {
    pub fn new(registry: NodeRegistry) -> Self {
        Self {
            registry,
            cache: EngineCacheStore::default(),
            pin_values: HashMap::new(),
        }
    }

    /// Sets or modifies an un-wired input pin's local parameter value.
    pub fn set_pin_value(&mut self, graph: &GraphStore, pin_id: PinId, val: PrimitiveValue) -> Option<HashSet<NodeId>> {
        self.pin_values.insert(pin_id, val);
        if let Some(pin) = graph.get_pin(pin_id) {
            Some(self.push_dirty(graph, pin.node_id))
        } else {
            None
        }
    }

    /// Recursively pushes dirty flags forward through downstream connections (Dirty-Push).
    /// Does NOT perform any computation until pulled!
    pub fn push_dirty(&mut self, graph: &GraphStore, root_node_id: NodeId) -> HashSet<NodeId> {
        let downstream = graph.downstream_dependents(root_node_id);
        self.cache.mark_dirty(root_node_id);
        for &dep_id in &downstream {
            self.cache.mark_dirty(dep_id);
        }
        downstream
    }

    /// Returns the cached output values for a node, if computed and clean.
    pub fn get_cached_outputs(&self, node_id: NodeId) -> Option<&[Value]> {
        self.cache.caches.get(&node_id).map(|c| c.cached_outputs.as_slice())
    }

    /// Resolves the current incoming value for a given input slot of a node.
    pub fn resolve_input_value(&self, graph: &GraphStore, node_id: NodeId, slot_idx: usize) -> Value {
        let node = match graph.get_node(node_id) {
            Some(n) => n,
            None => return Value::none(),
        };

        let in_pin_id = match node.inputs.get(slot_idx) {
            Some(&p) => p,
            None => return Value::none(),
        };

        let conn_ids = graph.get_pin_connections(in_pin_id);
        if let Some(&conn_id) = conn_ids.first() {
            if let Some(conn) = graph.get_connection(conn_id) {
                if let Some(src_pin) = graph.get_pin(conn.source_pin) {
                    if let Some(src_node) = graph.get_node(src_pin.node_id) {
                        let out_idx = src_node.outputs.iter().position(|&p| p == conn.source_pin).unwrap_or(0);
                        if let Some(outputs) = self.get_cached_outputs(src_pin.node_id) {
                            if let Some(val) = outputs.get(out_idx) {
                                return val.clone();
                            }
                        }
                    }
                }
            }
        }

        // Unwired: check pin_values or descriptor default
        if let Some(prim) = self.pin_values.get(&in_pin_id) {
            return Value::Primitive(prim.clone());
        }

        if let Some(desc) = self.registry.get(&node.kind) {
            if let Some(pin_desc) = desc.inputs.get(slot_idx) {
                return Value::Primitive(pin_desc.default_value.clone());
            }
        }

        Value::none()
    }

    /// Evaluates a node on demand by pulling upstream dependencies (Demand-Pull).

    /// Returns a slice of the computed or cached output values.
    pub fn pull_node(
        &mut self,
        graph: &GraphStore,
        target_node_id: NodeId,
        pinned_nodes: &HashSet<NodeId>,
    ) -> Result<Vec<Value>, ExecutionError> {
        self.cache.advance_tick();

        let node = graph
            .get_node(target_node_id)
            .ok_or_else(|| ExecutionError::Custom(format!("Node {} not found in graph", target_node_id)))?;

        // 1. Fetch node descriptor from registry
        let descriptor = match self.registry.get(&node.kind) {
            Some(d) => d.clone(),
            None => {
                let err_msg = format!("Unknown node type '{}'", node.kind);
                let cache = self.cache.get_or_create(target_node_id);
                cache.status = NodeExecutionStatus::Broken(err_msg.clone());
                return Err(ExecutionError::Custom(err_msg));
            }
        };

        // 2. Check if node is pure and already clean
        if descriptor.is_pure {
            let current_tick = self.cache.current_tick;
            let cache = self.cache.get_or_create(target_node_id);
            if cache.status == NodeExecutionStatus::Clean && !cache.cached_outputs.is_empty() {
                cache.last_accessed_tick = current_tick;
                return Ok(cache.cached_outputs.clone());
            }
        }

        // 3. Demand-Pull upstream inputs
        let mut input_values = Vec::new();
        let mut input_fingerprints = Vec::new();

        for (in_idx, &in_pin_id) in node.inputs.iter().enumerate() {
            let pin_desc = descriptor.inputs.get(in_idx);
            let conn_ids = graph.get_pin_connections(in_pin_id);

            let raw_val = if let Some(&conn_id) = conn_ids.first() {
                if let Some(conn) = graph.get_connection(conn_id) {
                    let src_pin = match graph.get_pin(conn.source_pin) {
                        Some(p) => p,
                        None => return Err(ExecutionError::InputMissing(SlotId::Fixed(in_idx as u16))),
                    };

                    // Pull upstream node
                    let upstream_outputs = self.pull_node(graph, src_pin.node_id, pinned_nodes)?;

                    // Find matching output slot index on upstream node
                    let upstream_node = graph.get_node(src_pin.node_id).unwrap();
                    let out_idx = upstream_node
                        .outputs
                        .iter()
                        .position(|&p| p == conn.source_pin)
                        .unwrap_or(0);

                    upstream_outputs
                        .get(out_idx)
                        .cloned()
                        .unwrap_or_else(Value::none)
                } else {
                    Value::none()
                }
            } else {
                // Pin has no wire: use local parameter override or default value from descriptor
                let prim = self
                    .pin_values
                    .get(&in_pin_id)
                    .cloned()
                    .or_else(|| pin_desc.map(|d| d.default_value.clone()))
                    .unwrap_or(PrimitiveValue::None);
                Value::Primitive(prim)
            };

            // Coerce value to expected pin type according to single source of truth
            let expected_type = pin_desc.map(|d| &d.data_type).unwrap_or(&crate::types::DataType::ANY);
            let coerced = coerce(&raw_val, expected_type).ok_or_else(|| {
                ExecutionError::TypeMismatch {
                    slot: SlotId::Fixed(in_idx as u16),
                    expected: expected_type.clone(),
                    found: raw_val.data_type(),
                }
            })?;

            input_fingerprints.push(coerced.fingerprint());
            input_values.push(coerced);
        }

        // 4. Memoization check: if pure and input fingerprints match previous run
        if descriptor.is_pure {
            let current_tick = self.cache.current_tick;
            let cache = self.cache.get_or_create(target_node_id);
            if cache.matches_inputs(&input_fingerprints) && !cache.cached_outputs.is_empty() {
                cache.status = NodeExecutionStatus::Clean;
                cache.last_accessed_tick = current_tick;
                return Ok(cache.cached_outputs.clone());
            }
        }

        // 5. Execute kernel
        let mut output_values = vec![Value::none(); descriptor.outputs.len()];

        match descriptor.kernel.as_ref() {
            NodeKernel::Sync(sync_fn) => {
                sync_fn(&input_values, &mut output_values)?;
            }
            NodeKernel::Async(async_fn) => {
                // For synchronous testing / evaluation, block on async future
                let task_ctx = Arc::new(crate::descriptor::TaskContext::new());
                let fut = async_fn(input_values.clone(), task_ctx);
                let res = futures_lite_block_on(fut)?;
                output_values = res;
            }
        }

        // 6. Early-Exit check: if outputs are identical to previous outputs, do not propagate dirty flags!
        let is_unchanged = if let Some(cache) = self.cache.caches.get(&target_node_id) {
            let old = &cache.cached_outputs;
            old.len() == output_values.len()
                && old
                    .iter()
                    .zip(&output_values)
                    .all(|(old, new)| new.matches_cached(old))
        } else {
            false
        };

        // 7. Update cache with LRU memory management
        self.cache.update_outputs(
            target_node_id,
            output_values.clone(),
            input_fingerprints,
            pinned_nodes,
        );

        if !is_unchanged {
            // New outputs differ: dirty immediate downstream dependents only if they weren't dirty already
            for conn_id in graph.connections().values() {
                if conn_id.source_node == target_node_id {
                    self.cache.mark_dirty(conn_id.target_node);
                }
            }
        }

        Ok(output_values)
    }
}

// Lightweight future runner for sync test evaluation without requiring full tokio runtime
fn futures_lite_block_on<F: std::future::Future>(mut fut: F) -> F::Output {
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

    fn dummy_raw_waker() -> RawWaker {
        fn no_op(_: *const ()) {}
        fn clone(_: *const ()) -> RawWaker {
            dummy_raw_waker()
        }
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, no_op, no_op, no_op);
        RawWaker::new(std::ptr::null(), &VTABLE)
    }

    let waker = unsafe { Waker::from_raw(dummy_raw_waker()) };
    let mut cx = Context::from_waker(&waker);
    // Pin on stack
    let mut pinned = unsafe { std::pin::Pin::new_unchecked(&mut fut) };
    loop {
        match pinned.as_mut().poll(&mut cx) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}
