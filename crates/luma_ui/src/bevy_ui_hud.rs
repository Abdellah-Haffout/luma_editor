use luma_canvas::plugin::{
    EditorCamera, EditorDataflowResource, EditorDomainPayloadResource, EditorGraphResource,
    EditorHistoryResource, EditorIdGenResource,
};
use luma_canvas::nodes::palette::NodePalette;
use crate::jackdaw_widgets::{spawn_separator, SeparatorProps};
use crate::{UiButton as Button, UiInteraction as Interaction};
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::ui::ScrollPosition;

#[derive(Component)]
pub struct BevyUiHudRoot;

#[derive(Component)]
pub struct BevyUiToast {
    pub timer: Timer,
}

#[derive(Component)]
pub struct BevyUiToastText;

#[derive(Component, Default)]
pub struct BevyUiSpawnPalette {
    pub spawn_canvas_pos: Vec2,
    pub active_category: String,
}

/// Marker for a category row in the left column of the Add Node menu.
#[derive(Component, Clone)]
pub struct SpawnCategoryRow {
    pub category: String,
}

/// Marker for the right-hand cascading submenu panel that appears when hovering a category.
#[derive(Component)]
pub struct SpawnSideSubmenuPanel;

/// Marker for the title text in the right-hand cascading submenu header.
#[derive(Component)]
pub struct SpawnSideSubmenuTitle;

/// Marker for the scrollable node list container inside the cascading side submenu.
#[derive(Component)]
pub struct SpawnSideScrollList;

#[derive(Component)]
pub struct SpawnPaletteButton {
    pub template: String,
    pub category: String,
}

#[derive(Component)]
pub struct ClosePaletteButton;

/// Spawns standard Bevy UI (bevy_ui) HUD elements:
/// 1. Quick Hotkeys & Telemetry reference dock (bottom-right)
use crate::theme::EditorTheme;

#[derive(Component)]
pub struct BevyUiBtnZoomIn;

#[derive(Component)]
pub struct BevyUiBtnZoomOut;

#[derive(Component)]
pub struct BevyUiBtnZoomReset;

#[derive(Component)]
pub struct BevyUiBtnZoomFit;

#[derive(Component)]
pub struct BevyUiBtnColorblind;

#[derive(Component)]
pub struct BevyUiMinimapRoot;

/// Marker for the floating zoom / accessibility toolbar overlay.
#[derive(Component)]
pub struct BevyUiFloatingToolbarRoot;

#[derive(Resource, Default)]
pub struct EditorAccessibilitySettings {
    pub colorblind_mode: bool,
    pub reduced_motion: bool,
}

