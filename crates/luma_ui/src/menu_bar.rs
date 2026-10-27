use bevy::app::AppExit;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use crate::{UiButton as Button, UiInteraction as Interaction};
use luma_canvas::plugin::{
    CameraAnimationResource, EditorCamera, EditorDataflowResource, EditorDispatcherResource,
    EditorDomainPayloadResource, EditorGraphResource, EditorHistoryResource, EditorIdGenResource,
    EditorSelectionResource,
};
use luma_commands::descriptor::CommandId;
use luma_commands::dispatcher::CommandContext;
use luma_core::id::NodeId;
use crate::bevy_ui_hud::{BevyUiSpawnPalette, BevyUiToast, BevyUiToastText};
use crate::bevy_ui_inspector::BevyUiInspectorRoot;
use crate::theme::EditorTheme;

/// Categorical top-level menu kinds in the application header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Default)]
pub enum MenuKind {
    #[default]
    File,
    Edit,
    View,
    Help,
}

impl MenuKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Edit => "Edit",
            Self::View => "View",
            Self::Help => "Help",
        }
    }
}

/// Dispatched actions from clicking menu items.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum MenuItemAction {
    // File
    FileNew,
    FileSaveAndBake,
    FileResetDemo,
    FileQuit,

    // Edit
    EditUndo,
    EditRedo,
    EditGroupSubgraph,
    EditDeleteSelected,
    EditSelectAll,
    EditClearSelection,
    EditSettings,

    // View
    ViewTogglePalette,
    ViewToggleInspector,
    ViewResetZoom,

    // Help
    HelpShortcuts,
    HelpAbout,
}

/// Global active state of the top menu bar.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorMenuState {
    pub open_menu: Option<MenuKind>,
}

/// Marker component for top-bar menu trigger buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BevyUiMenuTrigger(pub MenuKind);

/// Marker component for the dropdown popup panel of a specific menu.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BevyUiDropdownPanel(pub MenuKind);

/// Marker component for menu item action buttons inside dropdown panels.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BevyUiMenuItemButton(pub MenuItemAction);

/// Spawns the menu trigger buttons ("File", "Edit", "View", "Help") into the top bar.
pub fn spawn_menu_triggers(
    parent: &mut ChildSpawnerCommands,
    font: FontSource,
) {
    let menus = [
        MenuKind::File,
        MenuKind::Edit,
        MenuKind::View,
        MenuKind::Help,
    ];

    for menu in menus {
        parent
            .spawn((
                BevyUiMenuTrigger(menu),
                Button,
                Node {
                    height: Val::Px(22.0),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new(menu.label()),
                    TextFont {
                        font: font.clone(),
                        font_size: FontSize::Px(12.0),
                        ..default()
                    },
                    TextColor(EditorTheme::TEXT_PRIMARY),
                ));
            });
    }
}

