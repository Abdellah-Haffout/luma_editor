//! Demonstration of the new declarative macro-driven architecture (`#[luma_node]`, `#[derive(LumaResource)]`):
//!
//! - **Custom Object Passing**: `CustomMesh` is a user struct that automatically becomes an engine
//!   resource via `#[derive(LumaResource)]`.
//! - **Pure Function Nodes**: `generate_sphere` and `apply_tint` are plain Rust functions with `#[luma_node]`.
//!   They unpack inputs, execute, and emit outputs automatically.
//! - **Auto-Registration**: Zero manual registrations in `main`. All nodes in the binary are
//!   discovered and registered automatically by the engine at compile/link time!
//!
//! Run with:
//! ```bash
//! cargo run --example custom_mesh_pipeline
//! ```

use bevy::prelude::*;
use bevy_luma::prelude::*;
use luma_canvas::plugin::{EditorCamera, LumaEditorPlugin};
use luma_core::id::NodeId;
use luma_dataflow::value::LinearColor;
use luma_ui::{
    luma_node, EditorTheme, EditorUiCamera, LumaGraph, LumaResource, LumaSnarlAppExt, LumaUiPlugin,
    NodeDef, PinInfo,
};

// =============================================================================
// 1. Custom User-Defined Struct (Zero Core Edits, Driven by LumaResource)
// =============================================================================

#[derive(Clone, Debug, Default, LumaResource)]
#[resource("CustomMesh")]
pub struct CustomMesh {
    pub name: String,
    pub vertex_count: usize,
    pub color: LinearColor,
}

// =============================================================================
// 2. Pure Function Nodes Defined with `#[luma_node]` (Auto-Discovered!)
// =============================================================================

/// 1. Node that generates a `CustomMesh` without manual boxing or descriptor code
#[luma_node(category = "Geometry", title = "Sphere Generator", tag = "MESH")]
pub fn generate_sphere() -> CustomMesh {
    CustomMesh {
        name: "CyberSphere".into(),
        vertex_count: 1440,
        color: LinearColor::WHITE,
    }
}

/// 2. Node that receives `CustomMesh` + `LinearColor` and returns the tinted `CustomMesh`
/// Notice: Plain function parameters and return type!
#[luma_node(category = "Shader", title = "Apply Tint", tag = "TINT")]
pub fn apply_tint(mut mesh: CustomMesh, color: LinearColor) -> CustomMesh {
    mesh.color = color;
    mesh
}

// =============================================================================
// 3. Interactive UI Nodes (Color Palette & Live Mesh Inspector)
// =============================================================================

#[derive(Component)]
struct ColorPickerWidget {
    node_id: NodeId,
}

#[derive(Component)]
struct BtnSetColor(LinearColor);

#[derive(Component)]
struct MeshInspectorWidget {
    node_id: NodeId,
}

#[derive(Component)]
struct InspectorColorSwatch;

