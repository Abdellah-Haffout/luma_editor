use luma_core::id::{NodeId, PinId};
use luma_dataflow::descriptor::NodeBodyStyle;
use luma_canvas::nodes::card::NodeCardLayout;
use luma_canvas::nodes::custom::CustomNodeContent;
use bevy::prelude::*;
use bevy_luma::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Callback invoked when spawning a node card that has a registered custom body UI builder.
/// Receives `(commands, body_slot_entity, layout, font)` so external crates or examples can
/// mount arbitrary widgets (`UTextInput`, `UImage`, etc.) inside the node card without modifying the core!
pub type NodeBodyUiBuilder =
    Arc<dyn Fn(&mut Commands, Entity, &NodeCardLayout, Handle<Font>) + Send + Sync>;

/// Extensible registry allowing applications and examples to register custom body UI builders
/// for any node `kind` without touching the core node-graph infrastructure.
#[derive(Resource, Default, Clone)]
pub struct NodeBodyUiRegistry {
    builders: HashMap<String, NodeBodyUiBuilder>,
}

impl NodeBodyUiRegistry {
    /// Registers a custom UI builder callback for a specific node `kind`.
    pub fn register<F>(&mut self, kind: impl Into<String>, builder: F)
    where
        F: Fn(&mut Commands, Entity, &NodeCardLayout, Handle<Font>) + Send + Sync + 'static,
    {
        self.builders.insert(kind.into(), Arc::new(builder));
    }

    /// Retrieves the registered custom UI builder for `kind`, if any.
    pub fn get(&self, kind: &str) -> Option<&NodeBodyUiBuilder> {
        self.builders.get(kind)
    }
}

