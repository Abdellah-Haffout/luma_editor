//! Blender-Style Shading Workspace Example:
//!
//! Powered by `bevy_luma_widgets`:
//! - Pure functions as nodes via `#[luma_node]`.
//! - Interactive widgets powered by `bevy_luma_widgets` (DragValue, LumaSwatch, LumaBadge).
//! - Zero `.register_node(...)` calls in `main()`.
//! - Pure 2-panel Blender dock layout (3D Viewport on left, Shader Editor on right).
//!
//! Run with:
//! ```bash
//! cargo run --example blender_shading_workspace
//! ```

use std::sync::Arc;

use bevy::camera::visibility::RenderLayers;
use bevy::camera::RenderTarget;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy_luma::prelude::*;
use bevy_luma_widgets::prelude::*;
use jackdaw_feathers::{icons::Icon, tokens};
use jackdaw_panels::tree::{DockLeaf, DockNode, DockSplit, DockTree, SplitAxis};
use jackdaw_panels::{
    DockAreaStyle, DockTabContent, DockWindowDescriptor, LayoutState, WindowRegistry,
    WorkspaceDescriptor, WorkspaceRegistry,
};
use luma_canvas::plugin::{EditorCamera, LumaEditorPlugin};
use luma_core::id::NodeId;
use luma_dataflow::value::{LinearColor, PrimitiveValue, Value};
use luma_ui::{
    luma_body_node, luma_node, EditorTheme, EditorUiCamera, LumaGraph, LumaNode, LumaUiPlugin,
    NodeBodyContext, NodeStyle, PinInfo,
};

const SHADER_PREVIEW_WINDOW: &str = "shader_preview";
const SHADER_PREVIEW_LAYER: usize = 1;

// =============================================================================
// 1. Pure Function Nodes (Auto-Registered via #[luma_node])
// =============================================================================

/// Principled BSDF shader node implemented as a declarative Rust function.
/// All input pins and default parameters are declared directly in the signature.
#[luma_node(
    kind = "shader.principled_bsdf",
    category = "Shader",
    tag = "BSDF",
    title = "Principled BSDF",
    header_rgb = [0.18, 0.48, 0.36]
)]
pub fn principled_bsdf(
    #[default = LinearColor::new(1.00, 0.76, 0.33, 1.0)] base_color: LinearColor,
    #[default = 0.80] _metallic: f32,
    #[default = 0.25] _roughness: f32,
    #[default = 0.00] _emission: f32,
) -> LinearColor {
    base_color
}

/// Mix Color (MixRGB) node blending two linear colors via a scalar factor.
#[luma_node(
    kind = "shader.mix_color",
    category = "Color",
    tag = "MIX",
    title = "Mix Color",
    header_rgb = [0.46, 0.38, 0.18]
)]
pub fn mix_color(
    #[default = 0.5] factor: f32,
    #[default = LinearColor::new(1.00, 0.76, 0.33, 1.0)] a: LinearColor,
    #[default = LinearColor::new(0.18, 0.85, 0.98, 1.0)] b: LinearColor,
) -> LinearColor {
    let t = factor.clamp(0.0, 1.0);
    LinearColor::new(
        a.r * (1.0 - t) + b.r * t,
        a.g * (1.0 - t) + b.g * t,
        a.b * (1.0 - t) + b.b * t,
        1.0,
    )
}

/// Invert Color node reversing RGB channels.
#[luma_node(
    kind = "shader.invert_color",
    category = "Color",
    tag = "INV",
    title = "Invert Color",
    header_rgb = [0.46, 0.38, 0.18]
)]
pub fn invert_color(
    #[default = 1.0] factor: f32,
    #[default = LinearColor::new(1.0, 1.0, 1.0, 1.0)] color: LinearColor,
) -> LinearColor {
    let t = factor.clamp(0.0, 1.0);
    LinearColor::new(
        color.r * (1.0 - t) + (1.0 - color.r) * t,
        color.g * (1.0 - t) + (1.0 - color.g) * t,
        color.b * (1.0 - t) + (1.0 - color.b) * t,
        1.0,
    )
}

/// Color Ramp preset generating smooth gradients from a 0..1 scalar factor.
#[luma_node(
    kind = "shader.color_ramp",
    category = "Converter",
    tag = "RAMP",
    title = "Color Ramp",
    header_rgb = [0.24, 0.34, 0.46]
)]
pub fn color_ramp(#[default = 0.5] factor: f32) -> LinearColor {
    let t = factor.clamp(0.0, 1.0);
    if t < 0.33 {
        let f = t / 0.33;
        LinearColor::new(0.1 + f * 0.1, 0.2 + f * 0.6, 0.9 - f * 0.4, 1.0)
    } else if t < 0.66 {
        let f = (t - 0.33) / 0.33;
        LinearColor::new(0.2 + f * 0.8, 0.8 - f * 0.2, 0.5 - f * 0.4, 1.0)
    } else {
        let f = (t - 0.66) / 0.34;
        LinearColor::new(1.0, 0.6 - f * 0.4, 0.1, 1.0)
    }
}

/// Fresnel node computing viewing-angle reflection falloff using IOR.
#[luma_node(
    kind = "shader.fresnel",
    category = "Input",
    tag = "FRES",
    title = "Fresnel",
    header_rgb = [0.18, 0.36, 0.44]
)]
pub fn fresnel(#[default = 1.45] ior: f32) -> f32 {
    let r0 = ((1.0 - ior) / (1.0 + ior)).powi(2);
    (r0 + (1.0 - r0) * 0.65).clamp(0.0, 1.0)
}

/// Float Math Multiply node.
#[luma_node(
    kind = "shader.math_multiply",
    category = "Converter",
    tag = "MUL",
    title = "Math (Multiply)",
    header_rgb = [0.28, 0.32, 0.48]
)]
pub fn math_multiply(#[default = 1.0] a: f32, #[default = 2.0] b: f32) -> f32 {
    a * b
}

/// Float Math Add node.
#[luma_node(
    kind = "shader.math_add",
    category = "Converter",
    tag = "ADD",
    title = "Math (Add)",
    header_rgb = [0.28, 0.32, 0.48]
)]
pub fn math_add(#[default = 0.0] a: f32, #[default = 0.5] b: f32) -> f32 {
    a + b
}

/// Emission shader node multiplying color by luminous intensity.
#[luma_node(
    kind = "shader.emission",
    category = "Shader",
    tag = "EMIT",
    title = "Emission",
    header_rgb = [0.18, 0.48, 0.36]
)]
pub fn emission_shader(
    #[default = LinearColor::new(0.2, 0.85, 1.0, 1.0)] color: LinearColor,
    #[default = 3.0] strength: f32,
) -> LinearColor {
    let s = strength.max(0.0);
    LinearColor::new(color.r * s, color.g * s, color.b * s, 1.0)
}

/// Procedural Checker Texture pattern node.
#[luma_node(
    kind = "shader.checker_texture",
    category = "Texture",
    tag = "CHECK",
    title = "Checker Texture",
    header_rgb = [0.48, 0.28, 0.20]
)]
pub fn checker_texture(
    #[default = 4.0] scale: f32,
    #[default = LinearColor::new(0.90, 0.90, 0.92, 1.0)] color1: LinearColor,
    #[default = LinearColor::new(0.12, 0.14, 0.18, 1.0)] color2: LinearColor,
) -> LinearColor {
    if (scale as i32) % 2 == 0 {
        color1
    } else {
        color2
    }
}

