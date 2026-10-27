use luma_core::id::{NodeId, PinId};
use luma_dataflow::descriptor::NodeBodyStyle;
use luma_canvas::nodes::card::NodeCardLayout;
use crate::theme::EditorTheme;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

#[derive(Component, Clone, Copy, PartialEq, Debug, Default)]
pub struct NodeCardVisual {
    pub node_id: NodeId,
    pub type_color: Color,
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

/// Spawns a Blender-styled node card using pure bevy_ui with crisp borders,
/// colored header strip, input/output socket pins, and parameter steppers.
pub fn spawn_bevy_ui_node_card(
    commands: &mut Commands,
    layout: &NodeCardLayout,
    is_selected: bool,
    target_camera: Option<Entity>,
) -> Entity {
    let mut root_cmd = commands.spawn((
        NodeCardRoot { node_id: layout.node_id },
        NodeCardVisual {
            node_id: layout.node_id,
            type_color: layout.header_bg,
        },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Px(layout.size.x),
            height: Val::Px(layout.size.y),
            flex_direction: FlexDirection::Column,
            border: UiRect::all(Val::Px(if is_selected { 2.0 } else { 1.0 })),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            overflow: Overflow::visible(),
            ..default()
        },
        BackgroundColor(Color::srgb(0.133, 0.141, 0.157)),
        BorderColor::all(if is_selected {
            layout.header_bg
        } else {
            Color::srgba(0.086, 0.090, 0.102, 0.95)
        }),
        GlobalZIndex(20),
    ));

    if let Some(cam) = target_camera {
        root_cmd.insert(UiTargetCamera(cam));
    }

