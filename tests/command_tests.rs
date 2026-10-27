use bevy::math::Vec2;
use luma_editor::commands::*;
use luma_editor::core::*;

#[test]
fn test_command_registry_and_search() {
    let mut registry = CommandRegistry::new();

    registry.register(
        CommandDescriptor::new(
            "graph.spawn_node",
            "Spawn Node",
            CommandCategory::Graph,
            "Spawn a new node on the canvas",
            true,
        )
        .with_shortcut("Space"),
    );

    registry.register(
        CommandDescriptor::new(
            "edit.undo",
            "Undo Last Action",
            CommandCategory::Edit,
            "Revert the previous change",
            false,
        )
        .with_shortcut("Ctrl+Z"),
    );

    registry.register(
        CommandDescriptor::new(
            "view.zoom_fit",
            "Zoom to Fit",
            CommandCategory::View,
            "Frame all nodes within the canvas viewport",
            false,
        )
        .with_shortcut("F"),
    );

    // "spawn" matches only spawn_node
    assert_eq!(registry.search("spawn").len(), 1);
    assert_eq!(registry.search("spawn")[0].id.0, "graph.spawn_node");

    // "node" matches both spawn_node and zoom_fit (found in description)
    assert_eq!(registry.search("node").len(), 2);

    assert_eq!(registry.search("undo").len(), 1);
    assert_eq!(registry.search("undo")[0].id.0, "edit.undo");

    // Search across categories or descriptions
    assert_eq!(registry.search("Canvas").len(), 2);
}

#[test]
fn test_command_dispatcher_and_history_integration() {
    let mut dispatcher = CommandDispatcher::new(CommandRegistry::new());

    // Register a command that adds a node through history
    dispatcher.register_handler("graph.add_sample_node", |ctx| {
        let node_id = NodeId(999);
        let node = Node::new(node_id, "Sample", "MeshNode", Vec2::new(100.0, 100.0));
        ctx.history
            .apply(Box::new(AddNodeCommand::new(node)), ctx.graph)
            .map_err(|e| e.to_string())
    });

    let mut graph = GraphStore::new();
    let mut history = HistoryStack::default();
    let mut payloads = luma_editor::domain::evaluation::DomainPayloadStore::new();
    let mut dataflow = luma_editor::dataflow::engine::DataflowEngine::default();
    let mut ctx = CommandContext {
        graph: &mut graph,
        history: &mut history,
        payloads: &mut payloads,
        dataflow: &mut dataflow,
    };

    let cmd_id = CommandId::from("graph.add_sample_node");

    // Dispatch command
    let res = dispatcher.dispatch(&cmd_id, &mut ctx);
    assert!(res.is_ok());

    // Verify node exists
    assert!(ctx.graph.get_node(NodeId(999)).is_some());

    // Verify undo works
    assert!(ctx.history.can_undo());
    ctx.history.undo(ctx.graph).unwrap();
    assert!(ctx.graph.get_node(NodeId(999)).is_none());

    // Verify redo works
    assert!(ctx.history.can_redo());
    ctx.history.redo(ctx.graph).unwrap();
    assert!(ctx.graph.get_node(NodeId(999)).is_some());
}

#[test]
fn test_keybinding_map_and_serialization() {
    let mut map = KeybindingMap::with_defaults();

    let undo_chord = KeyChord::new(EditorKey::KeyZ).ctrl();
    assert_eq!(
        map.get_command(&undo_chord),
        Some(&CommandId::from("edit.undo"))
    );

    // Custom bind
    let custom_chord = KeyChord::new(EditorKey::KeyG).shift();
    map.bind(custom_chord, CommandId::from("custom.grab"));
    assert_eq!(
        map.get_command(&custom_chord),
        Some(&CommandId::from("custom.grab"))
    );

    // Test serialization to RON string
    let ron_string = ron::ser::to_string(&map).expect("Should serialize KeybindingMap");
    let deserialized: KeybindingMap = ron::from_str(&ron_string).expect("Should deserialize KeybindingMap");

    assert_eq!(
        deserialized.get_command(&undo_chord),
        Some(&CommandId::from("edit.undo"))
    );
}

#[test]
fn test_crash_journal_recording() {
    let mut journal = CrashJournal::new_in_memory();
    assert_eq!(journal.entries().len(), 0);

    journal.record(&CommandId::from("node.spawn"));
    journal.record(&CommandId::from("wire.connect"));

    assert_eq!(journal.entries().len(), 2);
    assert_eq!(journal.entries()[0].command_id.0, "node.spawn");
    assert_eq!(journal.entries()[1].command_id.0, "wire.connect");
    assert!(journal.entries()[0].timestamp_ms > 0);
}