// -----------------------------------------------------------------------------
// Physics Nodes (Auto-Registered via #[luma_node])
// -----------------------------------------------------------------------------

/// Physics Gravity Force node calculating weight F = m * g.
#[luma_node(
    kind = "physics.gravity_force",
    category = "Physics",
    tag = "GRAV",
    title = "Gravity Force",
    header_rgb = [0.26, 0.42, 0.58]
)]
pub fn gravity_force(
    #[default = 2.5] mass: f32,
    #[default = 9.81] gravity: f32,
) -> f32 {
    mass * gravity
}

/// Physics Kinetic Energy node computing E = 0.5 * m * v².
#[luma_node(
    kind = "physics.kinetic_energy",
    category = "Physics",
    tag = "ENGY",
    title = "Kinetic Energy",
    header_rgb = [0.42, 0.32, 0.56]
)]
pub fn kinetic_energy(
    #[default = 2.0] mass: f32,
    #[default = 4.0] velocity: f32,
) -> f32 {
    0.5 * mass * velocity.powi(2)
}

/// Physics Spring Force node implementing Hooke's Law with damping: F = -k*x - c*v.
#[luma_node(
    kind = "physics.spring_force",
    category = "Physics",
    tag = "SPRG",
    title = "Spring Force",
    header_rgb = [0.32, 0.46, 0.40]
)]
pub fn spring_force(
    #[default = 25.0] stiffness_k: f32,
    #[default = 0.4] displacement_x: f32,
    #[default = 1.2] damping_c: f32,
    #[default = 0.0] velocity: f32,
) -> f32 {
    -stiffness_k * displacement_x - damping_c * velocity
}

/// Physics Projectile Trajectory height y(t) = v₀*sin(θ)*t - 0.5*g*t².
#[luma_node(
    kind = "physics.projectile_trajectory",
    category = "Physics",
    tag = "TRAJ",
    title = "Projectile Motion",
    header_rgb = [0.40, 0.40, 0.54]
)]
pub fn projectile_trajectory(
    #[default = 12.0] initial_speed: f32,
    #[default = 45.0] angle_deg: f32,
    #[default = 0.5] time: f32,
    #[default = 9.81] gravity: f32,
) -> f32 {
    let theta = angle_deg.to_radians();
    let vy = initial_speed * theta.sin();
    (vy * time - 0.5 * gravity * time * time).max(0.0)
}

// =============================================================================
// 2. Interactive UI Body Nodes Powered by `bevy_luma_widgets`
// =============================================================================

#[derive(Component)]
struct BtnSetColor {
    node_id: NodeId,
    color: LinearColor,
}

#[derive(Component)]
struct WaveOscillatorMarker(NodeId);

#[derive(Component)]
struct ValueDragBinding(NodeId);

#[derive(Component)]
struct MetallicDragBinding(NodeId);

#[derive(Component)]
struct RoughnessDragBinding(NodeId);

#[derive(Component)]
struct EmissionDragBinding(NodeId);

#[derive(Component)]
#[allow(dead_code)]
struct PhysicsGravBinding(NodeId);

#[derive(Component)]
#[allow(dead_code)]
struct PhysicsBounceBinding(NodeId);

/// Interactive Physics Simulation node with live particle dynamics & DragValue scrubbers.
#[luma_body_node(system = sync_physics_simulation_system)]
pub struct PhysicsSimulationNode;

impl LumaNode for PhysicsSimulationNode {
    fn kind(&self) -> &'static str {
        "physics.simulation"
    }

    fn title(&self) -> &str {
        "Physics Simulation"
    }

    fn category(&self) -> (&'static str, &'static str) {
        ("Physics", "PHYS")
    }

    fn style(&self) -> NodeStyle {
        NodeStyle::header_rgb(0.22, 0.44, 0.60).width(240.0)
    }

    fn outputs(&self) -> Vec<PinInfo> {
        vec![
            PinInfo::float("Height"),
            PinInfo::float("Velocity"),
            PinInfo::float("Impact"),
        ]
    }

    fn body_height(&self) -> f32 {
        68.0
    }

    fn show_body(&self, ctx: &mut NodeBodyContext) {
        let node_id = ctx.node_id();

        // 1. Gravity DragValue Scrubber
        ctx.spawn_scene(
            DragValue::new(9.8)
                .range(0.0, 30.0)
                .speed(0.1)
                .step(0.5)
                .decimals(1)
                .prefix("Gravity: ")
                .suffix(" m/s²")
                .width(UVal::Percent(1.0))
                .font(ctx.font())
                .icon_font(ctx.icon_font()),
        )
        .insert(PhysicsGravBinding(node_id));

        // 2. Bounciness / Restitution DragValue Scrubber
        ctx.spawn_scene(
            DragValue::new(0.78)
                .range(0.0, 1.0)
                .speed(0.01)
                .step(0.05)
                .decimals(2)
                .prefix("Bounciness: ")
                .width(UVal::Percent(1.0))
                .font(ctx.font())
                .icon_font(ctx.icon_font()),
        )
        .insert(PhysicsBounceBinding(node_id));
    }
}

fn sync_physics_simulation_system(
    mut graph: LumaGraph,
    grav_q: Query<(&PhysicsGravBinding, &LumaDragValue), Changed<LumaDragValue>>,
    bounce_q: Query<(&PhysicsBounceBinding, &LumaDragValue), Changed<LumaDragValue>>,
    mut phys: ResMut<LivePhysicsState>,
) {
    for (_, dv) in &grav_q {
        phys.gravity = dv.value;
    }

    for (_, dv) in &bounce_q {
        phys.restitution = dv.value;
    }

    let phys_node_ids: Vec<NodeId> = graph
        .graph
        .0
        .nodes()
        .iter()
        .filter(|(_, node)| node.kind == "physics.simulation")
        .map(|(&node_id, _)| node_id)
        .collect();

    for node_id in phys_node_ids {
        let impact = (phys.last_impact / 10.0).clamp(0.0, 1.0);
        graph.set_output(node_id, 0, Value::float(phys.height.clamp(0.0, 5.0)));
        graph.set_output(node_id, 1, Value::float(phys.velocity.abs()));
        graph.set_output(node_id, 2, Value::float(impact));
    }
}

/// Interactive Float Value input node powered by `bevy_luma_widgets::DragValue`.
#[luma_body_node(system = sync_value_node_system)]
pub struct ValueNode;

impl LumaNode for ValueNode {
    fn kind(&self) -> &'static str {
        "shader.value"
    }

    fn title(&self) -> &str {
        "Value"
    }

    fn category(&self) -> (&'static str, &'static str) {
        ("Input", "VAL")
    }

    fn style(&self) -> NodeStyle {
        NodeStyle::header_rgb(0.18, 0.36, 0.44).width(230.0)
    }

    fn outputs(&self) -> Vec<PinInfo> {
        vec![PinInfo::float("Value")]
    }

    fn body_height(&self) -> f32 {
        34.0
    }

    fn show_body(&self, ctx: &mut NodeBodyContext) {
        let node_id = ctx.node_id();
        ctx.spawn_scene(
            DragValue::new(0.50)
                .speed(0.01)
                .step(0.05)
                .decimals(2)
                .prefix("Value: ")
                .width(UVal::Percent(1.0))
                .font(ctx.font())
                .icon_font(ctx.icon_font()),
        )
        .insert(ValueDragBinding(node_id));
    }
}

