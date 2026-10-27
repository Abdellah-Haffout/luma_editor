use luma_canvas::nodes::custom::set_node_instance_customization;
use luma_canvas::plugin::{
    CameraAnimationResource, EditorCamera, EditorDataflowResource, EditorDomainPayloadResource,
    EditorGraphResource, EditorHistoryResource, EditorIdGenResource, EditorSelectionResource,
};
use luma_core::history::{AddNodeCommand, RemoveNodeCommand};
use luma_dataflow::value::LinearColor;
use crate::components::node_card::NodeCardRoot;
use crate::jackdaw_widgets::{
    jackdaw_tokens, spawn_separator, SeparatorProps,
};
use crate::plugin::EditorNavigationResource;
use crate::theme::EditorTheme;
use crate::{UiButton as Button, UiInteraction as Interaction};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

#[derive(Component)]
pub struct BevyUiInspectorRoot;

#[derive(Component)]
pub struct BevyUiInspectorTitleText;

#[derive(Component)]
pub struct BevyUiInspectorKindBadge;

#[derive(Component)]
pub struct BevyUiInspectorKindText;

#[derive(Component)]
pub struct BevyUiInspectorPosText;

#[derive(Component)]
pub struct BevyUiInspectorPinsText;

#[derive(Component)]
pub struct BevyUiInspectorButton;

#[derive(Component)]
pub struct BevyUiInspectorBtnClose;

#[derive(Component)]
pub struct BevyUiInspectorBtnFocus;

#[derive(Component)]
pub struct BevyUiInspectorBtnDuplicate;

#[derive(Component)]
pub struct BevyUiInspectorBtnDelete;

#[derive(Component)]
pub struct InspectorParamSection;

#[derive(Component)]
pub struct InspectorParamValText;

#[derive(Component)]
pub struct InspectorBtnParamStep(pub f32);

/// Customizes the selected node's accent color strip & border.
#[derive(Component, Clone, Copy)]
pub struct InspectorBtnCustomAccent(pub [f32; 3]);

/// Customizes the selected node's card body background color.
#[derive(Component, Clone, Copy)]
pub struct InspectorBtnCustomBody(pub [f32; 3]);

fn spawn_inspector_btn<B: Component>(
    parent: &mut ChildSpawnerCommands,
    font: FontSource,
    label: &str,
    bundle_comp: B,
) {
    parent
        .spawn((
            bundle_comp,
            BevyUiInspectorButton,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(jackdaw_tokens::BORDER_RADIUS_SM)),
                border: UiRect::ZERO,
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.18, 0.27)),
            BorderColor::all(Color::NONE),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font,
                    font_size: FontSize::Px(10.0),
                    ..default()
                },
                TextColor(Color::srgb(0.90, 0.93, 0.98)),
            ));
        });
}

