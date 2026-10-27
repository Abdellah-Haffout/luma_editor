use crate::canvas::plugin::{
    CameraAnimationResource, EditorCamera, EditorDataflowResource, EditorDomainPayloadResource,
    EditorGraphResource, EditorHistoryResource, EditorIdGenResource, EditorSelectionResource,
    EditorTransformResource, EditorWiringResource,
};
use crate::commands::descriptor::CommandId;
use crate::commands::dispatcher::CommandContext;
use crate::core::id::NodeId;
use crate::focus::context::NavigationState;
use crate::focus::tools::ContextualToolRegistry;
use crate::nodes::card::NodeCardLayout;
use crate::nodes::interaction::NodeDragState;
use crate::ui::bevy_ui_hud::{
    bevy_ui_floating_toolbar_system, bevy_ui_quick_spawn_interaction_system, bevy_ui_toast_system,
    setup_bevy_ui_hud, toggle_bevy_ui_quick_spawn_system, BevyUiSpawnPalette, BevyUiToast,
    EditorAccessibilitySettings,
};
use crate::ui::bevy_ui_inspector::{
    bevy_ui_inspector_actions_system, setup_bevy_ui_inspector, update_bevy_ui_inspector_system,
};
use crate::ui::components::focus_toolbar::FocusExitButton;
use crate::ui::components::header::{
    ActionButtonRedo, ActionButtonSaveBake, ActionButtonSpawn, ActionButtonUndo, BreadcrumbLabel,
};
use crate::ui::components::node_card::{
    node_card_scene, update_node_socket_pins_system, DiveInButton, NodeCardRoot, NodeCardVisual,
    NodeInputValLabel, NodeOutputValLabel, NodeParamStepButton, NodeParamValueText,
};
use crate::ui::components::status_bar::{StatusCoordsLabel, StatusSelectionLabel};
use crate::ui::layout::editor_root_scene;
use crate::ui::menu_bar::{
    bevy_ui_menu_actions_system, bevy_ui_menu_dismiss_system, bevy_ui_menu_item_hover_system,
    bevy_ui_menu_toggle_system, bevy_ui_menu_trigger_hover_system, bevy_ui_menu_visibility_system,
    setup_bevy_ui_menu_dropdowns, EditorMenuState,
};
use crate::ui::theme::EditorTheme;
use crate::ui::wires::{draw_canvas_wires_system, interactive_pin_wiring_system};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy_luma::prelude::*;
use bevy_luma_widgets::prelude::LumaWidgetsPlugin;

#[derive(Resource, Default)]
pub struct EditorNavigationResource(pub NavigationState);

#[derive(Resource)]
pub struct EditorContextualToolsResource(pub ContextualToolRegistry);

impl Default for EditorContextualToolsResource {
    fn default() -> Self {
        Self(ContextualToolRegistry::with_default_toolsets())
    }
}

#[derive(Resource, Default)]
pub struct EditorDragResource(pub NodeDragState);

pub struct LumaUiPlugin;

