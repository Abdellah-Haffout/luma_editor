use luma_canvas::plugin::{
    canvas_pan_zoom_system, canvas_shortcuts_system, CameraAnimationResource, EditorCamera,
    EditorCanvasInputGuard, EditorDataflowResource, EditorDomainPayloadResource,
    EditorGraphResource, EditorHistoryResource, EditorIdGenResource, EditorSelectionResource,
    EditorWiringResource,
};
use luma_core::id::NodeId;
use luma_canvas::focus::context::NavigationState;
use luma_canvas::focus::tools::ContextualToolRegistry;
use luma_canvas::nodes::card::NodeCardLayout;
use luma_canvas::nodes::interaction::NodeDragState;
use crate::bevy_ui_chrome::{
    bevy_ui_chrome_actions_system, bevy_ui_chrome_hover_system, setup_bevy_ui_chrome,
    update_bevy_ui_chrome_telemetry_system, BevyUiBtnExitFocus,
};
use crate::bevy_ui_hud::{
    bevy_ui_floating_toolbar_system, bevy_ui_quick_spawn_interaction_system, bevy_ui_toast_system,
    setup_bevy_ui_hud, toggle_bevy_ui_quick_spawn_system, EditorAccessibilitySettings,
};
use crate::bevy_ui_inspector::{
    bevy_ui_inspector_actions_system, setup_bevy_ui_inspector, update_bevy_ui_inspector_system,
};
use crate::bevy_ui_settings::{
    bevy_ui_settings_category_sync_system, bevy_ui_settings_hover_system,
    bevy_ui_settings_hud_sync_system, bevy_ui_settings_interaction_system,
    bevy_ui_settings_toggle_sync_system, bevy_ui_settings_visibility_system,
    setup_bevy_ui_settings, EditorSettingsState,
};
use crate::components::focus_toolbar::FocusExitButton;
use crate::components::node_card::{
    spawn_luma_node_card_with_registry, update_node_socket_pins_system, DiveInButton,
    NodeBodyUiRegistry, NodeCardHeaderBar, NodeCardRoot, NodeCardSocketPin, NodeCardVisual,
    NodeInputValLabel, NodeLiveInputReadoutText, NodeLiveOutputReadoutText, NodeOutputValLabel,
    NodeParamStepButton, NodeParamValueText,
};
use crate::menu_bar::{
    bevy_ui_menu_actions_system, bevy_ui_menu_dismiss_system, bevy_ui_menu_item_hover_system,
    bevy_ui_menu_toggle_system, bevy_ui_menu_trigger_hover_system, bevy_ui_menu_visibility_system,
    setup_bevy_ui_menu_dropdowns, EditorMenuState,
};
use crate::snarl::{setup_snarl_infrastructure_system, PendingSnarlNodes};
use crate::theme::EditorTheme;
use crate::wires::{
    draw_canvas_wires_system, interactive_pin_wiring_system, sync_luma_connections_system,
    update_luma_connections_system, update_luma_ghost_connection_system,
};
use crate::UiInteraction as Interaction;
use bevy::prelude::*;
use bevy_luma::prelude::*;

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

/// Marker component for the dedicated screen UI overlay camera.
/// This camera stays fixed at screen space, immune to canvas zoom/pan.
use bevy::asset::embedded_asset;
use crate::connection_material::LumaConnectionMaterial;

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct EditorUiCamera;

pub struct LumaUiPlugin;