/// Spawns the 4 dropdown menu panels (File, Edit, View, Help) as top-level overlays.
pub fn setup_bevy_ui_menu_dropdowns(
    mut commands: Commands,
    camera_q: Query<Entity, With<crate::EditorUiCamera>>,
) {
    let cam_opt = camera_q.iter().next();
    let font = FontSource::default();

    // Definitions for all dropdown menus: (MenuKind, X-offset, Items: [(Label, Shortcut, Action), ...])
    let dropdowns = [
        (
            MenuKind::File,
            Val::Px(106.0),
            vec![
                ("New Graph", "Ctrl+N", MenuItemAction::FileNew),
                ("Save & Bake", "Ctrl+S", MenuItemAction::FileSaveAndBake),
                ("Reset Workspace", "F5", MenuItemAction::FileResetDemo),
                ("Quit Luma", "Alt+F4", MenuItemAction::FileQuit),
            ],
        ),
        (
            MenuKind::Edit,
            Val::Px(148.0),
            vec![
                ("Undo", "Ctrl+Z", MenuItemAction::EditUndo),
                ("Redo", "Ctrl+Y", MenuItemAction::EditRedo),
                ("Group Selection", "Ctrl+G", MenuItemAction::EditGroupSubgraph),
                ("Delete Selected", "Del", MenuItemAction::EditDeleteSelected),
                ("Select All", "Ctrl+A", MenuItemAction::EditSelectAll),
                ("Clear Selection", "Esc", MenuItemAction::EditClearSelection),
                ("Settings...", "Ctrl+,", MenuItemAction::EditSettings),
            ],
        ),
        (
            MenuKind::View,
            Val::Px(190.0),
            vec![
                ("Add Node (Palette)", "Tab", MenuItemAction::ViewTogglePalette),
                ("Inspector Panel", "I", MenuItemAction::ViewToggleInspector),
                ("Reset Camera Zoom", "Home", MenuItemAction::ViewResetZoom),
            ],
        ),
        (
            MenuKind::Help,
            Val::Px(236.0),
            vec![
                ("Keyboard Shortcuts", "?", MenuItemAction::HelpShortcuts),
                ("About Luma Editor", "F1", MenuItemAction::HelpAbout),
            ],
        ),
    ];

    for (kind, left_px, items) in dropdowns {
        let mut panel_cmd = commands.spawn((
            BevyUiDropdownPanel(kind),
            GlobalZIndex(250),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(70.0),
                left: left_px,
                width: Val::Px(210.0),
                padding: UiRect::all(Val::Px(4.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                border: UiRect::ZERO,
                border_radius: BorderRadius::all(Val::Px(8.0)),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.09, 0.12)),
            BorderColor::all(Color::NONE),
        ));

        if let Some(cam) = cam_opt {
            panel_cmd.insert(UiTargetCamera(cam));
        }

        panel_cmd.with_children(|panel| {
            for (idx, (label, shortcut, action)) in items.into_iter().enumerate() {
                // Add a visual separator between logical sections
                if kind == MenuKind::File && (idx == 2 || idx == 3)
                    || kind == MenuKind::Edit && (idx == 2 || idx == 4 || idx == 6)
                    || kind == MenuKind::View && idx == 2
                {
                    panel.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(1.0),
                            margin: UiRect::vertical(Val::Px(3.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.18, 0.20, 0.26)),
                    ));
                }

                // Menu item button
                panel
                    .spawn((
                        BevyUiMenuItemButton(action),
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(26.0),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Row,
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|row| {
                        // Action label
                        row.spawn((
                            Text::new(label),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(11.5),
                                ..default()
                            },
                            TextColor(EditorTheme::TEXT_PRIMARY),
                        ));

                        // Keyboard shortcut badge
                        row.spawn((
                            Text::new(shortcut),
                            TextFont {
                                font: font.clone(),
                                font_size: FontSize::Px(10.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.50, 0.55, 0.65)),
                        ));
                    });
            }
        });
    }
}

/// Toggles dropdown menus on click, and allows smooth switching on hover when open.
pub fn bevy_ui_menu_toggle_system(
    mut menu_state: ResMut<EditorMenuState>,
    triggers: Query<(&Interaction, &BevyUiMenuTrigger), Changed<Interaction>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        menu_state.open_menu = None;
        return;
    }

    for (interaction, trigger) in &triggers {
        match *interaction {
            Interaction::Pressed => {
                if menu_state.open_menu == Some(trigger.0) {
                    menu_state.open_menu = None;
                } else {
                    menu_state.open_menu = Some(trigger.0);
                }
            }
            Interaction::Hovered => {
                // If a menu is already open, hovering another menu trigger switches to it immediately!
                if menu_state.open_menu.is_some() {
                    menu_state.open_menu = Some(trigger.0);
                }
            }
            _ => {}
        }
    }
}

/// Synchronizes dropdown visibility with EditorMenuState.
pub fn bevy_ui_menu_visibility_system(
    menu_state: Res<EditorMenuState>,
    mut panels: Query<(&mut Node, &BevyUiDropdownPanel)>,
) {
    if !menu_state.is_changed() {
        return;
    }

    for (mut node, panel) in &mut panels {
        if menu_state.open_menu == Some(panel.0) {
            node.display = Display::Flex;
        } else {
            node.display = Display::None;
        }
    }
}

/// Smooth hover highlight for menu items inside dropdown panels.
pub fn bevy_ui_menu_item_hover_system(
    mut items: Query<
        (&Interaction, &mut BackgroundColor),
        (With<BevyUiMenuItemButton>, Changed<Interaction>),
    >,
) {
    for (interaction, mut bg) in &mut items {
        match *interaction {
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.18, 0.20, 0.26)),
            Interaction::Pressed => *bg = BackgroundColor(Color::srgb(0.24, 0.27, 0.35)),
            Interaction::None => *bg = BackgroundColor(Color::NONE),
        }
    }
}

