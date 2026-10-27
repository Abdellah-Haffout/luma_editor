use luma_core::graph::GraphStore;
use luma_core::id::{NodeId, PinId};
use crate::types::{AssetLoaderData, EntityAssemblerData, MaterialData, MeshData};
use bevy::math::{Quat, Vec3};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct AssembledSceneEntity {
    pub name: String,
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub mesh_source: Option<String>,
    pub material: Option<MaterialData>,
}

#[derive(Debug, Default, Clone)]
pub struct DomainPayloadStore {
    pub assets: HashMap<NodeId, AssetLoaderData>,
    pub meshes: HashMap<NodeId, MeshData>,
    pub materials: HashMap<NodeId, MaterialData>,
    pub entities: HashMap<NodeId, EntityAssemblerData>,
}

impl DomainPayloadStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Automatically initializes default payload data when a node of a given kind is added to the graph.
    pub fn init_default_payload(&mut self, node_id: NodeId, kind: &str) {
        match kind {
            "AssetLoader" => {
                self.assets.entry(node_id).or_default();
            }
            "MeshNode" => {
                self.meshes.entry(node_id).or_default();
            }
            "MaterialNode" => {
                self.materials.entry(node_id).or_default();
            }
            "EntityAssembler" => {
                self.entities.entry(node_id).or_default();
            }
            _ => {}
        }
    }

    /// Duplicates payload data from a source node to a newly created node.
    pub fn clone_payload(&mut self, from_id: NodeId, to_id: NodeId, kind: &str) {
        match kind {
            "AssetLoader" => {
                if let Some(data) = self.assets.get(&from_id).cloned() {
                    self.assets.insert(to_id, data);
                }
            }
            "MeshNode" => {
                if let Some(data) = self.meshes.get(&from_id).cloned() {
                    self.meshes.insert(to_id, data);
                }
            }
            "MaterialNode" => {
                if let Some(data) = self.materials.get(&from_id).cloned() {
                    self.materials.insert(to_id, data);
                }
            }
            "EntityAssembler" => {
                if let Some(mut data) = self.entities.get(&from_id).cloned() {
                    data.name = format!("{}_copy", data.name);
                    self.entities.insert(to_id, data);
                }
            }
            _ => {}
        }
    }

    /// Cleans up stored payload data when a node is removed from the graph.
    pub fn remove_payload(&mut self, node_id: NodeId) {
        self.assets.remove(&node_id);
        self.meshes.remove(&node_id);
        self.materials.remove(&node_id);
        self.entities.remove(&node_id);
    }

    /// Recursively traces an incoming connection pin backwards through Subgraphs and boundary nodes
    /// to locate the underlying Mesh source or Material data producer.
    fn resolve_source_producer(
        &self,
        source_pin_id: PinId,
        graph: &GraphStore,
    ) -> (Option<String>, Option<MaterialData>) {
        let source_pin = match graph.get_pin(source_pin_id) {
            Some(p) => p,
            None => return (None, None),
        };

        let src_node = match graph.get_node(source_pin.node_id) {
            Some(n) => n,
            None => return (None, None),
        };

        match src_node.kind.as_str() {
            "MeshNode" => {
                let mut upstream_mesh = None;
                for &in_pin_id in &src_node.inputs {
                    for &conn_id in graph.get_pin_connections(in_pin_id) {
                        if let Some(conn) = graph.get_connection(conn_id) {
                            let (u_mesh, _) = self.resolve_source_producer(conn.source_pin, graph);
                            if u_mesh.is_some() {
                                upstream_mesh = u_mesh;
                                break;
                            }
                        }
                    }
                    if upstream_mesh.is_some() {
                        break;
                    }
                }

                let mesh_src = upstream_mesh.or_else(|| {
                    self.meshes.get(&src_node.id).map(|m| m.mesh_source.clone())
                });
                (mesh_src, None)
            }
            "AssetLoader" => {
                let asset_path = self.assets.get(&src_node.id).map(|a| a.asset_path.clone());
                (asset_path, None)
            }
            "MaterialNode" => {
                let mat_data = self.materials.get(&src_node.id).cloned();
                (None, mat_data)
            }
            "Subgraph" => {
                // Outer pin connected to a Subgraph output:
                // Look inside the Subgraph's nested GraphStore for the GroupOutput node
                if let Some(inner) = &src_node.subgraph {
                    for (&_inner_node_id, inner_node) in inner.nodes() {
                        if inner_node.kind == "GroupOutput" {
                            // Find the GroupOutput input pin whose name matches the Subgraph output pin
                            for &in_pin_id in &inner_node.inputs {
                                if let Some(grp_pin) = inner.get_pin(in_pin_id) {
                                    if grp_pin.name == source_pin.name {
                                        // Trace backwards inside inner graph
                                        for &conn_id in inner.get_pin_connections(in_pin_id) {
                                            if let Some(conn) = inner.get_connection(conn_id) {
                                                return self.resolve_source_producer(conn.source_pin, inner);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                (None, None)
            }
            _ => (None, None),
        }
    }

    /// Evaluates an EntityAssembler node by tracing its incoming wires to upstream Mesh and Material nodes,
    /// seamlessly resolving through nested Subgraphs without breaking wiring continuity.
    pub fn evaluate_entity(
        &self,
        entity_node_id: NodeId,
        graph: &GraphStore,
    ) -> Result<AssembledSceneEntity, String> {
        let node = graph
            .get_node(entity_node_id)
            .ok_or_else(|| format!("Entity node not found: {}", entity_node_id))?;

        if node.kind != "EntityAssembler" {
            return Err(format!("Node {} is not an EntityAssembler", entity_node_id));
        }

        let entity_data = self
            .entities
            .get(&entity_node_id)
            .cloned()
            .unwrap_or_default();

        let mut mesh_source = None;
        let mut material = None;

        // Iterate through all incoming connections to this node's input pins
        for &pin_id in &node.inputs {
            let conn_ids = graph.get_pin_connections(pin_id);
            for &conn_id in conn_ids {
                if let Some(conn) = graph.get_connection(conn_id) {
                    let (res_mesh, res_mat) = self.resolve_source_producer(conn.source_pin, graph);
                    if res_mesh.is_some() {
                        mesh_source = res_mesh;
                    }
                    if res_mat.is_some() {
                        material = res_mat;
                    }
                }
            }
        }

        Ok(AssembledSceneEntity {
            name: entity_data.name,
            translation: entity_data.translation,
            rotation: entity_data.rotation,
            scale: entity_data.scale,
            mesh_source,
            material,
        })
    }

    /// Evaluates all EntityAssembler nodes in topological dependency order across the graph.
    pub fn evaluate_all_entities(&self, graph: &GraphStore) -> Vec<AssembledSceneEntity> {
        let sorted_nodes = graph.topological_sort().unwrap_or_else(|_| {
            graph.nodes().keys().copied().collect()
        });

        let mut entities = Vec::new();
        for node_id in sorted_nodes {
            if let Some(node) = graph.get_node(node_id) {
                if node.kind == "EntityAssembler" {
                    if let Ok(assembled) = self.evaluate_entity(node_id, graph) {
                        entities.push(assembled);
                    }
                }
            }
        }
        entities
    }
}
