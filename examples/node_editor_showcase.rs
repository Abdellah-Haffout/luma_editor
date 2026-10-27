//! Showcase example demonstrating the clean, declarative node architecture:
//!
//! - **Pure Functions as Nodes**: Write standard Rust functions with `#[luma_node]` and default parameter attributes `#[default = ...]`.
//! - **Zero-Boilerplate Auto-Registration**: All nodes register themselves into the engine compile-time inventory.
//! - **Pure Core**: The core crates (`luma_core`, `luma_canvas`, `luma_ui`) contain **zero** hardcoded domain nodes.
//! - **Clean UI Embedding**: Nodes requiring interactive widgets implement `LumaNode` and mount `bevy_luma` widgets in `.show_body()`.
//! - **Empty `main()`**: `main()` contains **zero** `.register_node(...)` calls. Whether 10 nodes or 10,000, all are auto-discovered!
//!
//! Run with:
//! ```bash
//! cargo run --example node_editor_showcase
//! ```

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_luma::prelude::*;
use luma_canvas::plugin::{EditorCamera, LumaEditorPlugin};
use luma_core::id::NodeId;
use luma_dataflow::value::Value;
use luma_ui::{
    luma_body_node, luma_node, EditorTheme, EditorUiCamera, LumaGraph, LumaNode, LumaResource,
    LumaUiPlugin, NodeBodyContext, NodeStyle, PinInfo, WebImageLoader,
};

// ============================================================================
// 1. Declarative Pure Function Nodes (Auto-Registered via #[luma_node])
// ============================================================================

/// Pure dataflow addition with default parameters.
#[luma_node(
    kind = "math.add",
    category = "Math",
    tag = "ADD",
    title = "Math (Add)",
    header_rgb = [0.16, 0.34, 0.48]
)]
pub fn math_add(
    #[default = 10.0] a: f32, 
    #[default = 25.0] b: f32
    ) -> f32 {
        a + b
}

/// Pure dataflow subtraction with default parameters.
#[luma_node(
    kind = "math.subtract",
    category = "Math",
    tag = "SUB",
    title = "Math (Subtract)",
    header_rgb = [0.16, 0.34, 0.48]
)]
pub fn math_subtract(#[default = 0.0] a: f32, #[default = 0.0] b: f32) -> f32 {
    a - b
}

/// Pure dataflow multiplication with default parameters.
#[luma_node(
    kind = "math.multiply",
    category = "Math",
    tag = "MUL",
    title = "Math (Multiply)",
    header_rgb = [0.18, 0.44, 0.36]
)]
pub fn math_multiply(#[default = 1.0] a: f32, #[default = 2.0] b: f32) -> f32 {
    a * b
}

/// Safe division node guarding against division-by-zero.
#[luma_node(
    kind = "math.divide",
    category = "Math",
    tag = "DIV",
    title = "Math (Divide)",
    header_rgb = [0.18, 0.44, 0.36]
)]
pub fn math_divide(#[default = 1.0] a: f32, #[default = 1.0] b: f32) -> f32 {
    if b.abs() < f32::EPSILON {
        0.0
    } else {
        a / b
    }
}

/// Harmonic sine wave oscillator node.
#[luma_node(
    kind = "math.wave",
    category = "Math",
    tag = "WAVE",
    title = "Harmonic Wave",
    header_rgb = [0.22, 0.36, 0.52]
)]
pub fn harmonic_wave(angle: f32, #[default = 1.0] amplitude: f32) -> f32 {
    angle.sin() * amplitude
}

/// Empty node that does nothing (no inputs, no outputs, zero logic).
#[luma_node(
    kind = "util.empty",
    category = "Utility",
    tag = "EMPTY",
    title = "Empty Node",
    header_rgb = [0.25, 0.25, 0.28]
)]
pub fn empty_node() {}

// ============================================================================
// 2. Interactive UI Nodes (Auto-Registered via inventory)
// ============================================================================

#[derive(Component)]
struct LinkInputWidget {
    node_id: NodeId,
    last_text: String,
}

#[derive(Component)]
struct NumberInputWidget {
    node_id: NodeId,
    last_text: String,
}

#[derive(Component)]
struct ImageViewerWidget {
    node_id: NodeId,
    image_entity: Entity,
    last_url: String,
}

/// Interactive Link (EditText) node mounting a live `UTextInput` widget.
/// The `system = sync_link_input_system` automatically registers its update logic with Bevy.
#[luma_body_node(system = sync_link_input_system)]
pub struct LinkInputNode;