impl Plugin for LumaUiPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/connection.wesl");

        app.add_plugins((
            bevy::feathers::FeathersPlugins,
            jackdaw_feathers::icons::IconFontPlugin,
            jackdaw_widgets::EditorWidgetsPlugins,
            jackdaw_panels::DockPlugin,
            crate::dock_layout::EditorDockLayoutPlugin,
            crate::web_image::WebImagePlugin,
            UiMaterialPlugin::<LumaConnectionMaterial>::default(),
        ))
        .init_resource::<EditorNavigationResource>()
        .init_resource::<EditorContextualToolsResource>()
        .init_resource::<EditorDragResource>()
        .init_resource::<EditorAccessibilitySettings>()
        .init_resource::<EditorMenuState>()
        .init_resource::<EditorSettingsState>()
        .init_resource::<NodeBodyUiRegistry>()
        .init_resource::<PendingSnarlNodes>()
        .add_systems(PreStartup, setup_snarl_infrastructure_system)
        .add_systems(
            PostStartup,
            (
                setup_bevy_ui_chrome,
                (
                    setup_bevy_ui_hud,
                    setup_bevy_ui_inspector,
                    setup_bevy_ui_settings,
                    setup_bevy_ui_menu_dropdowns,
                    crate::dock_layout::setup_editor_dock_layout,
                ),
                crate::dock_layout::attach_overlay_dock_anchors,
            )
                .chain(),
        )
            .add_systems(
                Update,
                update_canvas_input_guard_system.before(canvas_pan_zoom_system).before(canvas_shortcuts_system),
            )
            .add_systems(
                Update,
                (
                    sync_node_cards_system,
                    bind_node_cards_camera_system,
                    node_drag_system,
                    update_node_cards_position_system,
                    update_node_socket_pins_system,
                    interactive_pin_wiring_system,
                    delete_selected_system,
                    subgraph_grouping_shortcuts_system,
                    sync_luma_connections_system,
                    update_luma_connections_system,
                    update_luma_ghost_connection_system,
                    draw_canvas_wires_system,
                )
                    .after(update_canvas_input_guard_system),
            )
            .add_systems(
                Update,
                (
                    update_bevy_ui_chrome_telemetry_system,
                    bevy_ui_chrome_actions_system,
                    bevy_ui_chrome_hover_system,
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
                )
                    .after(update_canvas_input_guard_system),
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
            )
            .add_systems(
                Update,
                (
                    bevy_ui_settings_visibility_system,
                    bevy_ui_settings_interaction_system,
                    bevy_ui_settings_hover_system,
                    bevy_ui_settings_category_sync_system,
                    bevy_ui_settings_toggle_sync_system,
                    bevy_ui_settings_hud_sync_system,
                ),
            );

        // Auto-register any compile-time discovered UI / node systems
        for sys in inventory::iter::<crate::snarl::RegisteredLumaSystem> {
            (sys.register)(app);
        }
    }
}

/// Updates `EditorCanvasInputGuard` every frame before any canvas or node-graph system runs.
/// Blocks all node-graph interactions whenever:
/// 1. The mouse cursor is outside the window or outside the Node Editor dock area.
/// 2. A modal or popup menu is active (Settings modal, Top Menu dropdown, or Add Node palette).
/// 3. The mouse cursor is hovering or pressing any UI button / overlay.
///
/// While a mouse button is held, the decision taken on press is latched, so dragging a dock
/// gap or a tab across the canvas never starts a box-selection, and a canvas pan/drag that
/// leaves the Node Editor area keeps working until the button is released.
pub fn update_canvas_input_guard_system(
    mut guard: ResMut<EditorCanvasInputGuard>,
    windows: Query<&Window>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    settings_state: Res<EditorSettingsState>,
    menu_state: Res<EditorMenuState>,
    palette_q: Query<&Node, With<crate::bevy_ui_hud::BevyUiSpawnPalette>>,
    viewport_rect: Res<crate::dock_layout::NodeEditorViewportRect>,
    dock_drag: Option<Res<jackdaw_panels::drag::DockDragState>>,
    ui_interactions: Query<&Interaction>,
    mut press_latch: Local<Option<bool>>,
) {
    // Modals / menus always win.
    let is_palette_open = palette_q.iter().any(|n| n.display == Display::Flex);
    let dock_dragging = dock_drag
        .as_deref()
        .is_some_and(|d| !matches!(d, jackdaw_panels::drag::DockDragState::Idle));
    if settings_state.is_open || menu_state.open_menu.is_some() || is_palette_open || dock_dragging {
        guard.is_blocked = true;
        return;
    }

    let raw_blocked = (|| {
        let Some(window) = windows.iter().next() else {
            return true;
        };
        let Some(pos) = window.cursor_position() else {
            return true;
        };
        // Cursor must be inside the visible Node Editor area.
        let Some(rect) = viewport_rect.0 else {
            return true;
        };
        if !rect.contains(pos) {
            return true;
        }
        // Any hovered / pressed UI overlay (toolbar, minimap, inspector ...) blocks the canvas.
        ui_interactions.iter().any(|i| *i != Interaction::None)
    })();

    let any_pressed = mouse_buttons.get_pressed().next().is_some();
    if mouse_buttons.get_just_pressed().next().is_some() && press_latch.is_none() {
        *press_latch = Some(raw_blocked);
    }
    if !any_pressed {
        *press_latch = None;
    }

    guard.is_blocked = match *press_latch {
        Some(latched) if any_pressed => latched,
        _ => raw_blocked,
    };
}

