use std::collections::{HashMap, HashSet};

use luma_core::id::NodeId;
use crate::value::Value;

/// The execution lifecycle state of a node in the graph.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeExecutionStatus {
    /// Fully computed, cached, and up to date.
    Clean,
    /// Inputs or parameters changed; needs recalculation on demand.
    Dirty,
    /// Background asynchronous task running with progress fraction (0.0 to 1.0).
    Running { progress: f32 },
    /// Evaluation failed with explicit error details.
    Broken(String),
    /// Skipped because an upstream dependency failed or is blocked.
    Blocked,
}

/// Memoized cache entry for an individual node instance.
#[derive(Clone)]
pub struct NodeCache {
    pub execution_version: u64,
    /// 64-bit fingerprints of incoming inputs (ABA-resistant).
    pub input_fingerprints: Vec<u64>,
    /// Cached output values produced by the node kernel.
    pub cached_outputs: Vec<Value>,
    /// Status flag.
    pub status: NodeExecutionStatus,
    /// Logical clock tick when this node's cache was last accessed (for LRU eviction).
    pub last_accessed_tick: u64,
}

impl NodeCache {
    pub fn new() -> Self {
        Self {
            execution_version: 0,
            input_fingerprints: Vec::new(),
            cached_outputs: Vec::new(),
            status: NodeExecutionStatus::Dirty,
            last_accessed_tick: 0,
        }
    }

    /// Checks if incoming inputs match the cached fingerprints in O(N).
    pub fn matches_inputs(&self, current_fingerprints: &[u64]) -> bool {
        self.input_fingerprints.as_slice() == current_fingerprints
    }

    /// Calculates total heavy resource bytes held in this node's cached outputs.
    pub fn heavy_resource_bytes(&self) -> usize {
        self.cached_outputs
            .iter()
            .filter(|v| v.is_heavy_resource())
            .map(|v| v.footprint_bytes())
            .sum()
    }

    /// Evicts heavy resource buffers from this cache, leaving primitive values untouched!
    pub fn evict_heavy_resources(&mut self) -> usize {
        let mut freed = 0;
        for val in &mut self.cached_outputs {
            if val.is_heavy_resource() {
                freed += val.footprint_bytes();
                *val = Value::none();
            }
        }
        if freed > 0 {
            self.status = NodeExecutionStatus::Dirty;
        }
        freed
    }
}

/// Central cache store managing memoized node outputs and memory-budgeted LRU eviction.
pub struct EngineCacheStore {
    pub caches: HashMap<NodeId, NodeCache>,
    pub max_resource_bytes: usize,
    pub current_resource_bytes: usize,
    pub current_tick: u64,
}

impl Default for EngineCacheStore {
    fn default() -> Self {
        Self {
            caches: HashMap::new(),
            max_resource_bytes: 512 * 1024 * 1024, // 512 MB default budget
            current_resource_bytes: 0,
            current_tick: 0,
        }
    }
}

impl EngineCacheStore {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            caches: HashMap::new(),
            max_resource_bytes: max_bytes,
            current_resource_bytes: 0,
            current_tick: 0,
        }
    }

    pub fn advance_tick(&mut self) {
        self.current_tick = self.current_tick.wrapping_add(1);
    }

    pub fn get_or_create(&mut self, node_id: NodeId) -> &mut NodeCache {
        self.caches.entry(node_id).or_insert_with(NodeCache::new)
    }

    pub fn mark_dirty(&mut self, node_id: NodeId) {
        if let Some(cache) = self.caches.get_mut(&node_id) {
            cache.status = NodeExecutionStatus::Dirty;
        }
    }

    /// Stores newly computed outputs in the cache and enforces the memory budget via LRU.
    pub fn update_outputs(
        &mut self,
        node_id: NodeId,
        outputs: Vec<Value>,
        input_fingerprints: Vec<u64>,
        pinned_nodes: &HashSet<NodeId>,
    ) {
        let old_bytes = self.caches.get(&node_id).map(|c| c.heavy_resource_bytes()).unwrap_or(0);
        let new_bytes: usize = outputs.iter().filter(|v| v.is_heavy_resource()).map(|v| v.footprint_bytes()).sum();

        self.current_resource_bytes = self.current_resource_bytes.saturating_sub(old_bytes) + new_bytes;

        let tick = self.current_tick;
        let cache = self.get_or_create(node_id);
        cache.cached_outputs = outputs;
        cache.input_fingerprints = input_fingerprints;
        cache.status = NodeExecutionStatus::Clean;
        cache.execution_version = cache.execution_version.wrapping_add(1);
        cache.last_accessed_tick = tick;

        // Evict LRU intermediate resources if exceeding budget
        if self.current_resource_bytes > self.max_resource_bytes {
            self.evict_lru(pinned_nodes);
        }
    }

    /// Evicts oldest unpinned intermediate heavy resources when exceeding RAM threshold.
    /// Note: PrimitiveValues are NEVER evicted!
    fn evict_lru(&mut self, pinned_nodes: &HashSet<NodeId>) {
        let mut candidates: Vec<(NodeId, u64, usize)> = self
            .caches
            .iter()
            .filter(|(id, c)| !pinned_nodes.contains(id) && c.heavy_resource_bytes() > 0)
            .map(|(id, c)| (*id, c.last_accessed_tick, c.heavy_resource_bytes()))
            .collect();

        // Sort by oldest accessed first
        candidates.sort_by_key(|(_, tick, _)| *tick);

        for (node_id, _, _) in candidates {
            if self.current_resource_bytes <= self.max_resource_bytes {
                break;
            }
            if let Some(cache) = self.caches.get_mut(&node_id) {
                let freed = cache.evict_heavy_resources();
                self.current_resource_bytes = self.current_resource_bytes.saturating_sub(freed);
            }
        }
    }
}