/// Spawns standard Bevy UI (bevy_ui) HUD elements:
/// 1. Floating toolbar with zoom controls and colorblind mode (center-bottom)
/// 2. Interactive Minimap overview (bottom-right)
/// 3. Quick Hotkeys guide (bottom-left)
/// 4. Toast notification banner (top center)
/// 5. Subtle Node Search Palette modal in dark glass styling
pub fn setup_bevy_ui_hud(
    mut commands: Commands,
    camera_q: Query<Entity, With<crate::EditorUiCamera>>,
    dataflow_res: Res<EditorDataflowResource>,
) {
    let cam_opt = camera_q.iter().next();
    let font = FontSource::default();

    // 1. Floating Toolbar with Zoom Controls & Colorblind Toggle (Center-Bottom)
    let mut toolbar_cmd = commands.spawn((
        BevyUiFloatingToolbarRoot,
        Button,
        GlobalZIndex(220),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(50.0),
            left: Val::Percent(50.0),
            margin: UiRect {
                left: Val::Px(-140.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
            },
            width: Val::Px(280.0),
            height: Val::Px(32.0),
            padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.0),
            border: UiRect::ZERO,
            border_radius: BorderRadius::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(EditorTheme::WINDOW_BG),
        BorderColor::all(Color::NONE),
    ));
    if let Some(cam) = cam_opt {
        toolbar_cmd.insert(UiTargetCamera(cam));
    }
    toolbar_cmd.with_children(|bar| {
        // Zoom Out [-]
        bar.spawn((
            BevyUiBtnZoomOut,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
        )).with_children(|btn| {
            btn.spawn((
                Text::new("-"),
                TextFont { font: font.clone(), font_size: FontSize::Px(13.0), ..default() },
                TextColor(EditorTheme::TEXT_PRIMARY),
            ));
        });

        // Zoom 100% Reset
        bar.spawn((
            BevyUiBtnZoomReset,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
        )).with_children(|btn| {
            btn.spawn((
                Text::new("100%"),
                TextFont { font: font.clone(), font_size: FontSize::Px(11.0), ..default() },
                TextColor(EditorTheme::TEXT_PRIMARY),
            ));
        });

        // Zoom In [+]
        bar.spawn((
            BevyUiBtnZoomIn,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
        )).with_children(|btn| {
            btn.spawn((
                Text::new("+"),
                TextFont { font: font.clone(), font_size: FontSize::Px(13.0), ..default() },
                TextColor(EditorTheme::TEXT_PRIMARY),
            ));
        });

        spawn_separator(bar, SeparatorProps::vertical());

        // Fit to graph
        bar.spawn((
            BevyUiBtnZoomFit,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
        )).with_children(|btn| {
            btn.spawn((
                Text::new("Fit"),
                TextFont { font: font.clone(), font_size: FontSize::Px(11.0), ..default() },
                TextColor(EditorTheme::TEXT_MUTED),
            ));
        });

        spawn_separator(bar, SeparatorProps::vertical());

        // Colorblind Safe Shapes Toggle
        bar.spawn((
            BevyUiBtnColorblind,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
        )).with_children(|btn| {
            btn.spawn((
                Text::new("Shapes"),
                TextFont { font: font.clone(), font_size: FontSize::Px(10.5), ..default() },
                TextColor(EditorTheme::ACCENT_PRIMARY),
            ));
        });
    });

    // 2. Corner Minimap (Bottom-Right, Dark Glass Card)
    let mut minimap_cmd = commands.spawn((
        BevyUiMinimapRoot,
        Button,
        GlobalZIndex(190),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(50.0),
            right: Val::Px(16.0),
            width: Val::Px(120.0),
            height: Val::Px(72.0),
            padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            border: UiRect::ZERO,
            border_radius: BorderRadius::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(EditorTheme::WINDOW_BG),
        BorderColor::all(Color::NONE),
    ));
    if let Some(cam) = cam_opt {
        minimap_cmd.insert(UiTargetCamera(cam));
    }
    minimap_cmd.with_children(|map| {
        // Header
        map.spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        }).with_children(|hdr| {
            hdr.spawn((
                Text::new("MINIMAP"),
                TextFont { font: font.clone(), font_size: FontSize::Px(8.5), ..default() },
                TextColor(EditorTheme::TEXT_MUTED),
            ));
            hdr.spawn((
                Text::new("VIEWPORT"),
                TextFont { font: font.clone(), font_size: FontSize::Px(8.5), ..default() },
                TextColor(EditorTheme::ACCENT_PRIMARY),
            ));
        });

        // Clean camera viewport frame
        map.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(40.0),
            position_type: PositionType::Relative,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        }).with_children(|preview| {
            preview.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(14.0),
                    top: Val::Px(6.0),
                    width: Val::Px(80.0),
                    height: Val::Px(28.0),
                    border: UiRect::ZERO,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.14, 0.18)),
            ));
        });
    });

    // 3. Quick Hotkeys guide (Bottom-Left)
    let mut hud_cmd = commands.spawn((
        BevyUiHudRoot,
        Button,
        GlobalZIndex(200),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(50.0),
            left: Val::Px(16.0),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(10.0),
            align_items: AlignItems::Center,
            border: UiRect::ZERO,
            border_radius: BorderRadius::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(EditorTheme::WINDOW_BG),
        BorderColor::all(Color::NONE),
    ));
    if let Some(cam) = cam_opt {
        hud_cmd.insert(UiTargetCamera(cam));
    }
    hud_cmd.with_children(|parent| {
        parent.spawn((
            Text::new("[RMB] Pan  |  [Scroll] Zoom  |  [Space] Search Nodes"),
            TextFont {
                font: font.clone(),
                font_size: FontSize::Px(10.5),
                ..default()
            },
            TextColor(EditorTheme::TEXT_MUTED),
        ));
    });

    // 4. Toast Notification Banner (top center)
    let mut toast_cmd = commands.spawn((
        BevyUiToast {
            timer: Timer::from_seconds(3.0, TimerMode::Once),
        },
        GlobalZIndex(250),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(76.0),
            left: Val::Percent(50.0),
            margin: UiRect {
                left: Val::Px(-180.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
            },
            width: Val::Px(360.0),
            padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::ZERO,
            border_radius: BorderRadius::all(Val::Px(10.0)),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgb(0.04, 0.16, 0.10)),
        BorderColor::all(Color::NONE),
    ));
    if let Some(cam) = cam_opt {
        toast_cmd.insert(UiTargetCamera(cam));
    }
    toast_cmd.with_children(|parent| {
        parent.spawn((
            BevyUiToastText,
            Text::new("Done: Scene baked to scenes/main.bsn successfully!"),
            TextFont {
                font_size: FontSize::Px(12.0),
                ..default()
            },
            TextColor(EditorTheme::PIN_ENTITY),
        ));
    });

    // 5. Cascading Add Node Menu (Categories on Left + Hover Side Submenu with Scroll on Right)
    let palette_data = NodePalette::from_registry(&dataflow_res.0.registry);
    let grouped = palette_data.by_category();
    let default_cat = grouped
        .keys()
        .next()
        .cloned()
        .unwrap_or_else(|| "Custom & Utility".to_string());

    let mut palette_cmd = commands.spawn((
        BevyUiSpawnPalette {
            spawn_canvas_pos: Vec2::ZERO,
            active_category: default_cat.clone(),
        },
        Button,
        GlobalZIndex(300),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(200.0),
            top: Val::Px(160.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            column_gap: Val::Px(4.0),
            display: Display::None,
            ..default()
        },
    ));
    if let Some(cam) = cam_opt {
        palette_cmd.insert(UiTargetCamera(cam));
    }
    palette_cmd.with_children(|root| {
        // =========================================================
        // A. Left Column: Categories List (Bevy-Editor Figma Style)
        // =========================================================
        root.spawn((
            Node {
                width: Val::Px(215.0),
                flex_direction: FlexDirection::Column,
                border: UiRect::ZERO,
                border_radius: BorderRadius::all(Val::Px(6.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(jackdaw_feathers::tokens::MENU_BG),
            BorderColor::all(Color::NONE),
        ))
        .with_children(|left_panel| {
            // Header Bar: "Add Node" + [+ Custom] + [X]
            left_panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(30.0),
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        border: UiRect::ZERO,
                        ..default()
                    },
                    BackgroundColor(jackdaw_feathers::tokens::WINDOW_BG),
                ))
                .with_children(|hdr| {
                    hdr.spawn((
                        Text::new("Add Node"),
                        TextFont {
                            font_size: FontSize::Px(11.5),
                            ..default()
                        },
                        TextColor(EditorTheme::TEXT_PRIMARY),
                    ));

                    hdr.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|actions| {
                        actions
                            .spawn((
                                ClosePaletteButton,
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                    border: UiRect::ZERO,
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("X"),
                                    TextFont {
                                        font_size: FontSize::Px(10.5),
                                        ..default()
                                    },
                                    TextColor(EditorTheme::TEXT_MUTED),
                                ));
                            });
                    });
                });

            // Categories List
            left_panel
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(4.0)),
                    row_gap: Val::Px(2.0),
                    ..default()
                })
                .with_children(|cat_list| {
                    for (cat_name, templates) in &grouped {
                        let cat_color = templates
                            .first()
                            .and_then(|t| t.accent_color)
                            .map(|c| Color::srgb(c.r, c.g, c.b))
                            .unwrap_or(jackdaw_feathers::tokens::ACCENT_BLUE);
                        let is_initial = *cat_name == default_cat;
                        let count_label = format!("{} ›", templates.len());

                        cat_list
                            .spawn((
                                SpawnCategoryRow {
                                    category: cat_name.clone(),
                                },
                                Button,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(26.0),
                                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    border: UiRect::ZERO,
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(if is_initial {
                                    jackdaw_feathers::tokens::ELEVATED_BG
                                } else {
                                    Color::NONE
                                }),
                            ))
                            .with_children(|row| {
                                row.spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(7.0),
                                    ..default()
                                })
                                .with_children(|left| {
                                    // Category colored dot
                                    left.spawn((
                                        Node {
                                            width: Val::Px(6.0),
                                            height: Val::Px(6.0),
                                            border_radius: BorderRadius::all(Val::Px(3.0)),
                                            ..default()
                                        },
                                        BackgroundColor(cat_color),
                                    ));
                                    left.spawn((
                                        Text::new(cat_name.clone()),
                                        TextFont {
                                            font_size: FontSize::Px(11.0),
                                            ..default()
                                        },
                                        TextColor(EditorTheme::TEXT_PRIMARY),
                                    ));
                                });

                                row.spawn((
                                    Text::new(count_label),
                                    TextFont {
                                        font_size: FontSize::Px(10.0),
                                        ..default()
                                    },
                                    TextColor(EditorTheme::TEXT_MUTED),
                                ));
                            });
                    }
                });
        });

        // =========================================================
        // B. Right Cascading Side Submenu (Nodes in Hovered Category + Scroll)
        // =========================================================
        root.spawn((
            SpawnSideSubmenuPanel,
            Node {
                width: Val::Px(245.0),
                flex_direction: FlexDirection::Column,
                border: UiRect::ZERO,
                border_radius: BorderRadius::all(Val::Px(6.0)),
                overflow: Overflow::clip(),
                display: Display::Flex,
                ..default()
            },
            BackgroundColor(jackdaw_feathers::tokens::MENU_BG),
            BorderColor::all(Color::NONE),
        ))
        .with_children(|side_panel| {
            // Side Submenu Header: Active Category Title + Scroll Hint
            side_panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(30.0),
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        border: UiRect::ZERO,
                        ..default()
                    },
                    BackgroundColor(jackdaw_feathers::tokens::WINDOW_BG),
                ))
                .with_children(|shdr| {
                    shdr.spawn((
                        SpawnSideSubmenuTitle,
                        Text::new(default_cat.clone()),
                        TextFont {
                            font_size: FontSize::Px(11.0),
                            ..default()
                        },
                        TextColor(jackdaw_feathers::tokens::ACCENT_BLUE),
                    ));
                    shdr.spawn((
                        Text::new("Scroll ↕"),
                        TextFont {
                            font_size: FontSize::Px(9.5),
                            ..default()
                        },
                        TextColor(EditorTheme::TEXT_MUTED),
                    ));
                });

            // Scrollable Nodes Container
            side_panel
                .spawn((
                    SpawnSideScrollList,
                    Node {
                        width: Val::Percent(100.0),
                        max_height: Val::Px(240.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(4.0)),
                        row_gap: Val::Px(2.0),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    ScrollPosition::default(),
                ))
                .with_children(|list| {
                    for (cat_name, templates) in &grouped {
                        let is_initial = *cat_name == default_cat;
                        for tmpl in templates {
                            let accent_color = if let Some(c) = tmpl.accent_color {
                                Color::srgb(c.r, c.g, c.b)
                            } else if let Some(c) = tmpl.category.color {
                                Color::srgb(c.r, c.g, c.b)
                            } else if tmpl.kind.starts_with("math.vector") {
                                EditorTheme::PIN_VEC3
                            } else {
                                EditorTheme::PIN_FLOAT
                            };
                            let tag = tmpl.category.tag.to_string();
                            let label = tmpl.default_title.clone();
                            let kind = tmpl.kind.clone();

                            list.spawn((
                                SpawnPaletteButton {
                                    template: kind,
                                    category: cat_name.clone(),
                                },
                                Button,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(26.0),
                                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    border: UiRect::ZERO,
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    display: if is_initial {
                                        Display::Flex
                                    } else {
                                        Display::None
                                    },
                                    flex_shrink: 0.0,
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|row| {
                                row.spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(7.0),
                                    ..default()
                                })
                                .with_children(|left| {
                                    // 3px vertical accent bar per type
                                    left.spawn((
                                        Node {
                                            width: Val::Px(3.0),
                                            height: Val::Px(12.0),
                                            border_radius: BorderRadius::all(Val::Px(1.5)),
                                            ..default()
                                        },
                                        BackgroundColor(accent_color),
                                    ));
                                    left.spawn((
                                        Text::new(label),
                                        TextFont {
                                            font_size: FontSize::Px(11.0),
                                            ..default()
                                        },
                                        TextColor(EditorTheme::TEXT_PRIMARY),
                                    ));
                                });

                                row.spawn(Node {
                                    padding: UiRect::axes(Val::Px(5.0), Val::Px(1.5)),
                                    border_radius: BorderRadius::all(Val::Px(3.0)),
                                    ..default()
                                })
                                .insert(BackgroundColor(jackdaw_feathers::tokens::WINDOW_BG))
                                .with_children(|right| {
                                    right.spawn((
                                        Text::new(tag),
                                        TextFont {
                                            font_size: FontSize::Px(8.5),
                                            ..default()
                                        },
                                        TextColor(accent_color),
                                    ));
                                });
                            });
                        }
                    }
                });
        });
    });
}

