//! Blender-Style Shading Workspace Example:
//!
//! A pure two-panel workspace inspired by Blender's Shading workspace:
//! 1. **3D Viewport (Shader Preview)**: A live 3D preview of a PBR material on interactive
//!    3D geometry (UV Sphere, Torus, Cuboid) rendered to an offscreen texture with studio lighting.
//!    Supports mouse orbit and mesh switching with key `M`.
//! 2. **Shader Editor (Node Graph)**: The node graph canvas defining the material
//!    properties (Color Palette, Metallic, Roughness, Wave Oscillator, Principled BSDF).
//!
//! **Constraint Fulfilled**: Zero core modifications! All existing core crates
//! (`luma_core`, `luma_canvas`, `luma_dataflow`, etc.) are completely untouched.
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
    EditorTheme, EditorUiCamera, LumaGraph, LumaSnarlAppExt, LumaUiPlugin, NodeDef, PinInfo,
};

const SHADER_PREVIEW_WINDOW: &str = "shader_preview";
const SHADER_PREVIEW_LAYER: usize = 1;

// =============================================================================
// 1. Data Structures & Components
// =============================================================================

#[derive(Component)]
struct PreviewSphere;

#[derive(Component)]
struct ShaderPreviewRoot;

#[derive(Component)]
struct PreviewStatsText;

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
#[allow(dead_code)]
struct PrincipledBsdfWidget {
    node_id: NodeId,
}

#[derive(Component)]
struct BsdfColorSwatch {
    node_id: NodeId,
}

#[derive(Component)]
struct BsdfReadoutText {
    node_id: NodeId,
}

