use luma_canvas::plugin::{
    CameraAnimationResource, EditorCamera, EditorDataflowResource, EditorDispatcherResource,
    EditorDomainPayloadResource, EditorGraphResource, EditorHistoryResource, EditorSelectionResource,
    EditorTransformResource,
};
use luma_commands::descriptor::CommandId;
use luma_commands::dispatcher::CommandContext;
use crate::bevy_ui_hud::{BevyUiSpawnPalette, BevyUiToast};
use crate::plugin::EditorNavigationResource;
use crate::{UiButton as Button, UiInteraction as Interaction};
use bevy::prelude::*;

// Marker components for Bevy UI Screen Chrome
#[derive(Component)]
pub struct BevyUiHeaderRoot;

#[derive(Component)]
pub struct BevyUiStatusBarRoot;

#[derive(Component)]
pub struct BevyUiFocusToolbarRoot;

#[derive(Component)]
pub struct BevyUiBreadcrumbText;

#[derive(Component)]
pub struct BevyUiCoordsText;

#[derive(Component)]
pub struct BevyUiSelectionText;

#[derive(Component)]
pub struct BevyUiSceneSyncText;

#[derive(Component)]
pub struct BevyUiModeBadgeText;

#[derive(Component)]
pub struct BevyUiBtnSpawn;

#[derive(Component)]
pub struct BevyUiBtnBake;

#[derive(Component)]
pub struct BevyUiBtnUndo;

#[derive(Component)]
pub struct BevyUiBtnRedo;

#[derive(Component)]
pub struct BevyUiBtnExitFocus;

#[derive(Component)]
pub struct BevyUiChromeBtn;

use bevy_luma::style::prelude::Icon;
use crate::jackdaw_widgets::{
    jackdaw_tokens, spawn_chip, spawn_separator, ChipVariant, SeparatorProps,
};
use crate::theme::EditorTheme;

