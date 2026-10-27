use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct NodeId(pub u64);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node#{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct PinId(pub u64);

impl fmt::Display for PinId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pin#{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct ConnectionId(pub u64);

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Conn#{}", self.0)
    }
}

use std::sync::Arc;

/// A thread-safe, monotonically increasing ID generator for graph entities.
#[derive(Debug, Clone)]
pub struct IdGenerator {
    next_node_id: Arc<AtomicU64>,
    next_pin_id: Arc<AtomicU64>,
    next_connection_id: Arc<AtomicU64>,
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl IdGenerator {
    pub fn new() -> Self {
        Self {
            next_node_id: Arc::new(AtomicU64::new(1)),
            next_pin_id: Arc::new(AtomicU64::new(1)),
            next_connection_id: Arc::new(AtomicU64::new(1)),
        }
    }

    pub fn next_node_id(&self) -> NodeId {
        NodeId(self.next_node_id.fetch_add(1, Ordering::Relaxed))
    }

    pub fn next_pin_id(&self) -> PinId {
        PinId(self.next_pin_id.fetch_add(1, Ordering::Relaxed))
    }

    pub fn next_connection_id(&self) -> ConnectionId {
        ConnectionId(self.next_connection_id.fetch_add(1, Ordering::Relaxed))
    }

    pub fn seed(&self, max_node: u64, max_pin: u64, max_conn: u64) {
        self.next_node_id.store(max_node + 1, Ordering::Relaxed);
        self.next_pin_id.store(max_pin + 1, Ordering::Relaxed);
        self.next_connection_id.store(max_conn + 1, Ordering::Relaxed);
    }
}