impl Plugin for LumaUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<LumaWidgetsPlugin>() {
            app.add_plugins(LumaWidgetsPlugin);
        }

        app.init_resource::<EditorNavigationResource>()
            .init_resource::<EditorContextualToolsResource>()
            .init_resource::<EditorDragResource>()
            .init_resource::<EditorAccessibilitySettings>()
            .init_resource::<EditorMenuState>()
            // Top Navbar + Status Bar via bevy_luma URootUi::screen() (correct, no Taffy 0-size bug).
            // Inspector panel, HUD overlays, and menu dropdowns via bevy_ui with fixed pixel sizes.
            .add_systems(
                PostStartup,
                (
                    setup_luma_screen_ui_system,
                    setup_bevy_ui_hud,
                    setup_bevy_ui_inspector,
                    setup_bevy_ui_menu_dropdowns,
                ),
            )
            .add_systems(
                Update,
                (
                    sync_node_cards_system,
                    node_drag_system,
                    update_node_cards_position_system,
                    update_node_socket_pins_system,
                    interactive_pin_wiring_system,
                    delete_selected_system,
                    subgraph_grouping_shortcuts_system,
                    draw_canvas_wires_system,
                    // Update bevy_luma UText labels in navbar and status bar each frame
                    update_luma_chrome_telemetry_system,
                    // Handle clicks on bevy_luma navbar action buttons
                    luma_chrome_actions_system,
                ),
            )
            .add_systems(
                Update,
                (
                    update_bevy_ui_inspector_system,
                    bevy_ui_inspector_actions_system,
                    ui_focus_navigation_system,
                    bevy_ui_toast_system,
                    bevy_ui_quick_spawn_interaction_system,
                    toggle_bevy_ui_quick_spawn_system,
                    bevy_ui_floating_toolbar_system,
                    node_param_step_system,
                    update_node_io_values_system,
                ),
            )
            .add_systems(
                Update,
                (
                    bevy_ui_menu_toggle_system,
                    bevy_ui_menu_visibility_system,
                    bevy_ui_menu_actions_system,
                    bevy_ui_menu_item_hover_system,
                    bevy_ui_menu_trigger_hover_system,
                    bevy_ui_menu_dismiss_system,
                ),
            );
    }
}

/// Spawns the bevy_luma screen-space UI (Top Navbar + Status Bar + Focus Toolbar)
/// via URootUi::screen(). This renders correctly at full window size without the
/// Taffy 0x0 layout bug that affects bevy_ui absolute-positioned root nodes.
pub fn setup_luma_screen_ui_system(
    mut commands: Commands,
    theme: Res<Theme>,
    transform: Res<EditorTransformResource>,
    selection: Res<EditorSelectionResource>,
    nav_res: Res<EditorNavigationResource>,
    tools_res: Res<EditorContextualToolsResource>,
) {
    let scene = editor_root_scene(
        &theme,
        &transform,
        &selection,
        &nav_res.0,
        &tools_res.0,
    );
    commands.spawn_scene(scene).insert(URootUi::screen());
}

/// Continuously updates the UText labels in the bevy_luma Top Navbar and Bottom Status Bar
/// with live telemetry: pan/zoom coordinates, selection state, and breadcrumb trail.
pub fn update_luma_chrome_telemetry_system(
    transform: Res<EditorTransformResource>,
    selection: Res<EditorSelectionResource>,
    nav_res: Res<EditorNavigationResource>,
    mut coords_query: Query<&mut UText, (With<StatusCoordsLabel>, Without<StatusSelectionLabel>)>,
    mut sel_query: Query<&mut UText, (With<StatusSelectionLabel>, Without<StatusCoordsLabel>)>,
    mut breadcrumb_query: Query<&mut UText, With<BreadcrumbLabel>>,
) {
    let coords_text = format!(
        "Pan: ({:.1}, {:.1})  |  Zoom: {:.0}%",
        transform.0.pan.x,
        transform.0.pan.y,
        transform.0.zoom * 100.0
    );

    let sel_text = if selection.0.is_empty() {
        "No selection".to_string()
    } else {
        format!(
            "Selected: {} node(s), {} wire(s)",
            selection.0.selected_nodes.len(),
            selection.0.selected_connections.len()
        )
    };

    let breadcrumb_text = nav_res.0.breadcrumbs().join("  ›  ");

    for mut utext in &mut coords_query {
        if utext.text != coords_text {
            utext.text = coords_text.clone();
        }
    }

    for mut utext in &mut sel_query {
        if utext.text != sel_text {
            utext.text = sel_text.clone();
        }
    }

    for mut utext in &mut breadcrumb_query {
        if utext.text != breadcrumb_text {
            utext.text = breadcrumb_text.clone();
        }
    }
}