/// Spawns and despawns physical in-world 2D node cards to match the nodes in EditorGraphResource.
/// Nodes live in world space and are zoomed/panned naturally by the 2D Camera on the GPU.
pub fn sync_node_cards_system(
    mut commands: Commands,
    theme_opt: Option<Res<Theme>>,
    graph_res: Res<EditorGraphResource>,
    dataflow_res: Res<EditorDataflowResource>,
    ui_registry: Option<Res<NodeBodyUiRegistry>>,
    mut selection_res: ResMut<EditorSelectionResource>,
    existing_roots: Query<(Entity, &NodeCardRoot)>,
    editor_cam: Query<Entity, With<EditorCamera>>,
) {
    let target_cam = editor_cam.iter().next();
    let graph = &graph_res.0;
    let font = theme_opt.map(|t| t.text.font.inter_regular.clone()).unwrap_or_default();

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
            if let Some(layout) = NodeCardLayout::from_graph_with_registry(
                node_id,
                graph,
                Some(&dataflow_res.0.registry),
            ) {
                let is_selected = selection.is_node_selected(node_id);
                spawn_luma_node_card_with_registry(
                    &mut commands,
                    font.clone(),
                    &layout,
                    is_selected,
                    ui_registry.as_deref(),
                    target_cam,
                );
            }
        }
    }
}

/// Automatically binds node card `URootUi` instances explicitly to the `EditorCamera`.
/// Eliminates `AutoAmbiguousCamera` warnings when secondary 3D cameras exist in the scene.
pub fn bind_node_cards_camera_system(
    editor_cam: Query<Entity, With<EditorCamera>>,
    mut roots: Query<&mut URootUi, With<NodeCardRoot>>,
) {
    if let Some(cam) = editor_cam.iter().next() {
        for mut root in &mut roots {
            if root.camera == UiCameraRef::Auto {
                root.camera = UiCameraRef::Entity(cam);
            }
        }
    }
}