impl LumaNode for LinkInputNode {
    fn kind(&self) -> &'static str {
        "input.link"
    }

    fn title(&self) -> &str {
        "Link (EditText)"
    }

    fn category(&self) -> (&'static str, &'static str) {
        ("Input", "LINK")
    }

    fn style(&self) -> NodeStyle {
        NodeStyle::header_rgb(0.14, 0.34, 0.50).width(250.0)
    }

    fn outputs(&self) -> Vec<PinInfo> {
        vec![PinInfo::string("url")]
    }

    fn body_height(&self) -> f32 {
        34.0
    }

    fn show_body(&self, ctx: &mut NodeBodyContext) {
        let node_id = ctx.node_id();
        let initial = "https://raw.githubusercontent.com/bevyengine/bevy/main/assets/branding/bevy_bird_dark.png".to_string();
        let entity = spawn_text_input(
            ctx.commands(),
            UTextInput::new("Enter image URL or path...")
                .with_value(&initial)
                .with_font_size(10.0),
        );
        ctx.attach(entity).insert((
            LinkInputWidget {
                node_id,
                last_text: String::new(),
            },
            UNode {
                width: UVal::Percent(1.0),
                height: UVal::Px(26.0),
                padding: USides::axes(8.0, 4.0),
                background_color: Color::srgb(0.10, 0.11, 0.13),
                border_radius: UCornerRadius::all(4.0),
                ..default()
            },
        ));
    }
}

fn sync_link_input_system(
    mut graph: LumaGraph,
    mut link_inputs: Query<(&mut LinkInputWidget, &UTextInput)>,
) {
    let graph_changed = graph.graph.is_changed();
    for (mut widget, text_input) in &mut link_inputs {
        if text_input.value != widget.last_text || graph_changed {
            widget.last_text = text_input.value.clone();
            graph.set_output(widget.node_id, 0, Value::string(text_input.value.clone()));
        }
    }
}

/// Interactive Number Input node mounting a live `UTextInput` for numeric editing.
/// The `system = sync_number_input_system` automatically registers its update logic with Bevy.
#[luma_body_node(system = sync_number_input_system)]
pub struct NumberInputNode;

impl LumaNode for NumberInputNode {
    fn kind(&self) -> &'static str {
        "input.number"
    }

    fn title(&self) -> &str {
        "Number Input"
    }

    fn category(&self) -> (&'static str, &'static str) {
        ("Input", "NUM")
    }

    fn style(&self) -> NodeStyle {
        NodeStyle::header_rgb(0.12, 0.38, 0.44).width(200.0)
    }

    fn outputs(&self) -> Vec<PinInfo> {
        vec![PinInfo::float("value")]
    }

    fn body_height(&self) -> f32 {
        34.0
    }

    fn show_body(&self, ctx: &mut NodeBodyContext) {
        let node_id = ctx.node_id();
        let initial = "15.0".to_string();
        let entity = spawn_text_input(
            ctx.commands(),
            UTextInput::new("Enter number...")
                .with_value(&initial)
                .with_font_size(10.0),
        );
        ctx.attach(entity).insert((
            NumberInputWidget {
                node_id,
                last_text: String::new(),
            },
            UNode {
                width: UVal::Percent(1.0),
                height: UVal::Px(26.0),
                padding: USides::axes(8.0, 4.0),
                background_color: Color::srgb(0.10, 0.11, 0.13),
                border_radius: UCornerRadius::all(4.0),
                ..default()
            },
        ));
    }
}

fn sync_number_input_system(
    mut graph: LumaGraph,
    mut num_inputs: Query<(&mut NumberInputWidget, &UTextInput)>,
) {
    let graph_changed = graph.graph.is_changed();
    for (mut widget, text_input) in &mut num_inputs {
        if text_input.value != widget.last_text || graph_changed {
            widget.last_text = text_input.value.clone();
            let parsed = text_input.value.trim().parse::<f32>().unwrap_or(0.0);
            graph.set_output(widget.node_id, 0, Value::float(parsed));
        }
    }
}

/// Interactive Image Viewer node mounting a live `UImage` texture preview.
/// The `system = sync_image_viewer_system` automatically registers its update logic with Bevy.
#[luma_body_node(system = sync_image_viewer_system)]
pub struct ImageViewerNode;

