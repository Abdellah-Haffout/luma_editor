//! Blender-style editor layout built on top of `jackdaw_panels`.
//!
//! The whole area between the top header and the bottom status bar is a single
//! [`DockTreeHost`]. The layout itself is a pure binary [`DockTree`] (splits +
//! tabbed leaves) that `jackdaw_panels`' reconciler materializes into UI every
//! time it changes. That gives us, for free:
//!
//! * Smooth resizing of every area by dragging the thin gaps between them
//!   (`PanelHandle`s, which write straight back into the tree fractions).
//! * Tabs that can be dragged into another area, or onto an area edge to split
//!   it in two (drop overlay preview, just like Blender's area split/join).
//! * A `+` button in every tab bar to open any registered editor in that area.
//! * Multiple Blender-like Workspaces (Node Graph, Full Canvas, Dataflow)
//!   switchable from the top header bar via [`jackdaw_panels::WorkspaceTabStrip`].
//!
//! The node canvas is rendered by the world camera *behind* the UI, so the
//! [`window_ids::NODE_EDITOR`] area is kept transparent and acts as a "hole"
//! through which the canvas is visible. Screen-space overlays (inspector, zoom
//! toolbar, minimap, toasts ...) are glued to their dock area with
//! [`DockAnchored`] so they follow the area while it is resized or moved.

use std::sync::Arc;

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use jackdaw_feathers::{icons::Icon, tokens};
use jackdaw_panels::reconcile::DockTreeHost;
use jackdaw_panels::tree::{DockLeaf, DockNode, DockSplit, DockTree, SplitAxis};
use jackdaw_panels::{
    ActiveDockWindow, DockArea, DockAreaStyle, DockTabContent, DockWindowDescriptor, IconFontHandle,
    LayoutState, PanelHandle, WindowRegistry, WorkspaceDescriptor, WorkspaceRegistry,
};
use luma_canvas::nodes::palette::NodePalette;
use luma_dataflow::registry::NodeRegistry;

use crate::EditorUiCamera;

/// Stable ids of essential editor windows in the dock layout.
pub mod window_ids {
    pub const NODE_EDITOR: &str = "node_editor";
    pub const OUTLINER: &str = "outliner";
    pub const LIBRARY: &str = "library";
    pub const INSPECTOR: &str = "inspector";
}

/// Height of the fixed top header (menu + toolbar), in logical pixels.
pub const HEADER_HEIGHT: f32 = 34.0;
/// Height of the fixed bottom status bar, in logical pixels.
pub const STATUS_BAR_HEIGHT: f32 = 22.0;
/// Colour of the thin gaps between areas (Blender-like dark separators).
pub const AREA_GAP_COLOR: Color = Color::srgb(0.07, 0.07, 0.08);

/// Marker on the root dock host entity.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct EditorDockRoot;

/// Marker on the (transparent) content node of the node editor window.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct NodeEditorViewport;

/// Logical-pixel rectangle of the node editor area currently visible on
/// screen, or `None` when the node editor is not shown in any area.
/// Updated every frame in `PostUpdate`, consumed by the canvas input guard.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct NodeEditorViewportRect(pub Option<Rect>);

/// Where an overlay is positioned relative to its dock area.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DockAnchor {
    /// Cover the whole area.
    Fill,
    /// Horizontally centred, `offset` px below the top of the area.
    TopCenter(f32),
    /// Horizontally centred, `offset` px above the bottom of the area.
    BottomCenter(f32),
    /// `offset` px from the bottom-left corner.
    BottomLeft(Vec2),
    /// `offset` px from the bottom-right corner.
    BottomRight(Vec2),
}

/// Glues an absolutely-positioned overlay to the visible content of a dock
/// window. The overlay is hidden while that window is not visible anywhere.
#[derive(Component, Clone, Copy, Debug)]
pub struct DockAnchored {
    pub window: &'static str,
    pub anchor: DockAnchor,
}

