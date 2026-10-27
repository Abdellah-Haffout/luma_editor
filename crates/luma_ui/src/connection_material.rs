//! GPU SDF Connection Wire Material for Luma Editor.
//!
//! Pairs each active connection or dragging wire with a GPU material node
//! running `shaders/connection.wesl`. Computes exact Inigo Quilez signed distance
//! fields with anti-aliasing, two-pole color gradients, animated dataflow pulses,
//! and full support for all Luma wire routing styles (Bézier, Orthogonal, Circuit, Stepped, Straight).

use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::*;
use bevy::shader::ShaderRef;

pub const SHADER_CONNECTION_PATH: &str = "embedded://luma_ui/shaders/connection.wesl";

/// GPU material uniform for a single Luma connection wire.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct LumaConnectionMaterial {
    #[uniform(0)]
    pub p0: Vec2,
    #[uniform(0)]
    pub p1: Vec2,
    #[uniform(0)]
    pub p2: Vec2,
    #[uniform(0)]
    pub p3: Vec2,
    #[uniform(0)]
    pub src_color: Vec4,
    #[uniform(0)]
    pub tgt_color: Vec4,
    #[uniform(0)]
    pub width: f32,
    #[uniform(0)]
    pub feather: f32,
    #[uniform(0)]
    pub pulse_pos: f32,
    #[uniform(0)]
    pub pulse_enabled: f32,
    #[uniform(0)]
    pub dimmed: f32,
    #[uniform(0)]
    pub routing_mode: f32,
    #[uniform(0)]
    pub num_points: f32,
    #[uniform(0)]
    pub num_occluders: f32,
    #[uniform(0)]
    pub poly_points: [Vec4; 12],
    #[uniform(0)]
    pub occluders: [Vec4; 16],
}

impl Default for LumaConnectionMaterial {
    fn default() -> Self {
        Self {
            p0: Vec2::ZERO,
            p1: Vec2::ZERO,
            p2: Vec2::ZERO,
            p3: Vec2::ZERO,
            src_color: Vec4::new(0.6, 0.6, 0.7, 1.0),
            tgt_color: Vec4::new(0.6, 0.6, 0.7, 1.0),
            width: 2.2,
            feather: 1.0,
            pulse_pos: 0.0,
            pulse_enabled: 1.0,
            dimmed: 0.0,
            routing_mode: 0.0,
            num_points: 0.0,
            num_occluders: 0.0,
            poly_points: [Vec4::ZERO; 12],
            occluders: [Vec4::ZERO; 16],
        }
    }
}

impl UiMaterial for LumaConnectionMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_CONNECTION_PATH.into()
    }
}
