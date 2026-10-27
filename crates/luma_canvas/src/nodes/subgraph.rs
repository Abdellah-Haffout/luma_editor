use luma_core::graph::{
    GraphStore, Node, Pin, PinCapacity, PinDirection,
};
use luma_core::id::{IdGenerator, NodeId, PinId};
use bevy::math::Vec2;
use std::collections::{HashMap, HashSet};

/// Collapses a list of selected nodes into a single composite Subgraph node.
/// Automatically generates Group Input & Group Output boundary nodes inside the nested graph
/// and maps all external connections without breaking wiring continuity.
pub fn group_nodes(
    selected_node_ids: &[NodeId],
    graph: &mut GraphStore,
    id_gen: &IdGenerator,
) -> Result<NodeId, String> {
    if selected_node_ids.is_empty() {
        return Err("No nodes selected for grouping".to_string());
    }

    let selected_set: HashSet<NodeId> = selected_node_ids.iter().copied().collect();

    // 1. Calculate center position for the new Subgraph node
    let mut center = Vec2::ZERO;
    let mut count = 0.0;
    for &id in &selected_set {
        if let Some(node) = graph.get_node(id) {
            center += node.position;
            count += 1.0;
        }
    }
    if count > 0.0 {
        center /= count;
    }

    // 2. Prepare the inner subgraph container
    let mut inner_graph = GraphStore::new();

    // Spawn Group Input boundary node inside the subgraph
    let group_in_id = id_gen.next_node_id();
    let mut group_in_node = Node::new(group_in_id, "Group Input", "GroupInput", Vec2::new(-260.0, 0.0));
    group_in_node.size = Vec2::new(160.0, 76.0);
    inner_graph.add_node(group_in_node);

    // Spawn Group Output boundary node inside the subgraph
    let group_out_id = id_gen.next_node_id();
    let mut group_out_node = Node::new(group_out_id, "Group Output", "GroupOutput", Vec2::new(260.0, 0.0));
    group_out_node.size = Vec2::new(160.0, 76.0);
    inner_graph.add_node(group_out_node);

    // 3. Create the outer Subgraph node
    let subgraph_node_id = id_gen.next_node_id();
    let mut subgraph_node = Node::new(subgraph_node_id, "Node Group", "Subgraph", center);
    subgraph_node.size = Vec2::new(210.0, 76.0);

    // Keep track of external wire mappings:
    // (outer_source_pin, inner_target_pin) -> outer_subgraph_in_pin
    let mut in_pin_map: HashMap<PinId, PinId> = HashMap::new();
    // (inner_source_pin, outer_target_pin) -> outer_subgraph_out_pin
    let mut out_pin_map: HashMap<PinId, PinId> = HashMap::new();

    // 4. Analyze connections touching the selected nodes
    let all_conns: Vec<_> = graph.connections().values().cloned().collect();
    let mut conns_to_remove = Vec::new();

    for conn in all_conns {
        let src_in_group = selected_set.contains(&conn.source_node);
        let tgt_in_group = selected_set.contains(&conn.target_node);

        if src_in_group && tgt_in_group {
            // Internal connection: will be transferred to inner graph
            conns_to_remove.push(conn.id);
            let _ = inner_graph.add_connection(conn);
        } else if !src_in_group && tgt_in_group {
            // Incoming wire from outside into group:
            conns_to_remove.push(conn.id);
            let outer_in_pin = match in_pin_map.get(&conn.target_pin) {
                Some(&p) => p,
                None => {
                    let tgt_pin = match graph.get_pin(conn.target_pin) {
                        Some(p) => p.clone(),
                        None => continue,
                    };

                    // Add input pin to outer Subgraph node
                    let sub_in_id = id_gen.next_pin_id();
                    let sub_in_pin = Pin {
                        id: sub_in_id,
                        node_id: subgraph_node_id,
                        name: tgt_pin.name.clone(),
                        direction: PinDirection::Input,
                        data_type: tgt_pin.data_type.clone(),
                        capacity: PinCapacity::Single,
                    };
                    subgraph_node.inputs.push(sub_in_id);
                    let _ = graph.add_pin(sub_in_pin);

                    // Add matching output pin to inner GroupInput node
                    let grp_out_id = id_gen.next_pin_id();
                    let grp_out_pin = Pin {
                        id: grp_out_id,
                        node_id: group_in_id,
                        name: tgt_pin.name.clone(),
                        direction: PinDirection::Output,
                        data_type: tgt_pin.data_type.clone(),
                        capacity: PinCapacity::Multiple,
                    };
                    let _ = inner_graph.add_pin(grp_out_pin);
                    if let Some(in_node) = inner_graph.get_node_mut(group_in_id) {
                        in_node.outputs.push(grp_out_id);
                    }

                    // Connect GroupInput output to inner target pin
                    let inner_conn_id = id_gen.next_connection_id();
                    let _ = inner_graph.connect(inner_conn_id, grp_out_id, conn.target_pin);

                    in_pin_map.insert(conn.target_pin, sub_in_id);
                    sub_in_id
                }
            };

            // Wire outside source to outer Subgraph input pin
            let new_outer_conn_id = id_gen.next_connection_id();
            let _ = graph.connect(new_outer_conn_id, conn.source_pin, outer_in_pin);
        } else if src_in_group && !tgt_in_group {
            // Outgoing wire from group to outside:
            conns_to_remove.push(conn.id);
            let outer_out_pin = match out_pin_map.get(&conn.source_pin) {
                Some(&p) => p,
                None => {
                    let src_pin = match graph.get_pin(conn.source_pin) {
                        Some(p) => p.clone(),
                        None => continue,
                    };

                    // Add output pin to outer Subgraph node
                    let sub_out_id = id_gen.next_pin_id();
                    let sub_out_pin = Pin {
                        id: sub_out_id,
                        node_id: subgraph_node_id,
                        name: src_pin.name.clone(),
                        direction: PinDirection::Output,
                        data_type: src_pin.data_type.clone(),
                        capacity: PinCapacity::Multiple,
                    };
                    subgraph_node.outputs.push(sub_out_id);
                    let _ = graph.add_pin(sub_out_pin);

                    // Add matching input pin to inner GroupOutput node
                    let grp_in_id = id_gen.next_pin_id();
                    let grp_in_pin = Pin {
                        id: grp_in_id,
                        node_id: group_out_id,
                        name: src_pin.name.clone(),
                        direction: PinDirection::Input,
                        data_type: src_pin.data_type.clone(),
                        capacity: PinCapacity::Single,
                    };
                    let _ = inner_graph.add_pin(grp_in_pin);
                    if let Some(out_node) = inner_graph.get_node_mut(group_out_id) {
                        out_node.inputs.push(grp_in_id);
                    }

                    // Connect inner source pin to GroupOutput input
                    let inner_conn_id = id_gen.next_connection_id();
                    let _ = inner_graph.connect(inner_conn_id, conn.source_pin, grp_in_id);

                    out_pin_map.insert(conn.source_pin, sub_out_id);
                    sub_out_id
                }
            };

            // Wire outer Subgraph output pin to outside target
            let new_outer_conn_id = id_gen.next_connection_id();
            let _ = graph.connect(new_outer_conn_id, outer_out_pin, conn.target_pin);
        }
    }

    // 5. Remove disconnected/transferred outer connections
    for conn_id in conns_to_remove {
        let _ = graph.disconnect(conn_id);
    }

    // 6. Transfer selected nodes into inner graph (adjust positions relative to group center)
    for &id in &selected_set {
        if let Some((mut node, pins, _)) = graph.remove_node(id) {
            node.position -= center;
            inner_graph.add_node(node);
            for pin in pins {
                let _ = inner_graph.add_pin(pin);
            }
        }
    }

    // 7. Attach completed inner graph to outer Subgraph node
    subgraph_node.subgraph = Some(Box::new(inner_graph));
    graph.add_node(subgraph_node);

    Ok(subgraph_node_id)
}

