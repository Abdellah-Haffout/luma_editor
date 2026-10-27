use bevy::math::Vec2;
use luma_editor::core::*;

#[test]
fn test_node_and_pin_creation() {
    let ids = IdGenerator::new();
    let mut graph = GraphStore::new();

    let node_id = ids.next_node_id();
    let node = Node::new(node_id, "Test Node", "TestKind", Vec2::new(10.0, 20.0));
    graph.add_node(node);

    let pin_out_id = ids.next_pin_id();
    let pin_out = Pin {
        id: pin_out_id,
        node_id,
        name: "Out".into(),
        direction: PinDirection::Output,
        data_type: DataType::new("Mesh"),
        capacity: PinCapacity::Multiple,
    };
    graph.add_pin(pin_out).expect("Should add pin");

    assert!(graph.get_node(node_id).is_some());
    assert_eq!(graph.get_node(node_id).unwrap().outputs.len(), 1);
    assert_eq!(graph.get_node(node_id).unwrap().outputs[0], pin_out_id);
}

#[test]
fn test_wiring_validation() {
    let ids = IdGenerator::new();
    let mut graph = GraphStore::new();

    let n1 = ids.next_node_id();
    let n2 = ids.next_node_id();
    graph.add_node(Node::new(n1, "N1", "Kind", Vec2::ZERO));
    graph.add_node(Node::new(n2, "N2", "Kind", Vec2::ZERO));

    let p1_out = ids.next_pin_id();
    let p1_in = ids.next_pin_id();
    let p2_in_mesh = ids.next_pin_id();
    let p2_in_material = ids.next_pin_id();

    graph.add_pin(Pin {
        id: p1_out,
        node_id: n1,
        name: "OutMesh".into(),
        direction: PinDirection::Output,
        data_type: DataType::new("Mesh"),
        capacity: PinCapacity::Multiple,
    }).unwrap();

    graph.add_pin(Pin {
        id: p1_in,
        node_id: n1,
        name: "InMesh".into(),
        direction: PinDirection::Input,
        data_type: DataType::new("Mesh"),
        capacity: PinCapacity::Single,
    }).unwrap();

    graph.add_pin(Pin {
        id: p2_in_mesh,
        node_id: n2,
        name: "InMesh".into(),
        direction: PinDirection::Input,
        data_type: DataType::new("Mesh"),
        capacity: PinCapacity::Single,
    }).unwrap();

    graph.add_pin(Pin {
        id: p2_in_material,
        node_id: n2,
        name: "InMat".into(),
        direction: PinDirection::Input,
        data_type: DataType::new("Material"),
        capacity: PinCapacity::Single,
    }).unwrap();

    // 1. Self connection must fail
    assert_eq!(
        graph.connect(ids.next_connection_id(), p1_out, p1_in),
        Err(GraphError::SelfConnection(n1))
    );

    // 2. Incompatible types must fail
    assert!(matches!(
        graph.connect(ids.next_connection_id(), p1_out, p2_in_material),
        Err(GraphError::IncompatibleTypes { .. })
    ));

    // 3. Valid connection succeeds
    let conn_id = ids.next_connection_id();
    let conn = graph.connect(conn_id, p1_out, p2_in_mesh);
    assert!(conn.is_ok());

    // 4. Capacity single exceeded
    let n3 = ids.next_node_id();
    graph.add_node(Node::new(n3, "N3", "Kind", Vec2::ZERO));
    let p3_out = ids.next_pin_id();
    graph.add_pin(Pin {
        id: p3_out,
        node_id: n3,
        name: "OutMesh".into(),
        direction: PinDirection::Output,
        data_type: DataType::new("Mesh"),
        capacity: PinCapacity::Multiple,
    }).unwrap();

    assert_eq!(
        graph.connect(ids.next_connection_id(), p3_out, p2_in_mesh),
        Err(GraphError::CapacityExceeded(p2_in_mesh))
    );
}