fn sync_value_node_system(
    mut graph: LumaGraph,
    query: Query<(&ValueDragBinding, &LumaDragValue), Changed<LumaDragValue>>,
) {
    for (binding, drag) in &query {
        graph.set_output(binding.0, 0, Value::float(drag.value));
    }
}

/// Interactive Color Palette node utilizing `LumaSwatch` from `bevy_luma_widgets`.
#[luma_body_node(system = sync_color_palette_system)]
pub struct ColorPaletteNode;

impl LumaNode for ColorPaletteNode {
    fn kind(&self) -> &'static str {
        "shader.color_palette"
    }

    fn title(&self) -> &str {
        "Color Palette"
    }

    fn category(&self) -> (&'static str, &'static str) {
        ("Input", "COLOR")
    }

    fn style(&self) -> NodeStyle {
        NodeStyle::header_rgb(0.62, 0.45, 0.16).width(230.0)
    }

    fn outputs(&self) -> Vec<PinInfo> {
        vec![PinInfo::color("Color")]
    }

    fn body_height(&self) -> f32 {
        34.0
    }

    fn show_body(&self, ctx: &mut NodeBodyContext) {
        let node_id = ctx.node_id();

        let row = ctx
            .spawn((
                UNode {
                    width: UVal::Percent(1.0),
                    height: UVal::Px(28.0),
                    padding: USides::axes(4.0, 2.0),
                    background_color: Color::srgb(0.08, 0.09, 0.12),
                    border_radius: UCornerRadius::all(6.0),
                    ..default()
                },
                ULayout {
                    display: UDisplay::Flex,
                    flex_direction: UFlexDirection::Row,
                    align_items: UAlignItems::Center,
                    justify_content: UJustifyContent::SpaceEvenly,
                    gap: 6.0,
                    ..default()
                },
            ))
            .id();

        let presets = [
            LinearColor::new(1.00, 0.76, 0.33, 1.0), // Gold
            LinearColor::new(0.18, 0.85, 0.98, 1.0), // Cyan
            LinearColor::new(0.95, 0.18, 0.30, 1.0), // Ruby
            LinearColor::new(0.15, 0.88, 0.48, 1.0), // Emerald
            LinearColor::new(0.88, 0.90, 0.95, 1.0), // Silk
        ];

        for col in presets {
            let be_color = Color::srgb(col.r, col.g, col.b);
            ctx.commands().spawn((
                ChildOf(row),
                BtnSetColor { node_id, color: col },
                LumaSwatch {
                    color: be_color,
                    selected: false,
                },
                UNode {
                    width: UVal::Px(28.0),
                    height: UVal::Px(22.0),
                    background_color: be_color,
                    border_radius: UCornerRadius::all(4.0),
                    ..default()
                },
                UBorder {
                    color: Color::srgba(1.0, 1.0, 1.0, 0.25),
                    width: 1.0,
                    radius: UCornerRadius::all(4.0),
                    offset: 0.0,
                },
                UInteraction::default(),
                UInteractionColors {
                    normal: be_color,
                    hovered: be_color.lighter(0.12),
                    pressed: be_color.darker(0.12),
                },
            ));
        }
    }
}

fn sync_color_palette_system(
    mut graph: LumaGraph,
    btn_query: Query<(&UInteraction, &BtnSetColor), Changed<UInteraction>>,
) {
    for (interaction, btn) in &btn_query {
        if matches!(*interaction, UInteraction::Pressed | UInteraction::Clicked) {
            graph.set_output_color(btn.node_id, 0, btn.color);
        }
    }
}

/// Interactive Metallic Value node using `bevy_luma_widgets::DragValue` numeric scrubber.
#[luma_body_node(system = sync_metallic_node_system)]
pub struct MetallicNode;

impl LumaNode for MetallicNode {
    fn kind(&self) -> &'static str {
        "shader.metallic_val"
    }

    fn title(&self) -> &str {
        "Metallic Value"
    }

    fn category(&self) -> (&'static str, &'static str) {
        ("Input", "VAL")
    }

    fn style(&self) -> NodeStyle {
        NodeStyle::header_rgb(0.24, 0.36, 0.48).width(230.0)
    }

    fn outputs(&self) -> Vec<PinInfo> {
        vec![PinInfo::float("Metallic")]
    }

    fn body_height(&self) -> f32 {
        34.0
    }

    fn show_body(&self, ctx: &mut NodeBodyContext) {
        let node_id = ctx.node_id();
        ctx.spawn_scene(
            DragValue::new(0.80)
                .range(0.0, 1.0)
                .speed(0.005)
                .step(0.05)
                .decimals(2)
                .prefix("Metallic: ")
                .width(UVal::Percent(1.0))
                .font(ctx.font())
                .icon_font(ctx.icon_font()),
        )
        .insert(MetallicDragBinding(node_id));
    }
}

fn sync_metallic_node_system(
    mut graph: LumaGraph,
    query: Query<(&MetallicDragBinding, &LumaDragValue), Changed<LumaDragValue>>,
) {
    for (binding, drag) in &query {
        graph.set_output(binding.0, 0, Value::float(drag.value));
    }
}

/// Interactive Roughness Value node using `bevy_luma_widgets::DragValue` numeric scrubber.
#[luma_body_node(system = sync_roughness_node_system)]
pub struct RoughnessNode;

impl LumaNode for RoughnessNode {
    fn kind(&self) -> &'static str {
        "shader.roughness_val"
    }

    fn title(&self) -> &str {
        "Roughness Value"
    }

    fn category(&self) -> (&'static str, &'static str) {
        ("Input", "VAL")
    }

    fn style(&self) -> NodeStyle {
        NodeStyle::header_rgb(0.24, 0.36, 0.48).width(230.0)
    }

    fn outputs(&self) -> Vec<PinInfo> {
        vec![PinInfo::float("Roughness")]
    }

    fn body_height(&self) -> f32 {
        34.0
    }

    fn show_body(&self, ctx: &mut NodeBodyContext) {
        let node_id = ctx.node_id();
        ctx.spawn_scene(
            DragValue::new(0.25)
                .range(0.0, 1.0)
                .speed(0.005)
                .step(0.05)
                .decimals(2)
                .prefix("Roughness: ")
                .width(UVal::Percent(1.0))
                .font(ctx.font())
                .icon_font(ctx.icon_font()),
        )
        .insert(RoughnessDragBinding(node_id));
    }
}

fn sync_roughness_node_system(
    mut graph: LumaGraph,
    query: Query<(&RoughnessDragBinding, &LumaDragValue), Changed<LumaDragValue>>,
) {
    for (binding, drag) in &query {
        graph.set_output(binding.0, 0, Value::float(drag.value));
    }
}