    root_cmd.with_children(|card| {
        // A. Header Bar (32px)
        card.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(32.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                border_radius: BorderRadius::top(Val::Px(7.0)),
                ..default()
            },
            BackgroundColor(layout.header_bg),
        )).with_children(|header| {
            // Left: Category badge + Title
            header.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                ..default()
            }).with_children(|left| {
                // Category Badge Pill
                left.spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.25)),
                )).with_children(|badge| {
                    badge.spawn((
                        Text::new(&layout.category_tag),
                        TextFont { font_size: FontSize::Px(9.0), ..default() },
                        TextColor(Color::WHITE),
                    ));
                });

                // Node Title
                left.spawn((
                    Text::new(&layout.title),
                    TextFont { font_size: FontSize::Px(12.0), ..default() },
                    TextColor(Color::WHITE),
                ));
            });

            // Right (if Subgraph): Dive In Button
            if layout.kind == "Subgraph" {
                header.spawn((
                    DiveInButton { node_id: layout.node_id },
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
                )).with_children(|btn| {
                    btn.spawn((
                        Text::new("[Dive In] >"),
                        TextFont { font_size: FontSize::Px(9.5), ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            }
        });

        // B. Body Content (Slots & Pins)
        card.spawn(Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(Val::Px(4.0), Val::Px(6.0)),
            row_gap: Val::Px(4.0),
            ..default()
        }).with_children(|body| {
            let num_rows = layout.input_pins.len().max(layout.output_pins.len()).max(
                if layout.body_style == NodeBodyStyle::ParameterStepper { 1 } else { 0 }
            );

            for row_idx in 0..num_rows {
                body.spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(24.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    position_type: PositionType::Relative,
                    ..default()
                }).with_children(|row| {
                    // 1. Left: Input Pin Dot + Label
                    if let Some(in_pin) = layout.input_pins.get(row_idx) {
                        let is_connected = layout.is_pin_connected(in_pin.id);
                        let color = get_blender_pin_color(&in_pin.data_type.name);

                        row.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(6.0),
                            margin: UiRect::left(Val::Px(-9.0)),
                            ..default()
                        }).with_children(|in_group| {
                            // Socket dot
                            in_group.spawn((
                                NodeCardSocketPin {
                                    node_id: layout.node_id,
                                    pin_id: Some(in_pin.id),
                                },
                                Node {
                                    width: Val::Px(10.0),
                                    height: Val::Px(10.0),
                                    border: UiRect::all(Val::Px(if is_connected { 1.0 } else { 2.0 })),
                                    border_radius: BorderRadius::all(Val::Px(5.0)),
                                    ..default()
                                },
                                BackgroundColor(if is_connected { color } else { Color::srgb(0.133, 0.141, 0.157) }),
                                BorderColor::all(color),
                            ));

                            // Label
                            let text_val = layout.input_display_texts.get(row_idx).cloned().unwrap_or_else(|| in_pin.name.clone());
                            in_group.spawn((
                                NodeInputValLabel {
                                    node_id: layout.node_id,
                                    slot_idx: row_idx,
                                },
                                Text::new(text_val),
                                TextFont { font_size: FontSize::Px(10.5), ..default() },
                                TextColor(Color::srgb(0.85, 0.88, 0.92)),
                            ));
                        });
                    } else {
                        row.spawn(Node::default());
                    }

                    // 2. Center: Parameter Stepper [-] [Value] [+]
                    if layout.body_style == NodeBodyStyle::ParameterStepper && row_idx == 0 {
                        row.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.0),
                            padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        }).with_children(|stepper| {
                            // [-] button
                            stepper.spawn((
                                NodeParamStepButton {
                                    node_id: layout.node_id,
                                    delta: -1.0,
                                },
                                Button,
                                Node {
                                    width: Val::Px(16.0),
                                    height: Val::Px(16.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(Val::Px(3.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
                            )).with_children(|b| {
                                b.spawn((
                                    Text::new("-"),
                                    TextFont { font_size: FontSize::Px(11.0), ..default() },
                                    TextColor(Color::WHITE),
                                ));
                            });

                            // Value text
                            stepper.spawn((
                                NodeParamValueText {
                                    node_id: layout.node_id,
                                },
                                Text::new("10.0"),
                                TextFont { font_size: FontSize::Px(10.5), ..default() },
                                TextColor(Color::srgb(0.95, 0.85, 0.40)),
                            ));

                            // [+] button
                            stepper.spawn((
                                NodeParamStepButton {
                                    node_id: layout.node_id,
                                    delta: 1.0,
                                },
                                Button,
                                Node {
                                    width: Val::Px(16.0),
                                    height: Val::Px(16.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(Val::Px(3.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
                            )).with_children(|b| {
                                b.spawn((
                                    Text::new("+"),
                                    TextFont { font_size: FontSize::Px(11.0), ..default() },
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });
                    }

                    // 3. Right: Output Label + Pin Dot
                    if let Some(out_pin) = layout.output_pins.get(row_idx) {
                        let is_connected = layout.is_pin_connected(out_pin.id);
                        let color = get_blender_pin_color(&out_pin.data_type.name);

                        row.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(6.0),
                            margin: UiRect::right(Val::Px(-9.0)),
                            ..default()
                        }).with_children(|out_group| {
                            // Label
                            let text_val = layout.output_display_texts.get(row_idx).cloned().unwrap_or_else(|| out_pin.name.clone());
                            out_group.spawn((
                                NodeOutputValLabel {
                                    node_id: layout.node_id,
                                    slot_idx: row_idx,
                                },
                                Text::new(text_val),
                                TextFont { font_size: FontSize::Px(10.5), ..default() },
                                TextColor(Color::srgb(0.85, 0.88, 0.92)),
                            ));

                            // Socket dot
                            out_group.spawn((
                                NodeCardSocketPin {
                                    node_id: layout.node_id,
                                    pin_id: Some(out_pin.id),
                                },
                                Node {
                                    width: Val::Px(10.0),
                                    height: Val::Px(10.0),
                                    border: UiRect::all(Val::Px(if is_connected { 1.0 } else { 2.0 })),
                                    border_radius: BorderRadius::all(Val::Px(5.0)),
                                    ..default()
                                },
                                BackgroundColor(if is_connected { color } else { Color::srgb(0.133, 0.141, 0.157) }),
                                BorderColor::all(color),
                            ));
                        });
                    } else {
                        row.spawn(Node::default());
                    }
                });
            }
        });
    });

    root_cmd.id()
}

/// Compatibility builder bundle for test suites.
pub fn node_card_scene(
    _font: Handle<Font>,
    layout: &NodeCardLayout,
    is_selected: bool,
) -> impl Bundle {
    (
        NodeCardRoot { node_id: layout.node_id },
        NodeCardVisual {
            node_id: layout.node_id,
            type_color: layout.header_bg,
        },
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(layout.size.x),
            height: Val::Px(layout.size.y),
            border: UiRect::all(Val::Px(if is_selected { 2.0 } else { 1.0 })),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.133, 0.141, 0.157)),
    )
}

/// Continuously synchronizes the visual fill and border of socket dots
/// when wires are connected or disconnected in the editor graph.
pub fn update_node_socket_pins_system(
    graph_res: Res<luma_canvas::plugin::EditorGraphResource>,
    mut socket_query: Query<(&NodeCardSocketPin, &mut BackgroundColor, &mut BorderColor)>,
) {
    if !graph_res.is_changed() {
        return;
    }
    let graph = &graph_res.0;
    for (socket, mut bg, mut border) in &mut socket_query {
        if let Some(pin_id) = socket.pin_id {
            if let Some(pin) = graph.get_pin(pin_id) {
                let is_connected = !graph.get_pin_connections(pin_id).is_empty();
                let color = get_blender_pin_color(&pin.data_type.name);
                if is_connected {
                    *bg = BackgroundColor(color);
                    *border = BorderColor::all(color);
                } else {
                    *bg = BackgroundColor(Color::srgb(0.133, 0.141, 0.157));
                    *border = BorderColor::all(color);
                }
            }
        }
    }
}