impl LumaNode for ImageViewerNode {
    fn kind(&self) -> &'static str {
        "viewer.image"
    }

    fn title(&self) -> &str {
        "Image Viewer"
    }

    fn category(&self) -> (&'static str, &'static str) {
        ("Viewers", "IMAGE")
    }

    fn style(&self) -> NodeStyle {
        NodeStyle::header_rgb(0.42, 0.24, 0.12).width(240.0)
    }

    fn inputs(&self) -> Vec<PinInfo> {
        vec![PinInfo::string("url")]
    }

    fn outputs(&self) -> Vec<PinInfo> {
        vec![PinInfo::custom("entity", "Entity")]
    }

    fn body_height(&self) -> f32 {
        120.0
    }

    fn show_body(&self, ctx: &mut NodeBodyContext) {
        let node_id = ctx.node_id();
        let image_entity = ctx.spawn((
            UNode {
                width: UVal::Percent(1.0),
                height: UVal::Px(112.0),
                background_color: Color::WHITE,
                border_radius: UCornerRadius::all(6.0),
                ..default()
            },
            UImage {
                texture: Handle::default(),
                color: Color::WHITE,
                width: UVal::Percent(1.0),
                height: UVal::Px(112.0),
                radius: Some(UCornerRadius::all(6.0)),
            },
        )).id();

        ctx.spawn(ImageViewerWidget {
            node_id,
            image_entity,
            last_url: "__init__".into(),
        });
    }
}

fn sync_image_viewer_system(
    mut graph: LumaGraph,
    tex: Res<DemoFallbackTextures>,
    mut web_images: ResMut<WebImageLoader>,
    asset_server: Res<AssetServer>,
    mut viewers: Query<(&mut ImageViewerWidget, &mut UImage)>,
) {
    for (mut viewer, mut uimage) in &mut viewers {
        // Output this image entity so downstream nodes can receive and host it!
        graph.set_output_resource(viewer.node_id, 0, "Entity", PassedEntity(viewer.image_entity));

        let url = graph.input_string(viewer.node_id, 0);
        let trimmed = url.trim();

        let expected_texture = if trimmed.starts_with("http://") || trimmed.starts_with("https://")
        {
            web_images
                .get_or_request(trimmed)
                .unwrap_or_else(|| web_images.loading_texture.clone())
        } else if trimmed.is_empty() || trimmed.contains("grid") {
            tex.grid.clone()
        } else if trimmed.starts_with("sunset://") {
            tex.sunset.clone()
        } else {
            asset_server.load(trimmed.to_string())
        };

        if uimage.texture != expected_texture || url != viewer.last_url {
            viewer.last_url = url.clone();
            uimage.texture = expected_texture;
        }
    }
}

// ============================================================================
// 2.5 Entity Pipeline (Zero Core Changes — Driven by LumaResource)
// ============================================================================

/// Custom Entity resource passed through wires without modifying core engine crates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, LumaResource)]
#[resource("Entity")]
pub struct PassedEntity(pub Entity);

impl Default for PassedEntity {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

#[derive(Component)]
struct SpawnEntityBtn(NodeId);

/// 1. Node with a Button: spawns a Bevy Entity and pushes it to output pin
#[luma_body_node(system = sync_entity_spawner_system)]
pub struct EntitySpawnerNode;

impl LumaNode for EntitySpawnerNode {
    fn kind(&self) -> &'static str {
        "entity.spawner"
    }

    fn title(&self) -> &str {
        "Entity Spawner"
    }

    fn category(&self) -> (&'static str, &'static str) {
        ("Entity", "SPAWN")
    }

    fn style(&self) -> NodeStyle {
        NodeStyle::header_rgb(0.18, 0.44, 0.32).width(230.0)
    }

    fn outputs(&self) -> Vec<PinInfo> {
        vec![PinInfo::custom("entity", "Entity")]
    }

    fn body_height(&self) -> f32 {
        36.0
    }

    fn show_body(&self, ctx: &mut NodeBodyContext) {
        let node_id = ctx.node_id();
        let font = ctx.font();

        ctx.spawn((
            SpawnEntityBtn(node_id),
            UNode {
                width: UVal::Percent(1.0),
                height: UVal::Px(28.0),
                background_color: Color::srgb(0.20, 0.55, 0.38),
                border_radius: UCornerRadius::all(5.0),
                ..default()
            },
            UInteraction::default(),
            ULayout {
                display: UDisplay::Flex,
                justify_content: UJustifyContent::Center,
                align_items: UAlignItems::Center,
                ..default()
            },
        ))
        .with_children(|btn| {
            btn.spawn(UText {
                text: "✨ Spawn Bevy Entity".into(),
                font_size: 11.0,
                font,
                color: Color::WHITE,
                ..default()
            });
        });
    }
}

