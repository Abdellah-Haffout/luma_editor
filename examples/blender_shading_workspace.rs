//! Blender-Style Shading Workspace Example:
//!
//! Powered by `bevy_luma_widgets`:
//! - Pure functions as nodes via `#[luma_node]`.
//! - Interactive widgets powered by `bevy_luma_widgets` (LumaButton, LumaSwatch, LumaBadge).
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

// =============================================================================
// 2. Interactive UI Body Nodes Powered by `bevy_luma_widgets`
// =============================================================================

#[derive(Component)]
struct BtnSetColor {
    node_id: NodeId,
    color: LinearColor,
}

#[derive(Component)]
struct BtnSetVal {
    node_id: NodeId,
    val: f32,
}

#[derive(Component)]
struct WaveOscillatorMarker(NodeId);

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
        if *interaction == UInteraction::Pressed {
            graph.set_output_color(btn.node_id, 0, btn.color);
        }
    }
}

/// Interactive Metallic Value node using `LumaButton` presets.
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
        let font = ctx.font();

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
                    gap: 4.0,
                    ..default()
                },
            ))
            .id();

        for (lbl, val) in [("0.0", 0.0), ("0.25", 0.25), ("0.5", 0.5), ("0.85", 0.85), ("1.0", 1.0)] {
            ctx.commands()
                .spawn((
                    ChildOf(row),
                    BtnSetVal { node_id, val },
                    LumaButton {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Small,
                        disabled: false,
                    },
                    UNode {
                        width: UVal::Px(36.0),
                        height: UVal::Px(22.0),
                        background_color: Color::srgb(0.16, 0.20, 0.26),
                        border_radius: UCornerRadius::all(4.0),
                        ..default()
                    },
                    UInteraction::default(),
                    UInteractionColors {
                        normal: Color::srgb(0.16, 0.20, 0.26),
                        hovered: Color::srgb(0.24, 0.32, 0.42),
                        pressed: Color::srgb(0.12, 0.15, 0.20),
                    },
                    ULayout {
                        display: UDisplay::Flex,
                        justify_content: UJustifyContent::Center,
                        align_items: UAlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|b| {
                    b.spawn(UText {
                        text: lbl.into(),
                        font_size: 8.5,
                        font: font.clone(),
                        color: Color::WHITE,
                        ..default()
                    });
                });
        }
    }
}

fn sync_metallic_node_system(
    mut graph: LumaGraph,
    btn_query: Query<(&UInteraction, &BtnSetVal), Changed<UInteraction>>,
) {
    for (interaction, btn) in &btn_query {
        if *interaction == UInteraction::Pressed {
            graph.set_output(btn.node_id, 0, Value::float(btn.val));
        }
    }
}

/// Interactive Roughness Value node using `LumaButton` presets.
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
        let font = ctx.font();

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
                    gap: 4.0,
                    ..default()
                },
            ))
            .id();

        for (lbl, val) in [("0.05", 0.05), ("0.20", 0.20), ("0.40", 0.40), ("0.70", 0.70), ("1.0", 1.0)] {
            ctx.commands()
                .spawn((
                    ChildOf(row),
                    BtnSetVal { node_id, val },
                    LumaButton {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Small,
                        disabled: false,
                    },
                    UNode {
                        width: UVal::Px(36.0),
                        height: UVal::Px(22.0),
                        background_color: Color::srgb(0.18, 0.22, 0.28),
                        border_radius: UCornerRadius::all(4.0),
                        ..default()
                    },
                    UBorder {
                        color: Color::srgb(0.30, 0.36, 0.46),
                        width: 1.0,
                        radius: UCornerRadius::all(4.0),
                        offset: 0.0,
                    },
                    UInteraction::default(),
                    UInteractionColors {
                        normal: Color::srgb(0.18, 0.22, 0.28),
                        hovered: Color::srgb(0.26, 0.34, 0.44),
                        pressed: Color::srgb(0.14, 0.17, 0.22),
                    },
                    ULayout {
                        display: UDisplay::Flex,
                        justify_content: UJustifyContent::Center,
                        align_items: UAlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|b| {
                    b.spawn(UText {
                        text: lbl.into(),
                        font_size: 8.5,
                        font: font.clone(),
                        color: Color::WHITE,
                        ..default()
                    });
                });
        }
    }
}

