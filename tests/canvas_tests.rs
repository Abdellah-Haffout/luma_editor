use bevy::app::App;
use bevy::math::Vec2;
use luma_editor::canvas::*;
use luma_editor::core::id::{ConnectionId, NodeId};

#[test]
fn test_canvas_transform_projection_and_zoom() {
    let mut transform = CanvasTransform::new(Vec2::new(100.0, 50.0), 2.0);

    // Screen to canvas
    let screen_pt = Vec2::new(200.0, 150.0);
    let canvas_pt = transform.screen_to_canvas(screen_pt);
    assert_eq!(canvas_pt, Vec2::new(50.0, 50.0));

    // Canvas to screen roundtrip
    assert_eq!(transform.canvas_to_screen(canvas_pt), screen_pt);

    // Zoom at pivot: the point under the pivot must remain unchanged in Canvas space!
    let pivot = Vec2::new(300.0, 300.0);
    let canvas_pt_before = transform.screen_to_canvas(pivot);
    transform.zoom_at(pivot, 1.5);
    let canvas_pt_after = transform.screen_to_canvas(pivot);

    assert!((canvas_pt_before.x - canvas_pt_after.x).abs() < 1e-4);
    assert!((canvas_pt_before.y - canvas_pt_after.y).abs() < 1e-4);
}

#[test]
fn test_selection_state() {
    let mut state = SelectionState::new();
    let n1 = NodeId(1);
    let n2 = NodeId(2);
    let c1 = ConnectionId(10);

    state.select_node(n1, false);
    assert!(state.is_node_selected(n1));
    assert_eq!(state.selected_nodes_count(), 1);

    // Additive select
    state.select_node(n2, true);
    assert!(state.is_node_selected(n1));
    assert!(state.is_node_selected(n2));
    assert_eq!(state.selected_nodes_count(), 2);

    // Non-additive selection clears previous
    state.select_connection(c1, false);
    assert!(!state.is_node_selected(n1));
    assert!(state.is_connection_selected(c1));

    state.clear();
    assert!(state.is_empty());
}

#[test]
fn test_wire_bezier_spline_math() {
    let start = Vec2::new(0.0, 0.0);
    let end = Vec2::new(200.0, 100.0);

    let (p0, p1, p2, p3) = calculate_wire_bezier(start, end);
    assert_eq!(p0, start);
    assert_eq!(p3, end);
    assert!(p1.x > p0.x);
    assert!(p2.x < p3.x);

    // Endpoint checks
    assert_eq!(sample_cubic_bezier(p0, p1, p2, p3, 0.0), start);
    assert_eq!(sample_cubic_bezier(p0, p1, p2, p3, 1.0), end);

    // Sample polyline
    let points = sample_wire_spline_points(start, end, 10);
    assert_eq!(points.len(), 11);
    assert_eq!(points[0], start);
    assert_eq!(points[10], end);
}

#[test]
fn test_luma_editor_plugin_resource_initialization() {
    let mut app = App::new();
    app.add_plugins(LumaEditorPlugin);

    // Check all editor resources are registered
    assert!(app.world().contains_resource::<EditorGraphResource>());
    assert!(app.world().contains_resource::<EditorHistoryResource>());
    assert!(app.world().contains_resource::<EditorIdGenResource>());
    assert!(app.world().contains_resource::<EditorTransformResource>());
    assert!(app.world().contains_resource::<EditorSelectionResource>());
    assert!(app.world().contains_resource::<EditorWiringResource>());
    assert!(app.world().contains_resource::<EditorKeybindingsResource>());
    assert!(app.world().contains_resource::<EditorDispatcherResource>());
}