fn sync_entity_spawner_system(
    mut commands: Commands,
    mut graph: LumaGraph,
    theme_opt: Option<Res<Theme>>,
    btn_q: Query<(&UInteraction, &SpawnEntityBtn), Changed<UInteraction>>,
) {
    let font = theme_opt.map(|t| t.text.font.inter_regular.clone()).unwrap_or_default();
    for (interaction, btn) in &btn_q {
        if *interaction == UInteraction::Pressed {
            // Spawn a live, interactive visual button that will be transferred!
            let visual_btn = commands
                .spawn((
                    UNode {
                        width: UVal::Percent(1.0),
                        height: UVal::Px(34.0),
                        background_color: Color::srgb(0.18, 0.58, 0.88),
                        border_radius: UCornerRadius::all(6.0),
                        padding: USides::axes(8.0, 4.0),
                        ..default()
                    },
                    UInteraction::default(),
                    ULayout {
                        display: UDisplay::Flex,
                        justify_content: UJustifyContent::Center,
                        align_items: UAlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|b| {
                    b.spawn(UText {
                        text: "🎉 Live Transferred Button".into(),
                        font_size: 11.0,
                        font: font.clone(),
                        color: Color::WHITE,
                        ..default()
                    });
                })
                .id();

            graph.set_output_resource(btn.0, 0, "Entity", PassedEntity(visual_btn));
        }
    }
}

#[derive(Component)]
struct EntityReceiverWidget {
    node_id: NodeId,
    slot_entity: Entity,
    last_entity: Option<Entity>,
}

#[derive(Component)]
struct EntityReceiverPlaceholder;

/// 2. Node that receives any Entity from wire and mounts the visual widget inside its body
#[luma_body_node(system = sync_entity_receiver_system)]
pub struct EntityReceiverNode;

impl LumaNode for EntityReceiverNode {
    fn kind(&self) -> &'static str {
        "entity.receiver"
    }

    fn title(&self) -> &str {
        "Visual Entity Host"
    }

    fn category(&self) -> (&'static str, &'static str) {
        ("Entity", "HOST")
    }

    fn style(&self) -> NodeStyle {
        NodeStyle::header_rgb(0.38, 0.22, 0.52).width(260.0)
    }

    fn inputs(&self) -> Vec<PinInfo> {
        vec![PinInfo::custom("entity", "Entity")]
    }

    fn body_height(&self) -> f32 {
        64.0
    }

    fn show_body(&self, ctx: &mut NodeBodyContext) {
        let node_id = ctx.node_id();
        let slot_entity = ctx.slot_entity();
        let font = ctx.font();

        ctx.spawn((
            EntityReceiverWidget {
                node_id,
                slot_entity,
                last_entity: None,
            },
            UNode {
                width: UVal::Percent(1.0),
                height: UVal::Px(56.0),
                padding: USides::axes(8.0, 6.0),
                background_color: Color::srgb(0.12, 0.08, 0.16),
                border_radius: UCornerRadius::all(6.0),
                ..default()
            },
            UBorder {
                color: Color::srgba(0.65, 0.40, 0.85, 0.40),
                width: 1.2,
                radius: UCornerRadius::all(6.0),
                offset: 0.0,
            },
            ULayout {
                display: UDisplay::Flex,
                justify_content: UJustifyContent::Center,
                align_items: UAlignItems::Center,
                ..default()
            },
        ))
        .with_children(|panel| {
            panel.spawn((
                EntityReceiverPlaceholder,
                UText {
                    text: "[ Wire an Entity here to render inside ]".into(),
                    font_size: 9.5,
                    font,
                    color: Color::srgb(0.65, 0.60, 0.75),
                    ..default()
                },
            ));
        });
    }
}

fn sync_entity_receiver_system(
    mut commands: Commands,
    graph: LumaGraph,
    mut receivers: Query<(&mut EntityReceiverWidget, &Children)>,
    mut placeholders: Query<&mut Visibility, With<EntityReceiverPlaceholder>>,
) {
    for (mut receiver, children) in &mut receivers {
        let val = graph.input_value(receiver.node_id, 0);
        let current_entity = val.downcast_resource::<PassedEntity>().map(|p| p.0);

        if current_entity != receiver.last_entity {
            if let Some(old_ent) = receiver.last_entity {
                commands.entity(old_ent).remove::<ChildOf>();
            }

            receiver.last_entity = current_entity;

            if let Some(ent) = current_entity {
                // Physically attach the incoming visual entity inside this node's slot!
                commands.entity(ent).insert(ChildOf(receiver.slot_entity));

                for &child in children {
                    if let Ok(mut vis) = placeholders.get_mut(child) {
                        *vis = Visibility::Hidden;
                    }
                }
            } else {
                for &child in children {
                    if let Ok(mut vis) = placeholders.get_mut(child) {
                        *vis = Visibility::Inherited;
                    }
                }
            }
        }
    }
}

// ============================================================================
// 3. Fallback Resource & Main Application
// ============================================================================

#[derive(Resource, Default)]
struct DemoFallbackTextures {
    sunset: Handle<Image>,
    grid: Handle<Image>,
}

fn main() {
    // Notice: ZERO .register_node(...) and ZERO .add_systems(Update, ...) calls in main()!
    // All 7 nodes (functions + UI widgets + their update systems) are auto-discovered from compile-time inventory!
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Luma Editor — Declarative Node Architecture Showcase".into(),
                resolution: (1360, 840).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(EditorTheme::BG_CANVAS))
        .init_resource::<DemoFallbackTextures>()
        .add_plugins((LumaPlugin, LumaEditorPlugin, LumaUiPlugin))
        .add_systems(Startup, setup_showcase)
        .run();
}