/// Harmonic Sine Wave Oscillator node.
#[luma_body_node(system = sync_wave_oscillator_system)]
pub struct WaveOscillatorNode;

impl LumaNode for WaveOscillatorNode {
    fn kind(&self) -> &'static str {
        "shader.wave_oscillator"
    }

    fn title(&self) -> &str {
        "Sine Oscillator"
    }

    fn category(&self) -> (&'static str, &'static str) {
        ("Math", "WAVE")
    }

    fn style(&self) -> NodeStyle {
        NodeStyle::header_rgb(0.34, 0.24, 0.54).width(200.0)
    }

    fn outputs(&self) -> Vec<PinInfo> {
        vec![PinInfo::float("Wave")]
    }

    fn body_height(&self) -> f32 {
        28.0
    }

    fn show_body(&self, ctx: &mut NodeBodyContext) {
        let node_id = ctx.node_id();
        let font = ctx.font();

        ctx.spawn((
            WaveOscillatorMarker(node_id),
            UNode {
                width: UVal::Percent(1.0),
                height: UVal::Px(24.0),
                padding: USides::axes(6.0, 2.0),
                background_color: Color::srgb(0.12, 0.10, 0.16),
                border_radius: UCornerRadius::all(4.0),
                ..default()
            },
            ULayout {
                display: UDisplay::Flex,
                align_items: UAlignItems::Center,
                ..default()
            },
        ))
        .with_children(|row| {
            row.spawn(UText {
                text: "~ Live Breathing Sine ~".into(),
                font_size: 9.0,
                font,
                color: Color::srgb(0.75, 0.65, 0.95),
                ..default()
            });
        });
    }
}

fn sync_wave_oscillator_system(
    mut graph: LumaGraph,
    time: Res<Time>,
    oscillators: Query<&WaveOscillatorMarker>,
    mut last_tick: Local<f32>,
) {
    if oscillators.is_empty() {
        return;
    }
    let t = time.elapsed_secs();
    if t - *last_tick < 0.033 {
        return;
    }
    *last_tick = t;

    let wave_val = (t * 2.0).sin() * 0.5 + 0.5;
    for osc in &oscillators {
        graph.set_output(osc.0, 0, Value::float(wave_val));
    }
}

/// Interactive Emission Strength node utilizing `bevy_luma_widgets::DragValue` numeric scrubber.
#[luma_body_node(system = sync_emission_node_system)]
pub struct EmissionNode;

impl LumaNode for EmissionNode {
    fn kind(&self) -> &'static str {
        "shader.emission_val"
    }

    fn title(&self) -> &str {
        "Emission Strength"
    }

    fn category(&self) -> (&'static str, &'static str) {
        ("Shader", "EMIT")
    }

    fn style(&self) -> NodeStyle {
        NodeStyle::header_rgb(0.55, 0.35, 0.15).width(240.0)
    }

    fn outputs(&self) -> Vec<PinInfo> {
        vec![PinInfo::float("emission")]
    }

    fn body_height(&self) -> f32 {
        34.0
    }

    fn show_body(&self, ctx: &mut NodeBodyContext) {
        let node_id = ctx.node_id();
        ctx.spawn_scene(
            DragValue::new(0.0)
                .min(0.0)
                .max(50.0)
                .speed(0.1)
                .step(0.5)
                .decimals(1)
                .prefix("Emission: ")
                .width(UVal::Percent(1.0))
                .font(ctx.font())
                .icon_font(ctx.icon_font()),
        )
        .insert(EmissionDragBinding(node_id));
    }
}

fn sync_emission_node_system(
    mut graph: LumaGraph,
    query: Query<(&EmissionDragBinding, &LumaDragValue), Changed<LumaDragValue>>,
) {
    for (binding, drag) in &query {
        graph.set_output(binding.0, 0, Value::float(drag.value));
    }
}

// =============================================================================
// 3. Resources & Viewport Components
// =============================================================================

const PHYSICS_WINDOW: &str = "physics_panel";

#[derive(Component)]
struct PreviewSphere;

#[derive(Component)]
struct ShaderPreviewRoot;

#[derive(Component)]
struct PreviewStatsText;

#[derive(Component)]
struct PhysicsWindowRoot;

#[derive(Component)]
struct PhysTelemetryHeight;

#[derive(Component)]
struct PhysTelemetryVelocity;

#[derive(Component)]
struct PhysTelemetryEnergy;

#[derive(Component)]
struct PhysTelemetryBounces;

#[derive(Component)]
struct PhysBadgeText;

#[derive(Component)]
struct PhysStatusBadge;

#[derive(Component)]
struct PhysMeterBar;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum PhysButtonAction {
    Drop,
    SuperBounce,
    HeavyIron,
    TogglePause,
}

#[derive(Resource)]
pub struct LivePhysicsState {
    pub height: f32,
    pub velocity: f32,
    pub gravity: f32,
    pub restitution: f32,
    pub mass: f32,
    pub bounce_count: u32,
    pub last_impact: f32,
    pub paused: bool,
    pub settle_timer: f32,
}

impl Default for LivePhysicsState {
    fn default() -> Self {
        Self {
            height: 1.8,
            velocity: 0.0,
            gravity: 9.81,
            restitution: 0.78,
            mass: 1.2,
            bounce_count: 0,
            last_impact: 0.0,
            paused: false,
            settle_timer: 0.0,
        }
    }
}

#[derive(Resource)]
struct ShaderPreviewTexture {
    handle: Handle<Image>,
}

#[derive(Resource)]
struct ShaderLiveMaterial {
    material_handle: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
struct ShaderPreviewViewportRect(pub Option<Rect>);

#[derive(Resource)]
struct PreviewMeshes {
    sphere: Handle<Mesh>,
    torus: Handle<Mesh>,
    cuboid: Handle<Mesh>,
    current_index: usize,
}

// =============================================================================
// 4. Main Entry Point
// =============================================================================

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Luma Editor — Blender Shading & Physics Workspace".into(),
                resolution: (1440, 880).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(EditorTheme::BG_CANVAS))
        .init_resource::<ShaderPreviewViewportRect>()
        .init_resource::<LivePhysicsState>()
        // Auto-discovers all #[luma_node] and #[luma_body_node] from inventory!
        .add_plugins((LumaPlugin, LumaWidgetsPlugin, LumaEditorPlugin, LumaUiPlugin))
        .add_systems(Startup, (setup_3d_preview_scene, setup_pipeline).chain())
        .add_systems(
            PostStartup,
            setup_blender_shading_workspace.after(luma_ui::attach_overlay_dock_anchors),
        )
        .add_systems(
            Update,
            (
                sync_shader_to_preview_material,
                update_live_physics_system.before(orbit_preview_object),
                orbit_preview_object,
                toggle_preview_mesh,
                update_physics_window_telemetry_system,
                handle_physics_action_buttons_system,
            ),
        )
        .add_systems(
            PostUpdate,
            (bind_world_ui_camera, update_preview_viewport_rect),
        )
        .run();
}