/// Unpacks a composite Subgraph node back into its individual nodes in the outer graph,
/// restoring internal connections and preserving external wiring continuity.
pub fn ungroup_node(
    subgraph_node_id: NodeId,
    graph: &mut GraphStore,
    id_gen: &IdGenerator,
) -> Result<Vec<NodeId>, String> {
    let (subgraph_node, removed_pins, removed_connections) = graph
        .remove_node(subgraph_node_id)
        .ok_or_else(|| format!("Node {subgraph_node_id:?} not found"))?;

    let inner_graph = match subgraph_node.subgraph {
        Some(inner) => *inner,
        None => return Err(format!("Node {subgraph_node_id:?} is not a composite Subgraph")),
    };

    let group_pos = subgraph_node.position;

    // Find boundary node IDs inside inner graph
    let mut group_in_id = None;
    let mut group_out_id = None;
    for (&node_id, node) in inner_graph.nodes() {
        if node.kind == "GroupInput" {
            group_in_id = Some(node_id);
        } else if node.kind == "GroupOutput" {
            group_out_id = Some(node_id);
        }
    }

    // Map boundary pin names to target/source internal pins
    let mut boundary_in_wires: HashMap<String, PinId> = HashMap::new();
    let mut boundary_out_wires: HashMap<String, PinId> = HashMap::new();

    for conn in inner_graph.connections().values() {
        if let Some(gin) = group_in_id {
            if conn.source_node == gin {
                if let Some(pin) = inner_graph.get_pin(conn.source_pin) {
                    boundary_in_wires.insert(pin.name.clone(), conn.target_pin);
                }
            }
        }
        if let Some(gout) = group_out_id {
            if conn.target_node == gout {
                if let Some(pin) = inner_graph.get_pin(conn.target_pin) {
                    boundary_out_wires.insert(pin.name.clone(), conn.source_pin);
                }
            }
        }
    }

    // Map removed subgraph pins by ID
    let sub_pins_map: HashMap<PinId, Pin> = removed_pins.into_iter().map(|p| (p.id, p)).collect();

    // Transfer internal non-boundary nodes back into outer graph
    let mut unpacked_ids = Vec::new();
    for (&id, node) in inner_graph.nodes() {
        if Some(id) == group_in_id || Some(id) == group_out_id {
            continue;
        }
        let mut restored = node.clone();
        restored.position += group_pos;
        unpacked_ids.push(id);
        graph.add_node(restored);
    }

    // Transfer internal pins
    for (&_pin_id, pin) in inner_graph.pins() {
        if Some(pin.node_id) == group_in_id || Some(pin.node_id) == group_out_id {
            continue;
        }
        let _ = graph.add_pin(pin.clone());
    }

    // Transfer purely internal connections
    for conn in inner_graph.connections().values() {
        if Some(conn.source_node) == group_in_id
            || Some(conn.source_node) == group_out_id
            || Some(conn.target_node) == group_in_id
            || Some(conn.target_node) == group_out_id
        {
            continue;
        }
        let _ = graph.add_connection(conn.clone());
    }

    // Re-wire external incoming connections to inner target pins
    for conn in removed_connections {
        if conn.target_node == subgraph_node_id {
            if let Some(sub_pin) = sub_pins_map.get(&conn.target_pin) {
                if let Some(&inner_target_pin) = boundary_in_wires.get(&sub_pin.name) {
                    let new_conn_id = id_gen.next_connection_id();
                    let _ = graph.connect(new_conn_id, conn.source_pin, inner_target_pin);
                }
            }
        } else if conn.source_node == subgraph_node_id {
            if let Some(sub_pin) = sub_pins_map.get(&conn.source_pin) {
                if let Some(&inner_source_pin) = boundary_out_wires.get(&sub_pin.name) {
                    let new_conn_id = id_gen.next_connection_id();
                    let _ = graph.connect(new_conn_id, inner_source_pin, conn.target_pin);
                }
            }
        }
    }

    Ok(unpacked_ids)
}