#[derive(Component)]
struct InspectorInfoText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Luma Editor — Macro & Custom Struct Showcase".into(),
                resolution: (1360, 840).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(EditorTheme::BG_CANVAS))
        // Notice: `generate_sphere` and `apply_tint` are NOT registered here!
        // The engine discovers all `#[luma_node]` entries automatically via inventory.
        .add_plugins((LumaPlugin, LumaEditorPlugin, LumaUiPlugin))
        // UI Node for interactive color picking:
        .register_node(
            NodeDef::new("palette.color_picker", "Color Preset")
                .category("Color", "PALETTE")
                .header_rgb(0.55, 0.22, 0.35)
                .width(220.0)
                .output(PinInfo::color("color"))
                .body(36.0, |ctx| {
                    let node_id = ctx.node_id();
                    let font = ctx.font();

                    let row = ctx
                        .spawn((
                            ColorPickerWidget { node_id },
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
                                gap: 6.0,
                                ..default()
                            },
                        ))
                        .id();

                    let presets = [
                        ("Cyan", LinearColor::new(0.20, 0.85, 0.95, 1.0)),
                        ("Rose", LinearColor::new(0.95, 0.25, 0.45, 1.0)),
                        ("Gold", LinearColor::new(0.95, 0.75, 0.15, 1.0)),
                        ("Lime", LinearColor::new(0.30, 0.90, 0.40, 1.0)),
                    ];

                    for (name, col) in presets {
                        ctx.commands()
                            .spawn((
                                ChildOf(row),
                                BtnSetColor(col),
                                UNode {
                                    width: UVal::Px(42.0),
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
                                    font_size: 8.5,
                                    font: font.clone(),
                                    color: Color::BLACK,
                                    ..default()
                                });
                            });
                    }
                }),
        )
        // UI Node for live mesh inspection:
        .register_node(
            NodeDef::new("mesh.inspector", "Mesh Inspector")
                .category("Output", "VIEW")
                .header_rgb(0.22, 0.44, 0.30)
                .width(240.0)
                .input(PinInfo::custom("mesh", "CustomMesh"))
                .body(54.0, |ctx| {
                    let node_id = ctx.node_id();
                    let font = ctx.font();

                    ctx.spawn((
                        MeshInspectorWidget { node_id },
                        UNode {
                            width: UVal::Percent(1.0),
                            height: UVal::Px(46.0),
                            padding: USides::axes(6.0, 4.0),
                            background_color: Color::srgb(0.09, 0.11, 0.13),
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
                    .with_children(|row| {
                        row.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(2.0),
                            ..default()
                        })
                        .with_children(|col| {
                            col.spawn((
                                InspectorInfoText,
                                UText {
                                    text: "Mesh: Waiting...".into(),
                                    font_size: 9.5,
                                    font: font.clone(),
                                    color: Color::srgb(0.88, 0.90, 0.94),
                                    ..default()
                                },
                            ));
                        });

                        row.spawn((
                            InspectorColorSwatch,
                            UNode {
                                width: UVal::Px(36.0),
                                height: UVal::Px(36.0),
                                background_color: Color::WHITE,
                                border_radius: UCornerRadius::all(4.0),
                                ..default()
                            },
                            UBorder {
                                color: Color::srgb(0.30, 0.35, 0.40),
                                width: 1.0,
                                radius: UCornerRadius::all(4.0),
                                offset: 0.0,
                            },
                        ));
                    });
                }),
        )
        .add_systems(Startup, setup_pipeline)
        .add_systems(Update, (sync_color_picker_system, sync_mesh_inspector_system))
        .run();
}

fn setup_pipeline(mut commands: Commands, mut graph: LumaGraph) {
    commands.spawn((
        Camera2d,
        EditorCamera,
        EditorUiCamera,
        Transform::from_xyz(-60.0, 20.0, 0.0),
    ));

    // Spawn the function nodes generated by `#[luma_node]`!
    let n_mesh_gen = graph.spawn("generate_sphere", Vec2::new(-380.0, 100.0));
    let n_color_picker = graph.spawn("palette.color_picker", Vec2::new(-380.0, -80.0));
    let n_tint = graph.spawn("apply_tint", Vec2::new(-80.0, 20.0));
    let n_inspector = graph.spawn("mesh.inspector", Vec2::new(220.0, 20.0));

    // Connect Wires:
    //    generate_sphere (out 0) -> apply_tint (in 0 "mesh")
    //    color_picker   (out 0) -> apply_tint (in 1 "color")
    //    apply_tint     (out 0) -> mesh.inspector (in 0 "mesh")
    graph.connect(n_mesh_gen, 0, n_tint, 0);
    graph.connect(n_color_picker, 0, n_tint, 1);
    graph.connect(n_tint, 0, n_inspector, 0);

    // Initial color push from the preset
    graph.set_output_color(n_color_picker, 0, LinearColor::new(0.20, 0.85, 0.95, 1.0));

    graph.select(n_tint);
}

fn sync_color_picker_system(
    mut graph: LumaGraph,
    btn_query: Query<(&UInteraction, &BtnSetColor), Changed<UInteraction>>,
    picker_query: Query<&ColorPickerWidget>,
) {
    for (interaction, btn_color) in &btn_query {
        if *interaction == UInteraction::Pressed {
            for picker in &picker_query {
                graph.set_output_color(picker.node_id, 0, btn_color.0);
            }
        }
    }
}

fn sync_mesh_inspector_system(
    graph: LumaGraph,
    inspectors: Query<&MeshInspectorWidget>,
    mut swatches: Query<&mut UNode, With<InspectorColorSwatch>>,
    mut text_query: Query<&mut UText, With<InspectorInfoText>>,
) {
    for inspector in &inspectors {
        let val = graph.input_value(inspector.node_id, 0);

        if let Some(mesh) = val.downcast_resource::<CustomMesh>() {
            for mut swatch in &mut swatches {
                swatch.background_color = Color::srgba(
                    mesh.color.r,
                    mesh.color.g,
                    mesh.color.b,
                    mesh.color.a,
                );
            }
            for mut utext in &mut text_query {
                utext.text = format!("{}: {} verts", mesh.name, mesh.vertex_count);
            }
        }
    }
}
