use bevy::math::Vec2;
use luma_editor::core::*;
use luma_editor::nodes::*;

#[test]
fn test_node_card_layout_and_pin_positions() {
    let ids = IdGenerator::new();
    let mut graph = GraphStore::new();

    let node_id = ids.next_node_id();
    let mut node = Node::new(node_id, "Test Node", "MeshNode", Vec2::new(100.0, 100.0));
    node.size = Vec2::new(200.0, 150.0);
    graph.add_node(node);

    let p_in = ids.next_pin_id();
    let p_out = ids.next_pin_id();

    graph.add_pin(Pin {
        id: p_in,
        node_id,
        name: "Input".into(),
        direction: PinDirection::Input,
        data_type: DataType::new("Mesh"),
        capacity: PinCapacity::Single,
    }).unwrap();

    graph.add_pin(Pin {
        id: p_out,
        node_id,
        name: "Output".into(),
        direction: PinDirection::Output,
        data_type: DataType::new("Mesh"),
        capacity: PinCapacity::Multiple,
    }).unwrap();

    let layout = NodeCardLayout::from_graph(node_id, &graph).expect("Should build layout");
    assert_eq!(layout.title, "Test Node");
    assert_eq!(layout.input_pins.len(), 1);
    assert_eq!(layout.output_pins.len(), 1);

    // Pin positions
    let in_pos = layout.pin_local_position(p_in).unwrap();
    assert_eq!(in_pos.x, 0.0); // left edge

    let out_pos = layout.pin_local_position(p_out).unwrap();
    assert_eq!(out_pos.x, 200.0); // right edge
}

#[test]
fn test_node_drag_state_and_debounced_history() {
    let ids = IdGenerator::new();
    let mut graph = GraphStore::new();
    let mut history = HistoryStack::default();

    let n1 = ids.next_node_id();
    let n2 = ids.next_node_id();

    graph.add_node(Node::new(n1, "N1", "Kind", Vec2::new(10.0, 10.0)));
    graph.add_node(Node::new(n2, "N2", "Kind", Vec2::new(50.0, 50.0)));

    let mut drag = NodeDragState::new();
    assert!(!drag.is_dragging());

    // Begin drag at cursor (100, 100)
    drag.begin_drag(&[n1, n2], Vec2::new(100.0, 100.0), &graph);
    assert!(drag.is_dragging());

    // Update drag to cursor (150, 120) -> delta = (50, 20)
    drag.update_drag(Vec2::new(150.0, 120.0), &mut graph);
    assert_eq!(graph.get_node(n1).unwrap().position, Vec2::new(60.0, 30.0));
    assert_eq!(graph.get_node(n2).unwrap().position, Vec2::new(100.0, 70.0));

    // Finish drag: must record single MoveNodesCommand
    let recorded = drag.finish_drag(&mut graph, &mut history).unwrap();
    assert!(recorded);
    assert!(!drag.is_dragging());

    // Undo should restore both to start positions
    assert!(history.can_undo());
    history.undo(&mut graph).unwrap();
    assert_eq!(graph.get_node(n1).unwrap().position, Vec2::new(10.0, 10.0));
    assert_eq!(graph.get_node(n2).unwrap().position, Vec2::new(50.0, 50.0));

    // Redo should move them back
    assert!(history.can_redo());
    history.redo(&mut graph).unwrap();
    assert_eq!(graph.get_node(n1).unwrap().position, Vec2::new(60.0, 30.0));
    assert_eq!(graph.get_node(n2).unwrap().position, Vec2::new(100.0, 70.0));
}

#[test]
fn test_node_palette_spawning_and_undo() {
    let ids = IdGenerator::new();
    let mut graph = GraphStore::new();
    let mut history = HistoryStack::default();
    let palette = NodePalette::with_default_templates();

    assert!(palette.get("MeshNode").is_some());
    assert!(palette.get("MaterialNode").is_some());
    assert!(palette.get("EntityAssembler").is_some());

    // Spawn MeshNode at position (120, 80)
    let node_id = palette
        .spawn_node("MeshNode", Vec2::new(120.0, 80.0), &mut graph, &ids, &mut history)
        .expect("Should spawn node from template");

    let node = graph.get_node(node_id).expect("Node should exist in graph");
    assert_eq!(node.title, "3D Mesh");
    assert_eq!(node.inputs.len(), 1);
    assert_eq!(node.outputs.len(), 1);

    // Undo spawn transaction: should remove node cleanly
    assert!(history.can_undo());
    history.undo(&mut graph).unwrap();
    assert!(graph.get_node(node_id).is_none());

    // Redo spawn
    assert!(history.can_redo());
    history.redo(&mut graph).unwrap();
    assert!(graph.get_node(node_id).is_some());
}