#[derive(Component, Clone, Copy, PartialEq, Debug, Default)]
pub struct NodeCardVisual {
    pub node_id: NodeId,
    pub type_color: Color,
    pub header_bg: Color,
    pub body_bg: Color,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
pub struct NodeCardHeaderBar {
    pub node_id: NodeId,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
pub struct NodeCardRoot {
    pub node_id: NodeId,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
pub struct DiveInButton {
    pub node_id: NodeId,
}

/// Interactive value display label for parameter nodes (e.g. math.number).
#[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
pub struct NodeParamValueText {
    pub node_id: NodeId,
}

/// Interactive step button (- / +) for adjusting parameter values.
#[derive(Component, Clone, Copy, PartialEq, Default)]
pub struct NodeParamStepButton {
    pub node_id: NodeId,
    pub delta: f32,
}

/// Slot marker for injecting custom node UI widgets into a card's body.
#[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
pub struct NodeCustomBodySlot {
    pub node_id: NodeId,
}

/// Marker component for an input slot text label on a node card.
#[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
pub struct NodeInputValLabel {
    pub node_id: NodeId,
    pub slot_idx: usize,
}

/// Marker component for an output slot text label on a node card.
#[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
pub struct NodeOutputValLabel {
    pub node_id: NodeId,
    pub slot_idx: usize,
}

/// Marker component for an interactive pin socket circle on a node card.
#[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
pub struct NodeCardSocketPin {
    pub node_id: NodeId,
    pub pin_id: Option<PinId>,
}

/// Marker component for the live received-inputs readout line inside a Math node's body.
#[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
pub struct NodeLiveInputReadoutText {
    pub node_id: NodeId,
}

/// Marker component for the live computed-output readout line inside a Math node's body.
#[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
pub struct NodeLiveOutputReadoutText {
    pub node_id: NodeId,
}

/// Maps data type name to authentic Blender socket type colors.
pub fn get_blender_pin_color(type_name: &str) -> Color {
    match type_name {
        "Float" | "f32" | "Number" => Color::srgb(0.65, 0.65, 0.65),    // Silver / Gray (Float)
        "Int" | "Integer" => Color::srgb(0.35, 0.68, 0.78),             // Cyan (Integer)
        "Vec2" | "Vec3" | "Vec4" | "Vector" | "Float3" => Color::srgb(0.39, 0.39, 0.78), // Blue/Indigo (Vector)
        "Color" | "RGBA" => Color::srgb(0.78, 0.78, 0.16),              // Yellow (Color)
        "String" => Color::srgb(0.55, 0.75, 0.95),                      // Light Blue (String)
        "Shader" | "Material" => Color::srgb(0.39, 0.78, 0.39),         // Green (Shader/Material)
        "Geometry" | "Mesh" => Color::srgb(0.0, 0.84, 0.64),            // Emerald/Teal (Geometry)
        "Entity" | "Object" => Color::srgb(0.90, 0.54, 0.18),           // Orange (Object/Entity)
        "Asset" | "AssetRef" | "Model" => Color::srgb(1.0, 0.69, 0.12), // Amber (Asset)
        "Bool" | "Boolean" => Color::srgb(0.85, 0.40, 0.60),            // Pink (Boolean)
        _ => Color::srgb(0.80, 0.80, 0.80),                             // Untyped / Any
    }
}

/// Spawns a physical in-world 2D node card (`URootUi::world_2d`) using bevy_luma SDF widgets.
///
/// Features:
/// - Pure 2D World-Space rendering via `URootUi::world_2d(layout.size)`.
/// - Zooming is handled natively by the Bevy 2D Camera (zero CPU re-layouts, zero text pixelation).
/// - Anti-aliased SDF borders, rounded 8px corners, and subtle drop shadows.
/// - Crisp typography using embedded vector fonts (`UText`).
/// - Interactive socket pins with outer ring and glowing core positioned on the card edge.
/// - Center parameter steppers `[-] [value] [+]` with `UInteraction`.
pub fn spawn_luma_node_card(
    commands: &mut Commands,
    font: Handle<Font>,
    layout: &NodeCardLayout,
    is_selected: bool,
) -> Entity {
    spawn_luma_node_card_with_registry(commands, font, layout, is_selected, None)
}

pub fn spawn_luma_node_card_with_registry(
    commands: &mut Commands,
    font: Handle<Font>,
    layout: &NodeCardLayout,
    is_selected: bool,
    ui_registry: Option<&NodeBodyUiRegistry>,
) -> Entity {
    let mut root_ui = URootUi::world_2d(layout.size);
    root_ui.meters_per_unit = 1.0;
    root_ui.resolution_scale = 2.0;

    let radius = layout.corner_radius;
    let blender_outline_unselected = Color::srgb(0.075, 0.075, 0.085);
    let mut card_cmd = commands.spawn((
        root_ui,
        Transform::from_xyz(layout.position.x, layout.position.y, 0.0),
        NodeCardRoot { node_id: layout.node_id },
        NodeCardVisual {
            node_id: layout.node_id,
            type_color: layout.accent_color,
            header_bg: layout.header_bg,
            body_bg: layout.body_bg,
        },
        UNode {
            width: UVal::Px(layout.size.x),
            height: UVal::Px(layout.size.y),
            background_color: layout.body_bg,
            border_radius: UCornerRadius::all(radius),
            ..default()
        },
        UBorder {
            color: if is_selected {
                layout.accent_color
            } else {
                blender_outline_unselected
            },
            width: if is_selected { 2.0 } else { 1.4 },
            radius: UCornerRadius::all(radius),
            offset: 0.0,
        },
        UShadow {
            color: Color::srgba(0.0, 0.0, 0.0, 0.55),
            offset: Vec2::new(0.0, -4.0),
            blur: 12.0,
            spread: 0.0,
            is_inner: false,
        },
        ULayout {
            display: UDisplay::Flex,
            flex_direction: UFlexDirection::Column,
            ..default()
        },
    ));

    card_cmd.with_children(|card| {
        // A. Blender Node Header Banner (27px colored category header + collapse arrow + title + badge)
        card.spawn((
            NodeCardHeaderBar { node_id: layout.node_id },
            UNode {
                width: UVal::Percent(1.0),
                height: UVal::Px(27.0),
                background_color: layout.header_bg,
                border_radius: UCornerRadius::top(radius),
                padding: USides::axes(8.0, 4.0),
                ..default()
            },
            UBorder {
                color: Color::NONE,
                width: 0.0,
                radius: UCornerRadius::top(radius),
                offset: 0.0,
            },
            ULayout {
                display: UDisplay::Flex,
                flex_direction: UFlexDirection::Row,
                align_items: UAlignItems::Center,
                justify_content: UJustifyContent::SpaceBetween,
                ..default()
            },
        )).with_children(|header| {
            // Left: Blender Collapse Triangle (▼) + Crisp White Node Title
            header.spawn((
                UNode::default(),
                ULayout {
                    display: UDisplay::Flex,
                    flex_direction: UFlexDirection::Row,
                    align_items: UAlignItems::Center,
                    gap: 6.0,
                    ..default()
                },
            )).with_children(|left| {
                // Blender Collapse Chevron
                left.spawn(UText {
                    text: "▼".to_string(),
                    font_size: 8.0,
                    font: font.clone(),
                    color: Color::srgba(0.92, 0.92, 0.95, 0.80),
                    ..default()
                });

                // Node Title
                left.spawn(UText {
                    text: layout.title.clone(),
                    font_size: 11.5,
                    font: font.clone(),
                    color: Color::srgb(0.96, 0.96, 0.97),
                    ..default()
                });
            });

            // Right: Dive In Button (for Subgraphs) or Subtle Blender Category Pill
            if layout.kind == "Subgraph" {
                header.spawn((
                    DiveInButton { node_id: layout.node_id },
                    UNode {
                        background_color: Color::srgba(0.0, 0.0, 0.0, 0.35),
                        border_radius: UCornerRadius::all(4.0),
                        padding: USides::axes(6.0, 2.0),
                        ..default()
                    },
                    UInteraction::default(),
                    UInteractionColors {
                        normal: Color::srgba(0.0, 0.0, 0.0, 0.35),
                        hovered: Color::srgba(0.0, 0.0, 0.0, 0.55),
                        pressed: Color::srgba(0.0, 0.0, 0.0, 0.70),
                    },
                )).with_children(|btn| {
                    btn.spawn(UText {
                        text: "[Dive In] >".to_string(),
                        font_size: 9.5,
                        font: font.clone(),
                        color: Color::WHITE,
                        ..default()
                    });
                });
            } else {
                header.spawn((
                    UNode {
                        background_color: Color::srgba(0.0, 0.0, 0.0, 0.25),
                        border_radius: UCornerRadius::all(3.0),
                        padding: USides::axes(4.5, 1.5),
                        ..default()
                    },
                )).with_children(|badge| {
                    badge.spawn(UText {
                        text: layout.category_tag.clone(),
                        font_size: 8.5,
                        font: font.clone(),
                        color: Color::srgba(0.95, 0.95, 0.97, 0.85),
                        ..default()
                    });
                });
            }
        });

        // 1px Dark Header Separator Line (authentic Blender header-to-body divider)
        card.spawn(UNode {
            width: UVal::Percent(1.0),
            height: UVal::Px(1.0),
            background_color: Color::srgb(0.08, 0.08, 0.09),
            ..default()
        });

        // B. Body Content (Slots, Pins & Parameter Steppers)
        card.spawn((
            UNode {
                width: UVal::Percent(1.0),
                padding: USides {
                    left: 0.0,
                    right: 0.0,
                    top: 6.0,
                    bottom: 6.0,
                },
                ..default()
            },
            ULayout {
                display: UDisplay::Flex,
                flex_direction: UFlexDirection::Column,
                gap: 2.0,
                ..default()
            },
        )).with_children(|body| {
            let num_rows = layout.input_pins.len().max(layout.output_pins.len()).max(
                if layout.body_style == NodeBodyStyle::Parameters { 1 } else { 0 }
            );

            for row_idx in 0..num_rows {
                body.spawn((
                    UNode {
                        width: UVal::Percent(1.0),
                        height: UVal::Px(24.0),
                        ..default()
                    },
                    ULayout {
                        display: UDisplay::Flex,
                        flex_direction: UFlexDirection::Row,
                        align_items: UAlignItems::Center,
                        justify_content: UJustifyContent::SpaceBetween,
                        ..default()
                    },
                )).with_children(|row| {
                    // 1. Left: Input Pin Dot + Label
                    if let Some(in_pin) = layout.input_pins.get(row_idx) {
                        let is_connected = layout.is_pin_connected(in_pin.id);
                        let color = get_blender_pin_color(&in_pin.data_type.name);

                        row.spawn((
                            UNode::default(),
                            ULayout {
                                display: UDisplay::Flex,
                                flex_direction: UFlexDirection::Row,
                                align_items: UAlignItems::Center,
                                gap: 6.0,
                                ..default()
                            },
                        )).with_children(|in_group| {
                            // Blender Socket Dot (solid data-type fill + dark #121214 outline ring)
                            in_group.spawn((
                                NodeCardSocketPin {
                                    node_id: layout.node_id,
                                    pin_id: Some(in_pin.id),
                                },
                                UNode {
                                    width: UVal::Px(10.0),
                                    height: UVal::Px(10.0),
                                    background_color: color,
                                    border_radius: UCornerRadius::all(5.0),
                                    margin: USides { left: -5.0, ..default() },
                                    ..default()
                                },
                                UBorder {
                                    color: if is_connected {
                                        Color::WHITE
                                    } else {
                                        Color::srgb(0.07, 0.07, 0.08)
                                    },
                                    width: 1.6,
                                    radius: UCornerRadius::all(5.0),
                                    offset: 0.0,
                                },
                                UInteraction::default(),
                            ));

                            // Label
                            let text_val = layout.input_display_texts.get(row_idx).cloned().unwrap_or_else(|| in_pin.name.clone());
                            in_group.spawn((
                                NodeInputValLabel {
                                    node_id: layout.node_id,
                                    slot_idx: row_idx,
                                },
                                UText {
                                    text: text_val,
                                    font_size: 10.5,
                                    font: font.clone(),
                                    color: Color::srgb(0.88, 0.88, 0.90),
                                    ..default()
                                },
                            ));
                        });
                    } else {
                        row.spawn(UNode {
                            width: UVal::Px(0.0),
                            height: UVal::Px(0.0),
                            ..default()
                        });
                    }

                    // 2. Center: Math Parameter Number Display (Clean, centered, no surrounding shapes)
                    if layout.body_style == NodeBodyStyle::Parameters && row_idx == 0 {
                        let num_val = layout.number_value.unwrap_or(10.0);
                        let num_text = if num_val.fract() == 0.0 {
                            format!("{:.1}", num_val)
                        } else {
                            format!("{:.2}", num_val)
                        };

                        row.spawn((
                            UNode {
                                background_color: Color::NONE,
                                padding: USides::axes(4.0, 2.0),
                                ..default()
                            },
                            ULayout {
                                display: UDisplay::Flex,
                                flex_direction: UFlexDirection::Row,
                                justify_content: UJustifyContent::Center,
                                align_items: UAlignItems::Center,
                                ..default()
                            },
                        )).with_children(|center_slot| {
                            center_slot.spawn((
                                NodeParamValueText {
                                    node_id: layout.node_id,
                                },
                                UText {
                                    text: num_text,
                                    font_size: 11.5,
                                    font: font.clone(),
                                    color: Color::srgb(0.95, 0.95, 0.96),
                                    justify: Justify::Center,
                                    ..default()
                                },
                            ));
                        });
                    }

                    // 3. Right: Output Label + Pin Dot
                    if let Some(out_pin) = layout.output_pins.get(row_idx) {
                        let is_connected = layout.is_pin_connected(out_pin.id);
                        let color = get_blender_pin_color(&out_pin.data_type.name);

                        row.spawn((
                            UNode::default(),
                            ULayout {
                                display: UDisplay::Flex,
                                flex_direction: UFlexDirection::Row,
                                align_items: UAlignItems::Center,
                                gap: 6.0,
                                ..default()
                            },
                        )).with_children(|out_group| {
                            // Label
                            let text_val = layout.output_display_texts.get(row_idx).cloned().unwrap_or_else(|| out_pin.name.clone());
                            out_group.spawn((
                                NodeOutputValLabel {
                                    node_id: layout.node_id,
                                    slot_idx: row_idx,
                                },
                                UText {
                                    text: text_val,
                                    font_size: 10.5,
                                    font: font.clone(),
                                    color: Color::srgb(0.88, 0.88, 0.90),
                                    ..default()
                                },
                            ));

                            // Blender Socket Dot (solid data-type fill + dark #121214 outline ring)
                            out_group.spawn((
                                NodeCardSocketPin {
                                    node_id: layout.node_id,
                                    pin_id: Some(out_pin.id),
                                },
                                UNode {
                                    width: UVal::Px(10.0),
                                    height: UVal::Px(10.0),
                                    background_color: color,
                                    border_radius: UCornerRadius::all(5.0),
                                    margin: USides { right: -5.0, ..default() },
                                    ..default()
                                },
                                UBorder {
                                    color: if is_connected {
                                        Color::WHITE
                                    } else {
                                        Color::srgb(0.07, 0.07, 0.08)
                                    },
                                    width: 1.6,
                                    radius: UCornerRadius::all(5.0),
                                    offset: 0.0,
                                },
                                UInteraction::default(),
                            ));
                        });
                    } else {
                        row.spawn(UNode {
                            width: UVal::Px(0.0),
                            height: UVal::Px(0.0),
                            ..default()
                        });
                    }
                });
            }

            // Clean centered live number readout for Math Nodes without surrounding shapes
            if NodeCardLayout::has_math_value_readout(&layout.kind) {
                body.spawn((
                    UNode {
                        width: UVal::Percent(1.0),
                        height: UVal::Px(30.0),
                        background_color: Color::NONE,
                        margin: USides {
                            left: 6.0,
                            right: 6.0,
                            top: 2.0,
                            bottom: 2.0,
                        },
                        ..default()
                    },
                    ULayout {
                        display: UDisplay::Flex,
                        flex_direction: UFlexDirection::Column,
                        justify_content: UJustifyContent::Center,
                        align_items: UAlignItems::Center,
                        gap: 1.0,
                        ..default()
                    },
                ))
                .with_children(|readout| {
                    readout.spawn((
                        NodeLiveInputReadoutText {
                            node_id: layout.node_id,
                        },
                        UText {
                            text: String::new(),
                            font_size: 9.2,
                            font: font.clone(),
                            color: Color::srgb(0.70, 0.75, 0.82),
                            justify: Justify::Center,
                            ..default()
                        },
                    ));
                    readout.spawn((
                        NodeLiveOutputReadoutText {
                            node_id: layout.node_id,
                        },
                        UText {
                            text: "= 0".to_string(),
                            font_size: 12.0,
                            font: font.clone(),
                            color: Color::srgb(0.45, 0.88, 0.62),
                            justify: Justify::Center,
                            ..default()
                        },
                    ));
                });
            }
        });
    });

    let card_entity = card_cmd.id();

    // If a custom UI builder is registered for this node kind, invoke it on a dedicated slot container
    if let Some(reg) = ui_registry {
        if let Some(builder) = reg.get(&layout.kind) {
            let slot_h = match &layout.custom_content {
                CustomNodeContent::CustomWidget { height } => *height,
                _ => 36.0,
            };
            let slot_entity = commands
                .spawn((
                    ChildOf(card_entity),
                    NodeCustomBodySlot {
                        node_id: layout.node_id,
                    },
                    UNode {
                        width: UVal::Percent(1.0),
                        height: UVal::Px(slot_h),
                        padding: USides::axes(7.0, 3.0),
                        ..default()
                    },
                    ULayout {
                        display: UDisplay::Flex,
                        flex_direction: UFlexDirection::Column,
                        justify_content: UJustifyContent::Center,
                        gap: 3.0,
                        ..default()
                    },
                ))
                .id();
            builder(commands, slot_entity, layout, font);
        }
    }

    card_entity
}

/// Compatibility wrapper for older code calling `spawn_bevy_ui_node_card`.
pub fn spawn_bevy_ui_node_card(
    commands: &mut Commands,
    layout: &NodeCardLayout,
    is_selected: bool,
    _target_camera: Option<Entity>,
) -> Entity {
    spawn_luma_node_card(commands, Handle::default(), layout, is_selected)
}

/// Compatibility builder bundle.
pub fn node_card_scene(
    _font: Handle<Font>,
    layout: &NodeCardLayout,
    is_selected: bool,
) -> impl Bundle {
    let mut root_ui = URootUi::world_2d(layout.size);
    root_ui.meters_per_unit = 1.0;
    root_ui.resolution_scale = 2.0;

    (
        root_ui,
        Transform::from_xyz(layout.position.x, layout.position.y, 0.0),
        NodeCardRoot { node_id: layout.node_id },
        NodeCardVisual {
            node_id: layout.node_id,
            type_color: layout.accent_color,
            header_bg: layout.header_bg,
            body_bg: layout.body_bg,
        },
        UNode {
            width: UVal::Px(layout.size.x),
            height: UVal::Px(layout.size.y),
            background_color: layout.body_bg,
            border_radius: UCornerRadius::all(layout.corner_radius),
            ..default()
        },
        UBorder {
            color: if is_selected {
                layout.accent_color
            } else {
                Color::srgb(0.075, 0.075, 0.085)
            },
            width: if is_selected { 2.0 } else { 1.4 },
            radius: UCornerRadius::all(layout.corner_radius),
            offset: 0.0,
        },
    )
}

/// Continuously synchronizes the visual fill and border of socket dots
/// when wires are connected or disconnected in the editor graph.
pub fn update_node_socket_pins_system(
    graph_res: Res<luma_canvas::plugin::EditorGraphResource>,
    mut socket_query: Query<(&NodeCardSocketPin, &mut UNode, &mut UBorder)>,
) {
    if !graph_res.is_changed() {
        return;
    }
    let graph = &graph_res.0;
    for (socket, mut unode, mut border) in &mut socket_query {
        if let Some(pin_id) = socket.pin_id {
            if let Some(pin) = graph.get_pin(pin_id) {
                let is_connected = !graph.get_pin_connections(pin_id).is_empty();
                let color = get_blender_pin_color(&pin.data_type.name);
                unode.background_color = color;
                if is_connected {
                    border.color = Color::WHITE;
                    border.width = 1.6;
                } else {
                    border.color = Color::srgb(0.07, 0.07, 0.08);
                    border.width = 1.6;
                }
            }
        }
    }
}