impl DockAnchored {
    pub const fn new(window: &'static str, anchor: DockAnchor) -> Self {
        Self { window, anchor }
    }
}

pub struct EditorDockLayoutPlugin;

impl Plugin for EditorDockLayoutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NodeEditorViewportRect>()
            .add_observer(on_dock_tab_close_press)
            .add_observer(on_dock_tab_close_click)
            .add_observer(on_dock_tab_auxiliary_click)
            .add_observer(on_dock_grip_right_click)
            .add_observer(set_close_btn_hover::<PointerOver>(Color::srgba(0.85, 0.25, 0.25, 0.35)))
            .add_observer(set_close_btn_hover::<PointerOut>(Color::NONE))
            .add_systems(
                PostUpdate,
                (
                    sync_dock_backgrounds,
                    update_node_editor_viewport_rect,
                    follow_dock_anchors,
                )
                    .chain()
                    .before(bevy::ui::UiSystems::Layout),
            );
    }
}

/// Builds the default 3-column Blender-like layout:
///
/// ```text
/// +-------------------+------------------------------------------+-------------------+
/// |     OUTLINER      |                                          |    PROPERTIES     |
/// |   NODE LIBRARY    |               NODE EDITOR                |                   |
/// |                   |                                          |                   |
/// |                   |               (Full Height)              |                   |
/// |                   |                                          |                   |
/// +-------------------+------------------------------------------+-------------------+
/// ```
pub fn default_dock_tree() -> DockTree {
    use window_ids::*;

    let leaf = |area: &str, windows: &[&str]| {
        DockNode::Leaf(
            DockLeaf::new(area, DockAreaStyle::TabBar)
                .with_windows(windows.iter().map(|w| w.to_string()).collect()),
        )
    };
    let split = |axis, fraction, a, b| DockNode::Split(DockSplit { axis, fraction, a, b });

    let mut tree = DockTree::new();

    // Left sidebar: Outliner + Node Catalog tabs (20%)
    let left = tree.insert(leaf("left_sidebar", &[OUTLINER, LIBRARY]));

    // Center viewport: Full height Node Editor canvas (60%)
    let viewport = tree.insert(DockNode::Leaf(
        DockLeaf::new("center", DockAreaStyle::TabBar)
            .with_windows(vec![NODE_EDITOR.into()])
            .persistent(),
    ));

    // Right sidebar: Node Inspector & Properties (20%)
    let right = tree.insert(leaf("right_sidebar", &[INSPECTOR]));

    // Combine Center + Right, then Left + (Center + Right)
    let center_right = tree.insert(split(SplitAxis::Horizontal, 0.75, viewport, right));
    let root = tree.insert(split(SplitAxis::Horizontal, 0.20, left, center_right));

    tree.root = Some(root);
    tree
}

/// Builds a focused Full-Canvas workspace with a slim utility sidebar.
pub fn full_canvas_dock_tree() -> DockTree {
    use window_ids::*;

    let leaf = |area: &str, windows: &[&str]| {
        DockNode::Leaf(
            DockLeaf::new(area, DockAreaStyle::TabBar)
                .with_windows(windows.iter().map(|w| w.to_string()).collect()),
        )
    };

    let mut tree = DockTree::new();

    let viewport = tree.insert(DockNode::Leaf(
        DockLeaf::new("center", DockAreaStyle::TabBar)
            .with_windows(vec![NODE_EDITOR.into()])
            .persistent(),
    ));
    let right = tree.insert(leaf("right_sidebar", &[INSPECTOR, OUTLINER, LIBRARY]));
    let root = tree.insert(DockNode::Split(DockSplit {
        axis: SplitAxis::Horizontal,
        fraction: 0.82,
        a: viewport,
        b: right,
    }));

    tree.root = Some(root);
    tree
}

