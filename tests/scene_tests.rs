use bevy::math::{Quat, Vec2, Vec3};
use luma_editor::canvas::CanvasTransform;
use luma_editor::core::*;
use luma_editor::domain::*;
use luma_editor::scene::*;

#[test]
fn test_luma_graph_file_serialization_roundtrip() {
    let ids = IdGenerator::new();
    let mut graph = GraphStore::new();
    let mut payloads = DomainPayloadStore::new();

    let node_id = ids.next_node_id();
    let node = Node::new(node_id, "Serialized Node", "MeshNode", Vec2::new(100.0, 200.0));
    graph.add_node(node);

    payloads.meshes.insert(node_id, MeshData {
        mesh_source: "models/tree.glb".into(),
        dimensions: Vec3::new(2.0, 5.0, 2.0),
        wireframe_enabled: true,
    });

    let canvas = CanvasTransform::new(Vec2::new(50.0, -30.0), 1.25);
    let file = LumaGraphFile::new(canvas, graph, &payloads);

    // Serialize to RON string
    let ron_str = ron::ser::to_string(&file).expect("Should serialize LumaGraphFile");
    let loaded: LumaGraphFile = ron::from_str(&ron_str).expect("Should deserialize LumaGraphFile");

    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.canvas.pan, Vec2::new(50.0, -30.0));
    assert_eq!(loaded.canvas.zoom, 1.25);
    assert_eq!(loaded.graph.nodes().len(), 1);
    assert!(loaded.graph.get_node(node_id).is_some());

    let restored_payloads = loaded.to_domain_store();
    assert_eq!(
        restored_payloads.meshes.get(&node_id).unwrap().mesh_source,
        "models/tree.glb"
    );
}

#[test]
fn test_bsn_scene_compilation() {
    let ids = IdGenerator::new();
    let mut graph = GraphStore::new();
    let mut payloads = DomainPayloadStore::new();

    // Setup Mesh node
    let mesh_node = ids.next_node_id();
    let mesh_out = ids.next_pin_id();
    graph.add_node(Node::new(mesh_node, "Mesh", "MeshNode", Vec2::ZERO));
    graph.add_pin(Pin {
        id: mesh_out,
        node_id: mesh_node,
        name: "Out".into(),
        direction: PinDirection::Output,
        data_type: DataType::new("Mesh"),
        capacity: PinCapacity::Multiple,
    }).unwrap();
    payloads.meshes.insert(mesh_node, MeshData {
        mesh_source: "models/player.glb".into(),
        dimensions: Vec3::ONE,
        wireframe_enabled: false,
    });

    // Setup Entity Assembler node
    let entity_node = ids.next_node_id();
    let entity_in = ids.next_pin_id();
    graph.add_node(Node::new(entity_node, "Player", "EntityAssembler", Vec2::ZERO));
    graph.add_pin(Pin {
        id: entity_in,
        node_id: entity_node,
        name: "Mesh".into(),
        direction: PinDirection::Input,
        data_type: DataType::new("Mesh"),
        capacity: PinCapacity::Single,
    }).unwrap();
    payloads.entities.insert(entity_node, EntityAssemblerData {
        name: "MainHero".into(),
        translation: Vec3::new(10.0, 0.0, -5.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::splat(1.5),
    });

    graph.connect(ids.next_connection_id(), mesh_out, entity_in).unwrap();

    // Compile to BSN string
    let bsn_text = BsnCompiler::compile_scene(&graph, &payloads).expect("Should compile scene");

    assert!(bsn_text.contains("\"bevy_core::name::Name\": \"MainHero\""));
    assert!(bsn_text.contains("\"bevy_render::mesh::Mesh3d\": \"models/player.glb\""));
    assert!(bsn_text.contains("translation: (10.000, 0.000, -5.000)"));
    assert!(bsn_text.contains("scale: (1.500, 1.500, 1.500)"));
}

#[test]
fn test_auto_bake_pipeline_dual_format() {
    let ids = IdGenerator::new();
    let mut graph = GraphStore::new();
    let mut payloads = DomainPayloadStore::new();

    let entity_node = ids.next_node_id();
    graph.add_node(Node::new(entity_node, "Cube Entity", "EntityAssembler", Vec2::ZERO));
    payloads.entities.insert(entity_node, EntityAssemblerData {
        name: "TestCube".into(),
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });

    let canvas = CanvasTransform::default();
    let file = LumaGraphFile::new(canvas, graph, &payloads);

    let luma_path = "target/test_scene.luma";
    let bsn_path = "target/test_scene.bsn";

    let result = AutoBakePipeline::save_and_bake(luma_path, bsn_path, &file)
        .expect("Should save and bake");

    assert_eq!(result.entities_baked_count, 1);
    assert!(std::path::Path::new(luma_path).exists());
    assert!(std::path::Path::new(bsn_path).exists());

    // Verify .luma loads back
    let reloaded = LumaGraphFile::load_from_file(luma_path).expect("Should reload .luma");
    assert_eq!(reloaded.graph.nodes().len(), 1);

    // Verify .bsn has content
    let bsn_content = std::fs::read_to_string(bsn_path).expect("Should read .bsn");
    assert!(bsn_content.contains("\"TestCube\""));

    // Cleanup
    let _ = std::fs::remove_file(luma_path);
    let _ = std::fs::remove_file(bsn_path);
}