/// Dynamically updates the world transform translation, opacity dimming, and selection border styling of all node cards.
/// With bevy_luma in world space, the 2D camera naturally handles all panning and zooming on the GPU!
pub fn update_node_cards_position_system(
    graph_res: Res<EditorGraphResource>,
    selection_res: Res<EditorSelectionResource>,
    nav_res: Res<EditorNavigationResource>,
    mut roots: Query<(&NodeCardRoot, &mut Transform)>,
    mut visuals: Query<(&NodeCardVisual, &mut UNode, &mut UBorder), Without<NodeCardHeaderBar>>,
    mut headers: Query<(&NodeCardHeaderBar, &mut UNode), Without<NodeCardVisual>>,
) {
    let graph = &graph_res.0;
    let selection = &selection_res.0;
    let is_focus = nav_res.0.is_in_focus();
    let focused_id = nav_res.0.focused_node_id();
    let blender_unselected_border = Color::srgb(0.075, 0.075, 0.085);

    for (root, mut transform) in &mut roots {
        if let Some(node) = graph.get_node(root.node_id) {
            transform.translation.x = node.position.x;
            transform.translation.y = node.position.y;
        }
    }

    for (header_bar, mut header_node) in &mut headers {
        if let Some(ov) =
            luma_canvas::nodes::custom::get_node_instance_customization(header_bar.node_id)
        {
            if let Some(c) = ov.header_bg {
                header_node.background_color = Color::srgba(c.r, c.g, c.b, c.a);
            }
        }
    }

    for (visual, mut unode, mut uborder) in &mut visuals {
        let is_selected = selection.is_node_selected(visual.node_id);
        let (body_bg, accent_color) = if let Some(ov) =
            luma_canvas::nodes::custom::get_node_instance_customization(visual.node_id)
        {
            let b = ov
                .body_bg
                .map(|c| Color::srgba(c.r, c.g, c.b, c.a))
                .unwrap_or(visual.body_bg);
            let a = ov
                .accent_color
                .map(|c| Color::srgba(c.r, c.g, c.b, c.a))
                .unwrap_or(visual.type_color);
            (b, a)
        } else {
            (visual.body_bg, visual.type_color)
        };
        if is_focus {
            if Some(visual.node_id) == focused_id {
                unode.background_color = body_bg;
                uborder.color = EditorTheme::BORDER_FOCUSED;
                uborder.width = 2.0;
            } else {
                unode.background_color = Color::srgb(0.06, 0.065, 0.075);
                uborder.color = blender_unselected_border;
                uborder.width = 1.0;
            }
        } else {
            unode.background_color = body_bg;
            if is_selected {
                uborder.color = accent_color;
                uborder.width = 2.0;
            } else {
                uborder.color = blender_unselected_border;
                uborder.width = 1.4;
            }
        }
    }
}