/// Handles button clicks on the bevy_luma Navbar action buttons:
/// Spawn Node (opens the quick-spawn palette), Save & Bake, Undo, Redo.
pub fn luma_chrome_actions_system(
    spawn_clicks: Query<&UInteraction, (With<ActionButtonSpawn>, Changed<UInteraction>)>,
    bake_clicks: Query<&UInteraction, (With<ActionButtonSaveBake>, Changed<UInteraction>)>,
    undo_clicks: Query<&UInteraction, (With<ActionButtonUndo>, Changed<UInteraction>)>,
    redo_clicks: Query<&UInteraction, (With<ActionButtonRedo>, Changed<UInteraction>)>,
    mut palette_nodes: Query<(&mut Node, &mut BevyUiSpawnPalette), Without<BevyUiToast>>,
    mut toast_query: Query<(&mut BevyUiToast, &mut Node), Without<BevyUiSpawnPalette>>,
    mut dispatcher: ResMut<crate::canvas::plugin::EditorDispatcherResource>,
    mut graph_res: ResMut<EditorGraphResource>,
    mut history_res: ResMut<EditorHistoryResource>,
    mut payload_res: ResMut<EditorDomainPayloadResource>,
    mut dataflow_res: ResMut<EditorDataflowResource>,
) {
    for interaction in &spawn_clicks {
        if *interaction == UInteraction::Pressed {
            for (mut p_node, mut p_palette) in &mut palette_nodes {
                if p_node.display == Display::Flex {
                    p_node.display = Display::None;
                } else {
                    p_node.left = Val::Px(510.0);
                    p_node.top = Val::Px(72.0);
                    p_node.margin = UiRect::ZERO;
                    p_palette.spawn_canvas_pos = Vec2::ZERO;
                    p_node.display = Display::Flex;
                }
            }
        }
    }

    for interaction in &bake_clicks {
        if *interaction == UInteraction::Pressed {
            let mut ctx = CommandContext {
                graph: &mut graph_res.0,
                history: &mut history_res.0,
                payloads: &mut payload_res.0,
                dataflow: &mut dataflow_res.0,
            };
            let _ = dispatcher.0.dispatch(&CommandId::from("app.save_and_bake"), &mut ctx);
            for (mut toast, mut t_node) in &mut toast_query {
                t_node.display = Display::Flex;
                toast.timer.reset();
            }
        }
    }

    for interaction in &undo_clicks {
        if *interaction == UInteraction::Pressed {
            let _ = history_res.0.undo(&mut graph_res.0);
        }
    }

    for interaction in &redo_clicks {
        if *interaction == UInteraction::Pressed {
            let _ = history_res.0.redo(&mut graph_res.0);
        }
    }
}

/// Spawns and despawns in-world 2D node cards to match the nodes in EditorGraphResource.
/// Nodes are spawned ONCE without any CPU layout recalculations on zoom; the Camera handles all zooming!
pub fn sync_node_cards_system(
    mut commands: Commands,
    theme: Res<Theme>,
    graph_res: Res<EditorGraphResource>,
    mut selection_res: ResMut<EditorSelectionResource>,
    existing_roots: Query<(Entity, &NodeCardRoot)>,
) {
    let font = theme.text.font.inter_regular.clone();
    let graph = &graph_res.0;

    // Retain only nodes and connections that still exist in the graph (cleans up after Undo/Redo)
    selection_res.0.selected_nodes.retain(|id| graph.nodes().contains_key(id));
    selection_res.0.selected_connections.retain(|id| graph.connections().contains_key(id));

    let selection = &selection_res.0;

    let mut spawned_ids = std::collections::HashSet::new();
    for (entity, root) in &existing_roots {
        if !graph.nodes().contains_key(&root.node_id) {
            commands.entity(entity).despawn();
        } else {
            spawned_ids.insert(root.node_id);
        }
    }

    for (&node_id, _node) in graph.nodes() {
        if !spawned_ids.contains(&node_id) {
            if let Some(layout) = NodeCardLayout::from_graph(node_id, graph) {
                let is_selected = selection.is_node_selected(node_id);
                let mut root_ui = URootUi::world_2d(layout.size);
                root_ui.meters_per_unit = 1.0;
                root_ui.resolution_scale = 2.5;
                commands
                    .spawn_scene(node_card_scene(
                        font.clone(),
                        &layout,
                        is_selected,
                    ))
                    .insert(root_ui);
            }
        }
    }
}

