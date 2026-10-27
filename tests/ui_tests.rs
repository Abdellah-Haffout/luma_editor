use bevy::prelude::*;
use luma_editor::focus::tools::ContextualTool;
use luma_editor::nodes::card::NodeCardLayout;
use luma_editor::nodes::palette::NodePalette;
use luma_editor::ui::components::focus_toolbar::focus_toolbar_dock_scene;
use luma_editor::ui::components::header::top_navbar_scene;
use luma_editor::ui::components::node_card::node_card_scene;
use luma_editor::ui::components::status_bar::status_bar_scene;
use luma_editor::ui::theme::EditorTheme;
use luma_editor::ui::{
    EditorContextualToolsResource, EditorDragResource, EditorNavigationResource, LumaUiPlugin,
};

#[test]
fn test_ui_theme_constants() {
    assert_ne!(EditorTheme::BG_CANVAS, EditorTheme::BG_HEADER);
    assert_ne!(EditorTheme::ACCENT_PRIMARY, EditorTheme::ACCENT_DANGER);
    assert_ne!(EditorTheme::PIN_MESH, EditorTheme::PIN_MATERIAL);
    assert_ne!(EditorTheme::PIN_MATERIAL, EditorTheme::PIN_ENTITY);
}

#[test]
fn test_ui_scenes_construction() {
    let dummy_font = Handle::<Font>::default();

    // 1. Top Navbar Scene
    let breadcrumbs = vec!["Graph".to_string(), "MeshNode".to_string()];
    let _navbar = top_navbar_scene(dummy_font.clone(), breadcrumbs, true);

    // 2. Status Bar Scene
    let transform = luma_editor::canvas::CanvasTransform::default();
    let selection = luma_editor::canvas::SelectionState::default();
    let _status_bar = status_bar_scene(dummy_font.clone(), &transform, &selection, "scenes/test.bsn");

    // 3. Focus Toolbar Scene
    let tools = vec![
        ContextualTool::new("geom.gizmo", "Gizmo", "Geometry", "tool"),
    ];
    let _focus_toolbar = focus_toolbar_dock_scene(dummy_font.clone(), "MeshNode", "Mesh", &tools, true);
}

#[test]
fn test_node_card_scene_generation() {
    let mut graph = luma_editor::core::GraphStore::new();
    let id_gen = luma_editor::core::IdGenerator::default();
    let mut history = luma_editor::core::HistoryStack::default();
    let palette = NodePalette::with_default_templates();

    let node_id = palette
        .spawn_node(
            "MeshNode",
            Vec2::new(100.0, 150.0),
            &mut graph,
            &id_gen,
            &mut history,
        )
        .expect("Should spawn node");

    let layout = NodeCardLayout::from_graph(node_id, &graph).expect("Should compute card layout");
    let dummy_font = Handle::<Font>::default();

    let _card_unselected = node_card_scene(dummy_font.clone(), &layout, false);
    let _card_selected = node_card_scene(dummy_font, &layout, true);
}

#[test]
fn test_luma_ui_plugin_resources_and_state() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.add_plugins(bevy::text::TextPlugin);
    app.add_plugins(luma_editor::canvas::LumaEditorPlugin);
    app.add_plugins(LumaUiPlugin);

    assert!(app.world().contains_resource::<EditorNavigationResource>());
    assert!(app.world().contains_resource::<EditorContextualToolsResource>());
    assert!(app.world().contains_resource::<EditorDragResource>());

    let nav_res = app.world().resource::<EditorNavigationResource>();
    assert!(!nav_res.0.is_in_focus());
    assert_eq!(nav_res.0.breadcrumbs(), vec!["Root Graph"]);

    let drag_res = app.world().resource::<EditorDragResource>();
    assert!(!drag_res.0.is_dragging());
}