fn sync_roughness_node_system(
    mut graph: LumaGraph,
    btn_query: Query<(&UInteraction, &BtnSetVal), Changed<UInteraction>>,
) {
    for (interaction, btn) in &btn_query {
        if *interaction == UInteraction::Pressed {
            graph.set_output(btn.node_id, 0, Value::float(btn.val));
        }
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
) {
    let t = time.elapsed_secs();
    let wave_val = (t * 2.5).sin() * 0.5 + 0.5;

    for osc in &oscillators {
        graph.set_output(osc.0, 0, Value::float(wave_val));
    }
}

// =============================================================================
// 3. Resources & Viewport Components
// =============================================================================

#[derive(Component)]
struct PreviewSphere;

#[derive(Component)]
struct ShaderPreviewRoot;

#[derive(Component)]
struct PreviewStatsText;

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
                title: "Luma Editor — Blender Shading Workspace".into(),
                resolution: (1440, 880).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(EditorTheme::BG_CANVAS))
        .init_resource::<ShaderPreviewViewportRect>()
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
                orbit_preview_object,
                toggle_preview_mesh,
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

    let sphere = meshes.add(Sphere::new(1.25).mesh().uv(48, 24));
    let torus = meshes.add(Torus::new(0.45, 1.15));
    let cuboid = meshes.add(Cuboid::new(1.8, 1.8, 1.8));

    commands.insert_resource(PreviewMeshes {
        sphere: sphere.clone(),
        torus,
        cuboid,
        current_index: 0,
    });

    let preview_layer = RenderLayers::layer(SHADER_PREVIEW_LAYER);

    commands.spawn((
        Mesh3d(sphere),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(0.0, 0.0, 0.0),
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

    // 3D Camera rendering into offscreen texture
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: -1,
            clear_color: Color::srgb(0.10, 0.11, 0.13).into(),
            ..default()
        },
        RenderTarget::Image(image_handle.into()),
        Transform::from_xyz(0.0, 1.2, 3.4).looking_at(Vec3::ZERO, Vec3::Y),
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

    let n_palette = graph.spawn("shader.color_palette", Vec2::new(-360.0, 140.0));
    let n_metallic = graph.spawn("shader.metallic_val", Vec2::new(-360.0, -20.0));
    let n_roughness = graph.spawn("shader.roughness_val", Vec2::new(-360.0, -180.0));
    let n_wave = graph.spawn("shader.wave_oscillator", Vec2::new(-360.0, -340.0));
    let n_bsdf = graph.spawn("shader.principled_bsdf", Vec2::new(120.0, -20.0));

    graph.connect(n_palette, 0, n_bsdf, 0);
    graph.connect(n_metallic, 0, n_bsdf, 1);
    graph.connect(n_roughness, 0, n_bsdf, 2);
    graph.connect(n_wave, 0, n_bsdf, 3);

    graph.set_output_color(n_palette, 0, LinearColor::new(1.00, 0.76, 0.33, 1.0));
    graph.set_output(n_metallic, 0, Value::float(0.80));
    graph.set_output(n_roughness, 0, Value::float(0.25));

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
                            Text::new("3D Viewport • Principled BSDF Preview [M: Switch Mesh]"),
                            TextFont {
                                font_size: tokens::TEXT_SIZE_SM,
                                ..default()
                            },
                            TextColor(tokens::TEXT_PRIMARY),
                        ));

                        bar.spawn((
                            PreviewStatsText,
                            Text::new("Base Color: Gold | Met: 0.80 | Rough: 0.25"),
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

    // Tab 2: Shader Node Editor
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

    // Pure 2-Tab Binary Split: 45% Preview / 55% Editor
    let mut tree = DockTree::new();
    let preview_leaf = tree.insert(DockNode::Leaf(
        DockLeaf::new("preview_area", DockAreaStyle::TabBar)
            .with_windows(vec![SHADER_PREVIEW_WINDOW.into()])
            .persistent(),
    ));
    let editor_leaf = tree.insert(DockNode::Leaf(
        DockLeaf::new("editor_area", DockAreaStyle::TabBar)
            .with_windows(vec![luma_ui::window_ids::NODE_EDITOR.into()])
            .persistent(),
    ));
    let root = tree.insert(DockNode::Split(DockSplit {
        axis: SplitAxis::Horizontal,
        fraction: 0.45,
        a: preview_leaf,
        b: editor_leaf,
    }));
    tree.root = Some(root);
    *dock_tree = tree.clone();

    // Single "Shading" Workspace
    workspace_registry.workspaces.clear();
    workspace_registry.register(WorkspaceDescriptor {
        id: "shading".into(),
        name: "Shading".into(),
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