/// Updates the Bevy UI toast timer and visibility.
pub fn bevy_ui_toast_system(
    time: Res<Time>,
    mut toasts: Query<(&mut BevyUiToast, &mut Node)>,
) {
    for (mut toast, mut node) in &mut toasts {
        if node.display == Display::Flex {
            toast.timer.tick(time.delta());
            if toast.timer.is_finished() {
                node.display = Display::None;
            }
        }
    }
}

/// Handles interaction with the Cascading Add Node Palette:
/// - Hovering over a category row reveals that category's nodes in the right-hand side submenu.
/// - Mouse wheel scrolling scrolls the side submenu node list (`ScrollPosition`).
/// - Clicking a node spawns it on the canvas.
/// - Clicking `[+ Custom]` registers and spawns a brand-new customizable node type.
pub fn bevy_ui_quick_spawn_interaction_system(
    mut category_rows: Query<
        (&Interaction, &SpawnCategoryRow, &mut BackgroundColor),
        (Without<SpawnPaletteButton>, Without<ClosePaletteButton>),
    >,
    mut buttons: Query<
        (&Interaction, &SpawnPaletteButton, &mut BackgroundColor, &mut Node),
        (Without<SpawnCategoryRow>, Without<ClosePaletteButton>, Without<BevyUiSpawnPalette>, Without<BevyUiToast>),
    >,
    mut close_buttons: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ClosePaletteButton>, Without<SpawnPaletteButton>, Without<SpawnCategoryRow>),
    >,
    mut submenu_title_q: Query<&mut Text, (With<SpawnSideSubmenuTitle>, Without<BevyUiToastText>)>,
    mut scroll_list_q: Query<(&mut ScrollPosition, &Children), With<SpawnSideScrollList>>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut palette_query: Query<(&mut Node, &mut BevyUiSpawnPalette), Without<BevyUiToast>>,
    mut graph_res: ResMut<EditorGraphResource>,
    mut history_res: ResMut<EditorHistoryResource>,
    mut payload_res: ResMut<EditorDomainPayloadResource>,
    dataflow_res: Res<EditorDataflowResource>,
    id_gen: Res<EditorIdGenResource>,
) {
    // 1. Check if palette is open; handle mouse-wheel scrolling on the side submenu
    let is_palette_open = palette_query
        .iter()
        .next()
        .map(|(n, _)| n.display == Display::Flex)
        .unwrap_or(false);

    if is_palette_open && mouse_scroll.delta.y.abs() > 0.001 {
        let visible_count = buttons
            .iter()
            .filter(|(_, _, _, n)| n.display == Display::Flex)
            .count();
        let max_scroll = ((visible_count as f32 * 28.0) - 220.0).max(0.0);
        for (mut scroll_pos, _) in &mut scroll_list_q {
            scroll_pos.y = (scroll_pos.y - mouse_scroll.delta.y * 24.0).clamp(0.0, max_scroll);
        }
    }

    // 2. Category hover -> switch active category & update cascading side submenu
    let mut newly_hovered_cat: Option<String> = None;
    for (interaction, cat_row, _) in &category_rows {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            newly_hovered_cat = Some(cat_row.category.clone());
            break;
        }
    }

    if let Some(hovered_cat) = newly_hovered_cat {
        let mut changed_cat = false;
        for (_, mut pal) in &mut palette_query {
            if pal.active_category != hovered_cat {
                pal.active_category = hovered_cat.clone();
                changed_cat = true;
            }
        }

        if changed_cat {
            for mut title_text in &mut submenu_title_q {
                **title_text = hovered_cat.clone();
            }
            for (mut scroll_pos, _) in &mut scroll_list_q {
                scroll_pos.y = 0.0;
            }
            for (_, btn, _, mut node) in &mut buttons {
                node.display = if btn.category == hovered_cat {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
    }

    // Sync category row backgrounds
    let active_cat = palette_query
        .iter()
        .next()
        .map(|(_, p)| p.active_category.clone())
        .unwrap_or_default();
    for (interaction, cat_row, mut bg) in &mut category_rows {
        if cat_row.category == active_cat {
            *bg = BackgroundColor(jackdaw_feathers::tokens::ACCENT_BLUE);
        } else if *interaction == Interaction::Hovered {
            *bg = BackgroundColor(jackdaw_feathers::tokens::ELEVATED_BG);
        } else {
            *bg = BackgroundColor(Color::NONE);
        }
    }

    // 3. Close on [X] button
    for (interaction, mut bg) in &mut close_buttons {
        match *interaction {
            Interaction::Pressed => {
                for (mut node, _) in &mut palette_query {
                    node.display = Display::None;
                }
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(jackdaw_feathers::tokens::ELEVATED_BG);
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::NONE);
            }
        }
    }

    // 4. Spawn on template button click in the right side submenu
    for (interaction, btn, mut bg, _) in &mut buttons {
        match *interaction {
            Interaction::Pressed => {
                let spawn_canvas = palette_query
                    .iter()
                    .next()
                    .map(|(_, pal)| pal.spawn_canvas_pos)
                    .unwrap_or(Vec2::ZERO);

                let palette = NodePalette::from_registry(&dataflow_res.0.registry);
                if let Ok(new_id) = palette.spawn_node(
                    &btn.template,
                    spawn_canvas,
                    &mut graph_res.0,
                    &id_gen.0,
                    &mut history_res.0,
                ) {
                    payload_res.0.init_default_payload(new_id, &btn.template);
                }

                // Hide palette after spawning
                for (mut node, _) in &mut palette_query {
                    node.display = Display::None;
                }
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(jackdaw_feathers::tokens::ELEVATED_BG);
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::NONE);
            }
        }
    }
}