/// Dynamically updates the world transform translation, opacity dimming, and selection border styling of all node cards.
pub fn update_node_cards_position_system(
    graph_res: Res<EditorGraphResource>,
    selection_res: Res<EditorSelectionResource>,
    nav_res: Res<EditorNavigationResource>,
    mut roots: Query<(&NodeCardRoot, &mut Transform)>,
    mut visuals: Query<(&NodeCardVisual, &mut UNode, &mut UBorder)>,
) {
    let graph = &graph_res.0;
    let selection = &selection_res.0;
    let is_focus = nav_res.0.is_in_focus();
    let focused_id = nav_res.0.focused_node_id();

    for (root, mut transform) in &mut roots {
        if let Some(node) = graph.get_node(root.node_id) {
            transform.translation.x = node.position.x;
            transform.translation.y = node.position.y;
        }
    }

    for (visual, mut unode, mut uborder) in &mut visuals {
        let is_selected = selection.is_node_selected(visual.node_id);
        if is_focus {
            if Some(visual.node_id) == focused_id {
                // Focused node: elevated background and bold focus accent border
                unode.background_color = Color::srgb(0.18, 0.20, 0.25);
                uborder.color = EditorTheme::BORDER_FOCUSED;
                uborder.width = 2.0;
            } else {
                // Dimmed node: reduced to 15% opacity to focus user attention
                unode.background_color = Color::srgba(0.14, 0.15, 0.17, 0.15);
                uborder.color = Color::srgba(1.0, 1.0, 1.0, 0.02);
                uborder.width = 1.0;
            }
        } else {
            // Standard macro canvas styling
            unode.background_color = Color::srgb(0.133, 0.141, 0.157);
            if is_selected {
                uborder.color = visual.type_color;
                uborder.width = 1.8;
            } else {
                uborder.color = Color::srgba(0.086, 0.090, 0.102, 0.95);
                uborder.width = 1.0;
            }
        }
    }
}