#[test]
fn test_dag_cycle_detection() {
    let ids = IdGenerator::new();
    let mut graph = GraphStore::new();

    let a = ids.next_node_id();
    let b = ids.next_node_id();
    let c = ids.next_node_id();

    graph.add_node(Node::new(a, "A", "Kind", Vec2::ZERO));
    graph.add_node(Node::new(b, "B", "Kind", Vec2::ZERO));
    graph.add_node(Node::new(c, "C", "Kind", Vec2::ZERO));

    let a_out = ids.next_pin_id();
    let a_in = ids.next_pin_id();
    let b_out = ids.next_pin_id();
    let b_in = ids.next_pin_id();
    let c_out = ids.next_pin_id();
    let c_in = ids.next_pin_id();

    graph.add_pin(Pin { id: a_out, node_id: a, name: "out".into(), direction: PinDirection::Output, data_type: DataType::any(), capacity: PinCapacity::Multiple }).unwrap();
    graph.add_pin(Pin { id: a_in, node_id: a, name: "in".into(), direction: PinDirection::Input, data_type: DataType::any(), capacity: PinCapacity::Multiple }).unwrap();

    graph.add_pin(Pin { id: b_out, node_id: b, name: "out".into(), direction: PinDirection::Output, data_type: DataType::any(), capacity: PinCapacity::Multiple }).unwrap();
    graph.add_pin(Pin { id: b_in, node_id: b, name: "in".into(), direction: PinDirection::Input, data_type: DataType::any(), capacity: PinCapacity::Multiple }).unwrap();

    graph.add_pin(Pin { id: c_out, node_id: c, name: "out".into(), direction: PinDirection::Output, data_type: DataType::any(), capacity: PinCapacity::Multiple }).unwrap();
    graph.add_pin(Pin { id: c_in, node_id: c, name: "in".into(), direction: PinDirection::Input, data_type: DataType::any(), capacity: PinCapacity::Multiple }).unwrap();

    // Connect A -> B
    graph.connect(ids.next_connection_id(), a_out, b_in).unwrap();
    // Connect B -> C
    graph.connect(ids.next_connection_id(), b_out, c_in).unwrap();

    // Now connecting C -> A must be blocked because it forms a cycle (A -> B -> C -> A)
    assert_eq!(
        graph.connect(ids.next_connection_id(), c_out, a_in),
        Err(GraphError::CycleDetected { from: c, to: a })
    );
}

#[test]
fn test_history_undo_redo_and_transactions() {
    let ids = IdGenerator::new();
    let mut graph = GraphStore::new();
    let mut history = HistoryStack::default();

    let node_id = ids.next_node_id();
    let node = Node::new(node_id, "Undoable Node", "Kind", Vec2::new(50.0, 50.0));

    // 1. Add node through history
    history.apply(Box::new(AddNodeCommand::new(node)), &mut graph).unwrap();
    assert!(graph.get_node(node_id).is_some());

    // 2. Undo Add
    assert!(history.can_undo());
    history.undo(&mut graph).unwrap();
    assert!(graph.get_node(node_id).is_none());

    // 3. Redo Add
    assert!(history.can_redo());
    history.redo(&mut graph).unwrap();
    assert!(graph.get_node(node_id).is_some());

    // 4. Move node through history
    let old_pos = Vec2::new(50.0, 50.0);
    let new_pos = Vec2::new(150.0, 250.0);
    history.apply(Box::new(MoveNodesCommand::new(vec![(node_id, old_pos, new_pos)])), &mut graph).unwrap();
    assert_eq!(graph.get_node(node_id).unwrap().position, new_pos);

    history.undo(&mut graph).unwrap();
    assert_eq!(graph.get_node(node_id).unwrap().position, old_pos);

    history.redo(&mut graph).unwrap();
    assert_eq!(graph.get_node(node_id).unwrap().position, new_pos);

    // 5. Compound transaction: Delete node with connection and undo in one step
    let n2 = ids.next_node_id();
    let p1_out = ids.next_pin_id();
    let p2_in = ids.next_pin_id();
    let conn_id = ids.next_connection_id();

    graph.add_node(Node::new(n2, "N2", "Kind", Vec2::ZERO));
    graph.add_pin(Pin { id: p1_out, node_id, name: "out".into(), direction: PinDirection::Output, data_type: DataType::any(), capacity: PinCapacity::Multiple }).unwrap();
    graph.add_pin(Pin { id: p2_in, node_id: n2, name: "in".into(), direction: PinDirection::Input, data_type: DataType::any(), capacity: PinCapacity::Multiple }).unwrap();
    graph.connect(conn_id, p1_out, p2_in).unwrap();

    assert_eq!(graph.connections().len(), 1);

    // Begin compound delete transaction
    history.begin_transaction("Delete Node with Wires");
    history.apply(Box::new(DisconnectCommand::new(conn_id)), &mut graph).unwrap();
    history.apply(Box::new(RemoveNodeCommand::new(node_id)), &mut graph).unwrap();
    history.commit_transaction();

    assert!(graph.get_node(node_id).is_none());
    assert_eq!(graph.connections().len(), 0);

    // Undo should restore BOTH the node and the wire!
    history.undo(&mut graph).unwrap();
    assert!(graph.get_node(node_id).is_some());
    assert_eq!(graph.connections().len(), 1);
    assert_eq!(graph.get_connection(conn_id).unwrap().source_pin, p1_out);
}
