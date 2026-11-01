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

/// Physics simulation body types for rigid bodies.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PhysicsBodyType {
    #[default]
    Dynamic,
    Static,
    Kinematic,
}

/// Domain payload for physical rigid body and collision dynamics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhysicsData {
    pub body_type: PhysicsBodyType,
    pub mass: f32,
    pub friction: f32,
    pub restitution: f32,
    pub gravity_scale: f32,
    pub linear_damping: f32,
}

impl Default for PhysicsData {
    fn default() -> Self {
        Self {
            body_type: PhysicsBodyType::Dynamic,
            mass: 1.0,
            friction: 0.5,
            restitution: 0.7,
            gravity_scale: 1.0,
            linear_damping: 0.1,
        }
    }
}