/// Handles instant first-click node selection, double-click dive-in, marquee box selection, and interactive dragging across the infinite canvas in world space.
/// Prevents clicks in the Top Navbar, Bottom Status Bar, or Pin Sockets from interfering with node drag.
pub fn node_drag_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    _motion: Res<AccumulatedMouseMotion>,
    keys: Res<ButtonInput<KeyCode>>,
    camera_query: Query<(&Camera, &GlobalTransform, &Projection), With<EditorCamera>>,
    wiring_res: Res<EditorWiringResource>,
    mut selection_res: ResMut<EditorSelectionResource>,
    mut graph_res: ResMut<EditorGraphResource>,
    mut history_res: ResMut<EditorHistoryResource>,
    mut drag_res: ResMut<EditorDragResource>,
    mut nav_res: ResMut<EditorNavigationResource>,
    mut anim: ResMut<CameraAnimationResource>,
    mut box_start_pos: Local<Option<Vec2>>,
    mut last_click: Local<Option<(NodeId, std::time::Instant)>>,
) {
    if keys.pressed(KeyCode::Space) || wiring_res.0.is_active() {
        return;
    }

    let window = match windows.iter().next() {
        Some(w) => w,
        None => return,
    };

    let cursor_screen = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    // Ignore clicks inside top navigation bar (y <= 68) or bottom status bar (y >= window.height() - 44)
    let in_top_chrome = cursor_screen.y <= 68.0;
    let in_bottom_chrome = cursor_screen.y >= (window.height() - 44.0);
    if in_top_chrome || in_bottom_chrome {
        return;
    }

    let (camera, cam_gt, projection) = match camera_query.iter().next() {
        Some(c) => c,
        None => return,
    };

    let scale = match projection {
        Projection::Orthographic(proj) => proj.scale,
        _ => 1.0,
    };

    let cursor_world = match camera.viewport_to_world_2d(cam_gt, cursor_screen).ok() {
        Some(p) => p,
        None => return,
    };

    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if mouse_buttons.just_pressed(MouseButton::Left) {
        // First check if click was near a pin socket: if so, leave for wiring system!
        let hit_radius = (16.0f32 * scale).max(12.0f32);
        let mut hit_pin = false;
        for (_node_id, node) in graph_res.0.nodes() {
            for &pin_id in node.inputs.iter().chain(node.outputs.iter()) {
                if let Some(pin_world) = crate::ui::wires::compute_pin_world_position(node, pin_id) {
                    if (cursor_world - pin_world).length() <= hit_radius {
                        hit_pin = true;
                        break;
                    }
                }
            }
            if hit_pin {
                break;
            }
        }
        if hit_pin {
            return;
        }

        // Hit-test nodes on canvas in world space
        let mut hit_node_id = None;
        for (&node_id, node) in graph_res.0.nodes() {
            let half_w = node.size.x * 0.5;
            let half_h = node.size.y * 0.5;
            if cursor_world.x >= node.position.x - half_w
                && cursor_world.x <= node.position.x + half_w
                && cursor_world.y >= node.position.y - half_h
                && cursor_world.y <= node.position.y + half_h
            {
                hit_node_id = Some(node_id);
                break;
            }
        }

        if let Some(node_id) = hit_node_id {
            // Check double-click to dive in (< 350ms on same node)
            let now = std::time::Instant::now();
            let is_double_click = if let Some((prev_id, prev_time)) = *last_click {
                prev_id == node_id && now.duration_since(prev_time).as_millis() < 350
            } else {
                false
            };
            *last_click = Some((node_id, now));

            if is_double_click {
                if let Some(node) = graph_res.0.get_node(node_id) {
                    let cam_pos = Vec2::new(cam_gt.translation().x, cam_gt.translation().y);
                    nav_res.0.dive_into(
                        node.id,
                        node.kind.clone(),
                        node.title.clone(),
                        cam_pos.x,
                        cam_pos.y,
                        scale,
                    );
                    anim.animate_to(cam_pos, scale, node.position, 0.65);
                }
                return;
            }

            // In atomic focus mode, dimmed nodes cannot be dragged
            if nav_res.0.is_in_focus() && Some(node_id) != nav_res.0.focused_node_id() {
                return;
            }

            if shift {
                selection_res.0.select_node(node_id, true);
            } else if !selection_res.0.is_node_selected(node_id) {
                selection_res.0.select_node(node_id, false);
            }

            // Immediately start dragging on the very first click
            let selected_nodes: Vec<_> = selection_res.0.selected_nodes.iter().copied().collect();
            if !selected_nodes.is_empty() {
                drag_res.0.begin_drag(&selected_nodes, cursor_world, &graph_res.0);
            }
            *box_start_pos = None;
            selection_res.0.box_selection = None;
        } else {
            // Clicked empty canvas space: start Marquee Box Selection
            *box_start_pos = Some(cursor_world);
            if !shift {
                selection_res.0.clear();
            }
        }
    }

    if mouse_buttons.pressed(MouseButton::Left) {
        if drag_res.0.is_dragging() {
            drag_res.0.update_drag(cursor_world, &mut graph_res.0);
        } else if let Some(start) = *box_start_pos {
            // Update marquee box selection rectangle
            if (cursor_world - start).length() > 4.0 {
                let min = start.min(cursor_world);
                let max = start.max(cursor_world);
                let rect = Rect { min, max };
                selection_res.0.box_selection = Some(rect);

                // Multi-select all nodes intersecting with the marquee rectangle
                for (&node_id, node) in graph_res.0.nodes() {
                    let half = node.size * 0.5;
                    let node_min = node.position - half;
                    let node_max = node.position + half;
                    let intersects = !(rect.max.x < node_min.x
                        || rect.min.x > node_max.x
                        || rect.max.y < node_min.y
                        || rect.min.y > node_max.y);
                    if intersects {
                        selection_res.0.selected_nodes.insert(node_id);
                    } else if !shift {
                        selection_res.0.selected_nodes.remove(&node_id);
                    }
                }
            }
        }
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        if drag_res.0.is_dragging() {
            let _ = drag_res.0.finish_drag(&mut graph_res.0, &mut history_res.0);
        }
        *box_start_pos = None;
        selection_res.0.box_selection = None;
    }
}

