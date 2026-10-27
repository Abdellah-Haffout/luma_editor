use crate::transform::CanvasTransform;
use luma_core::graph::GraphStore;
use luma_domain::evaluation::DomainPayloadStore;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LumaGraphMeta {
    pub author: String,
    pub created_at_sec: u64,
    pub editor_version: String,
}

impl Default for LumaGraphMeta {
    fn default() -> Self {
        let created_at_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            author: "User".to_string(),
            created_at_sec,
            editor_version: "0.1.0".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LumaGraphFile {
    pub version: u32,
    pub meta: LumaGraphMeta,
    pub canvas: CanvasTransform,
    pub graph: GraphStore,
    pub domain_assets: Vec<(u64, luma_domain::types::AssetLoaderData)>,
    pub domain_meshes: Vec<(u64, luma_domain::types::MeshData)>,
    pub domain_materials: Vec<(u64, luma_domain::types::MaterialData)>,
    pub domain_entities: Vec<(u64, luma_domain::types::EntityAssemblerData)>,
}

impl LumaGraphFile {
    pub fn new(
        canvas: CanvasTransform,
        graph: GraphStore,
        payloads: &DomainPayloadStore,
    ) -> Self {
        let domain_assets = payloads
            .assets
            .iter()
            .map(|(k, v)| (k.0, v.clone()))
            .collect();
        let domain_meshes = payloads
            .meshes
            .iter()
            .map(|(k, v)| (k.0, v.clone()))
            .collect();
        let domain_materials = payloads
            .materials
            .iter()
            .map(|(k, v)| (k.0, v.clone()))
            .collect();
        let domain_entities = payloads
            .entities
            .iter()
            .map(|(k, v)| (k.0, v.clone()))
            .collect();

        Self {
            version: 1,
            meta: LumaGraphMeta::default(),
            canvas,
            graph,
            domain_assets,
            domain_meshes,
            domain_materials,
            domain_entities,
        }
    }

    pub fn to_domain_store(&self) -> DomainPayloadStore {
        let mut store = DomainPayloadStore::new();
        for (id, val) in &self.domain_assets {
            store.assets.insert(luma_core::id::NodeId(*id), val.clone());
        }
        for (id, val) in &self.domain_meshes {
            store.meshes.insert(luma_core::id::NodeId(*id), val.clone());
        }
        for (id, val) in &self.domain_materials {
            store.materials.insert(luma_core::id::NodeId(*id), val.clone());
        }
        for (id, val) in &self.domain_entities {
            store.entities.insert(luma_core::id::NodeId(*id), val.clone());
        }
        store
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let pretty = ron::ser::PrettyConfig::default();
        let serialized = ron::ser::to_string_pretty(self, pretty).map_err(|e| e.to_string())?;
        std::fs::write(path, serialized).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let mut file: Self = ron::from_str(&content).map_err(|e| e.to_string())?;
        file.graph.rebuild_indices();
        Ok(file)
    }
}