/// Toggles the quick spawn palette when pressing Space or Escape, anchoring it directly under the mouse cursor.
pub fn toggle_bevy_ui_quick_spawn_system(
    keys: Res<ButtonInput<KeyCode>>,
    input_guard: Res<luma_canvas::plugin::EditorCanvasInputGuard>,
    luma_focus: Option<Res<bevy_luma::prelude::UFocusState>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    mut palette_query: Query<(&mut Node, &mut BevyUiSpawnPalette)>,
) {
    if luma_focus.as_ref().is_some_and(|f| f.focused.is_some()) {
        return;
    }
    let window = match windows.iter().next() {
        Some(w) => w,
        None => return,
    };

    if keys.just_pressed(KeyCode::Space) {
        for (mut node, mut palette) in &mut palette_query {
            if node.display == Display::Flex {
                node.display = Display::None;
            } else if !input_guard.is_blocked {
                let cursor_screen = match window.cursor_position() {
                    Some(pos) => pos,
                    None => return,
                };

                // Clamp popover position so it stays within the visible viewport
                let left = cursor_screen.x.clamp(8.0, (window.width() - 470.0).max(8.0));
                let top = cursor_screen.y.clamp(38.0, (window.height() - 280.0).max(38.0));
                node.left = Val::Px(left);
                node.top = Val::Px(top);
                node.margin = UiRect::ZERO;

                // Calculate 2D world-space position under cursor where the new node will spawn
                if let Some((camera, cam_gt)) = camera_query.iter().next() {
                    if let Ok(world_pos) = camera.viewport_to_world_2d(cam_gt, cursor_screen) {
                        palette.spawn_canvas_pos = world_pos;
                    }
                }

                node.display = Display::Flex;
            }
        }
    }

    if keys.just_pressed(KeyCode::Escape) {
        for (mut node, _) in &mut palette_query {
            if node.display == Display::Flex {
                node.display = Display::None;
            }
        }
    }
}