/// Subtle hover highlight for top-bar menu triggers.
pub fn bevy_ui_menu_trigger_hover_system(
    menu_state: Res<EditorMenuState>,
    mut triggers: Query<
        (&Interaction, &BevyUiMenuTrigger, &mut BackgroundColor),
        (With<BevyUiMenuTrigger>, Changed<Interaction>),
    >,
) {
    for (interaction, trigger, mut bg) in &mut triggers {
        let is_open = menu_state.open_menu == Some(trigger.0);
        if is_open {
            *bg = BackgroundColor(Color::srgb(0.20, 0.22, 0.28));
        } else {
            match *interaction {
                Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.18, 0.20, 0.26)),
                Interaction::Pressed => *bg = BackgroundColor(Color::srgb(0.24, 0.27, 0.35)),
                Interaction::None => *bg = BackgroundColor(Color::NONE),
            }
        }
    }
}

/// Dismisses open menus when clicking anywhere outside the menu bars.
pub fn bevy_ui_menu_dismiss_system(
    mouse: Res<ButtonInput<MouseButton>>,
    mut menu_state: ResMut<EditorMenuState>,
    triggers: Query<&Interaction, With<BevyUiMenuTrigger>>,
    items: Query<&Interaction, With<BevyUiMenuItemButton>>,
) {
    if menu_state.open_menu.is_none() {
        return;
    }

    if mouse.just_pressed(MouseButton::Left) {
        let over_trigger = triggers.iter().any(|i| *i == Interaction::Hovered || *i == Interaction::Pressed);
        let over_item = items.iter().any(|i| *i == Interaction::Hovered || *i == Interaction::Pressed);

        if !over_trigger && !over_item {
            menu_state.open_menu = None;
        }
    }
}