/// Builds a Node Library & Dataflow workspace with an expanded node catalog.
pub fn dataflow_dock_tree() -> DockTree {
    use window_ids::*;

    let leaf = |area: &str, windows: &[&str]| {
        DockNode::Leaf(
            DockLeaf::new(area, DockAreaStyle::TabBar)
                .with_windows(windows.iter().map(|w| w.to_string()).collect()),
        )
    };
    let split = |axis, fraction, a, b| DockNode::Split(DockSplit { axis, fraction, a, b });

    let mut tree = DockTree::new();

    let left = tree.insert(leaf("left_sidebar", &[LIBRARY, OUTLINER]));

    let viewport = tree.insert(DockNode::Leaf(
        DockLeaf::new("center", DockAreaStyle::TabBar)
            .with_windows(vec![NODE_EDITOR.into()])
            .persistent(),
    ));
    let right = tree.insert(leaf("right_sidebar", &[INSPECTOR]));
    let center_right = tree.insert(split(SplitAxis::Horizontal, 0.72, viewport, right));

    let root = tree.insert(split(SplitAxis::Horizontal, 0.28, left, center_right));
    tree.root = Some(root);
    tree
}

/// Registers every dock window, seeds the default workspaces and spawns the dock host.
pub fn setup_editor_dock_layout(
    mut commands: Commands,
    ui_cam_query: Query<Entity, With<EditorUiCamera>>,
    icon_font: Res<jackdaw_feathers::icons::IconFont>,
    dataflow_res: Res<luma_canvas::plugin::EditorDataflowResource>,
    mut registry: ResMut<WindowRegistry>,
    mut workspace_registry: ResMut<WorkspaceRegistry>,
    mut dock_tree: ResMut<DockTree>,
) {
    commands.insert_resource(IconFontHandle(icon_font.0.clone()));

    register_dock_windows(&mut registry, &dataflow_res.0.registry);
    setup_workspaces(&mut workspace_registry, &mut dock_tree);

    let mut host = commands.spawn((
        EditorDockRoot,
        DockTreeHost::default(),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            top: Val::Px(HEADER_HEIGHT),
            bottom: Val::Px(STATUS_BAR_HEIGHT),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::NONE),
        GlobalZIndex(10),
    ));
    if let Some(cam) = ui_cam_query.iter().next() {
        host.insert(UiTargetCamera(cam));
    }
}

fn setup_workspaces(registry: &mut WorkspaceRegistry, dock_tree: &mut DockTree) {
    let graph_tree = default_dock_tree();
    let canvas_tree = full_canvas_dock_tree();
    let debug_tree = dataflow_dock_tree();

    *dock_tree = graph_tree.clone();

    registry.register(WorkspaceDescriptor {
        id: "graph".into(),
        name: "Node Graph".into(),
        icon: Some(String::from(Icon::Workflow.unicode())),
        accent_color: tokens::ACCENT_BLUE,
        layout: LayoutState::default(),
        tree: graph_tree,
    });

    registry.register(WorkspaceDescriptor {
        id: "canvas".into(),
        name: "Full Canvas".into(),
        icon: Some(String::from(Icon::Maximize2.unicode())),
        accent_color: tokens::GUIDE_LINE,
        layout: LayoutState::default(),
        tree: canvas_tree,
    });

    registry.register(WorkspaceDescriptor {
        id: "dataflow".into(),
        name: "Dataflow".into(),
        icon: Some(String::from(Icon::Activity.unicode())),
        accent_color: tokens::TEXT_SUCCESS,
        layout: LayoutState::default(),
        tree: debug_tree,
    });

    registry.active = Some("graph".into());
}