// ============================================================================
// 4. Graph Setup
// ============================================================================

fn setup_showcase(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut tex: ResMut<DemoFallbackTextures>,
    mut graph: LumaGraph,
) {
    commands.spawn((
        Camera2d,
        EditorCamera,
        EditorUiCamera,
        Transform::from_xyz(-40.0, 20.0, 0.0),
    ));

    tex.sunset = build_demo_texture(&mut images, false);
    tex.grid = build_demo_texture(&mut images, true);

    // Row 1: Image pipeline (Link text input -> Image viewer)
    let link_node = graph.spawn("input.link", Vec2::new(-260.0, 100.0));
    let viewer_node = graph.spawn("viewer.image", Vec2::new(100.0, 100.0));
    graph.connect(link_node, 0, viewer_node, 0);

    // Row 2: Math pipeline (Number input -> Math Add -> Math Multiply)
    let num_node = graph.spawn("input.number", Vec2::new(-340.0, -140.0));
    let add_node = graph.spawn("math.add", Vec2::new(-80.0, -140.0));
    let mul_node = graph.spawn("math.multiply", Vec2::new(180.0, -140.0));
    graph.connect(num_node, 0, add_node, 0);
    graph.connect(add_node, 0, mul_node, 0);

    // Row 3: Empty standalone node
    graph.spawn("util.empty", Vec2::new(-80.0, 300.0));

    // Row 4: Entity pipeline (Button Spawner -> Visual Entity Host)
    let spawner_node = graph.spawn("entity.spawner", Vec2::new(-260.0, -320.0));
    let receiver_node = graph.spawn("entity.receiver", Vec2::new(100.0, -320.0));
    graph.connect(spawner_node, 0, receiver_node, 0);

    graph.select(add_node);
}

fn build_demo_texture(images: &mut Assets<Image>, grid_mode: bool) -> Handle<Image> {
    let (w, h) = (256u32, 128u32);
    let mut data = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        let v = y as f32 / h as f32;
        for x in 0..w {
            let u = x as f32 / w as f32;
            let (r, g, b) = if grid_mode {
                let gx = ((u * 16.0).fract() - 0.5).abs() > 0.44;
                let gy = ((v * 8.0).fract() - 0.5).abs() > 0.44;
                if gx || gy {
                    (45, 210, 245)
                } else {
                    (24, 30, 42)
                }
            } else {
                let sun = ((u - 0.5).hypot((v - 0.5) * 0.5) < 0.18) as u8;
                (
                    ((0.25 + v * 0.70) * 255.0) as u8 | (sun * 240),
                    ((0.15 + v * 0.45) * 255.0) as u8 | (sun * 180),
                    ((0.55 - v * 0.30) * 255.0) as u8,
                )
            };
            data.extend_from_slice(&[r, g, b, 255]);
        }
    }
    images.add(Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}