fn bind_world_ui_camera(
    editor_camera: Query<Entity, With<EditorCamera>>,
    mut roots: Query<&mut URootUi>,
) {
    if let Some(cam) = editor_camera.iter().next() {
        for mut root in &mut roots {
            if root.camera == UiCameraRef::Auto {
                root.camera = UiCameraRef::Entity(cam);
            }
        }
    }
}

// =============================================================================
// 5. 3D Preview Scene Setup (Offscreen Texture on Layer 1)
// =============================================================================

fn setup_3d_preview_scene(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let image = Image::new_target_texture(
        1024,
        1024,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    );
    let image_handle = images.add(image);
    commands.insert_resource(ShaderPreviewTexture {
        handle: image_handle.clone(),
    });

    let material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(1.00, 0.76, 0.33),
        metallic: 0.80,
        perceptual_roughness: 0.25,
        reflectance: 0.50,
        ..default()
    });
    commands.insert_resource(ShaderLiveMaterial {
        material_handle: material_handle.clone(),
    });

    // Small Sphere mesh (replacing the previous 1.25 radius large sphere)
    let sphere = meshes.add(Sphere::new(0.38).mesh().uv(36, 18));
    let torus = meshes.add(Torus::new(0.14, 0.38));
    let cuboid = meshes.add(Cuboid::new(0.65, 0.65, 0.65));

    commands.insert_resource(PreviewMeshes {
        sphere: sphere.clone(),
        torus,
        cuboid,
        current_index: 0,
    });

    let preview_layer = RenderLayers::layer(SHADER_PREVIEW_LAYER);

    // Contact Pedestal for physical bouncing
    let pedestal_mesh = meshes.add(Cylinder::new(1.6, 0.06).mesh());
    let pedestal_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.13, 0.16),
        metallic: 0.15,
        perceptual_roughness: 0.85,
        reflectance: 0.20,
        ..default()
    });
    commands.spawn((
        Mesh3d(pedestal_mesh),
        MeshMaterial3d(pedestal_mat),
        Transform::from_xyz(0.0, -0.65, 0.0),
        preview_layer.clone(),
    ));

    // Small Sphere with dynamic height
    commands.spawn((
        Mesh3d(sphere),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(0.0, -0.24 + 1.8, 0.0),
        PreviewSphere,
        preview_layer.clone(),
    ));

    // Studio 3-Point Lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 14_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(4.5, 7.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        preview_layer.clone(),
    ));

    commands.spawn((
        PointLight {
            intensity: 450_000.0,
            range: 20.0,
            color: Color::srgb(0.70, 0.85, 1.00),
            ..default()
        },
        Transform::from_xyz(-5.0, 2.5, -2.0),
        preview_layer.clone(),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 6_000.0,
            shadow_maps_enabled: false,
            color: Color::srgb(1.0, 0.95, 0.85),
            ..default()
        },
        Transform::from_xyz(-3.0, 4.0, -4.0).looking_at(Vec3::ZERO, Vec3::Y),
        preview_layer.clone(),
    ));

    // 3D Camera rendering into offscreen texture looking at small sphere + pedestal
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: -1,
            clear_color: Color::srgb(0.10, 0.11, 0.13).into(),
            ..default()
        },
        RenderTarget::Image(image_handle.into()),
        Transform::from_xyz(0.0, 1.1, 3.2).looking_at(Vec3::new(0.0, 0.15, 0.0), Vec3::Y),
        preview_layer,
    ));
}

// =============================================================================
// 6. 2D Node Pipeline Setup
// =============================================================================