use luma_core::history::{EditCommand, HistoryError};

pub struct GroupNodesCommand {
    selected_node_ids: Vec<NodeId>,
    id_gen: IdGenerator,
    created_subgraph_id: Option<NodeId>,
    unpacked_nodes_on_undo: Vec<NodeId>,
    created_id_out: Option<std::sync::Arc<std::sync::atomic::AtomicU64>>,
}

impl GroupNodesCommand {
    pub fn new(selected_node_ids: Vec<NodeId>, id_gen: IdGenerator) -> Self {
        Self {
            selected_node_ids,
            id_gen,
            created_subgraph_id: None,
            unpacked_nodes_on_undo: Vec::new(),
            created_id_out: None,
        }
    }

    pub fn with_output_channel(mut self, holder: std::sync::Arc<std::sync::atomic::AtomicU64>) -> Self {
        self.created_id_out = Some(holder);
        self
    }

    pub fn created_subgraph_id(&self) -> Option<NodeId> {
        self.created_subgraph_id
    }
}

impl EditCommand for GroupNodesCommand {
    fn execute(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        let targets = if !self.unpacked_nodes_on_undo.is_empty() {
            &self.unpacked_nodes_on_undo
        } else {
            &self.selected_node_ids
        };

        match group_nodes(targets, graph, &self.id_gen) {
            Ok(sub_id) => {
                self.created_subgraph_id = Some(sub_id);
                if let Some(holder) = &self.created_id_out {
                    holder.store(sub_id.0, std::sync::atomic::Ordering::Relaxed);
                }
                Ok(())
            }
            Err(_) => Err(HistoryError::Graph(luma_core::graph::GraphError::NodeNotFound(
                self.selected_node_ids.first().copied().unwrap_or(NodeId(0)),
            ))),
        }
    }