/// Spawns the entire screen-space Chrome (Top Navbar, Bottom Status Bar, Focus Toolbar) using pure bevy_ui.
pub fn setup_bevy_ui_chrome(
    mut commands: Commands,
    camera_q: Query<Entity, With<crate::EditorUiCamera>>,
    asset_server: Res<AssetServer>,
) {
    let cam_opt = camera_q.iter().next();
    eprintln!("DEBUG: setup_bevy_ui_chrome running! cam_opt = {:?}", cam_opt);
    let font = FontSource::default();
    let icon_font = FontSource::Handle(
        asset_server.load("embedded://luma_style/style/assets/icons/Lucide.ttf"),
    );

    // ==========================================
    // 1. Top Navigation Bar (Fixed 34px Header - Jackdaw / Figma)
    // ==========================================
    let mut header_cmd = commands.spawn((
        BevyUiHeaderRoot,
        GlobalZIndex(150),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Px(34.0),
            display: Display::Flex,
            padding: UiRect::axes(Val::Px(12.0), Val::Px(0.0)),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            border: UiRect::ZERO,
            border_radius: BorderRadius::bottom(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(EditorTheme::WINDOW_BG),
        BorderColor::all(Color::NONE),
    ));
    if let Some(cam) = cam_opt {
        header_cmd.insert(UiTargetCamera(cam));
    }
    header_cmd.with_children(|parent| {
        // Left: Clean Luma Brand & Breadcrumbs trail (Jackdaw Figma style)
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|left| {
                // Vertical accent bar (Jackdaw title bar token)
                left.spawn((
                    Node {
                        width: Val::Px(3.0),
                        height: Val::Px(13.0),
                        border_radius: BorderRadius::all(Val::Px(1.5)),
                        ..default()
                    },
                    BackgroundColor(EditorTheme::BORDER_FOCUSED),
                ));

                // Elegant Brand Logo (pure text, no bulky border)
                left.spawn((
                    Text::new("Luma"),
                    TextFont {
                        font: font.clone(),
                        font_size: FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(EditorTheme::TEXT_PRIMARY),
                ));

                // Vertical separator between Brand and Menus (Jackdaw separator)
                spawn_separator(left, SeparatorProps::vertical());

                // Top Menu Triggers: File, Edit, View, Help
                crate::menu_bar::spawn_menu_triggers(left, font.clone());

                // Path separator
                left.spawn((
                    Text::new("/"),
                    TextFont {
                        font: font.clone(),
                        font_size: FontSize::Px(11.5),
                        ..default()
                    },
                    TextColor(Color::srgb(0.40, 0.40, 0.45)),
                ));

                // Breadcrumbs trail
                left.spawn((
                    BevyUiBreadcrumbText,
                    Text::new("Root Graph"),
                    TextFont {
                        font: font.clone(),
                        font_size: FontSize::Px(12.0),
                        ..default()
                    },
                    TextColor(EditorTheme::TEXT_MUTED),
                ));
            });

        // Center: Central Document Tab & Ghost Action Buttons (Zero border, clean Figma styling)
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|center| {
                // Active document tab pill (Jackdaw document tab strip spec)
                center
                    .spawn((
                        Node {
                            height: Val::Px(24.0),
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(0.0)),
                            border_radius: BorderRadius::all(Val::Px(jackdaw_tokens::BORDER_RADIUS_SM)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(6.0),
                            border: UiRect::ZERO,
                            ..default()
                        },
                        BackgroundColor(jackdaw_tokens::DOC_TAB_ACTIVE_BG),
                    ))
                    .with_children(|pill| {
                        // Blue scene accent dot
                        pill.spawn((
                            Node {
                                width: Val::Px(5.0),
                                height: Val::Px(5.0),
                                border_radius: BorderRadius::all(Val::Px(2.5)),
                                border: UiRect::ZERO,
                                ..default()
                            },
                            BackgroundColor(jackdaw_tokens::DOC_TAB_SCENE_ACCENT),
                        ));

                        pill.spawn((
                            Text::new("scenes/main.luma"),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(11.5),
                                ..default()
                            },
                            TextColor(jackdaw_tokens::DOC_TAB_ACTIVE_LABEL),
                        ));
                    });

                // Subtle vertical separator (Jackdaw separator)
                spawn_separator(center, SeparatorProps::vertical());

                // Undo Ghost Button
                center
                    .spawn((
                        BevyUiBtnUndo,
                        BevyUiChromeBtn,
                        Button,
                        Node {
                            height: Val::Px(24.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(0.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Undo"),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(11.5),
                                ..default()
                            },
                            TextColor(EditorTheme::TEXT_MUTED),
                        ));
                    });

                // Redo Ghost Button
                center
                    .spawn((
                        BevyUiBtnRedo,
                        BevyUiChromeBtn,
                        Button,
                        Node {
                            height: Val::Px(24.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(0.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Redo"),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(11.5),
                                ..default()
                            },
                            TextColor(EditorTheme::TEXT_MUTED),
                        ));
                    });

                // Subtle vertical separator (Jackdaw separator)
                spawn_separator(center, SeparatorProps::vertical());

                // + Add Node Button
                center
                    .spawn((
                        BevyUiBtnSpawn,
                        BevyUiChromeBtn,
                        Button,
                        Node {
                            height: Val::Px(24.0),
                            padding: UiRect::axes(Val::Px(9.0), Val::Px(0.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.13, 0.16)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("+ Add Node"),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(11.5),
                                ..default()
                            },
                            TextColor(EditorTheme::TEXT_PRIMARY),
                        ));
                    });

                // Save & Bake Button (Solid EditorTheme::ACCENT_PRIMARY, the ONLY strongly colored button)
                center
                    .spawn((
                        BevyUiBtnBake,
                        BevyUiChromeBtn,
                        Button,
                        Node {
                            height: Val::Px(24.0),
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(0.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(EditorTheme::ACCENT_PRIMARY),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Save & Bake"),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(11.5),
                                ..default()
                            },
                            TextColor(Color::srgb(0.04, 0.05, 0.07)),
                        ));
                    });
            });

        // Right: Mode Badge (Clean, borderless subtle badge)
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|right| {
                right
                    .spawn((
                        Node {
                            height: Val::Px(22.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(0.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(5.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.13, 0.16)),
                    ))
                    .with_children(|badge| {
                        badge.spawn((
                            Node {
                                width: Val::Px(4.0),
                                height: Val::Px(4.0),
                                border_radius: BorderRadius::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(EditorTheme::ACCENT_SUCCESS),
                        ));

                        badge.spawn((
                            BevyUiModeBadgeText,
                            Text::new("Macro Graph Mode"),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(11.0),
                                ..default()
                            },
                            TextColor(EditorTheme::TEXT_MUTED),
                        ));
                    });

                // Settings button in top chrome with Lucide Icon::SETTINGS
                right
                    .spawn((
                        crate::bevy_ui_settings::BevyUiBtnSettings,
                        BevyUiChromeBtn,
                        Button,
                        Node {
                            height: Val::Px(22.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(0.0)),
                            margin: UiRect::left(Val::Px(6.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(5.0),
                            border: UiRect::ZERO,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.13, 0.16)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(Icon::SETTINGS),
                            TextFont {
                                font: icon_font.clone(),
                                font_size: FontSize::Px(12.0),
                                ..default()
                            },
                            TextColor(EditorTheme::BORDER_FOCUSED),
                        ));
                        btn.spawn((
                            Text::new("Settings"),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(11.0),
                                ..default()
                            },
                            TextColor(EditorTheme::TEXT_PRIMARY),
                        ));
                    });
            });
    });

    // ==========================================
    // 2. Bottom Status Bar (Fixed 22px Footer - Jackdaw / Figma)
    // ==========================================
    let mut footer_cmd = commands.spawn((
        BevyUiStatusBarRoot,
        GlobalZIndex(150),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Px(22.0),
            display: Display::Flex,
            padding: UiRect::axes(Val::Px(12.0), Val::Px(0.0)),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            border: UiRect::ZERO,
            ..default()
        },
        BackgroundColor(EditorTheme::WINDOW_BG),
        BorderColor::all(Color::NONE),
    ));
    if let Some(cam) = cam_opt {
        footer_cmd.insert(UiTargetCamera(cam));
    }
    footer_cmd.with_children(|parent| {
        // Left: Coords & Zoom Telemetry
        parent.spawn((
            BevyUiCoordsText,
            Text::new("Pan: (0.0, 0.0)  •  Zoom: 100%"),
            TextFont {
                font: font.clone(),
                font_size: FontSize::Px(11.0),
                ..default()
            },
            TextColor(EditorTheme::TEXT_MUTED),
        ));

        // Center: Selection Summary
        parent.spawn((
            BevyUiSelectionText,
            Text::new("No selection"),
            TextFont {
                font: font.clone(),
                font_size: FontSize::Px(11.0),
                ..default()
            },
            TextColor(EditorTheme::TEXT_MUTED),
        ));

        // Right: Engine Sync Status + Jackdaw Status Chips
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                border: UiRect::ZERO,
                ..default()
            })
            .with_children(|right| {
                right.spawn((
                    BevyUiSceneSyncText,
                    Text::new("scenes/main.bsn"),
                    TextFont {
                        font: font.clone(),
                        font_size: FontSize::Px(11.0),
                        ..default()
                    },
                    TextColor(EditorTheme::TEXT_MUTED),
                ));

                spawn_chip(right, &font, "Synced", ChipVariant::Success);
                spawn_chip(right, &font, "BSN v1.0", ChipVariant::Neutral);
            });
    });

    // ==========================================
    // 3. Contextual Focus Toolbar Dock (Top Center)
    // ==========================================
    let mut focus_cmd = commands.spawn((
        BevyUiFocusToolbarRoot,
        GlobalZIndex(160),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(42.0),
            left: Val::Percent(50.0),
            margin: UiRect {
                left: Val::Px(-160.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
            },
            width: Val::Px(320.0),
            height: Val::Px(30.0),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(0.0)),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(4.0)),
            display: Display::None, // hidden when not in focus
            ..default()
        },
        BackgroundColor(EditorTheme::PANEL_BG),
    ));
    if let Some(cam) = cam_opt {
        focus_cmd.insert(UiTargetCamera(cam));
    }
    focus_cmd.with_children(|parent| {
        // Exit button
        parent
            .spawn((
                BevyUiBtnExitFocus,
                BevyUiChromeBtn,
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.14, 0.15, 0.19)),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("< Back to Graph [Esc]"),
                    TextFont {
                        font: font.clone(),
                        font_size: FontSize::Px(10.5),
                        ..default()
                    },
                    TextColor(EditorTheme::TEXT_PRIMARY),
                ));
            });

        // Contextual indicator
        parent.spawn((
            Text::new("Focus Workspace Active"),
            TextFont {
                font: font.clone(),
                font_size: FontSize::Px(11.0),
                ..default()
            },
            TextColor(EditorTheme::BORDER_FOCUSED),
        ));
    });
}

