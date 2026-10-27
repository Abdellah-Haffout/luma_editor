use crate::graph::{Connection, GraphError, GraphStore, Node, Pin};
use crate::id::{ConnectionId, NodeId, PinId};
use bevy::math::Vec2;

#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    #[error("Graph error during command execution: {0}")]
    Graph(#[from] GraphError),

    #[error("Nothing to undo")]
    NothingToUndo,

    #[error("Nothing to redo")]
    NothingToRedo,

    #[error("Cannot perform operation while a transaction is in progress")]
    TransactionInProgress,
}

pub trait EditCommand: Send + Sync {
    fn execute(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError>;
    fn undo(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError>;
    fn description(&self) -> &str;
}

// -----------------------------------------------------------------------------
// Commands
// -----------------------------------------------------------------------------

pub struct AddNodeCommand {
    node: Option<Node>,
    added_id: Option<NodeId>,
}

impl AddNodeCommand {
    pub fn new(node: Node) -> Self {
        Self {
            node: Some(node),
            added_id: None,
        }
    }
}

impl EditCommand for AddNodeCommand {
    fn execute(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        let node = self.node.take().expect("Node data must be present for execute");
        let id = node.id;
        graph.add_node(node);
        self.added_id = Some(id);
        Ok(())
    }

    fn undo(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        let id = self.added_id.take().expect("Node ID must be present for undo");
        if let Some((node, _, _)) = graph.remove_node(id) {
            self.node = Some(node);
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "Add Node"
    }
}

pub struct RemoveNodeCommand {
    node_id: NodeId,
    cached_node: Option<Node>,
    cached_pins: Vec<Pin>,
    cached_connections: Vec<Connection>,
}

impl RemoveNodeCommand {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            cached_node: None,
            cached_pins: Vec::new(),
            cached_connections: Vec::new(),
        }
    }
}

impl EditCommand for RemoveNodeCommand {
    fn execute(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        if let Some((node, pins, conns)) = graph.remove_node(self.node_id) {
            self.cached_node = Some(node);
            self.cached_pins = pins;
            self.cached_connections = conns;
            Ok(())
        } else {
            Err(GraphError::NodeNotFound(self.node_id).into())
        }
    }

    fn undo(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        if let Some(node) = self.cached_node.take() {
            graph.add_node(node);
            for pin in self.cached_pins.drain(..) {
                graph.add_pin(pin)?;
            }
            for conn in self.cached_connections.drain(..) {
                let _ = graph.connect(conn.id, conn.source_pin, conn.target_pin);
            }
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "Remove Node"
    }
}

pub struct ConnectCommand {
    conn_id: ConnectionId,
    source_pin: PinId,
    target_pin: PinId,
}

impl ConnectCommand {
    pub fn new(conn_id: ConnectionId, source_pin: PinId, target_pin: PinId) -> Self {
        Self {
            conn_id,
            source_pin,
            target_pin,
        }
    }
}

impl EditCommand for ConnectCommand {
    fn execute(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        graph.connect(self.conn_id, self.source_pin, self.target_pin)?;
        Ok(())
    }

    fn undo(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        graph.disconnect(self.conn_id);
        Ok(())
    }

    fn description(&self) -> &str {
        "Connect Pins"
    }
}

pub struct DisconnectCommand {
    conn_id: ConnectionId,
    cached_connection: Option<Connection>,
}

impl DisconnectCommand {
    pub fn new(conn_id: ConnectionId) -> Self {
        Self {
            conn_id,
            cached_connection: None,
        }
    }
}

impl EditCommand for DisconnectCommand {
    fn execute(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        if let Some(conn) = graph.disconnect(self.conn_id) {
            self.cached_connection = Some(conn);
            Ok(())
        } else {
            Err(GraphError::ConnectionNotFound(self.conn_id).into())
        }
    }