    fn undo(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        if let Some(sub_id) = self.created_subgraph_id.take() {
            match ungroup_node(sub_id, graph, &self.id_gen) {
                Ok(unpacked) => {
                    self.unpacked_nodes_on_undo = unpacked;
                    Ok(())
                }
                Err(_) => Err(HistoryError::Graph(luma_core::graph::GraphError::NodeNotFound(sub_id))),
            }
        } else {
            Ok(())
        }
    }

    fn description(&self) -> &str {
        "Group Nodes"
    }
}

pub struct UngroupNodeCommand {
    subgraph_node_id: NodeId,
    id_gen: IdGenerator,
    unpacked_nodes: Vec<NodeId>,
    unpacked_out: Option<std::sync::Arc<std::sync::Mutex<Vec<NodeId>>>>,
}

impl UngroupNodeCommand {
    pub fn new(subgraph_node_id: NodeId, id_gen: IdGenerator) -> Self {
        Self {
            subgraph_node_id,
            id_gen,
            unpacked_nodes: Vec::new(),
            unpacked_out: None,
        }
    }

    pub fn with_output_channel(mut self, holder: std::sync::Arc<std::sync::Mutex<Vec<NodeId>>>) -> Self {
        self.unpacked_out = Some(holder);
        self
    }

    pub fn unpacked_nodes(&self) -> &[NodeId] {
        &self.unpacked_nodes
    }
}

impl EditCommand for UngroupNodeCommand {
    fn execute(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        match ungroup_node(self.subgraph_node_id, graph, &self.id_gen) {
            Ok(unpacked) => {
                if let Some(holder) = &self.unpacked_out {
                    if let Ok(mut lock) = holder.lock() {
                        *lock = unpacked.clone();
                    }
                }
                self.unpacked_nodes = unpacked;
                Ok(())
            }
            Err(_) => Err(HistoryError::Graph(luma_core::graph::GraphError::NodeNotFound(self.subgraph_node_id))),
        }
    }

    fn undo(&mut self, graph: &mut GraphStore) -> Result<(), HistoryError> {
        if !self.unpacked_nodes.is_empty() {
            match group_nodes(&self.unpacked_nodes, graph, &self.id_gen) {
                Ok(sub_id) => {
                    self.subgraph_node_id = sub_id;
                    Ok(())
                }
                Err(_) => Err(HistoryError::Graph(luma_core::graph::GraphError::NodeNotFound(
                    self.unpacked_nodes.first().copied().unwrap_or(NodeId(0)),
                ))),
            }
        } else {
            Ok(())
        }
    }

    fn description(&self) -> &str {
        "Ungroup Nodes"
    }
}