/// Updates telemetry texts in the native Bevy UI chrome.
pub fn update_bevy_ui_chrome_telemetry_system(
    transform: Res<EditorTransformResource>,
    selection: Res<EditorSelectionResource>,
    nav_res: Res<EditorNavigationResource>,
    mut coords_query: Query<&mut Text, (With<BevyUiCoordsText>, Without<BevyUiSelectionText>, Without<BevyUiBreadcrumbText>, Without<BevyUiModeBadgeText>)>,
    mut sel_query: Query<&mut Text, (With<BevyUiSelectionText>, Without<BevyUiCoordsText>, Without<BevyUiBreadcrumbText>, Without<BevyUiModeBadgeText>)>,
    mut breadcrumb_query: Query<&mut Text, (With<BevyUiBreadcrumbText>, Without<BevyUiCoordsText>, Without<BevyUiSelectionText>, Without<BevyUiModeBadgeText>)>,
    mut mode_badge_query: Query<&mut Text, (With<BevyUiModeBadgeText>, Without<BevyUiCoordsText>, Without<BevyUiSelectionText>, Without<BevyUiBreadcrumbText>)>,
    mut focus_toolbar_query: Query<&mut Node, With<BevyUiFocusToolbarRoot>>,
) {
    for mut text in &mut coords_query {
        **text = format!(
            "Pan: ({:.1}, {:.1})  |  Zoom: {:.0}%",
            transform.0.pan.x,
            transform.0.pan.y,
            transform.0.zoom * 100.0
        );
    }

    for mut text in &mut sel_query {
        **text = if selection.0.is_empty() {
            "No selection".to_string()
        } else {
            format!(
                "Selected: {} node(s), {} wire(s)",
                selection.0.selected_nodes.len(),
                selection.0.selected_connections.len()
            )
        };
    }

    for mut text in &mut breadcrumb_query {
        **text = nav_res.0.breadcrumbs().join("  ›  ");
    }

    let is_in_focus = nav_res.0.is_in_focus();
    for mut text in &mut mode_badge_query {
        **text = if is_in_focus {
            "Micro Focus Mode".to_string()
        } else {
            "Macro Graph Mode".to_string()
        };
    }

    for mut node in &mut focus_toolbar_query {
        node.display = if is_in_focus {
            Display::Flex
        } else {
            Display::None
        };
    }
}