/// Spawns the native Bevy UI Node Inspector side-panel.
pub fn setup_bevy_ui_inspector(
    mut commands: Commands,
    camera_q: Query<Entity, With<crate::EditorUiCamera>>,
) {
    let cam_opt = camera_q.iter().next();
    let font = FontSource::default();

    let mut inspector_cmd = commands.spawn((
        BevyUiInspectorRoot,
        Button,
        GlobalZIndex(140),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(72.0),
            right: Val::Px(16.0),
            width: Val::Px(290.0),
            max_height: Val::Px(720.0),
            padding: UiRect::all(Val::Px(14.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            border: UiRect::ZERO,
            border_radius: BorderRadius::all(Val::Px(12.0)),
            display: Display::None, // initially hidden until a node is selected
            ..default()
        },
        BackgroundColor(EditorTheme::WINDOW_BG),
        BorderColor::all(Color::NONE),
    ));

    if let Some(cam) = cam_opt {
        inspector_cmd.insert(UiTargetCamera(cam));
    }

    inspector_cmd.with_children(|panel| {
        // 1. Header (Title + Close button)
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::bottom(Val::Px(4.0)),
                border: UiRect::ZERO,
                ..default()
            })
            .insert(BorderColor::all(Color::NONE))
            .with_children(|hdr| {
                hdr.spawn((
                    Text::new("NODE INSPECTOR"),
                    TextFont {
                        font: font.clone(),
                        font_size: FontSize::Px(11.5),
                        ..default()
                    },
                    TextColor(jackdaw_tokens::TEXT_SECONDARY),
                ));

                hdr.spawn((
                    BevyUiInspectorBtnClose,
                    BevyUiInspectorButton,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(7.0), Val::Px(3.0)),
                        border: UiRect::ZERO,
                        border_radius: BorderRadius::all(Val::Px(jackdaw_tokens::BORDER_RADIUS_SM)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.18, 0.26)),
                    BorderColor::all(Color::NONE),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("X"),
                        TextFont {
                            font: font.clone(),
                            font_size: FontSize::Px(10.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            });

        spawn_separator(panel, SeparatorProps::horizontal());

        // 2. Node Identity & Kind Badge
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    BevyUiInspectorKindBadge,
                    Node {
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(2.5)),
                        border_radius: BorderRadius::all(Val::Px(jackdaw_tokens::BORDER_RADIUS_SM)),
                        border: UiRect::ZERO,
                        ..default()
                    },
                    BackgroundColor(jackdaw_tokens::CHIP_BG),
                    BorderColor::all(Color::NONE),
                ))
                .with_children(|badge| {
                    badge.spawn((
                        BevyUiInspectorKindText,
                        Text::new("NODE"),
                        TextFont {
                            font: font.clone(),
                            font_size: FontSize::Px(10.0),
                            ..default()
                        },
                        TextColor(jackdaw_tokens::CHIP_TEXT),
                    ));
                });

                row.spawn((
                    BevyUiInspectorTitleText,
                    Text::new("Selected Node"),
                    TextFont {
                        font: font.clone(),
                        font_size: FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

        spawn_separator(panel, SeparatorProps::horizontal());

        // 3. Properties info
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(jackdaw_tokens::BORDER_RADIUS_SM)),
                border: UiRect::ZERO,
                ..default()
            })
            .insert(BackgroundColor(Color::srgb(0.04, 0.05, 0.08)))
            .insert(BorderColor::all(Color::NONE))
            .with_children(|props| {
                props.spawn((
                    BevyUiInspectorPosText,
                    Text::new("Position: (0, 0)"),
                    TextFont {
                        font: font.clone(),
                        font_size: FontSize::Px(11.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.70, 0.75, 0.85)),
                ));

                props.spawn((
                    BevyUiInspectorPinsText,
                    Text::new("Pins: 0 in, 0 out"),
                    TextFont {
                        font: font.clone(),
                        font_size: FontSize::Px(11.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.70, 0.75, 0.85)),
                ));
            });

        // 4. Generic Numeric Parameter Section (shown whenever the node has a numeric input pin)
        panel
            .spawn((
                InspectorParamSection,
                Node {
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    border: UiRect::ZERO,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.04, 0.05, 0.08)),
                BorderColor::all(Color::NONE),
            ))
            .with_children(|sec| {
                sec.spawn((
                    Text::new("NUMERIC PARAMETER"),
                    TextFont {
                        font: font.clone(),
                        font_size: FontSize::Px(10.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.40, 0.70, 1.0)),
                ));

                sec.spawn((
                    InspectorParamValText,
                    Text::new("Value: 0.00"),
                    TextFont {
                        font: font.clone(),
                        font_size: FontSize::Px(12.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                sec.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_inspector_btn(row, font.clone(), "-10", InspectorBtnParamStep(-10.0));
                    spawn_inspector_btn(row, font.clone(), "-1", InspectorBtnParamStep(-1.0));
                    spawn_inspector_btn(row, font.clone(), "+1", InspectorBtnParamStep(1.0));
                    spawn_inspector_btn(row, font.clone(), "+10", InspectorBtnParamStep(10.0));
                });
            });

        spawn_separator(panel, SeparatorProps::horizontal());

        // 4.5 Node Appearance & Content Customizer (Always available for any selected node)
        panel
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    border_radius: BorderRadius::all(Val::Px(jackdaw_tokens::BORDER_RADIUS_SM)),
                    border: UiRect::ZERO,
                    ..default()
                },
                BackgroundColor(jackdaw_tokens::WINDOW_BG),
            ))
            .with_children(|cust| {
                cust.spawn((
                    Text::new("Customize Appearance & Content"),
                    TextFont {
                        font: font.clone(),
                        font_size: FontSize::Px(10.5),
                        ..default()
                    },
                    TextColor(jackdaw_tokens::ACCENT_BLUE),
                ));

                // Accent Color Row
                cust.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_inspector_btn(
                        row,
                        font.clone(),
                        "Blue",
                        InspectorBtnCustomAccent([0.126, 0.431, 0.784]),
                    );
                    spawn_inspector_btn(
                        row,
                        font.clone(),
                        "Rose",
                        InspectorBtnCustomAccent([0.94, 0.31, 0.53]),
                    );
                    spawn_inspector_btn(
                        row,
                        font.clone(),
                        "Green",
                        InspectorBtnCustomAccent([0.26, 0.73, 0.51]),
                    );
                    spawn_inspector_btn(
                        row,
                        font.clone(),
                        "Amber",
                        InspectorBtnCustomAccent([0.96, 0.65, 0.14]),
                    );
                });

                // Body Surface Theme Row
                cust.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_inspector_btn(
                        row,
                        font.clone(),
                        "Slate",
                        InspectorBtnCustomBody([0.141, 0.149, 0.169]),
                    );
                    spawn_inspector_btn(
                        row,
                        font.clone(),
                        "Navy",
                        InspectorBtnCustomBody([0.11, 0.14, 0.21]),
                    );
                    spawn_inspector_btn(
                        row,
                        font.clone(),
                        "Plum",
                        InspectorBtnCustomBody([0.17, 0.12, 0.20]),
                    );
                    spawn_inspector_btn(
                        row,
                        font.clone(),
                        "Carbon",
                        InspectorBtnCustomBody([0.09, 0.09, 0.10]),
                    );
                });
            });

        spawn_separator(panel, SeparatorProps::horizontal());

        // 5. Action Buttons (Focus, Duplicate, Delete)
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::top(Val::Px(4.0)),
                ..default()
            })
            .with_children(|actions| {
                // Focus / Dive-In Button
                actions
                    .spawn((
                        BevyUiInspectorBtnFocus,
                        BevyUiInspectorButton,
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::ZERO,
                            border_radius: BorderRadius::all(Val::Px(jackdaw_tokens::BORDER_RADIUS_SM)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.18, 0.22, 0.35)),
                        BorderColor::all(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Focus Node [Tab]"),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(11.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });

                // Duplicate Button
                actions
                    .spawn((
                        BevyUiInspectorBtnDuplicate,
                        BevyUiInspectorButton,
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::ZERO,
                            border_radius: BorderRadius::all(Val::Px(jackdaw_tokens::BORDER_RADIUS_SM)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.14, 0.17, 0.25)),
                        BorderColor::all(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Duplicate Node"),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(11.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.85, 0.90, 0.98)),
                        ));
                    });

                // Delete Button
                actions
                    .spawn((
                        BevyUiInspectorBtnDelete,
                        BevyUiInspectorButton,
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::ZERO,
                            border_radius: BorderRadius::all(Val::Px(jackdaw_tokens::BORDER_RADIUS_SM)),
                            ..default()
                        },
                        BackgroundColor(jackdaw_tokens::BUTTON_DESTRUCTIVE_BG),
                        BorderColor::all(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Delete Node [Del]"),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(11.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            });
    });
}

/// Dynamically updates inspector panel visibility and generic node telemetry based on the active selection.
pub fn update_bevy_ui_inspector_system(
    selection_res: Res<EditorSelectionResource>,
    graph_res: Res<EditorGraphResource>,
    dataflow_res: Res<EditorDataflowResource>,
    mut panel_query: Query<&mut Node, With<BevyUiInspectorRoot>>,
    mut badge_bg_query: Query<&mut BackgroundColor, With<BevyUiInspectorKindBadge>>,
    mut param_section_query: Query<&mut Node, (With<InspectorParamSection>, Without<BevyUiInspectorRoot>)>,
    mut text_query: Query<
        (
            &mut Text,
            Option<&BevyUiInspectorTitleText>,
            Option<&BevyUiInspectorKindText>,
            Option<&BevyUiInspectorPosText>,
            Option<&BevyUiInspectorPinsText>,
            Option<&InspectorParamValText>,
        ),
        Or<(
            With<BevyUiInspectorTitleText>,
            With<BevyUiInspectorKindText>,
            With<BevyUiInspectorPosText>,
            With<BevyUiInspectorPinsText>,
            With<InspectorParamValText>,
        )>,
    >,
) {
    let single_selected_node_id = if selection_res.0.selected_nodes.len() == 1 {
        selection_res.0.selected_nodes.iter().next().copied()
    } else {
        None
    };

    let node_opt = single_selected_node_id.and_then(|id| graph_res.0.get_node(id));

    for mut panel_node in &mut panel_query {
        panel_node.display = if node_opt.is_some() {
            Display::Flex
        } else {
            Display::None
        };
    }

    if let Some(node) = node_opt {
        let mut badge_tag = "NODE".to_string();
        if let Some(desc) = dataflow_res.0.registry.get(&node.kind) {
            badge_tag = desc.ui.category.tag.to_string();
        }

        for mut bg in &mut badge_bg_query {
            *bg = BackgroundColor(jackdaw_tokens::CHIP_BG);
        }

        let has_numeric_param = node
            .inputs
            .first()
            .and_then(|&p| graph_res.0.get_pin(p))
            .is_some_and(|pin| pin.data_type.name == "Float" || pin.data_type.name == "Int");

        for mut sec_node in &mut param_section_query {
            sec_node.display = if has_numeric_param {
                Display::Flex
            } else {
                Display::None
            };
        }

        for (mut text, title_opt, kind_opt, pos_opt, pins_opt, param_opt) in &mut text_query {
            if title_opt.is_some() {
                **text = node.title.clone();
            } else if kind_opt.is_some() {
                **text = badge_tag.clone();
            } else if pos_opt.is_some() {
                **text = format!("Position: ({:.0}, {:.0})", node.position.x, node.position.y);
            } else if pins_opt.is_some() {
                **text = format!("Pins: {} Inputs, {} Outputs", node.inputs.len(), node.outputs.len());
            } else if param_opt.is_some() {
                if let Some(&pin_id) = node.inputs.first() {
                    let val = match dataflow_res.0.pin_values.get(&pin_id) {
                        Some(luma_dataflow::value::PrimitiveValue::Float(f)) => *f,
                        Some(luma_dataflow::value::PrimitiveValue::Int(i)) => *i as f32,
                        _ => 0.0,
                    };
                    **text = format!("Value: {:.2}", val);
                }
            }
        }
    }
}

#[derive(SystemParam)]
pub struct InspectorActionQueries<'w, 's> {
    pub close_clicks: Query<'w, 's, &'static Interaction, (With<BevyUiInspectorBtnClose>, Changed<Interaction>)>,
    pub focus_clicks: Query<'w, 's, &'static Interaction, (With<BevyUiInspectorBtnFocus>, Changed<Interaction>)>,
    pub duplicate_clicks: Query<'w, 's, &'static Interaction, (With<BevyUiInspectorBtnDuplicate>, Changed<Interaction>)>,
    pub delete_clicks: Query<'w, 's, &'static Interaction, (With<BevyUiInspectorBtnDelete>, Changed<Interaction>)>,
    pub param_step_clicks: Query<'w, 's, (&'static Interaction, &'static InspectorBtnParamStep), Changed<Interaction>>,
    pub custom_accent_clicks: Query<'w, 's, (&'static Interaction, &'static InspectorBtnCustomAccent), Changed<Interaction>>,
    pub custom_body_clicks: Query<'w, 's, (&'static Interaction, &'static InspectorBtnCustomBody), Changed<Interaction>>,
}

/// Handles button clicks inside the Inspector panel (Close, Focus, Duplicate, Delete, plus Parameter & Appearance edits).
pub fn bevy_ui_inspector_actions_system(
    mut commands: Commands,
    actions: InspectorActionQueries,
    card_roots: Query<(Entity, &NodeCardRoot)>,
    camera_query: Query<(&Transform, &Projection), With<EditorCamera>>,
    mut anim: ResMut<CameraAnimationResource>,
    mut selection_res: ResMut<EditorSelectionResource>,
    mut graph_res: ResMut<EditorGraphResource>,
    mut history_res: ResMut<EditorHistoryResource>,
    mut nav_res: ResMut<EditorNavigationResource>,
    mut payload_res: ResMut<EditorDomainPayloadResource>,
    mut dataflow_res: ResMut<EditorDataflowResource>,
    id_gen: Res<EditorIdGenResource>,
) {
    let selected_id = if selection_res.0.selected_nodes.len() == 1 {
        selection_res.0.selected_nodes.iter().next().copied()
    } else {
        None
    };

    // Close: deselect
    for interaction in &actions.close_clicks {
        if *interaction == Interaction::Pressed {
            selection_res.0.clear();
        }
    }

    if let Some(node_id) = selected_id {
        // Focus / Dive-In
        for interaction in &actions.focus_clicks {
            if *interaction == Interaction::Pressed {
                if let Some(node) = graph_res.0.get_node(node_id) {
                    let (cam_pos, cam_scale) = if let Some((t, p)) = camera_query.iter().next() {
                        let scale = match p {
                            Projection::Orthographic(proj) => proj.scale,
                            _ => 1.0,
                        };
                        (Vec2::new(t.translation.x, t.translation.y), scale)
                    } else {
                        (Vec2::ZERO, 1.0)
                    };
                    nav_res.0.dive_into(
                        node.id,
                        node.kind.clone(),
                        node.title.clone(),
                        cam_pos.x,
                        cam_pos.y,
                        cam_scale,
                    );
                    anim.animate_to(cam_pos, cam_scale, node.position, 0.65);
                }
            }
        }

        // Duplicate
        for interaction in &actions.duplicate_clicks {
            if *interaction == Interaction::Pressed {
                if let Some((kind, title, pos)) = graph_res
                    .0
                    .get_node(node_id)
                    .map(|n| (n.kind.clone(), n.title.clone(), n.position))
                {
                    let new_id = id_gen.0.next_node_id();
                    let new_node = luma_core::graph::Node::new(
                        new_id,
                        format!("{} Copy", title),
                        kind.clone(),
                        pos + Vec2::new(40.0, 40.0),
                    );
                    let cmd = Box::new(AddNodeCommand::new(new_node));
                    let _ = history_res.0.apply(cmd, &mut graph_res.0);
                    payload_res.0.clone_payload(node_id, new_id, &kind);
                    selection_res.0.select_node(new_id, false);
                }
            }
        }

        // Delete
        for interaction in &actions.delete_clicks {
            if *interaction == Interaction::Pressed {
                let cmd = Box::new(RemoveNodeCommand::new(node_id));
                let _ = history_res.0.apply(cmd, &mut graph_res.0);
                payload_res.0.remove_payload(node_id);
                selection_res.0.clear();
            }
        }

        // Numeric Parameter Step & Downstream Re-evaluation
        for (interaction, step) in &actions.param_step_clicks {
            if *interaction == Interaction::Pressed {
                if let Some(node) = graph_res.0.get_node(node_id) {
                    if let Some(&pin_id) = node.inputs.first() {
                        let cur = match dataflow_res.0.pin_values.get(&pin_id) {
                            Some(luma_dataflow::value::PrimitiveValue::Float(f)) => *f,
                            Some(luma_dataflow::value::PrimitiveValue::Int(i)) => *i as f32,
                            _ => 0.0,
                        };
                        let new_val = cur + step.0;
                        let downstream = dataflow_res.0.set_pin_value(
                            &graph_res.0,
                            pin_id,
                            luma_dataflow::value::PrimitiveValue::Float(new_val),
                        );
                        let pinned = std::collections::HashSet::new();
                        let _ = dataflow_res.0.pull_node(&graph_res.0, node_id, &pinned);
                        if let Some(deps) = downstream {
                            for dep_id in deps {
                                let _ = dataflow_res.0.pull_node(&graph_res.0, dep_id, &pinned);
                            }
                        }
                    }
                }
            }
        }

        // Custom Node Appearance & Content overrides
        let mut rebuild_card = false;
        for (interaction, accent) in &actions.custom_accent_clicks {
            if *interaction == Interaction::Pressed {
                let [r, g, b] = accent.0;
                set_node_instance_customization(node_id, |ov| {
                    ov.accent_color = Some(LinearColor::new(r, g, b, 1.0));
                    ov.header_bg = Some(LinearColor::new(r * 0.55, g * 0.55, b * 0.55, 1.0));
                });
                rebuild_card = true;
            }
        }

        for (interaction, body) in &actions.custom_body_clicks {
            if *interaction == Interaction::Pressed {
                let [r, g, b] = body.0;
                set_node_instance_customization(node_id, |ov| {
                    ov.body_bg = Some(LinearColor::new(r, g, b, 1.0));
                });
                rebuild_card = true;
            }
        }

        if rebuild_card {
            for (entity, root) in &card_roots {
                if root.node_id == node_id {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