/// Handles interaction with floating zoom controls and colorblind mode toggle.
pub fn bevy_ui_floating_toolbar_system(
    zoom_in: Query<&Interaction, (Changed<Interaction>, With<BevyUiBtnZoomIn>)>,
    zoom_out: Query<&Interaction, (Changed<Interaction>, With<BevyUiBtnZoomOut>)>,
    zoom_reset: Query<&Interaction, (Changed<Interaction>, With<BevyUiBtnZoomReset>)>,
    zoom_fit: Query<&Interaction, (Changed<Interaction>, With<BevyUiBtnZoomFit>)>,
    colorblind_btn: Query<&Interaction, (Changed<Interaction>, With<BevyUiBtnColorblind>)>,
    mut camera_q: Query<(&mut Transform, &mut Projection), With<EditorCamera>>,
    mut access_res: ResMut<EditorAccessibilitySettings>,
) {
    if let Some((mut cam_tf, mut proj)) = camera_q.iter_mut().next() {
        if let Projection::Orthographic(ref mut ortho) = *proj {
            for interaction in &zoom_in {
                if *interaction == Interaction::Pressed {
                    ortho.scale = (ortho.scale * 0.82).clamp(0.2, 5.0);
                }
            }
            for interaction in &zoom_out {
                if *interaction == Interaction::Pressed {
                    ortho.scale = (ortho.scale * 1.22).clamp(0.2, 5.0);
                }
            }
            for interaction in &zoom_reset {
                if *interaction == Interaction::Pressed {
                    ortho.scale = 1.0;
                }
            }
            for interaction in &zoom_fit {
                if *interaction == Interaction::Pressed {
                    ortho.scale = 1.1;
                    cam_tf.translation.x = -100.0;
                    cam_tf.translation.y = 0.0;
                }
            }
        }
    }
    for interaction in &colorblind_btn {
        if *interaction == Interaction::Pressed {
            access_res.colorblind_mode = !access_res.colorblind_mode;
        }
    }
}