/// Allows deleting selected nodes and connections when pressing Delete or Backspace.
pub fn delete_selected_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut selection_res: ResMut<EditorSelectionResource>,
    mut graph_res: ResMut<EditorGraphResource>,
    mut history_res: ResMut<EditorHistoryResource>,
    mut payload_res: ResMut<EditorDomainPayloadResource>,
) {
    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        if !selection_res.0.selected_nodes.is_empty() {
            let selected_nodes: Vec<_> = selection_res.0.selected_nodes.iter().copied().collect();
            for node_id in selected_nodes {
                let cmd = Box::new(crate::core::history::RemoveNodeCommand::new(node_id));
                let _ = history_res.0.apply(cmd, &mut graph_res.0);
                payload_res.0.remove_payload(node_id);
            }
            selection_res.0.clear();
        }
    }
}

/// Handles Dive-In focus buttons, Tab hotkey focus toggle, and Focus Exit button interactions with smooth camera animation.
pub fn ui_focus_navigation_system(
    dive_clicks: Query<(&UInteraction, &DiveInButton), Changed<UInteraction>>,
    exit_clicks: Query<&UInteraction, (With<FocusExitButton>, Changed<UInteraction>)>,
    keys: Res<ButtonInput<KeyCode>>,
    selection_res: Res<EditorSelectionResource>,
    mut nav_res: ResMut<EditorNavigationResource>,
    graph_res: Res<EditorGraphResource>,
    camera_query: Query<(&Transform, &Projection), With<EditorCamera>>,
    mut anim: ResMut<CameraAnimationResource>,
) {
    let get_cam_state = || {
        if let Some((t, p)) = camera_query.iter().next() {
            let scale = match p {
                Projection::Orthographic(proj) => proj.scale,
                _ => 1.0,
            };
            (Vec2::new(t.translation.x, t.translation.y), scale)
        } else {
            (Vec2::ZERO, 1.0)
        }
    };

    for (interaction, btn) in &dive_clicks {
        if *interaction == UInteraction::Pressed {
            if let Some(node) = graph_res.0.get_node(btn.node_id) {
                let (cam_pos, cam_scale) = get_cam_state();
                nav_res.0.dive_into(node.id, node.kind.clone(), node.title.clone(), cam_pos.x, cam_pos.y, cam_scale);
                anim.animate_to(cam_pos, cam_scale, node.position, 0.65);
            }
        }
    }

    for interaction in &exit_clicks {
        if *interaction == UInteraction::Pressed {
            let (cur_pos, cur_scale) = get_cam_state();
            if let Some(crate::focus::WorkspaceContext::FocusedNode { saved_cam_x, saved_cam_y, saved_cam_scale, .. }) = nav_res.0.exit_focus() {
                anim.animate_to(cur_pos, cur_scale, Vec2::new(saved_cam_x, saved_cam_y), saved_cam_scale);
            } else {
                anim.animate_to(cur_pos, cur_scale, Vec2::ZERO, 1.0);
            }
        }
    }

    // Tab hotkey toggles focus into selected node or back to graph
    if keys.just_pressed(KeyCode::Tab) {
        if nav_res.0.is_in_focus() {
            let (cur_pos, cur_scale) = get_cam_state();
            if let Some(crate::focus::WorkspaceContext::FocusedNode { saved_cam_x, saved_cam_y, saved_cam_scale, .. }) = nav_res.0.exit_focus() {
                anim.animate_to(cur_pos, cur_scale, Vec2::new(saved_cam_x, saved_cam_y), saved_cam_scale);
            } else {
                anim.animate_to(cur_pos, cur_scale, Vec2::ZERO, 1.0);
            }
        } else if selection_res.0.selected_nodes.len() == 1 {
            let node_id = *selection_res.0.selected_nodes.iter().next().unwrap();
            if let Some(node) = graph_res.0.get_node(node_id) {
                let (cam_pos, cam_scale) = get_cam_state();
                nav_res.0.dive_into(node.id, node.kind.clone(), node.title.clone(), cam_pos.x, cam_pos.y, cam_scale);
                anim.animate_to(cam_pos, cam_scale, node.position, 0.65);
            }
        }
    }

    // Esc hotkey exits focus
    if keys.just_pressed(KeyCode::Escape) && nav_res.0.is_in_focus() {
        let (cur_pos, cur_scale) = get_cam_state();
        if let Some(crate::focus::WorkspaceContext::FocusedNode { saved_cam_x, saved_cam_y, saved_cam_scale, .. }) = nav_res.0.exit_focus() {
            anim.animate_to(cur_pos, cur_scale, Vec2::new(saved_cam_x, saved_cam_y), saved_cam_scale);
        } else {
            anim.animate_to(cur_pos, cur_scale, Vec2::ZERO, 1.0);
        }
    }
}