#[derive(Component)]
struct WaveOscillatorMarker {
    node_id: NodeId,
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
// 2. Main Entry Point
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
        .add_plugins((LumaPlugin, LumaEditorPlugin, LumaUiPlugin))
        // Register custom shader domain nodes
        .register_node(
            NodeDef::new("shader.color_palette", "Color Palette")
                .category("Input", "COLOR")
                .header_rgb(0.62, 0.45, 0.16)
                .width(220.0)
                .output(PinInfo::color("Color"))
                .body(36.0, |ctx| {
                    let node_id = ctx.node_id();
                    let font = ctx.font();

                    let row = ctx
                        .spawn((
                            UNode {
                                width: UVal::Percent(1.0),
                                height: UVal::Px(28.0),
                                padding: USides::axes(4.0, 2.0),
                                background_color: Color::srgb(0.10, 0.11, 0.14),
                                border_radius: UCornerRadius::all(4.0),
                                ..default()
                            },
                            ULayout {
                                display: UDisplay::Flex,
                                flex_direction: UFlexDirection::Row,
                                align_items: UAlignItems::Center,
                                gap: 5.0,
                                ..default()
                            },
                        ))
                        .id();

                    let presets = [
                        ("Gold", LinearColor::new(1.00, 0.76, 0.33, 1.0)),
                        ("Cyan", LinearColor::new(0.18, 0.85, 0.98, 1.0)),
                        ("Ruby", LinearColor::new(0.95, 0.18, 0.30, 1.0)),
                        ("Lime", LinearColor::new(0.15, 0.88, 0.48, 1.0)),
                        ("Silk", LinearColor::new(0.88, 0.90, 0.95, 1.0)),
                    ];

                    for (name, col) in presets {
                        ctx.commands()
                            .spawn((
                                ChildOf(row),
                                BtnSetColor { node_id, color: col },
                                UNode {
                                    width: UVal::Px(35.0),
                                    height: UVal::Px(20.0),
                                    background_color: Color::srgba(col.r, col.g, col.b, col.a),
                                    border_radius: UCornerRadius::all(3.0),
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
                                    text: name.into(),
                                    font_size: 8.0,
                                    font: font.clone(),
                                    color: Color::BLACK,
                                    ..default()
                                });
                            });
                    }
                }),
        )
        .register_node(
            NodeDef::new("shader.metallic_val", "Metallic Value")
                .category("Input", "VAL")
                .header_rgb(0.24, 0.36, 0.48)
                .width(220.0)
                .output(PinInfo::float("Metallic"))
                .body(36.0, |ctx| {
                    let node_id = ctx.node_id();
                    let font = ctx.font();

                    let row = ctx
                        .spawn((
                            UNode {
                                width: UVal::Percent(1.0),
                                height: UVal::Px(28.0),
                                padding: USides::axes(4.0, 2.0),
                                background_color: Color::srgb(0.10, 0.11, 0.14),
                                border_radius: UCornerRadius::all(4.0),
                                ..default()
                            },
                            ULayout {
                                display: UDisplay::Flex,
                                flex_direction: UFlexDirection::Row,
                                align_items: UAlignItems::Center,
                                gap: 4.0,
                                ..default()
                            },
                        ))
                        .id();

                    let steps = [("0.0", 0.0), ("0.25", 0.25), ("0.5", 0.5), ("0.85", 0.85), ("1.0", 1.0)];

                    for (lbl, val) in steps {
                        ctx.commands()
                            .spawn((
                                ChildOf(row),
                                BtnSetVal { node_id, val },
                                UNode {
                                    width: UVal::Px(35.0),
                                    height: UVal::Px(20.0),
                                    background_color: Color::srgb(0.18, 0.22, 0.28),
                                    border_radius: UCornerRadius::all(3.0),
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
                                    text: lbl.into(),
                                    font_size: 8.5,
                                    font: font.clone(),
                                    color: Color::WHITE,
                                    ..default()
                                });
                            });
                    }
                }),
        )
        .register_node(
            NodeDef::new("shader.roughness_val", "Roughness Value")
                .category("Input", "VAL")
                .header_rgb(0.24, 0.36, 0.48)
                .width(220.0)
                .output(PinInfo::float("Roughness"))
                .body(36.0, |ctx| {
                    let node_id = ctx.node_id();
                    let font = ctx.font();

                    let row = ctx
                        .spawn((
                            UNode {
                                width: UVal::Percent(1.0),
                                height: UVal::Px(28.0),
                                padding: USides::axes(4.0, 2.0),
                                background_color: Color::srgb(0.10, 0.11, 0.14),
                                border_radius: UCornerRadius::all(4.0),
                                ..default()
                            },
                            ULayout {
                                display: UDisplay::Flex,
                                flex_direction: UFlexDirection::Row,
                                align_items: UAlignItems::Center,
                                gap: 4.0,
                                ..default()
                            },
                        ))
                        .id();

                    let steps = [("0.05", 0.05), ("0.20", 0.20), ("0.40", 0.40), ("0.70", 0.70), ("1.0", 1.0)];

                    for (lbl, val) in steps {
                        ctx.commands()
                            .spawn((
                                ChildOf(row),
                                BtnSetVal { node_id, val },
                                UNode {
                                    width: UVal::Px(35.0),
                                    height: UVal::Px(20.0),
                                    background_color: Color::srgb(0.20, 0.24, 0.30),
                                    border_radius: UCornerRadius::all(3.0),
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
                                    text: lbl.into(),
                                    font_size: 8.5,
                                    font: font.clone(),
                                    color: Color::WHITE,
                                    ..default()
                                });
                            });
                    }
                }),
        )
        .register_node(
            NodeDef::new("shader.wave_oscillator", "Sine Oscillator")
                .category("Math", "WAVE")
                .header_rgb(0.34, 0.24, 0.54)
                .width(200.0)
                .output(PinInfo::float("Wave"))
                .body(28.0, |ctx| {
                    let node_id = ctx.node_id();
                    let font = ctx.font();

                    ctx.spawn((
                        WaveOscillatorMarker { node_id },
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
                            font: font.clone(),
                            color: Color::srgb(0.75, 0.65, 0.95),
                            ..default()
                        });
                    });
                }),
        )
        .register_node(
            NodeDef::new("shader.principled_bsdf", "Principled BSDF")
                .category("Shader", "BSDF")
                .header_rgb(0.18, 0.48, 0.36)
                .width(250.0)
                .input(PinInfo::color("Base Color"))
                .input(PinInfo::float("Metallic").default_float(0.0))
                .input(PinInfo::float("Roughness").default_float(0.3))
                .input(PinInfo::float("Emission").default_float(0.0))
                .output(PinInfo::color("Surface"))
                .body(50.0, |ctx| {
                    let node_id = ctx.node_id();
                    let font = ctx.font();

                    let row = ctx
                        .spawn((
                            PrincipledBsdfWidget { node_id },
                            UNode {
                                width: UVal::Percent(1.0),
                                height: UVal::Px(44.0),
                                padding: USides::axes(6.0, 3.0),
                                background_color: Color::srgb(0.09, 0.12, 0.11),
                                border_radius: UCornerRadius::all(4.0),
                                ..default()
                            },
                            ULayout {
                                display: UDisplay::Flex,
                                flex_direction: UFlexDirection::Row,
                                align_items: UAlignItems::Center,
                                justify_content: UJustifyContent::SpaceBetween,
                                ..default()
                            },
                        ))
                        .id();

                    ctx.commands()
                        .spawn((
                            ChildOf(row),
                            UNode {
                                width: UVal::Px(120.0),
                                height: UVal::Px(36.0),
                                ..default()
                            },
                            ULayout {
                                display: UDisplay::Flex,
                                flex_direction: UFlexDirection::Column,
                                gap: 2.0,
                                ..default()
                            },
                        ))
                        .with_children(|col| {
                            col.spawn((
                                BsdfReadoutText { node_id },
                                UText {
                                    text: "Material: Standard PBR\nTarget: 3D Viewport".into(),
                                    font_size: 9.0,
                                    font: font.clone(),
                                    color: Color::srgb(0.85, 0.90, 0.88),
                                    ..default()
                                },
                            ));
                        });

                    ctx.commands().spawn((
                        ChildOf(row),
                        BsdfColorSwatch { node_id },
                        UNode {
                            width: UVal::Px(32.0),
                            height: UVal::Px(32.0),
                            background_color: Color::srgb(1.00, 0.76, 0.33),
                            border_radius: UCornerRadius::all(4.0),
                            ..default()
                        },
                        UBorder {
                            color: Color::srgb(0.30, 0.40, 0.35),
                            width: 1.0,
                            radius: UCornerRadius::all(4.0),
                            offset: 0.0,
                        },
                    ));
                }),
        )
        .add_systems(Startup, (setup_3d_preview_scene, setup_pipeline).chain())
        .add_systems(
            PostStartup,
            setup_blender_shading_workspace.after(luma_ui::attach_overlay_dock_anchors),
        )
        .add_systems(
            Update,
            (
                sync_color_picker_system,
                sync_value_slider_system,
                sync_wave_oscillator_system,
                sync_shader_to_preview_material,
                orbit_preview_object,
                toggle_preview_mesh,
            ),
        )
        .add_systems(PostUpdate, update_preview_viewport_rect)
        .run();
}

