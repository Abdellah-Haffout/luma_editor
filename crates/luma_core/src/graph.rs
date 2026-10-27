use crate::id::{ConnectionId, NodeId, PinId};
use bevy::math::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PinDirection {
    #[default]
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PinCapacity {
    Single,
    Multiple,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DataType {
    pub name: String,
}

impl DataType {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn any() -> Self {
        Self::new("Any")
    }

    pub fn is_compatible_with(&self, other: &DataType) -> bool {
        self.name == "Any" || other.name == "Any" || self.name == other.name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pin {
    pub id: PinId,
    pub node_id: NodeId,
    pub name: String,
    pub direction: PinDirection,
    pub data_type: DataType,
    pub capacity: PinCapacity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub title: String,
    pub kind: String,
    pub position: Vec2,
    pub size: Vec2,
    pub inputs: Vec<PinId>,
    pub outputs: Vec<PinId>,
    pub is_collapsed: bool,
    #[serde(default)]
    pub subgraph: Option<Box<GraphStore>>,
}

impl Node {
    pub fn new(id: NodeId, title: impl Into<String>, kind: impl Into<String>, position: Vec2) -> Self {
        Self {
            id,
            title: title.into(),
            kind: kind.into(),
            position,
            size: Vec2::new(200.0, 150.0),
            inputs: Vec::new(),
            outputs: Vec::new(),
            is_collapsed: false,
            subgraph: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    pub id: ConnectionId,
    pub source_node: NodeId,
    pub source_pin: PinId,
    pub target_node: NodeId,
    pub target_pin: PinId,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum GraphError {
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),

    #[error("Pin not found: {0}")]
    PinNotFound(PinId),

    #[error("Connection not found: {0}")]
    ConnectionNotFound(ConnectionId),

    #[error("Invalid connection direction: source must be Output and target must be Input")]
    InvalidDirection,

    #[error("Self connection is not allowed on node: {0}")]
    SelfConnection(NodeId),

    #[error("Incompatible data types: '{source_type}' cannot connect to '{target_type}'")]
    IncompatibleTypes { source_type: String, target_type: String },

    #[error("Target pin {0} capacity exceeded")]
    CapacityExceeded(PinId),

    #[error("Connecting node {from} to {to} would create a circular dependency")]
    CycleDetected { from: NodeId, to: NodeId },
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GraphStore {
    nodes: HashMap<NodeId, Node>,
    pins: HashMap<PinId, Pin>,
    connections: HashMap<ConnectionId, Connection>,

    #[serde(skip)]
    pin_connections: HashMap<PinId, Vec<ConnectionId>>,
}

impl GraphStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.pins.clear();
        self.connections.clear();
        self.pin_connections.clear();
    }

    pub fn nodes(&self) -> &HashMap<NodeId, Node> {
        &self.nodes
    }

    pub fn pins(&self) -> &HashMap<PinId, Pin> {
        &self.pins
    }

    pub fn connections(&self) -> &HashMap<ConnectionId, Connection> {
        &self.connections
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    pub fn get_pin(&self, id: PinId) -> Option<&Pin> {
        self.pins.get(&id)
    }

    pub fn get_connection(&self, id: ConnectionId) -> Option<&Connection> {
        self.connections.get(&id)
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
    }

    pub fn remove_node(&mut self, id: NodeId) -> Option<(Node, Vec<Pin>, Vec<Connection>)> {
        let node = self.nodes.remove(&id)?;

        // Remove all pins belonging to this node
        let mut removed_pins = Vec::new();
        let all_pin_ids: Vec<PinId> = node.inputs.iter().chain(node.outputs.iter()).copied().collect();
        for pin_id in all_pin_ids {
            if let Some(pin) = self.pins.remove(&pin_id) {
                removed_pins.push(pin);
            }
        }

        // Find and remove all connections attached to these pins
        let mut removed_connections = Vec::new();
        let conn_ids_to_remove: Vec<ConnectionId> = self
            .connections
            .values()
            .filter(|c| c.source_node == id || c.target_node == id)
            .map(|c| c.id)
            .collect();

        for conn_id in conn_ids_to_remove {
            if let Some(conn) = self.disconnect(conn_id) {
                removed_connections.push(conn);
            }
        }

        Some((node, removed_pins, removed_connections))
    }

    pub fn add_pin(&mut self, pin: Pin) -> Result<(), GraphError> {
        let node = self.nodes.get_mut(&pin.node_id).ok_or(GraphError::NodeNotFound(pin.node_id))?;
        match pin.direction {
            PinDirection::Input => {
                if !node.inputs.contains(&pin.id) {
                    node.inputs.push(pin.id);
                }
            }
            PinDirection::Output => {
                if !node.outputs.contains(&pin.id) {
                    node.outputs.push(pin.id);
                }
            }
        }
        self.pins.insert(pin.id, pin);
        Ok(())
    }

    pub fn remove_pin(&mut self, pin_id: PinId) -> Option<(Pin, Vec<Connection>)> {
        let pin = self.pins.remove(&pin_id)?;

        if let Some(node) = self.nodes.get_mut(&pin.node_id) {
            node.inputs.retain(|&id| id != pin_id);
            node.outputs.retain(|&id| id != pin_id);
        }

        let mut removed_conns = Vec::new();
        if let Some(conn_ids) = self.pin_connections.remove(&pin_id) {
            for conn_id in conn_ids {
                if let Some(conn) = self.connections.remove(&conn_id) {
                    // Also clean up from the other pin's connection list
                    let other_pin = if conn.source_pin == pin_id {
                        conn.target_pin
                    } else {
                        conn.source_pin
                    };
                    if let Some(conns) = self.pin_connections.get_mut(&other_pin) {
                        conns.retain(|&id| id != conn_id);
                    }
                    removed_conns.push(conn);
                }
            }
        }

        Some((pin, removed_conns))
    }

    pub fn validate_connection(&self, source_pin_id: PinId, target_pin_id: PinId) -> Result<(), GraphError> {
        let source_pin = self.pins.get(&source_pin_id).ok_or(GraphError::PinNotFound(source_pin_id))?;
        let target_pin = self.pins.get(&target_pin_id).ok_or(GraphError::PinNotFound(target_pin_id))?;

        if source_pin.direction != PinDirection::Output || target_pin.direction != PinDirection::Input {
            return Err(GraphError::InvalidDirection);
        }

        if source_pin.node_id == target_pin.node_id {
            return Err(GraphError::SelfConnection(source_pin.node_id));
        }

        if !source_pin.data_type.is_compatible_with(&target_pin.data_type) {
            return Err(GraphError::IncompatibleTypes {
                source_type: source_pin.data_type.name.clone(),
                target_type: target_pin.data_type.name.clone(),
            });
        }

        if target_pin.capacity == PinCapacity::Single {
            let existing_count = self
                .pin_connections
                .get(&target_pin_id)
                .map(|v| v.len())
                .unwrap_or(0);
            if existing_count >= 1 {
                return Err(GraphError::CapacityExceeded(target_pin_id));
            }
        }

        if self.would_create_cycle(source_pin.node_id, target_pin.node_id) {
            return Err(GraphError::CycleDetected {
                from: source_pin.node_id,
                to: target_pin.node_id,
            });
        }

        Ok(())
    }

    pub fn would_create_cycle(&self, from_node: NodeId, to_node: NodeId) -> bool {
        if from_node == to_node {
            return true;
        }

        // BFS/DFS: starting from to_node, can we reach from_node via existing connections?
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(to_node);
        visited.insert(to_node);

        while let Some(current) = queue.pop_front() {
            if current == from_node {
                return true;
            }

            // Find all outgoing connections from `current` node
            for conn in self.connections.values() {
                if conn.source_node == current && visited.insert(conn.target_node) {
                    queue.push_back(conn.target_node);
                }
            }
        }

        false
    }

    pub fn connect(&mut self, conn_id: ConnectionId, source_pin_id: PinId, target_pin_id: PinId) -> Result<Connection, GraphError> {
        self.validate_connection(source_pin_id, target_pin_id)?;

        let source_pin = self.pins.get(&source_pin_id).unwrap();
        let target_pin = self.pins.get(&target_pin_id).unwrap();

        let connection = Connection {
            id: conn_id,
            source_node: source_pin.node_id,
            source_pin: source_pin_id,
            target_node: target_pin.node_id,
            target_pin: target_pin_id,
        };

        self.connections.insert(conn_id, connection.clone());
        self.pin_connections.entry(source_pin_id).or_default().push(conn_id);
        self.pin_connections.entry(target_pin_id).or_default().push(conn_id);

        Ok(connection)
    }

    pub fn add_connection(&mut self, conn: Connection) {
        self.pin_connections.entry(conn.source_pin).or_default().push(conn.id);
        self.pin_connections.entry(conn.target_pin).or_default().push(conn.id);
        self.connections.insert(conn.id, conn);
    }

    pub fn disconnect(&mut self, conn_id: ConnectionId) -> Option<Connection> {
        let conn = self.connections.remove(&conn_id)?;

        if let Some(conns) = self.pin_connections.get_mut(&conn.source_pin) {
            conns.retain(|&id| id != conn_id);
        }
        if let Some(conns) = self.pin_connections.get_mut(&conn.target_pin) {
            conns.retain(|&id| id != conn_id);
        }

        Some(conn)
    }

    pub fn get_pin_connections(&self, pin_id: PinId) -> &[ConnectionId] {
        self.pin_connections.get(&pin_id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn rebuild_indices(&mut self) {
        self.pin_connections.clear();
        for conn in self.connections.values() {
            self.pin_connections.entry(conn.source_pin).or_default().push(conn.id);
            self.pin_connections.entry(conn.target_pin).or_default().push(conn.id);
        }
    }

    /// Returns all node IDs in topological order (dependency order) such that for any connection
    /// A -> B, A appears before B.
    /// Returns `Err(GraphError::CycleDetected)` if a circular dependency is detected.
    pub fn topological_sort(&self) -> Result<Vec<NodeId>, GraphError> {
        let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
        let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

        for &node_id in self.nodes.keys() {
            in_degree.insert(node_id, 0);
            adjacency.insert(node_id, Vec::new());
        }

        for conn in self.connections.values() {
            if self.nodes.contains_key(&conn.source_node) && self.nodes.contains_key(&conn.target_node) {
                *in_degree.entry(conn.target_node).or_insert(0) += 1;
                adjacency.entry(conn.source_node).or_default().push(conn.target_node);
            }
        }

        let mut queue: VecDeque<NodeId> = in_degree
            .iter()
            .filter(|&(_, deg)| *deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut sorted = Vec::with_capacity(self.nodes.len());

        while let Some(node_id) = queue.pop_front() {
            sorted.push(node_id);

            if let Some(neighbors) = adjacency.get(&node_id) {
                for &neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(&neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        if sorted.len() != self.nodes.len() {
            let remaining: Vec<NodeId> = in_degree
                .into_iter()
                .filter(|(_, deg)| *deg > 0)
                .map(|(id, _)| id)
                .collect();
            let from = remaining.first().copied().unwrap_or(NodeId(0));
            let to = remaining.get(1).copied().unwrap_or(from);
            return Err(GraphError::CycleDetected { from, to });
        }

        Ok(sorted)
    }

    /// Recursively collects all node IDs that feed data into `node_id`.
    pub fn upstream_dependencies(&self, node_id: NodeId) -> HashSet<NodeId> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(node) = self.nodes.get(&node_id) {
            for &pin_id in &node.inputs {
                for &conn_id in self.get_pin_connections(pin_id) {
                    if let Some(conn) = self.connections.get(&conn_id) {
                        if visited.insert(conn.source_node) {
                            queue.push_back(conn.source_node);
                        }
                    }
                }
            }
        }

        while let Some(curr) = queue.pop_front() {
            if let Some(curr_node) = self.nodes.get(&curr) {
                for &pin_id in &curr_node.inputs {
                    for &conn_id in self.get_pin_connections(pin_id) {
                        if let Some(conn) = self.connections.get(&conn_id) {
                            if visited.insert(conn.source_node) {
                                queue.push_back(conn.source_node);
                            }
                        }
                    }
                }
            }
        }

        visited
    }

    /// Recursively collects all node IDs that consume data from `node_id`.
    pub fn downstream_dependents(&self, node_id: NodeId) -> HashSet<NodeId> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(node) = self.nodes.get(&node_id) {
            for &pin_id in &node.outputs {
                for &conn_id in self.get_pin_connections(pin_id) {
                    if let Some(conn) = self.connections.get(&conn_id) {
                        if visited.insert(conn.target_node) {
                            queue.push_back(conn.target_node);
                        }
                    }
                }
            }
        }

        while let Some(curr) = queue.pop_front() {
            if let Some(curr_node) = self.nodes.get(&curr) {
                for &pin_id in &curr_node.outputs {
                    for &conn_id in self.get_pin_connections(pin_id) {
                        if let Some(conn) = self.connections.get(&conn_id) {
                            if visited.insert(conn.target_node) {
                                queue.push_back(conn.target_node);
                            }
                        }
                    }
                }
            }
        }

        visited
    }
}