fn setup_pipeline(mut commands: Commands, mut graph: LumaGraph) {
    commands.spawn((
        Camera2d,
        EditorCamera,
        EditorUiCamera,
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Left Column: Generators & Inputs
    let n_palette = graph.spawn("shader.color_palette", Vec2::new(-660.0, 200.0));
    let n_invert = graph.spawn("shader.invert_color", Vec2::new(-660.0, 30.0));
    let _n_wave = graph.spawn("shader.wave_oscillator", Vec2::new(-660.0, -160.0));
    let _n_val = graph.spawn("shader.value", Vec2::new(-660.0, -300.0));
    let _n_grav = graph.spawn("physics.gravity_force", Vec2::new(-660.0, -440.0));

    // Middle Column: Shading Math, Physics & DragValue Numeric Scrubbers
    let n_mix = graph.spawn("shader.mix_color", Vec2::new(-350.0, 160.0));
    let n_metallic = graph.spawn("shader.metallic_val", Vec2::new(-350.0, -10.0));
    let n_roughness = graph.spawn("shader.roughness_val", Vec2::new(-350.0, -160.0));
    let n_emission = graph.spawn("shader.emission_val", Vec2::new(-350.0, -310.0));
    let _n_phys = graph.spawn("physics.simulation", Vec2::new(-350.0, -490.0));

    // Right: BSDF Output
    let n_bsdf = graph.spawn("shader.principled_bsdf", Vec2::new(140.0, -20.0));

    // Wiring the Shader Graph (Palette, Metallic, Roughness, Emission -> BSDF)
    graph.connect(n_palette, 0, n_bsdf, 0);   // Base Color
    graph.connect(n_metallic, 0, n_bsdf, 1);  // Metallic DragValue -> Principled BSDF
    graph.connect(n_roughness, 0, n_bsdf, 2); // Roughness DragValue -> Principled BSDF
    graph.connect(n_emission, 0, n_bsdf, 3);  // Emission DragValue -> Principled BSDF

    // Pre-wire Mix & Invert so user can easily switch or connect
    graph.connect(n_palette, 0, n_invert, 1); // Palette feeds invert
    graph.connect(n_palette, 0, n_mix, 1);    // Palette feeds Mix A
    graph.connect(n_invert, 0, n_mix, 2);     // Inverted color feeds Mix B

    // Default output values
    graph.set_output_color(n_palette, 0, LinearColor::new(1.00, 0.76, 0.33, 1.0));
    graph.set_output(_n_val, 0, Value::float(0.50));
    graph.set_output(n_metallic, 0, Value::float(0.80));
    graph.set_output(n_roughness, 0, Value::float(0.25));
    graph.set_output(n_emission, 0, Value::float(0.00));

    graph.select(n_bsdf);
}

// =============================================================================
// 7. Dock Layout: Pure 2-Tab Blender Workspace
// =============================================================================

fn setup_blender_shading_workspace(
    mut registry: ResMut<WindowRegistry>,
    mut workspace_registry: ResMut<WorkspaceRegistry>,
    mut dock_tree: ResMut<DockTree>,
    preview_texture: Res<ShaderPreviewTexture>,
) {
    registry.unregister(luma_ui::window_ids::OUTLINER);
    registry.unregister(luma_ui::window_ids::LIBRARY);
    registry.unregister(luma_ui::window_ids::INSPECTOR);

    // Tab 1: 3D Viewport (Shader Preview)
    let preview_handle = preview_texture.handle.clone();
    registry.register(DockWindowDescriptor {
        id: SHADER_PREVIEW_WINDOW.into(),
        name: "3D Viewport".into(),
        icon: Some(String::from(Icon::Box.unicode())),
        default_area: "preview_area".into(),
        priority: 0,
        build: Arc::new(move |parent| {
            parent
                .spawn((
                    ShaderPreviewRoot,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.08, 0.09, 0.11)),
                ))
                .with_children(|vp| {
                    vp.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(28.0),
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(0.0)),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            border: UiRect::bottom(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.06, 0.07, 0.09, 0.95)),
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.06)),
                    ))
                    .with_children(|bar| {
                        bar.spawn((
                            Text::new("3D Viewport • Principled BSDF Preview [M: Switch Mesh | Space: Drop Ball]"),
                            TextFont {
                                font_size: tokens::TEXT_SIZE_SM,
                                ..default()
                            },
                            TextColor(tokens::TEXT_PRIMARY),
                        ));

                        bar.spawn((
                            PreviewStatsText,
                            Text::new("Base Color: Gold | Met: 0.80 | Rough: 0.25 | Small Sphere (r=0.38)"),
                            TextFont {
                                font_size: tokens::TEXT_SIZE_XS,
                                ..default()
                            },
                            TextColor(Color::srgb(0.70, 0.75, 0.85)),
                        ));
                    });

                    vp.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        ImageNode {
                            image: preview_handle.clone(),
                            ..default()
                        },
                    ));
                });
        }),
    });

    // Tab 2: Physics Simulation & Live Telemetry Window
    registry.register(DockWindowDescriptor {
        id: PHYSICS_WINDOW.into(),
        name: "Physics".into(),
        icon: Some(String::from(Icon::Activity.unicode())),
        default_area: "physics_area".into(),
        priority: 1,
        build: Arc::new(|parent| {
            parent
                .spawn((
                    PhysicsWindowRoot,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(10.0)),
                        row_gap: Val::Px(8.0),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.09, 0.10, 0.12)),
                ))
                .with_children(|root| {
                    // Header Bar with status badge
                    root.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::bottom(Val::Px(6.0)),
                            border: UiRect::bottom(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.06)),
                    ))
                    .with_children(|hdr| {
                        hdr.spawn((
                            Text::new("Rigid Body • Small Sphere Dynamics"),
                            TextFont {
                                font_size: tokens::TEXT_SIZE_SM,
                                ..default()
                            },
                            TextColor(Color::srgb(0.88, 0.90, 0.96)),
                        ));

                        hdr.spawn((
                            PhysStatusBadge,
                            Node {
                                padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.20, 0.70, 0.40, 0.20)),
                            BorderColor::all(Color::srgba(0.20, 0.80, 0.45, 0.40)),
                        ))
                        .with_children(|badge| {
                            badge.spawn((
                                PhysBadgeText,
                                Text::new("● RUNNING"),
                                TextFont {
                                    font_size: tokens::TEXT_SIZE_XS,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.35, 0.90, 0.55)),
                            ));
                        });
                    });

                    // Row 1: Height & Velocity
                    root.spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        // Card 1: Height
                        row.spawn((
                            Node {
                                flex_grow: 1.0,
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(6.0)),
                                border_radius: BorderRadius::all(Val::Px(6.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                row_gap: Val::Px(2.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.12, 0.13, 0.16)),
                            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                        ))
                        .with_children(|card| {
                            card.spawn((
                                Text::new("Height (Y)"),
                                TextFont {
                                    font_size: tokens::TEXT_SIZE_XS,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.60, 0.65, 0.72)),
                            ));
                            card.spawn((
                                PhysTelemetryHeight,
                                Text::new("1.80 m"),
                                TextFont {
                                    font_size: tokens::TEXT_SIZE_SM,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.35, 0.75, 1.0)),
                            ));
                        });

                        // Card 2: Velocity
                        row.spawn((
                            Node {
                                flex_grow: 1.0,
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(6.0)),
                                border_radius: BorderRadius::all(Val::Px(6.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                row_gap: Val::Px(2.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.12, 0.13, 0.16)),
                            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                        ))
                        .with_children(|card| {
                            card.spawn((
                                Text::new("Velocity (Vy)"),
                                TextFont {
                                    font_size: tokens::TEXT_SIZE_XS,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.60, 0.65, 0.72)),
                            ));
                            card.spawn((
                                PhysTelemetryVelocity,
                                Text::new("0.00 m/s"),
                                TextFont {
                                    font_size: tokens::TEXT_SIZE_SM,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.95, 0.65, 0.25)),
                            ));
                        });
                    });

                    // Row 2: Kinetic Energy & Bounces
                    root.spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        // Card 3: Kinetic Energy
                        row.spawn((
                            Node {
                                flex_grow: 1.0,
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(6.0)),
                                border_radius: BorderRadius::all(Val::Px(6.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                row_gap: Val::Px(2.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.12, 0.13, 0.16)),
                            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                        ))
                        .with_children(|card| {
                            card.spawn((
                                Text::new("Kinetic Energy"),
                                TextFont {
                                    font_size: tokens::TEXT_SIZE_XS,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.60, 0.65, 0.72)),
                            ));
                            card.spawn((
                                PhysTelemetryEnergy,
                                Text::new("0.00 J"),
                                TextFont {
                                    font_size: tokens::TEXT_SIZE_SM,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.85, 0.45, 0.95)),
                            ));
                        });

                        // Card 4: Bounces
                        row.spawn((
                            Node {
                                flex_grow: 1.0,
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(6.0)),
                                border_radius: BorderRadius::all(Val::Px(6.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                row_gap: Val::Px(2.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.12, 0.13, 0.16)),
                            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.05)),
                        ))
                        .with_children(|card| {
                            card.spawn((
                                Text::new("Bounces"),
                                TextFont {
                                    font_size: tokens::TEXT_SIZE_XS,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.60, 0.65, 0.72)),
                            ));
                            card.spawn((
                                PhysTelemetryBounces,
                                Text::new("0 bounces"),
                                TextFont {
                                    font_size: tokens::TEXT_SIZE_SM,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.40, 0.85, 0.65)),
                            ));
                        });
                    });

                    // Height Gauge Progress Bar
                    root.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.14, 0.15, 0.18, 0.8)),
                    ))
                    .with_children(|gauge| {
                        gauge.spawn(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Text::new("Sphere Elevation Gauge"),
                                TextFont {
                                    font_size: tokens::TEXT_SIZE_XS,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.65, 0.70, 0.80)),
                            ));
                            row.spawn((
                                Text::new("Max: 3.0 m"),
                                TextFont {
                                    font_size: tokens::TEXT_SIZE_XS,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.50, 0.55, 0.65)),
                            ));
                        });

                        gauge.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(6.0),
                                border_radius: BorderRadius::all(Val::Px(3.0)),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.08, 0.09, 0.11)),
                        ))
                        .with_children(|bar_bg| {
                            bar_bg.spawn((
                                PhysMeterBar,
                                Node {
                                    width: Val::Percent(60.0),
                                    height: Val::Percent(100.0),
                                    border_radius: BorderRadius::all(Val::Px(3.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.30, 0.75, 1.0)),
                            ));
                        });
                    });

                    // Action Buttons Row
                    root.spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|btns| {
                        // Drop Button
                        btns.spawn((
                            Interaction::default(),
                            PhysButtonAction::Drop,
                            Node {
                                flex_grow: 1.0,
                                height: Val::Px(26.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                padding: UiRect::axes(Val::Px(4.0), Val::Px(0.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.18, 0.28, 0.42)),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("Drop Ball"),
                                TextFont {
                                    font_size: tokens::TEXT_SIZE_XS,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.90, 0.92, 0.96)),
                            ));
                        });

                        // Super Bouncy Button
                        btns.spawn((
                            Interaction::default(),
                            PhysButtonAction::SuperBounce,
                            Node {
                                flex_grow: 1.0,
                                height: Val::Px(26.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                padding: UiRect::axes(Val::Px(4.0), Val::Px(0.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.35, 0.22, 0.42)),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("Bouncy (0.9)"),
                                TextFont {
                                    font_size: tokens::TEXT_SIZE_XS,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.90, 0.92, 0.96)),
                            ));
                        });

                        // Heavy Iron Button
                        btns.spawn((
                            Interaction::default(),
                            PhysButtonAction::HeavyIron,
                            Node {
                                flex_grow: 1.0,
                                height: Val::Px(26.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                padding: UiRect::axes(Val::Px(4.0), Val::Px(0.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.35, 0.28, 0.16)),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("Heavy Iron"),
                                TextFont {
                                    font_size: tokens::TEXT_SIZE_XS,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.90, 0.92, 0.96)),
                            ));
                        });

                        // Pause / Play Button
                        btns.spawn((
                            Interaction::default(),
                            PhysButtonAction::TogglePause,
                            Node {
                                flex_grow: 1.0,
                                height: Val::Px(26.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                padding: UiRect::axes(Val::Px(4.0), Val::Px(0.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.20, 0.23, 0.28)),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("Pause / Play"),
                                TextFont {
                                    font_size: tokens::TEXT_SIZE_XS,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.90, 0.92, 0.96)),
                            ));
                        });
                    });

                    // Bottom info label
                    root.spawn((
                        Text::new("Earth Gravity (g=9.81 m/s²) • Small Sphere (r=0.38m) • Contact Floor (Y=-0.65)"),
                        TextFont {
                            font_size: tokens::TEXT_SIZE_XS,
                            ..default()
                        },
                        TextColor(Color::srgb(0.48, 0.52, 0.60)),
                    ));
                });
        }),
    });

    // Tab 3: Shader Node Editor
    registry.unregister(luma_ui::window_ids::NODE_EDITOR);
    registry.register(DockWindowDescriptor {
        id: luma_ui::window_ids::NODE_EDITOR.into(),
        name: "Shader Editor".into(),
        icon: Some(String::from(Icon::Workflow.unicode())),
        default_area: "editor_area".into(),
        priority: 0,
        build: Arc::new(|parent| {
            parent.spawn((
                luma_ui::NodeEditorViewport,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                Pickable::IGNORE,
            ));
        }),
    });

    // 3-Panel Blender Layout:
    // Left column: 3D Viewport on top (55%), Physics Panel on bottom (45%)
    // Full Workspace: Left column (38%) and Shader Editor (62%)
    let mut tree = DockTree::new();
    let preview_leaf = tree.insert(DockNode::Leaf(
        DockLeaf::new("preview_area", DockAreaStyle::TabBar)
            .with_windows(vec![SHADER_PREVIEW_WINDOW.into()])
            .persistent(),
    ));
    let physics_leaf = tree.insert(DockNode::Leaf(
        DockLeaf::new("physics_area", DockAreaStyle::TabBar)
            .with_windows(vec![PHYSICS_WINDOW.into()])
            .persistent(),
    ));
    let left_column = tree.insert(DockNode::Split(DockSplit {
        axis: SplitAxis::Vertical,
        fraction: 0.55,
        a: preview_leaf,
        b: physics_leaf,
    }));

    let editor_leaf = tree.insert(DockNode::Leaf(
        DockLeaf::new("editor_area", DockAreaStyle::TabBar)
            .with_windows(vec![luma_ui::window_ids::NODE_EDITOR.into()])
            .persistent(),
    ));

    let root = tree.insert(DockNode::Split(DockSplit {
        axis: SplitAxis::Horizontal,
        fraction: 0.38,
        a: left_column,
        b: editor_leaf,
    }));
    tree.root = Some(root);
    *dock_tree = tree.clone();

    // Single "Shading & Physics" Workspace
    workspace_registry.workspaces.clear();
    workspace_registry.register(WorkspaceDescriptor {
        id: "shading".into(),
        name: "Shading & Physics".into(),
        icon: Some(String::from(Icon::Sparkles.unicode())),
        accent_color: Color::srgb(0.95, 0.65, 0.20),
        layout: LayoutState::default(),
        tree,
    });
    workspace_registry.active = Some("shading".into());
}