    fn undo(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        if let Some(conn) = self.cached_connection.take() {
            graph.connect(conn.id, conn.source_pin, conn.target_pin)?;
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "Disconnect Pins"
    }
}

pub struct MoveNodesCommand {
    moves: Vec<(NodeId, Vec2, Vec2)>, // (id, old_pos, new_pos)
}

impl MoveNodesCommand {
    pub fn new(moves: Vec<(NodeId, Vec2, Vec2)>) -> Self {
        Self { moves }
    }
}

impl EditCommand for MoveNodesCommand {
    fn execute(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        for (id, _, new_pos) in &self.moves {
            if let Some(node) = graph.get_node_mut(*id) {
                node.position = *new_pos;
            }
        }
        Ok(())
    }

    fn undo(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        for (id, old_pos, _) in &self.moves {
            if let Some(node) = graph.get_node_mut(*id) {
                node.position = *old_pos;
            }
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "Move Nodes"
    }
}

pub struct CompoundTransaction {
    description: String,
    commands: Vec<Box<dyn EditCommand>>,
}

impl CompoundTransaction {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            commands: Vec::new(),
        }
    }

    pub fn add(&mut self, command: Box<dyn EditCommand>) {
        self.commands.push(command);
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl EditCommand for CompoundTransaction {
    fn execute(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        for cmd in &mut self.commands {
            cmd.execute(graph)?;
        }
        Ok(())
    }

    fn undo(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo(graph)?;
        }
        Ok(())
    }

    fn description(&self) -> &str {
        &self.description
    }
}

// -----------------------------------------------------------------------------
// History Stack
// -----------------------------------------------------------------------------

pub struct HistoryStack {
    undo_stack: Vec<Box<dyn EditCommand>>,
    redo_stack: Vec<Box<dyn EditCommand>>,
    active_transaction: Option<CompoundTransaction>,
    max_depth: usize,
}

impl Default for HistoryStack {
    fn default() -> Self {
        Self::new(100)
    }
}

impl HistoryStack {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            active_transaction: None,
            max_depth,
        }
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.active_transaction = None;
    }

    pub fn begin_transaction(&mut self, description: impl Into<String>) {
        self.active_transaction = Some(CompoundTransaction::new(description));
    }

    pub fn commit_transaction(&mut self) {
        if let Some(tx) = self.active_transaction.take() {
            if !tx.is_empty() {
                self.undo_stack.push(Box::new(tx));
                self.redo_stack.clear();
                self.trim();
            }
        }
    }

    pub fn cancel_transaction(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        if let Some(mut tx) = self.active_transaction.take() {
            tx.undo(graph)?;
        }
        Ok(())
    }

    pub fn apply(&mut self, mut cmd: Box<dyn EditCommand>, graph: &mut GraphStore) -> Result<(), HistoryError> {
        cmd.execute(graph)?;

        if let Some(tx) = &mut self.active_transaction {
            tx.add(cmd);
        } else {
            self.undo_stack.push(cmd);
            self.redo_stack.clear();
            self.trim();
        }

        Ok(())
    }

    pub fn undo(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        if self.active_transaction.is_some() {
            return Err(HistoryError::TransactionInProgress);
        }

        let mut cmd = self.undo_stack.pop().ok_or(HistoryError::NothingToUndo)?;
        cmd.undo(graph)?;
        self.redo_stack.push(cmd);
        Ok(())
    }

    pub fn redo(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        if self.active_transaction.is_some() {
            return Err(HistoryError::TransactionInProgress);
        }

        let mut cmd = self.redo_stack.pop().ok_or(HistoryError::NothingToRedo)?;
        cmd.execute(graph)?;
        self.undo_stack.push(cmd);
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty() && self.active_transaction.is_none()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty() && self.active_transaction.is_none()
    }

    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.last().map(|c| c.description())
    }

    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.last().map(|c| c.description())
    }

    fn trim(&mut self) {
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
    }
}