/// Handles button clicks on the native Bevy UI Chrome (Spawn, Bake, Undo, Redo, Exit Focus).
pub fn bevy_ui_chrome_actions_system(
    spawn_clicks: Query<&Interaction, (With<BevyUiBtnSpawn>, Changed<Interaction>)>,
    bake_clicks: Query<&Interaction, (With<BevyUiBtnBake>, Changed<Interaction>)>,
    undo_clicks: Query<&Interaction, (With<BevyUiBtnUndo>, Changed<Interaction>)>,
    redo_clicks: Query<&Interaction, (With<BevyUiBtnRedo>, Changed<Interaction>)>,
    exit_clicks: Query<&Interaction, (With<BevyUiBtnExitFocus>, Changed<Interaction>)>,
    camera_query: Query<(&Transform, &Projection), With<EditorCamera>>,
    mut anim: ResMut<CameraAnimationResource>,
    mut palette_nodes: Query<(&mut Node, &mut BevyUiSpawnPalette), Without<BevyUiToast>>,
    mut toast_query: Query<(&mut BevyUiToast, &mut Node), Without<BevyUiSpawnPalette>>,
    mut dispatcher: ResMut<EditorDispatcherResource>,
    mut graph_res: ResMut<EditorGraphResource>,
    mut history_res: ResMut<EditorHistoryResource>,
    mut nav_res: ResMut<EditorNavigationResource>,
    mut payload_res: ResMut<EditorDomainPayloadResource>,
    mut dataflow_res: ResMut<EditorDataflowResource>,
) {
    for interaction in &spawn_clicks {
        if *interaction == Interaction::Pressed {
            for (mut p_node, mut p_palette) in &mut palette_nodes {
                if p_node.display == Display::Flex {
                    p_node.display = Display::None;
                } else {
                    p_node.left = Val::Px(510.0);
                    p_node.top = Val::Px(36.0);
                    p_node.margin = UiRect::ZERO;
                    p_palette.spawn_canvas_pos = Vec2::ZERO;
                    p_node.display = Display::Flex;
                }
            }
        }
    }

    for interaction in &bake_clicks {
        if *interaction == Interaction::Pressed {
            let mut ctx = CommandContext {
                graph: &mut graph_res.0,
                history: &mut history_res.0,
                payloads: &mut payload_res.0,
                dataflow: &mut dataflow_res.0,
            };
            let _ = dispatcher.0.dispatch(&CommandId::from("app.save_and_bake"), &mut ctx);

            // Show Toast
            for (mut toast, mut t_node) in &mut toast_query {
                t_node.display = Display::Flex;
                toast.timer.reset();
            }
        }
    }

    for interaction in &undo_clicks {
        if *interaction == Interaction::Pressed {
            let _ = history_res.0.undo(&mut graph_res.0);
        }
    }

    for interaction in &redo_clicks {
        if *interaction == Interaction::Pressed {
            let _ = history_res.0.redo(&mut graph_res.0);
        }
    }

    for interaction in &exit_clicks {
        if *interaction == Interaction::Pressed {
            let (cur_pos, cur_scale) = if let Some((t, p)) = camera_query.iter().next() {
                let scale = match p {
                    Projection::Orthographic(proj) => proj.scale,
                    _ => 1.0,
                };
                (Vec2::new(t.translation.x, t.translation.y), scale)
            } else {
                (Vec2::ZERO, 1.0)
            };

            if let Some(luma_canvas::focus::WorkspaceContext::FocusedNode { saved_cam_x, saved_cam_y, saved_cam_scale, .. }) = nav_res.0.exit_focus() {
                anim.animate_to(cur_pos, cur_scale, Vec2::new(saved_cam_x, saved_cam_y), saved_cam_scale);
            } else {
                anim.animate_to(cur_pos, cur_scale, Vec2::ZERO, 1.0);
            }
        }
    }
}