// =============================================================================
// 8. Viewport & Material Sync Systems
// =============================================================================

fn update_preview_viewport_rect(
    contents: Query<(&DockTabContent, &Node, &ComputedNode, &UiGlobalTransform, &InheritedVisibility)>,
    mut rect: ResMut<ShaderPreviewViewportRect>,
) {
    let new = contents
        .iter()
        .filter(|(content, node, computed, _, visibility)| {
            content.window_id == SHADER_PREVIEW_WINDOW
                && node.display != Display::None
                && visibility.get()
                && computed.size().min_element() > 1.0
        })
        .map(|(_, _, computed, transform, _)| {
            let inv = computed.inverse_scale_factor();
            let size = computed.size() * inv;
            let (_scale, _angle, center) = transform.to_scale_angle_translation();
            Rect::from_center_size(center * inv, size)
        })
        .next();

    if rect.0 != new {
        rect.0 = new;
    }
}

fn orbit_preview_object(
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut sphere_query: Query<&mut Transform, With<PreviewSphere>>,
    preview_rect: Res<ShaderPreviewViewportRect>,
    windows: Query<&Window>,
) {
    let delta = mouse_motion.delta;

    let is_mouse_over_preview = windows
        .iter()
        .next()
        .and_then(|w| w.cursor_position())
        .is_some_and(|pos| preview_rect.0.is_some_and(|r| r.contains(pos)));

    let is_dragging = (mouse_button.pressed(MouseButton::Left)
        || mouse_button.pressed(MouseButton::Middle)
        || mouse_button.pressed(MouseButton::Right))
        && is_mouse_over_preview;

    for mut transform in &mut sphere_query {
        if is_dragging && delta.length_squared() > 0.001 {
            transform.rotate_y(delta.x * 0.008);
            transform.rotate_x(delta.y * 0.008);
        } else {
            transform.rotate_y(0.30 * time.delta_secs());
        }
    }
}

fn toggle_preview_mesh(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut meshes_res: ResMut<PreviewMeshes>,
    mut sphere_query: Query<&mut Mesh3d, With<PreviewSphere>>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        meshes_res.current_index = (meshes_res.current_index + 1) % 3;
        let next_handle = match meshes_res.current_index {
            0 => meshes_res.sphere.clone(),
            1 => meshes_res.torus.clone(),
            _ => meshes_res.cuboid.clone(),
        };
        for mut mesh3d in &mut sphere_query {
            mesh3d.0 = next_handle.clone();
        }
    }
}