/// Handles instant first-click node selection, double-click dive-in, marquee box selection, and interactive dragging across the infinite canvas in world space.
/// Prevents clicks in the Top Navbar, Bottom Status Bar, or Pin Sockets from interfering with node drag.
pub fn node_drag_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    keys: Res<ButtonInput<KeyCode>>,
    input_guard: Res<EditorCanvasInputGuard>,
    camera_query: Query<(&Camera, &GlobalTransform, &Projection), With<EditorCamera>>,
    (card_roots, socket_pins, interactive_widgets): (
        Query<(&NodeCardRoot, &GlobalTransform)>,
        Query<(&NodeCardSocketPin, &GlobalTransform)>,
        Query<
            &UInteraction,
            (
                Without<NodeCardSocketPin>,
                Without<NodeCardRoot>,
            ),
        >,
    ),
    wiring_res: Res<EditorWiringResource>,
    mut selection_res: ResMut<EditorSelectionResource>,
    mut graph_res: ResMut<EditorGraphResource>,
    mut history_res: ResMut<EditorHistoryResource>,
    mut drag_res: ResMut<EditorDragResource>,
    mut nav_res: ResMut<EditorNavigationResource>,
    mut anim: ResMut<CameraAnimationResource>,
    luma_focus: Option<ResMut<bevy_luma::prelude::UFocusState>>,
    (mut box_start_pos, mut last_click): (
        Local<Option<Vec2>>,
        Local<Option<(NodeId, std::time::Instant)>>,
    ),
    settings_state: Res<EditorSettingsState>,
) {
    if mouse_buttons.just_released(MouseButton::Left) && input_guard.is_blocked {
        if drag_res.0.is_dragging() {
            let _ = drag_res.0.finish_drag(&mut graph_res.0, &mut history_res.0);
        }
        *box_start_pos = None;
        selection_res.0.box_selection = None;
        return;
    }

    if input_guard.is_blocked || keys.pressed(KeyCode::Space) || wiring_res.0.is_active() || settings_state.is_open {
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
        let live_pins = crate::wires::build_live_pin_positions_map(&graph_res.0, &card_roots, &socket_pins);
        let mut hit_pin = false;
        for (_node_id, node) in graph_res.0.nodes() {
            for &pin_id in node.inputs.iter().chain(node.outputs.iter()) {
                if let Some(pin_world) = live_pins
                    .get(&pin_id)
                    .copied()
                    .or_else(|| crate::wires::compute_pin_world_position(node, pin_id))
                {
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

        // If cursor is hovering or pressing an interactive in-node widget (e.g. UTextInput, Stepper, DiveIn),
        // let the widget handle the click instead of starting a node card drag!
        if interactive_widgets
            .iter()
            .any(|ui| *ui == UInteraction::Hovered || *ui == UInteraction::Pressed)
        {
            return;
        }

        // Hit-test nodes on canvas in world space using their actual rendered card size
        let mut hit_node_id = None;
        for (&node_id, node) in graph_res.0.nodes() {
            let sz = NodeCardLayout::compute_size_fast(&node.kind, node.size, node.inputs.len(), node.outputs.len());
            let half_w = sz.x * 0.5;
            let half_h = sz.y * 0.5;
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
            if let Some(mut focus) = luma_focus {
                focus.focused = None;
            }
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
            if let Some(mut focus) = luma_focus {
                focus.focused = None;
            }
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
    input_guard: Res<EditorCanvasInputGuard>,
    luma_focus: Option<Res<bevy_luma::prelude::UFocusState>>,
    mut selection_res: ResMut<EditorSelectionResource>,
    mut graph_res: ResMut<EditorGraphResource>,
    mut history_res: ResMut<EditorHistoryResource>,
    mut payload_res: ResMut<EditorDomainPayloadResource>,
) {
    if input_guard.is_blocked || luma_focus.as_ref().is_some_and(|f| f.focused.is_some()) {
        return;
    }
    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        if !selection_res.0.selected_nodes.is_empty() {
            let selected_nodes: Vec<_> = selection_res.0.selected_nodes.iter().copied().collect();
            for node_id in selected_nodes {
                let cmd = Box::new(luma_core::history::RemoveNodeCommand::new(node_id));
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
    exit_clicks: Query<&Interaction, (Or<(With<FocusExitButton>, With<BevyUiBtnExitFocus>)>, Changed<Interaction>)>,
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
        if *interaction == Interaction::Pressed {
            let (cur_pos, cur_scale) = get_cam_state();
            if let Some(luma_canvas::focus::WorkspaceContext::FocusedNode { saved_cam_x, saved_cam_y, saved_cam_scale, .. }) = nav_res.0.exit_focus() {
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
            if let Some(luma_canvas::focus::WorkspaceContext::FocusedNode { saved_cam_x, saved_cam_y, saved_cam_scale, .. }) = nav_res.0.exit_focus() {
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
        if let Some(luma_canvas::focus::WorkspaceContext::FocusedNode { saved_cam_x, saved_cam_y, saved_cam_scale, .. }) = nav_res.0.exit_focus() {
            anim.animate_to(cur_pos, cur_scale, Vec2::new(saved_cam_x, saved_cam_y), saved_cam_scale);
        } else {
            anim.animate_to(cur_pos, cur_scale, Vec2::ZERO, 1.0);
        }
    }
}

/// Handles grouping selected nodes into a composite Subgraph (`Ctrl + G`) and ungrouping (`Ctrl + Alt + G` or `Ctrl + Shift + G`).
pub fn subgraph_grouping_shortcuts_system(
    keys: Res<ButtonInput<KeyCode>>,
    input_guard: Res<EditorCanvasInputGuard>,
    mut selection_res: ResMut<EditorSelectionResource>,
    mut graph_res: ResMut<EditorGraphResource>,
    mut history_res: ResMut<EditorHistoryResource>,
    id_gen_res: Res<EditorIdGenResource>,
    mut toast_query: Query<(&mut crate::bevy_ui_hud::BevyUiToast, &mut Node)>,
    mut toast_text: Query<&mut Text, With<crate::bevy_ui_hud::BevyUiToastText>>,
) {
    if input_guard.is_blocked {
        return;
    }
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
                    luma_canvas::nodes::subgraph::UngroupNodeCommand::new(sub_id, id_gen_res.0.clone())
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
                    luma_canvas::nodes::subgraph::GroupNodesCommand::new(selected.clone(), id_gen_res.0.clone())
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
                        Some(luma_dataflow::value::PrimitiveValue::Float(f)) => *f,
                        Some(luma_dataflow::value::PrimitiveValue::Int(i)) => *i as f32,
                        _ => 10.0,
                    };
                    let new_val = cur_val + delta;
                    let downstream = dataflow_res.0.set_pin_value(
                        &graph_res.0,
                        pin_id,
                        luma_dataflow::value::PrimitiveValue::Float(new_val),
                    );

                    for (mut text, text_marker) in &mut text_query {
                        if text_marker.node_id == node_id {
                            let formatted = if new_val.fract() == 0.0 {
                                format!("{:.1}", new_val)
                            } else {
                                format!("{:.2}", new_val)
                            };
                            text.text = formatted;
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
/// to the textual socket labels, inline parameter steppers, and Math node live readout boxes.
pub fn update_node_io_values_system(
    graph_res: Res<EditorGraphResource>,
    mut dataflow_res: ResMut<EditorDataflowResource>,
    mut in_text_query: Query<
        (&mut UText, &NodeInputValLabel),
        (
            Without<NodeOutputValLabel>,
            Without<NodeParamValueText>,
            Without<NodeLiveInputReadoutText>,
            Without<NodeLiveOutputReadoutText>,
        ),
    >,
    mut out_text_query: Query<
        (&mut UText, &NodeOutputValLabel),
        (
            Without<NodeInputValLabel>,
            Without<NodeParamValueText>,
            Without<NodeLiveInputReadoutText>,
            Without<NodeLiveOutputReadoutText>,
        ),
    >,
    mut param_text_query: Query<
        (&mut UText, &NodeParamValueText),
        (
            Without<NodeInputValLabel>,
            Without<NodeOutputValLabel>,
            Without<NodeLiveInputReadoutText>,
            Without<NodeLiveOutputReadoutText>,
        ),
    >,
    mut live_in_readout_q: Query<
        (&mut UText, &NodeLiveInputReadoutText),
        (
            Without<NodeInputValLabel>,
            Without<NodeOutputValLabel>,
            Without<NodeParamValueText>,
            Without<NodeLiveOutputReadoutText>,
        ),
    >,
    mut live_out_readout_q: Query<
        (&mut UText, &NodeLiveOutputReadoutText),
        (
            Without<NodeInputValLabel>,
            Without<NodeOutputValLabel>,
            Without<NodeParamValueText>,
            Without<NodeLiveInputReadoutText>,
        ),
    >,
) {
    let graph = &graph_res.0;

    // 1. Whenever the graph changes (new wires, disconnected wires, spawned nodes) or any
    // registered node lacks a clean cache entry, evaluate the dataflow graph automatically.
    let needs_eval = graph_res.is_changed()
        || graph.nodes().iter().any(|(&nid, n)| {
            dataflow_res.0.registry.get(&n.kind).is_some()
                && dataflow_res.0.get_cached_outputs(nid).is_none()
        });

    if needs_eval {
        let node_ids: Vec<NodeId> = graph.nodes().keys().copied().collect();
        for &nid in &node_ids {
            dataflow_res.0.cache.mark_dirty(nid);
        }
        let pinned = std::collections::HashSet::new();
        for &nid in &node_ids {
            if let Some(node) = graph.get_node(nid) {
                if dataflow_res.0.registry.get(&node.kind).is_some() {
                    let _ = dataflow_res.0.pull_node(graph, nid, &pinned);
                }
            }
        }
    }

    let engine = &dataflow_res.0;

    // 2. Sync inline parameter stepper text (e.g. math.number, math.integer) to its live value
    for (mut text, param_marker) in &mut param_text_query {
        if graph.get_node(param_marker.node_id).is_some() {
            let val = engine.resolve_input_value(graph, param_marker.node_id, 0);
            let val_str = val.to_concise_string();
            if !val_str.is_empty() && text.text != val_str {
                text.text = val_str;
            }
        }
    }

    // 3. Sync input pin labels (keep socket labels clean on Math nodes that have a dedicated readout box)
    for (mut text, in_label) in &mut in_text_query {
        if let Some(node) = graph.get_node(in_label.node_id) {
            if let Some(&pin_id) = node.inputs.get(in_label.slot_idx) {
                if let Some(pin) = graph.get_pin(pin_id) {
                    let has_readout_box = NodeCardLayout::has_math_value_readout(&node.kind)
                        || matches!(node.kind.as_str(), "math.number" | "math.integer");
                    let target = if has_readout_box {
                        pin.name.clone()
                    } else {
                        let val = engine.resolve_input_value(graph, in_label.node_id, in_label.slot_idx);
                        let val_str = val.to_concise_string();
                        if !val_str.is_empty() {
                            format!("{}: {}", pin.name, val_str)
                        } else {
                            pin.name.clone()
                        }
                    };
                    if text.text != target {
                        text.text = target;
                    }
                }
            } else if !text.text.is_empty() {
                text.text.clear();
            }
        }
    }

    // 4. Sync output pin labels (keep socket labels clean on Math nodes that have a dedicated readout box)
    for (mut text, out_label) in &mut out_text_query {
        if let Some(node) = graph.get_node(out_label.node_id) {
            if let Some(&pin_id) = node.outputs.get(out_label.slot_idx) {
                if let Some(pin) = graph.get_pin(pin_id) {
                    let has_readout_box = NodeCardLayout::has_math_value_readout(&node.kind);
                    let target = if has_readout_box {
                        pin.name.clone()
                    } else if let Some(outputs) = engine.get_cached_outputs(out_label.node_id) {
                        if let Some(val) = outputs.get(out_label.slot_idx) {
                            let val_str = val.to_concise_string();
                            if !val_str.is_empty() {
                                format!("{} = {}", pin.name, val_str)
                            } else {
                                pin.name.clone()
                            }
                        } else {
                            pin.name.clone()
                        }
                    } else {
                        pin.name.clone()
                    };
                    if text.text != target {
                        text.text = target;
                    }
                }
            } else if !text.text.is_empty() {
                text.text.clear();
            }
        }
    }

    // 5. Sync Live Received Inputs Readout on Math Nodes (clean centered numbers)
    for (mut text, readout_marker) in &mut live_in_readout_q {
        if let Some(node) = graph.get_node(readout_marker.node_id) {
            let target = if node.inputs.is_empty() {
                String::new()
            } else {
                let mut parts = Vec::with_capacity(node.inputs.len());
                for (idx, &pin_id) in node.inputs.iter().enumerate() {
                    let pin_name = graph
                        .get_pin(pin_id)
                        .map(|p| p.name.as_str())
                        .unwrap_or("in");
                    let val = engine.resolve_input_value(graph, readout_marker.node_id, idx);
                    let val_str = val.to_concise_string();
                    if node.inputs.len() == 1 {
                        parts.push(val_str);
                    } else {
                        parts.push(format!("{pin_name}: {val_str}"));
                    }
                }
                parts.join("  |  ")
            };
            if text.text != target {
                text.text = target;
            }
        }
    }

    // 6. Sync Live Computed Output Readout on Math Nodes (clean centered numbers)
    for (mut text, readout_marker) in &mut live_out_readout_q {
        if let Some(node) = graph.get_node(readout_marker.node_id) {
            let target = if let Some(outputs) = engine.get_cached_outputs(readout_marker.node_id) {
                let mut parts = Vec::with_capacity(outputs.len());
                for (idx, val) in outputs.iter().enumerate() {
                    let val_str = val.to_concise_string();
                    if outputs.len() == 1 {
                        parts.push(format!("= {val_str}"));
                    } else {
                        let pin_name = node
                            .outputs
                            .get(idx)
                            .and_then(|&pid| graph.get_pin(pid))
                            .map(|p| p.name.as_str())
                            .unwrap_or("out");
                        parts.push(format!("{pin_name} = {val_str}"));
                    }
                }
                parts.join("  |  ")
            } else {
                "= 0.0".to_string()
            };
            if text.text != target {
                text.text = target;
            }
        }
    }
}