/// Smooth hover and pressed visual feedback for all borderless buttons on the Bevy UI Chrome.
pub fn bevy_ui_chrome_hover_system(
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor, Option<&BevyUiBtnBake>),
        (With<BevyUiChromeBtn>, Changed<Interaction>),
    >,
) {
    for (interaction, mut bg, bake_opt) in &mut buttons {
        if bake_opt.is_some() {
            match *interaction {
                Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.40, 0.88, 1.0)),
                Interaction::Pressed => *bg = BackgroundColor(Color::srgb(0.18, 0.75, 0.90)),
                Interaction::None => *bg = BackgroundColor(EditorTheme::ACCENT_PRIMARY),
            }
        } else {
            match *interaction {
                Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.18, 0.20, 0.26)),
                Interaction::Pressed => *bg = BackgroundColor(Color::srgb(0.24, 0.27, 0.35)),
                Interaction::None => *bg = BackgroundColor(Color::NONE),
            }
        }
    }
}

/// Continuously ensures all UI roots (Header Navbar, Status Bar, Focus Toolbar)
/// are targeted to the 2D Camera, guaranteeing they never disappear or get orphaned.
pub fn ensure_ui_camera_target_system(
    camera_q: Query<Entity, With<crate::EditorUiCamera>>,
    unattached_ui: Query<
        Entity,
        (
            With<Node>,
            Without<UiTargetCamera>,
            Or<(
                With<BevyUiHeaderRoot>,
                With<BevyUiStatusBarRoot>,
                With<BevyUiFocusToolbarRoot>,
                With<crate::bevy_ui_settings::BevyUiSettingsRoot>,
            )>, 
        ),
    >,
    mut commands: Commands,
) {
    if let Some(cam) = camera_q.iter().next() {
        for entity in &unattached_ui {
            commands.entity(entity).insert(UiTargetCamera(cam));
        }
    }
}