fn sync_shader_to_preview_material(
    graph: LumaGraph,
    mut materials: ResMut<Assets<StandardMaterial>>,
    live_mat: Res<ShaderLiveMaterial>,
    mut stats_text: Query<&mut Text, With<PreviewStatsText>>,
    mut last_state: Local<Option<([u8; 3], u32, u32, u32)>>,
) {
    for (&node_id, node) in graph.graph.0.nodes() {
        if node.kind == "shader.principled_bsdf" {
            let base_color = graph
                .input_color(node_id, 0)
                .unwrap_or(LinearColor::new(1.00, 0.76, 0.33, 1.0));

            let metallic = match graph.input_value(node_id, 1) {
                Value::Primitive(PrimitiveValue::Float(f)) => f.clamp(0.0, 1.0),
                _ => 0.80,
            };

            let roughness = match graph.input_value(node_id, 2) {
                Value::Primitive(PrimitiveValue::Float(f)) => f.clamp(0.04, 1.0),
                _ => 0.25,
            };

            let emission = match graph.input_value(node_id, 3) {
                Value::Primitive(PrimitiveValue::Float(f)) => f.max(0.0),
                _ => 0.0,
            };

            let state_key = (
                [
                    (base_color.r * 255.0) as u8,
                    (base_color.g * 255.0) as u8,
                    (base_color.b * 255.0) as u8,
                ],
                (metallic * 1000.0) as u32,
                (roughness * 1000.0) as u32,
                (emission * 1000.0) as u32,
            );

            if *last_state == Some(state_key) {
                continue;
            }
            *last_state = Some(state_key);

            if let Some(mut mat) = materials.get_mut(&live_mat.material_handle) {
                mat.base_color = Color::srgb(base_color.r, base_color.g, base_color.b);
                mat.metallic = metallic;
                mat.perceptual_roughness = roughness;
                if emission > 0.02 {
                    mat.emissive = LinearRgba::rgb(
                        base_color.r * emission * 3.0,
                        base_color.g * emission * 3.0,
                        base_color.b * emission * 3.0,
                    );
                } else {
                    mat.emissive = LinearRgba::BLACK;
                }
            }

            for mut text in &mut stats_text {
                **text = format!(
                    "RGB: ({:.2}, {:.2}, {:.2}) | Met: {:.2} | Rough: {:.2} | Emit: {:.2}",
                    base_color.r, base_color.g, base_color.b, metallic, roughness, emission
                );
            }
        }
    }
}

fn update_live_physics_system(
    time: Res<Time>,
    mut phys: ResMut<LivePhysicsState>,
    mut sphere_query: Query<&mut Transform, With<PreviewSphere>>,
) {
    if !phys.paused {
        let dt = time.delta_secs().min(0.05);
        phys.velocity -= phys.gravity * dt;
        phys.height += phys.velocity * dt;

        if phys.height <= 0.0 {
            phys.height = 0.0;
            let impact = phys.velocity.abs();
            phys.last_impact = impact;
            if impact > 0.15 {
                phys.velocity = -phys.velocity * phys.restitution;
                phys.bounce_count += 1;
            } else {
                phys.velocity = 0.0;
                phys.settle_timer += dt;
                // Auto-relaunch ball after settling for 2 seconds
                if phys.settle_timer > 2.0 {
                    phys.height = 1.8;
                    phys.velocity = 0.0;
                    phys.settle_timer = 0.0;
                    phys.bounce_count = 0;
                }
            }
        }
    }

    for mut transform in &mut sphere_query {
        transform.translation.y = -0.24 + phys.height;
    }
}

fn handle_physics_action_buttons_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut interaction_query: Query<
        (&Interaction, &PhysButtonAction, &mut BackgroundColor),
        (Changed<Interaction>, With<PhysButtonAction>),
    >,
    mut phys: ResMut<LivePhysicsState>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        phys.height = 2.4;
        phys.velocity = 0.0;
        phys.bounce_count = 0;
        phys.settle_timer = 0.0;
        phys.paused = false;
    }

    for (interaction, action, mut bg) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                match action {
                    PhysButtonAction::Drop => {
                        phys.height = 2.5;
                        phys.velocity = 0.0;
                        phys.bounce_count = 0;
                        phys.settle_timer = 0.0;
                        phys.paused = false;
                    }
                    PhysButtonAction::SuperBounce => {
                        phys.height = 2.4;
                        phys.velocity = 0.0;
                        phys.restitution = 0.90;
                        phys.bounce_count = 0;
                        phys.settle_timer = 0.0;
                        phys.paused = false;
                    }
                    PhysButtonAction::HeavyIron => {
                        phys.height = 2.2;
                        phys.velocity = 0.0;
                        phys.mass = 5.0;
                        phys.restitution = 0.35;
                        phys.bounce_count = 0;
                        phys.settle_timer = 0.0;
                        phys.paused = false;
                    }
                    PhysButtonAction::TogglePause => {
                        phys.paused = !phys.paused;
                    }
                }
                *bg = BackgroundColor(Color::srgb(0.50, 0.65, 0.85));
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgb(0.28, 0.35, 0.46));
            }
            Interaction::None => {
                *bg = BackgroundColor(match action {
                    PhysButtonAction::Drop => Color::srgb(0.18, 0.28, 0.42),
                    PhysButtonAction::SuperBounce => Color::srgb(0.35, 0.22, 0.42),
                    PhysButtonAction::HeavyIron => Color::srgb(0.35, 0.28, 0.16),
                    PhysButtonAction::TogglePause => Color::srgb(0.20, 0.23, 0.28),
                });
            }
        }
    }
}

fn update_physics_window_telemetry_system(
    phys: Res<LivePhysicsState>,
    mut texts: ParamSet<(
        Query<&mut Text, With<PhysTelemetryHeight>>,
        Query<&mut Text, With<PhysTelemetryVelocity>>,
        Query<&mut Text, With<PhysTelemetryEnergy>>,
        Query<&mut Text, With<PhysTelemetryBounces>>,
        Query<&mut Text, With<PhysBadgeText>>,
    )>,
    mut q_meter: Query<&mut Node, With<PhysMeterBar>>,
    mut q_badge: Query<&mut BackgroundColor, With<PhysStatusBadge>>,
) {
    for mut text in &mut texts.p0() {
        **text = format!("{:.2} m", phys.height);
    }
    for mut text in &mut texts.p1() {
        **text = format!("{:.2} m/s", phys.velocity);
    }
    for mut text in &mut texts.p2() {
        let ke = 0.5 * phys.mass * phys.velocity.powi(2);
        **text = format!("{:.2} J", ke);
    }
    for mut text in &mut texts.p3() {
        **text = format!("{} bounces", phys.bounce_count);
    }
    for mut text in &mut texts.p4() {
        **text = if phys.paused {
            "❚❚ PAUSED".into()
        } else {
            "● RUNNING".into()
        };
    }
    for mut node in &mut q_meter {
        let pct = (phys.height / 3.0).clamp(0.01, 1.0) * 100.0;
        node.width = Val::Percent(pct);
    }
    for mut bg in &mut q_badge {
        *bg = if phys.paused {
            BackgroundColor(Color::srgba(0.85, 0.50, 0.15, 0.25))
        } else {
            BackgroundColor(Color::srgba(0.20, 0.70, 0.40, 0.20))
        };
    }
}
