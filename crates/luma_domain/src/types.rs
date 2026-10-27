use bevy::math::{Quat, Vec3, Vec4};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetLoaderData {
    pub asset_path: String,
    pub is_loaded: bool,
}

impl Default for AssetLoaderData {
    fn default() -> Self {
        Self {
            asset_path: "models/character.glb".to_string(),
            is_loaded: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeshData {
    pub mesh_source: String,
    pub dimensions: Vec3,
    pub wireframe_enabled: bool,
}

impl Default for MeshData {
    fn default() -> Self {
        Self {
            mesh_source: "Cube".to_string(),
            dimensions: Vec3::ONE,
            wireframe_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaterialData {
    pub base_color: Vec4,
    pub perceptual_roughness: f32,
    pub metallic: f32,
    pub emissive: Vec4,
}

impl Default for MaterialData {
    fn default() -> Self {
        Self {
            base_color: Vec4::new(0.8, 0.8, 0.8, 1.0),
            perceptual_roughness: 0.5,
            metallic: 0.0,
            emissive: Vec4::ZERO,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntityAssemblerData {
    pub name: String,
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for EntityAssemblerData {
    fn default() -> Self {
        Self {
            name: "Entity".to_string(),
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}