/// Handles grouping selected nodes into a composite Subgraph (`Ctrl + G`) and ungrouping (`Ctrl + Alt + G` or `Ctrl + Shift + G`).
pub fn subgraph_grouping_shortcuts_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut selection_res: ResMut<EditorSelectionResource>,
    mut graph_res: ResMut<EditorGraphResource>,
    mut history_res: ResMut<EditorHistoryResource>,
    id_gen_res: Res<EditorIdGenResource>,
    mut toast_query: Query<(&mut crate::ui::bevy_ui_hud::BevyUiToast, &mut Node)>,
    mut toast_text: Query<&mut Text, With<crate::ui::bevy_ui_hud::BevyUiToastText>>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let alt = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if ctrl && keys.just_pressed(KeyCode::KeyG) {
        if alt || shift {
            // Ungroup selected Subgraph (Undoable with Ctrl+Z)
            let selected: Vec<_> = selection_res.0.selected_nodes.iter().copied().collect();
            if selected.len() == 1 {
                let sub_id = selected[0];
                let unpacked_holder = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
                let cmd = Box::new(
                    crate::nodes::subgraph::UngroupNodeCommand::new(sub_id, id_gen_res.0.clone())
                        .with_output_channel(unpacked_holder.clone()),
                );

                if history_res.0.apply(cmd, &mut graph_res.0).is_ok() {
                    let unpacked = unpacked_holder.lock().unwrap().clone();
                    selection_res.0.clear();
                    for id in &unpacked {
                        selection_res.0.selected_nodes.insert(*id);
                    }
                    for (mut toast, mut t_node) in &mut toast_query {
                        t_node.display = Display::Flex;
                        toast.timer.reset();
                    }
                    for mut text in &mut toast_text {
                        **text = format!("✔ Ungrouped Subgraph into {} node(s) (Undo with Ctrl+Z)", unpacked.len());
                    }
                }
            }
        } else {
            // Group selected nodes (Undoable with Ctrl+Z)
            let selected: Vec<_> = selection_res.0.selected_nodes.iter().copied().collect();
            if !selected.is_empty() {
                let created_holder = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
                let cmd = Box::new(
                    crate::nodes::subgraph::GroupNodesCommand::new(selected.clone(), id_gen_res.0.clone())
                        .with_output_channel(created_holder.clone()),
                );

                if history_res.0.apply(cmd, &mut graph_res.0).is_ok() {
                    let sub_id = NodeId(created_holder.load(std::sync::atomic::Ordering::Relaxed));
                    selection_res.0.clear();
                    if sub_id.0 > 0 {
                        selection_res.0.selected_nodes.insert(sub_id);
                    }
                    for (mut toast, mut t_node) in &mut toast_query {
                        t_node.display = Display::Flex;
                        toast.timer.reset();
                    }
                    for mut text in &mut toast_text {
                        **text = format!("✔ Grouped {} node(s) into Subgraph (Undo with Ctrl+Z)", selected.len());
                    }
                }
            }
        }
    }
}