/// Executes actions triggered by clicking menu items.
pub fn bevy_ui_menu_actions_system(
    item_clicks: Query<(&Interaction, &BevyUiMenuItemButton), Changed<Interaction>>,
    mut menu_state: ResMut<EditorMenuState>,
    mut app_exit: MessageWriter<AppExit>,
    camera_query: Query<(&Transform, &Projection), With<EditorCamera>>,
    mut anim: ResMut<CameraAnimationResource>,
    mut palette_nodes: Query<(&mut Node, &mut BevyUiSpawnPalette), (Without<BevyUiToast>, Without<BevyUiDropdownPanel>, Without<BevyUiInspectorRoot>)>,
    mut inspector_nodes: Query<&mut Node, (With<BevyUiInspectorRoot>, Without<BevyUiToast>, Without<BevyUiDropdownPanel>)>,
    mut toast_query: Query<(&mut BevyUiToast, &mut Node), (Without<BevyUiSpawnPalette>, Without<BevyUiDropdownPanel>, Without<BevyUiInspectorRoot>)>,
    mut toast_text: Query<&mut Text, With<BevyUiToastText>>,
    mut dispatcher: ResMut<EditorDispatcherResource>,
    mut graph_res: ResMut<EditorGraphResource>,
    mut history_res: ResMut<EditorHistoryResource>,
    mut payload_res: ResMut<EditorDomainPayloadResource>,
    mut dataflow_res: ResMut<EditorDataflowResource>,
    mut selection_res: ResMut<EditorSelectionResource>,
    id_gen_res: Res<EditorIdGenResource>,
) {
    for (interaction, btn) in &item_clicks {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Close menu on click
        menu_state.open_menu = None;

        match btn.0 {
            MenuItemAction::FileNew => {
                graph_res.0.clear();
                selection_res.0.clear();
                history_res.0.clear();
                dataflow_res.0.cache.caches.clear();
                dataflow_res.0.pin_values.clear();

                for (mut toast, mut t_node) in &mut toast_query {
                    t_node.display = Display::Flex;
                    toast.timer.reset();
                }
                for mut text in &mut toast_text {
                    **text = "✔ Blank graph created".to_string();
                }
            }

            MenuItemAction::FileSaveAndBake => {
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
                for mut text in &mut toast_text {
                    **text = "✔ Saved to scenes/main.luma & baked to scenes/main.bsn".to_string();
                }
            }

            MenuItemAction::FileResetDemo => {
                graph_res.0.clear();
                selection_res.0.clear();
                history_res.0.clear();
                dataflow_res.0.cache.caches.clear();
                dataflow_res.0.pin_values.clear();

                for (mut toast, mut t_node) in &mut toast_query {
                    t_node.display = Display::Flex;
                    toast.timer.reset();
                }
                for mut text in &mut toast_text {
                    **text = "✔ Workspace reset to clean state".to_string();
                }
            }

            MenuItemAction::FileQuit => {
                app_exit.write(AppExit::Success);
            }

            MenuItemAction::EditUndo => {
                let _ = history_res.0.undo(&mut graph_res.0);
                for (mut toast, mut t_node) in &mut toast_query {
                    t_node.display = Display::Flex;
                    toast.timer.reset();
                }
                for mut text in &mut toast_text {
                    **text = "↺ Undo performed".to_string();
                }
            }

            MenuItemAction::EditRedo => {
                let _ = history_res.0.redo(&mut graph_res.0);
                for (mut toast, mut t_node) in &mut toast_query {
                    t_node.display = Display::Flex;
                    toast.timer.reset();
                }
                for mut text in &mut toast_text {
                    **text = "↻ Redo performed".to_string();
                }
            }

            MenuItemAction::EditGroupSubgraph => {
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
                            **text = format!("✔ Grouped {} node(s) into Subgraph", selected.len());
                        }
                    }
                }
            }

            MenuItemAction::EditDeleteSelected => {
                let selected: Vec<_> = selection_res.0.selected_nodes.iter().copied().collect();
                for node_id in &selected {
                    graph_res.0.remove_node(*node_id);
                }
                selection_res.0.clear();
                for (mut toast, mut t_node) in &mut toast_query {
                    t_node.display = Display::Flex;
                    toast.timer.reset();
                }
                for mut text in &mut toast_text {
                    **text = format!("✔ Deleted {} selected node(s)", selected.len());
                }
            }

            MenuItemAction::EditSelectAll => {
                for &id in graph_res.0.nodes().keys() {
                    selection_res.0.selected_nodes.insert(id);
                }
                for (mut toast, mut t_node) in &mut toast_query {
                    t_node.display = Display::Flex;
                    toast.timer.reset();
                }
                for mut text in &mut toast_text {
                    **text = format!("✔ Selected {} node(s)", selection_res.0.selected_nodes.len());
                }
            }

            MenuItemAction::EditClearSelection => {
                selection_res.0.clear();
                for (mut toast, mut t_node) in &mut toast_query {
                    t_node.display = Display::Flex;
                    toast.timer.reset();
                }
                for mut text in &mut toast_text {
                    **text = "✔ Selection cleared".to_string();
                }
            }

            MenuItemAction::EditSettings => {
                // Handled directly by bevy_ui_settings_interaction_system
            }

            MenuItemAction::ViewTogglePalette => {
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

            MenuItemAction::ViewToggleInspector => {
                for mut node in &mut inspector_nodes {
                    node.display = if node.display == Display::Flex {
                        Display::None
                    } else {
                        Display::Flex
                    };
                }
            }

            MenuItemAction::ViewResetZoom => {
                let (cur_pos, cur_scale) = if let Some((t, p)) = camera_query.iter().next() {
                    let scale = match p {
                        Projection::Orthographic(proj) => proj.scale,
                        _ => 1.0,
                    };
                    (Vec2::new(t.translation.x, t.translation.y), scale)
                } else {
                    (Vec2::ZERO, 1.0)
                };

                anim.animate_to(cur_pos, cur_scale, Vec2::ZERO, 1.0);

                for (mut toast, mut t_node) in &mut toast_query {
                    t_node.display = Display::Flex;
                    toast.timer.reset();
                }
                for mut text in &mut toast_text {
                    **text = "✔ Camera reset to origin (1.0x)".to_string();
                }
            }

            MenuItemAction::HelpShortcuts => {
                for (mut toast, mut t_node) in &mut toast_query {
                    t_node.display = Display::Flex;
                    toast.timer.reset();
                }
                for mut text in &mut toast_text {
                    **text = "⌨ Tab: Palette | Del: Delete | Ctrl+G: Group | Ctrl+Z/Y: Undo/Redo | Ctrl+S: Bake".to_string();
                }
            }

            MenuItemAction::HelpAbout => {
                for (mut toast, mut t_node) in &mut toast_query {
                    t_node.display = Display::Flex;
                    toast.timer.reset();
                }
                for mut text in &mut toast_text {
                    **text = "ℹ Luma Editor v0.1.0 — Pure GPU SDF Node & Dataflow Engine".to_string();
                }
            }
        }
    }
}