// =============================================================================
// 3. 3D Preview Scene Setup (Offscreen Texture on Layer 1)
// =============================================================================

fn setup_3d_preview_scene(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 1. Offscreen Render Target Texture
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

    // 2. Initial PBR Material
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

    // 3. Meshes collection
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

    // 4. Spawn 3D Mesh
    commands.spawn((
        Mesh3d(sphere),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(0.0, 0.0, 0.0),
        PreviewSphere,
        preview_layer.clone(),
    ));

    // 5. Studio 3-Point Lighting Setup
    // Key Light
    commands.spawn((
        DirectionalLight {
            illuminance: 14_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(4.5, 7.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        preview_layer.clone(),
    ));

    // Cool Fill Light
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

    // Rim / Backlight
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

    // 6. 3D Camera rendering into the offscreen texture
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
// 4. 2D Node Pipeline Setup
// =============================================================================

fn setup_pipeline(mut commands: Commands, mut graph: LumaGraph) {
    // 2D Camera for world canvas and screen UI
    commands.spawn((
        Camera2d,
        EditorCamera,
        EditorUiCamera,
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Spawn classic Blender shader nodes:
    let n_palette = graph.spawn("shader.color_palette", Vec2::new(-360.0, 140.0));
    let n_metallic = graph.spawn("shader.metallic_val", Vec2::new(-360.0, -20.0));
    let n_roughness = graph.spawn("shader.roughness_val", Vec2::new(-360.0, -180.0));
    let n_wave = graph.spawn("shader.wave_oscillator", Vec2::new(-360.0, -340.0));
    let n_bsdf = graph.spawn("shader.principled_bsdf", Vec2::new(120.0, -20.0));

    // Connect node graph:
    graph.connect(n_palette, 0, n_bsdf, 0); // Color Palette -> BSDF Base Color
    graph.connect(n_metallic, 0, n_bsdf, 1); // Metallic Val -> BSDF Metallic
    graph.connect(n_roughness, 0, n_bsdf, 2); // Roughness Val -> BSDF Roughness
    graph.connect(n_wave, 0, n_bsdf, 3);      // Sine Oscillator -> BSDF Emission

    // Set initial outputs:
    graph.set_output_color(n_palette, 0, LinearColor::new(1.00, 0.76, 0.33, 1.0));
    graph.set_output(n_metallic, 0, Value::float(0.80));
    graph.set_output(n_roughness, 0, Value::float(0.25));

    graph.select(n_bsdf);
}

// =============================================================================
// 5. Dock Layout: Pure 2-Tab Blender Workspace
// =============================================================================

fn setup_blender_shading_workspace(
    mut registry: ResMut<WindowRegistry>,
    mut workspace_registry: ResMut<WorkspaceRegistry>,
    mut dock_tree: ResMut<DockTree>,
    preview_texture: Res<ShaderPreviewTexture>,
) {
    // 1. Remove non-shading windows to keep ONLY the two tabs
    registry.unregister(luma_ui::window_ids::OUTLINER);
    registry.unregister(luma_ui::window_ids::LIBRARY);
    registry.unregister(luma_ui::window_ids::INSPECTOR);

    // 2. Register Tab 1: Shader Preview (3D Viewport)
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
                    // Blender-style 3D Viewport header banner
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

                    // Offscreen 3D rendered texture displayed via ImageNode
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

    // 3. Register Tab 2: Shader Node Editor (Transparent hole for world canvas)
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

    // 4. Construct the pure 2-tab binary split DockTree:
    // Left: 3D Viewport (45%) | Right: Shader Editor (55%)
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

    // 5. Workspace Tab: Exactly one active "Shading" workspace
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
// 6. Interactive Update Systems
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
            // Gentle idle turntable rotation
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

fn sync_color_picker_system(
    mut graph: LumaGraph,
    btn_query: Query<(&UInteraction, &BtnSetColor), Changed<UInteraction>>,
) {
    for (interaction, btn) in &btn_query {
        if *interaction == UInteraction::Pressed {
            graph.set_output_color(btn.node_id, 0, btn.color);
        }
    }
}

fn sync_value_slider_system(
    mut graph: LumaGraph,
    btn_query: Query<(&UInteraction, &BtnSetVal), Changed<UInteraction>>,
) {
    for (interaction, btn) in &btn_query {
        if *interaction == UInteraction::Pressed {
            graph.set_output(btn.node_id, 0, Value::float(btn.val));
        }
    }
}

fn sync_wave_oscillator_system(
    mut graph: LumaGraph,
    time: Res<Time>,
    oscillators: Query<&WaveOscillatorMarker>,
) {
    let t = time.elapsed_secs();
    let wave_val = (t * 2.5).sin() * 0.5 + 0.5; // Smooth 0.0 .. 1.0 sine

    for osc in &oscillators {
        graph.set_output(osc.node_id, 0, Value::float(wave_val));
    }
}

fn sync_shader_to_preview_material(
    graph: LumaGraph,
    mut materials: ResMut<Assets<StandardMaterial>>,
    live_mat: Res<ShaderLiveMaterial>,
    mut stats_text: Query<&mut Text, With<PreviewStatsText>>,
    mut swatches: Query<(&BsdfColorSwatch, &mut UNode)>,
    mut readouts: Query<(&BsdfReadoutText, &mut UText)>,
) {
    for (&node_id, node) in graph.graph.0.nodes() {
        if node.kind == "shader.principled_bsdf" {
            // Input 0: Base Color
            let base_color = graph
                .input_color(node_id, 0)
                .unwrap_or(LinearColor::new(1.00, 0.76, 0.33, 1.0));

            // Input 1: Metallic
            let metallic = match graph.input_value(node_id, 1) {
                Value::Primitive(PrimitiveValue::Float(f)) => f.clamp(0.0, 1.0),
                _ => 0.0,
            };

            // Input 2: Roughness
            let roughness = match graph.input_value(node_id, 2) {
                Value::Primitive(PrimitiveValue::Float(f)) => f.clamp(0.04, 1.0),
                _ => 0.30,
            };

            // Input 3: Emission
            let emission = match graph.input_value(node_id, 3) {
                Value::Primitive(PrimitiveValue::Float(f)) => f.max(0.0),
                _ => 0.0,
            };

            // 1. Update 3D PBR StandardMaterial
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

            // 2. Update Viewport Header Banner Text
            for mut text in &mut stats_text {
                **text = format!(
                    "RGB: ({:.2}, {:.2}, {:.2}) | Met: {:.2} | Rough: {:.2} | Emit: {:.2}",
                    base_color.r, base_color.g, base_color.b, metallic, roughness, emission
                );
            }

            // 3. Update BSDF Node Swatch and Text
            for (swatch, mut unode) in &mut swatches {
                if swatch.node_id == node_id {
                    unode.background_color = Color::srgb(base_color.r, base_color.g, base_color.b);
                }
            }
            for (readout, mut utext) in &mut readouts {
                if readout.node_id == node_id {
                    utext.text = format!("Met: {:.2}  Rough: {:.2}\nEmit: {:.2}", metallic, roughness, emission);
                }
            }
        }
    }
}