fn register_dock_windows(registry: &mut WindowRegistry, node_registry: &NodeRegistry) {
    use window_ids::*;

    registry.register(DockWindowDescriptor {
        id: NODE_EDITOR.into(),
        name: "Node Editor".into(),
        icon: Some(String::from(Icon::Workflow.unicode())),
        default_area: "center".into(),
        priority: 0,
        build: Arc::new(|parent| {
            parent.spawn((
                NodeEditorViewport,
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

    registry.register(DockWindowDescriptor {
        id: OUTLINER.into(),
        name: "Outliner".into(),
        icon: Some(String::from(Icon::ListTree.unicode())),
        default_area: "left_sidebar".into(),
        priority: 10,
        build: Arc::new(|parent| {
            parent
                .spawn((
                    jackdaw_widgets::tree_view::TreeRoot,
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(tokens::SPACING_SM)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                ))
                .with_children(|tree| {
                    tree.spawn((
                        jackdaw_widgets::tree_view::TreeRowContent,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(tokens::FIELD_ROW_HEIGHT),
                            padding: UiRect::horizontal(Val::Px(8.0)),
                            border_radius: BorderRadius::all(Val::Px(tokens::BORDER_RADIUS_SM)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        children![(
                            jackdaw_widgets::tree_view::TreeRowLabel,
                            Text::new("Root Graph"),
                            TextFont {
                                font_size: tokens::TEXT_SIZE_SM,
                                ..default()
                            },
                            TextColor(tokens::TEXT_SECONDARY),
                        )],
                    ));
                });
        }),
    });

    let palette_data = NodePalette::from_registry(node_registry);
    let library_items: Vec<(String, Vec<(String, String, usize, usize)>)> = palette_data
        .by_category()
        .into_iter()
        .map(|(cat, templates)| {
            let items = templates
                .into_iter()
                .map(|t| {
                    let in_count = t
                        .pins
                        .iter()
                        .filter(|p| p.direction == luma_core::graph::PinDirection::Input)
                        .count();
                    let out_count = t
                        .pins
                        .iter()
                        .filter(|p| p.direction == luma_core::graph::PinDirection::Output)
                        .count();
                    (t.kind.clone(), t.default_title.clone(), in_count, out_count)
                })
                .collect();
            (cat, items)
        })
        .collect();

    registry.register(DockWindowDescriptor {
        id: LIBRARY.into(),
        name: "Node Library".into(),
        icon: Some(String::from(Icon::Boxes.unicode())),
        default_area: "left_sidebar".into(),
        priority: 20,
        build: Arc::new(move |parent| {
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(tokens::SPACING_SM)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(tokens::SPACING_SM),
                    overflow: Overflow::scroll_y(),
                    ..default()
                })
                .with_children(|container| {
                    for (cat, items) in &library_items {
                        container.spawn((
                            Text::new(cat.to_uppercase()),
                            TextFont {
                                font_size: tokens::TEXT_SIZE_XS,
                                ..default()
                            },
                            TextColor(tokens::TEXT_SECONDARY),
                        ));

                        for (kind, title, in_count, out_count) in items {
                            container.spawn((
                                crate::bevy_ui_hud::SpawnPaletteButton {
                                    template: kind.clone(),
                                    category: cat.clone(),
                                },
                                crate::UiButton,
                                Node {
                                    width: Val::Percent(100.0),
                                    min_height: Val::Px(24.0),
                                    padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                    border_radius: BorderRadius::all(Val::Px(tokens::BORDER_RADIUS_SM)),
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(tokens::TOOLBAR_BUTTON_BG),
                                children![
                                    (
                                        Text::new(title.clone()),
                                        TextFont {
                                            font_size: tokens::TEXT_SIZE_SM,
                                            ..default()
                                        },
                                        TextColor(tokens::TEXT_PRIMARY),
                                    ),
                                    (
                                        Text::new(format!("{in_count} in / {out_count} out")),
                                        TextFont {
                                            font_size: tokens::TEXT_SIZE_XS,
                                            ..default()
                                        },
                                        TextColor(tokens::TEXT_SECONDARY),
                                    )
                                ],
                            ));
                        }
                    }
                });
        }),
    });

    registry.register(DockWindowDescriptor {
        id: INSPECTOR.into(),
        name: "Properties".into(),
        icon: Some(String::from(Icon::SlidersHorizontal.unicode())),
        default_area: "right_sidebar".into(),
        priority: 30,
        build: Arc::new(|parent| {
            // The live inspector is an overlay glued on top of this area (see
            // `DockAnchored`); this is what shows when nothing is selected.
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(tokens::SPACING_LG)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![(
                    Text::new("Select a node to inspect its properties"),
                    TextFont {
                        font_size: tokens::TEXT_SIZE_SM,
                        ..default()
                    },
                    TextColor(tokens::TEXT_SECONDARY),
                )],
            ));
        }),
    });
}

/// Attaches [`DockAnchored`] to the pre-existing floating overlays so they live
/// inside their dock area instead of being pinned to the window.
pub fn attach_overlay_dock_anchors(
    mut commands: Commands,
    inspector: Query<Entity, With<crate::bevy_ui_inspector::BevyUiInspectorRoot>>,
    toolbar: Query<Entity, With<crate::bevy_ui_hud::BevyUiFloatingToolbarRoot>>,
    minimap: Query<Entity, With<crate::bevy_ui_hud::BevyUiMinimapRoot>>,
    hotkeys: Query<Entity, With<crate::bevy_ui_hud::BevyUiHudRoot>>,
    toast: Query<Entity, With<crate::bevy_ui_hud::BevyUiToast>>,
    focus: Query<Entity, With<crate::bevy_ui_chrome::BevyUiFocusToolbarRoot>>,
) {
    use window_ids::*;
    let mut attach = |entity: Entity, window: &'static str, anchor: DockAnchor| {
        commands.entity(entity).insert(DockAnchored::new(window, anchor));
    };
    for e in &inspector {
        attach(e, INSPECTOR, DockAnchor::Fill);
    }
    for e in &toolbar {
        attach(e, NODE_EDITOR, DockAnchor::BottomCenter(12.0));
    }
    for e in &minimap {
        attach(e, NODE_EDITOR, DockAnchor::BottomRight(Vec2::splat(12.0)));
    }
    for e in &hotkeys {
        attach(e, NODE_EDITOR, DockAnchor::BottomLeft(Vec2::splat(12.0)));
    }
    for e in &toast {
        attach(e, NODE_EDITOR, DockAnchor::TopCenter(48.0));
    }
    for e in &focus {
        attach(e, NODE_EDITOR, DockAnchor::TopCenter(10.0));
    }
}

/// Logical-pixel rect of a UI node, as used by `Val::Px` positions.
fn logical_rect(computed: &ComputedNode, transform: &UiGlobalTransform) -> Rect {
    let inv = computed.inverse_scale_factor();
    let size = computed.size() * inv;
    let (_scale, _angle, center) = transform.to_scale_angle_translation();
    Rect::from_center_size(center * inv, size)
}

/// Rect of the visible content node of `window`, if it is shown somewhere.
fn visible_window_rect(
    window: &str,
    contents: &Query<(&DockTabContent, &Node, &ComputedNode, &UiGlobalTransform, &InheritedVisibility)>,
) -> Option<Rect> {
    contents
        .iter()
        .filter(|(content, node, computed, _, visibility)| {
            content.window_id == window
                && node.display != Display::None
                && visibility.get()
                && computed.size().min_element() > 1.0
        })
        .map(|(_, _, computed, transform, _)| logical_rect(computed, transform))
        .next()
}

/// Keeps every container transparent, every regular area opaque, and the area
/// currently showing the node editor transparent so the canvas shows through.
fn sync_dock_backgrounds(
    containers: Query<
        Entity,
        Or<(With<jackdaw_panels::Panel>, With<DockTreeHost>, With<jackdaw_panels::PanelGroup>)>,
    >,
    areas: Query<(Entity, &ActiveDockWindow, Option<&Children>), With<DockArea>>,
    contents: Query<&DockTabContent>,
    handles: Query<Entity, With<PanelHandle>>,
    mut backgrounds: Query<&mut BackgroundColor>,
) {
    for entity in &containers {
        if let Ok(mut bg) = backgrounds.get_mut(entity) {
            bg.set_if_neq(BackgroundColor(Color::NONE));
        }
    }

    for (entity, active, children) in &areas {
        let shows_canvas = active.0.is_some_and(|active_tab| {
            children.into_iter().flatten().any(|child| {
                contents.get(*child).is_ok_and(|c| {
                    c.tab_id == active_tab && c.window_id == window_ids::NODE_EDITOR
                })
            })
        });
        let color = if shows_canvas { Color::NONE } else { tokens::PANEL_BG };
        match backgrounds.get_mut(entity) {
            Ok(mut bg) => {
                bg.set_if_neq(BackgroundColor(color));
            }
            Err(_) => {}
        }
    }

    // Idle handles are transparent in jackdaw; paint them as dark gaps so the
    // canvas doesn't bleed between areas. Hover colour is left untouched.
    for entity in &handles {
        if let Ok(mut bg) = backgrounds.get_mut(entity)
            && bg.0 == Color::NONE
        {
            bg.0 = AREA_GAP_COLOR;
        }
    }
}

fn update_node_editor_viewport_rect(
    contents: Query<(&DockTabContent, &Node, &ComputedNode, &UiGlobalTransform, &InheritedVisibility)>,
    mut rect: ResMut<NodeEditorViewportRect>,
) {
    let new = visible_window_rect(window_ids::NODE_EDITOR, &contents);
    if rect.0 != new {
        rect.0 = new;
    }
}

fn follow_dock_anchors(
    contents: Query<(&DockTabContent, &Node, &ComputedNode, &UiGlobalTransform, &InheritedVisibility)>,
    mut overlays: Query<
        (&DockAnchored, &mut Node, &mut Visibility, &ComputedNode),
        Without<DockTabContent>,
    >,
) {
    for (anchored, mut node, mut visibility, computed) in &mut overlays {
        let Some(area) = visible_window_rect(anchored.window, &contents) else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        visibility.set_if_neq(Visibility::Inherited);

        let own = computed.size() * computed.inverse_scale_factor();
        let (left, top) = match anchored.anchor {
            DockAnchor::Fill => (area.min.x, area.min.y),
            DockAnchor::TopCenter(offset) => (area.center().x - own.x * 0.5, area.min.y + offset),
            DockAnchor::BottomCenter(offset) => {
                (area.center().x - own.x * 0.5, area.max.y - own.y - offset)
            }
            DockAnchor::BottomLeft(offset) => (area.min.x + offset.x, area.max.y - own.y - offset.y),
            DockAnchor::BottomRight(offset) => {
                (area.max.x - own.x - offset.x, area.max.y - own.y - offset.y)
            }
        };

        let mut target = node.clone();
        target.position_type = PositionType::Absolute;
        target.left = Val::Px(left.round());
        target.top = Val::Px(top.round());
        target.right = Val::Auto;
        target.bottom = Val::Auto;
        target.margin = UiRect::ZERO;
        if anchored.anchor == DockAnchor::Fill {
            target.width = Val::Px(area.width().round());
            target.height = Val::Px(area.height().round());
            target.max_height = Val::Auto;
            target.overflow = Overflow::scroll_y();
            target.border_radius = BorderRadius::ZERO;
        }
        if *node != target {
            *node = target;
        }
    }
}

fn set_close_btn_hover<E: EntityEvent>(
    color: Color,
) -> impl Fn(On<E>, Commands, Query<(), With<jackdaw_panels::area::DockTabCloseButton>>) {
    move |event, mut commands, filter| {
        if filter.contains(event.event_target()) {
            commands
                .entity(event.event_target())
                .insert(BackgroundColor(color));
        }
    }
}

/// Intercepts PointerPress on DockTabCloseButton so it does not bubble up to DockTab
/// (which would activate the tab instead of closing it).
fn on_dock_tab_close_press(
    mut press: On<PointerPress>,
    close_buttons: Query<&jackdaw_panels::area::DockTabCloseButton>,
    parent_q: Query<&ChildOf>,
) {
    let target = press.event_target();
    if close_buttons.contains(target)
        || parent_q.iter_ancestors(target).any(|p| close_buttons.contains(p))
    {
        press.propagate(false);
    }
}

/// Closes a dock tab when its [X] close button is clicked.
fn on_dock_tab_close_click(
    mut click: On<PointerClick>,
    close_buttons: Query<&jackdaw_panels::area::DockTabCloseButton>,
    parent_q: Query<&ChildOf>,
    mut tree: ResMut<DockTree>,
) {
    if click.event().button != PointerButton::Primary {
        return;
    }
    let target = click.event_target();
    let tab_id = if let Ok(btn) = close_buttons.get(target) {
        Some(btn.tab_id)
    } else {
        parent_q
            .iter_ancestors(target)
            .find_map(|p| close_buttons.get(p).ok().map(|b| b.tab_id))
    };

    if let Some(tab_id) = tab_id {
        // Prevent closing the primary node editor canvas if it's the only one
        let is_node_editor = tree
            .tabs()
            .any(|(_, t)| t.id == tab_id && t.window_id == window_ids::NODE_EDITOR);
        let node_editor_count = tree
            .tabs()
            .filter(|(_, t)| t.window_id == window_ids::NODE_EDITOR)
            .count();
        if is_node_editor && node_editor_count <= 1 {
            return;
        }

        click.propagate(false);
        tree.remove_tab(tab_id);
    }
}

/// Closes a dock tab when clicked with Middle mouse button or Right mouse button (Secondary).
fn on_dock_tab_auxiliary_click(
    mut click: On<PointerClick>,
    tabs: Query<&jackdaw_panels::area::DockTab>,
    parent_q: Query<&ChildOf>,
    mut tree: ResMut<DockTree>,
) {
    let button = click.event().button;
    if button != PointerButton::Middle && button != PointerButton::Secondary {
        return;
    }
    let target = click.event_target();
    let tab = if let Ok(t) = tabs.get(target) {
        Some(t)
    } else {
        parent_q
            .iter_ancestors(target)
            .find_map(|p| tabs.get(p).ok())
    };

    if let Some(tab) = tab {
        let is_node_editor = tree
            .tabs()
            .any(|(_, t)| t.id == tab.tab_id && t.window_id == window_ids::NODE_EDITOR);
        let node_editor_count = tree
            .tabs()
            .filter(|(_, t)| t.window_id == window_ids::NODE_EDITOR)
            .count();
        if is_node_editor && node_editor_count <= 1 {
            return;
        }

        click.propagate(false);
        tree.remove_tab(tab.tab_id);
    }
}

/// Closes the entire area / panel when its right-hand grip handle is right-clicked.
fn on_dock_grip_right_click(
    mut click: On<PointerClick>,
    grips: Query<&ChildOf, With<jackdaw_panels::tabs::DockTabGrip>>,
    parent_q: Query<&ChildOf>,
    bindings: Query<&jackdaw_panels::reconcile::LeafBinding>,
    mut tree: ResMut<DockTree>,
) {
    if click.event().button != PointerButton::Secondary {
        return;
    }
    let target = click.event_target();
    let is_grip = grips.contains(target)
        || parent_q.iter_ancestors(target).any(|p| grips.contains(p));
    if !is_grip {
        return;
    }

    if let Some(area) = parent_q.iter_ancestors(target).find(|p| bindings.contains(*p)) {
        if let Ok(binding) = bindings.get(area) {
            click.propagate(false);
            let leaf_tabs: Vec<_> = if let Some(leaf) = tree.get(binding.0).and_then(|n| n.as_leaf()) {
                leaf.windows.iter().map(|t| t.id).collect()
            } else {
                Vec::new()
            };

            for tab_id in leaf_tabs {
                let is_node_editor = tree
                    .tabs()
                    .any(|(_, t)| t.id == tab_id && t.window_id == window_ids::NODE_EDITOR);
                let node_editor_count = tree
                    .tabs()
                    .filter(|(_, t)| t.window_id == window_ids::NODE_EDITOR)
                    .count();
                if is_node_editor && node_editor_count <= 1 {
                    continue;
                }
                tree.remove_tab(tab_id);
            }
        }
    }
}
