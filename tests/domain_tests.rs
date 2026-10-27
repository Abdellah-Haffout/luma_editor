use bevy::math::{Quat, Vec3, Vec4};
use luma_editor::core::*;
use luma_editor::domain::*;

#[test]
fn test_viewport_throttling_and_culling() {
    let mut manager = ViewportManager::new();
    let node_id = NodeId(42);

    manager.register(node_id, (256, 256));
    let target = manager.get_target_mut(node_id).unwrap();

    // Initial state: dirty and visible -> should render at t = 1.0
    assert!(target.should_render(1.0));
    target.mark_rendered(1.0);
    assert!(!target.is_dirty);

    // Frame immediately after (t = 1.01) -> should NOT render (not dirty, throttled)
    assert!(!target.should_render(1.01));

    // Marked dirty, but within throttled 15 FPS window (needs at least ~0.067s interval)
    target.is_dirty = true;
    assert!(!target.should_render(1.02)); // only 0.02s passed

    // After 0.08s passed -> interval satisfied!
    assert!(target.should_render(1.08));

    // Frustum culling: if hidden, never renders even if dirty and interval met
    target.is_visible = false;
    assert!(!target.should_render(2.0));
}

#[test]
fn test_domain_entity_assembly_evaluation() {
    let ids = IdGenerator::new();
    let mut graph = GraphStore::new();
    let mut store = DomainPayloadStore::new();

    // 1. Asset Loader Node
    let asset_node_id = ids.next_node_id();
    let asset_out_pin = ids.next_pin_id();
    graph.add_node(Node::new(asset_node_id, "Asset", "AssetLoader", bevy::math::Vec2::ZERO));
    graph.add_pin(Pin {
        id: asset_out_pin,
        node_id: asset_node_id,
        name: "Model".into(),
        direction: PinDirection::Output,
        data_type: DataType::new("Mesh"),
        capacity: PinCapacity::Multiple,
    }).unwrap();
    store.assets.insert(asset_node_id, AssetLoaderData {
        asset_path: "characters/knight.glb".into(),
        is_loaded: true,
    });

    // 2. Material Node
    let mat_node_id = ids.next_node_id();
    let mat_out_pin = ids.next_pin_id();
    graph.add_node(Node::new(mat_node_id, "Mat", "MaterialNode", bevy::math::Vec2::ZERO));
    graph.add_pin(Pin {
        id: mat_out_pin,
        node_id: mat_node_id,
        name: "Material".into(),
        direction: PinDirection::Output,
        data_type: DataType::new("Material"),
        capacity: PinCapacity::Multiple,
    }).unwrap();
    store.materials.insert(mat_node_id, MaterialData {
        base_color: Vec4::new(1.0, 0.0, 0.0, 1.0),
        perceptual_roughness: 0.25,
        metallic: 0.9,
        emissive: Vec4::ZERO,
    });

    // 3. Entity Assembler Node
    let entity_node_id = ids.next_node_id();
    let entity_mesh_in = ids.next_pin_id();
    let entity_mat_in = ids.next_pin_id();
    graph.add_node(Node::new(entity_node_id, "Knight Entity", "EntityAssembler", bevy::math::Vec2::ZERO));
    graph.add_pin(Pin {
        id: entity_mesh_in,
        node_id: entity_node_id,
        name: "Mesh".into(),
        direction: PinDirection::Input,
        data_type: DataType::new("Mesh"),
        capacity: PinCapacity::Single,
    }).unwrap();
    graph.add_pin(Pin {
        id: entity_mat_in,
        node_id: entity_node_id,
        name: "Material".into(),
        direction: PinDirection::Input,
        data_type: DataType::new("Material"),
        capacity: PinCapacity::Single,
    }).unwrap();
    store.entities.insert(entity_node_id, EntityAssemblerData {
        name: "Red Knight".into(),
        translation: Vec3::new(0.0, 2.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });

    // Connect Asset -> Entity and Material -> Entity
    graph.connect(ids.next_connection_id(), asset_out_pin, entity_mesh_in).unwrap();
    graph.connect(ids.next_connection_id(), mat_out_pin, entity_mat_in).unwrap();

    // Evaluate the assembled entity
    let assembled = store.evaluate_entity(entity_node_id, &graph).expect("Should evaluate entity");
    assert_eq!(assembled.name, "Red Knight");
    assert_eq!(assembled.translation, Vec3::new(0.0, 2.0, 0.0));
    assert_eq!(assembled.mesh_source, Some("characters/knight.glb".into()));

    let mat = assembled.material.expect("Material should be assembled");
    assert_eq!(mat.base_color, Vec4::new(1.0, 0.0, 0.0, 1.0));
    assert_eq!(mat.metallic, 0.9);
}

#[test]
fn test_subgraph_deep_wire_evaluation() {
    let ids = IdGenerator::new();
    let mut graph = GraphStore::new();
    let mut store = DomainPayloadStore::new();

    // 1. Asset Loader Node
    let asset_node_id = ids.next_node_id();
    let asset_out_pin = ids.next_pin_id();
    graph.add_node(Node::new(asset_node_id, "Asset", "AssetLoader", bevy::math::Vec2::ZERO));
    graph.add_pin(Pin {
        id: asset_out_pin,
        node_id: asset_node_id,
        name: "Model".into(),
        direction: PinDirection::Output,
        data_type: DataType::new("Mesh"),
        capacity: PinCapacity::Multiple,
    }).unwrap();
    store.assets.insert(asset_node_id, AssetLoaderData {
        asset_path: "characters/hero.glb".into(),
        is_loaded: true,
    });

    // 2. Material Node
    let mat_node_id = ids.next_node_id();
    let mat_out_pin = ids.next_pin_id();
    graph.add_node(Node::new(mat_node_id, "Mat", "MaterialNode", bevy::math::Vec2::ZERO));
    graph.add_pin(Pin {
        id: mat_out_pin,
        node_id: mat_node_id,
        name: "Material".into(),
        direction: PinDirection::Output,
        data_type: DataType::new("Material"),
        capacity: PinCapacity::Multiple,
    }).unwrap();
    store.materials.insert(mat_node_id, MaterialData {
        base_color: Vec4::new(0.0, 1.0, 0.5, 1.0),
        perceptual_roughness: 0.1,
        metallic: 0.95,
        emissive: Vec4::ZERO,
    });

    // 3. Group Asset and Material into Subgraph
    let sub_id = luma_editor::nodes::group_nodes(&[asset_node_id, mat_node_id], &mut graph, &ids)
        .expect("Should group nodes into subgraph");

    // 4. Entity Assembler Node
    let entity_node_id = ids.next_node_id();
    let entity_mesh_in = ids.next_pin_id();
    let entity_mat_in = ids.next_pin_id();
    graph.add_node(Node::new(entity_node_id, "Hero Entity", "EntityAssembler", bevy::math::Vec2::ZERO));
    graph.add_pin(Pin {
        id: entity_mesh_in,
        node_id: entity_node_id,
        name: "Mesh".into(),
        direction: PinDirection::Input,
        data_type: DataType::new("Mesh"),
        capacity: PinCapacity::Single,
    }).unwrap();
    graph.add_pin(Pin {
        id: entity_mat_in,
        node_id: entity_node_id,
        name: "Material".into(),
        direction: PinDirection::Input,
        data_type: DataType::new("Material"),
        capacity: PinCapacity::Single,
    }).unwrap();
    store.entities.insert(entity_node_id, EntityAssemblerData {
        name: "Hero Entity".into(),
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });

    // 5. Connect Subgraph output pins to Entity Assembler
    let sub_outputs: Vec<_> = graph
        .get_node(sub_id)
        .unwrap()
        .outputs
        .clone()
        .into_iter()
        .map(|pin_id| (pin_id, graph.get_pin(pin_id).unwrap().name.clone()))
        .collect();

    for (sub_out_pin_id, pin_name) in sub_outputs {
        if pin_name == "Model" {
            graph.connect(ids.next_connection_id(), sub_out_pin_id, entity_mesh_in).unwrap();
        } else if pin_name == "Material" {
            graph.connect(ids.next_connection_id(), sub_out_pin_id, entity_mat_in).unwrap();
        }
    }

    // 6. Evaluate EntityAssembler: should seamlessly resolve backwards through Subgraph
    let assembled = store.evaluate_entity(entity_node_id, &graph).expect("Should evaluate through Subgraph");
    assert_eq!(assembled.name, "Hero Entity");
    assert_eq!(assembled.mesh_source, Some("characters/hero.glb".into()));
    let mat = assembled.material.expect("Should resolve material from inside Subgraph");
    assert_eq!(mat.base_color, Vec4::new(0.0, 1.0, 0.5, 1.0));
    assert_eq!(mat.metallic, 0.95);
}