/// Handles interactive +/- stepping for numeric parameter nodes (UnivisEditor visual hook pattern).
pub fn node_param_step_system(
    buttons: Query<(&UInteraction, &NodeParamStepButton), Changed<UInteraction>>,
    mut text_query: Query<(&mut UText, &NodeParamValueText)>,
    graph_res: Res<EditorGraphResource>,
    mut dataflow_res: ResMut<EditorDataflowResource>,
) {
    for (interaction, btn) in &buttons {
        if *interaction == UInteraction::Pressed {
            let node_id = btn.node_id;
            let delta = btn.delta;

            if let Some(node) = graph_res.0.get_node(node_id) {
                if let Some(&pin_id) = node.inputs.first() {
                    let cur_val = match dataflow_res.0.pin_values.get(&pin_id) {
                        Some(crate::dataflow::value::PrimitiveValue::Float(f)) => *f,
                        Some(crate::dataflow::value::PrimitiveValue::Int(i)) => *i as f32,
                        _ => 10.0,
                    };
                    let new_val = cur_val + delta;
                    let downstream = dataflow_res.0.set_pin_value(
                        &graph_res.0,
                        pin_id,
                        crate::dataflow::value::PrimitiveValue::Float(new_val),
                    );

                    for (mut utext, text_marker) in &mut text_query {
                        if text_marker.node_id == node_id {
                            utext.text = if new_val.fract() == 0.0 {
                                format!("{:.1}", new_val)
                            } else {
                                format!("{:.2}", new_val)
                            };
                        }
                    }

                    // Propagate dirty push and evaluate downstream pipeline
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
}

/// Continuously synchronizes live input values and computed output results
/// to the textual socket labels on node cards.
pub fn update_node_io_values_system(
    graph_res: Res<EditorGraphResource>,
    dataflow_res: Res<EditorDataflowResource>,
    mut in_text_query: Query<(&mut UText, &NodeInputValLabel), Without<NodeOutputValLabel>>,
    mut out_text_query: Query<(&mut UText, &NodeOutputValLabel), Without<NodeInputValLabel>>,
) {
    let graph = &graph_res.0;
    let engine = &dataflow_res.0;

    for (mut utext, in_label) in &mut in_text_query {
        if let Some(node) = graph.get_node(in_label.node_id) {
            if let Some(&pin_id) = node.inputs.get(in_label.slot_idx) {
                if let Some(pin) = graph.get_pin(pin_id) {
                    let val = engine.resolve_input_value(graph, in_label.node_id, in_label.slot_idx);
                    let val_str = val.to_concise_string();
                    let target = if !val_str.is_empty() {
                        format!("{}: {}", pin.name, val_str)
                    } else {
                        pin.name.clone()
                    };
                    if utext.text != target {
                        utext.text = target;
                    }
                }
            } else if !utext.text.is_empty() {
                utext.text.clear();
            }
        }
    }

    for (mut utext, out_label) in &mut out_text_query {
        if let Some(node) = graph.get_node(out_label.node_id) {
            if let Some(&pin_id) = node.outputs.get(out_label.slot_idx) {
                if let Some(pin) = graph.get_pin(pin_id) {
                    let target = if let Some(outputs) = engine.get_cached_outputs(out_label.node_id) {
                        if let Some(val) = outputs.get(out_label.slot_idx) {
                            let val_str = val.to_concise_string();
                            if !val_str.is_empty() {
                                format!("{}: = {}", pin.name, val_str)
                            } else {
                                pin.name.clone()
                            }
                        } else {
                            pin.name.clone()
                        }
                    } else {
                        pin.name.clone()
                    };
                    if utext.text != target {
                        utext.text = target;
                    }
                }
            } else if !utext.text.is_empty() {
                utext.text.clear();
            }
        }
    }
}
