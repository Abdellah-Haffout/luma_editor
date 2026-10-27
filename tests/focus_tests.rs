use luma_editor::core::id::NodeId;
use luma_editor::focus::*;

#[test]
fn test_navigation_state_breadcrumbs_and_dive_in() {
    let mut nav = NavigationState::new();

    // 1. Initial Root state
    assert_eq!(*nav.current(), WorkspaceContext::RootGraph);
    assert!(!nav.is_in_focus());
    assert_eq!(nav.focused_node_id(), None);
    assert_eq!(nav.breadcrumbs(), vec!["Root Graph".to_string()]);

    // 2. Dive into Mesh Node
    let mesh_id = NodeId(10);
    nav.dive_into(mesh_id, "MeshNode", "Hero Model", 0.0, 0.0, 1.0);

    assert!(nav.is_in_focus());
    assert_eq!(nav.focused_node_id(), Some(mesh_id));
    assert_eq!(nav.focused_node_kind(), Some("MeshNode"));
    assert_eq!(nav.depth(), 2);
    assert_eq!(
        nav.breadcrumbs(),
        vec!["Root Graph".to_string(), "Hero Model".to_string()]
    );

    // 3. Nested dive-in (e.g. into sub-material or sub-graph)
    let mat_id = NodeId(20);
    nav.dive_into(mat_id, "MaterialNode", "Armor Shader", 0.0, 0.0, 1.0);
    assert_eq!(nav.depth(), 3);
    assert_eq!(
        nav.breadcrumbs(),
        vec![
            "Root Graph".to_string(),
            "Hero Model".to_string(),
            "Armor Shader".to_string()
        ]
    );

    // 4. Exit focus back to Mesh Node
    let popped = nav.exit_focus();
    assert!(popped.is_some());
    assert_eq!(nav.focused_node_id(), Some(mesh_id));
    assert_eq!(nav.depth(), 2);

    // 5. Exit focus back to Root Graph
    nav.exit_focus();
    assert_eq!(*nav.current(), WorkspaceContext::RootGraph);
    assert!(!nav.is_in_focus());
    assert_eq!(nav.depth(), 1);

    // 6. Cannot exit Root Graph
    assert_eq!(nav.exit_focus(), None);
}

#[test]
fn test_contextual_tool_registry_and_strict_isolation() {
    let registry = ContextualToolRegistry::with_default_toolsets();

    // 1. Mesh Node tools
    let mesh_tools = registry.get_tools_for_kind("MeshNode");
    assert_eq!(mesh_tools.len(), 5);
    assert!(mesh_tools.iter().any(|t| t.id == "mesh.translate"));
    assert!(mesh_tools.iter().any(|t| t.id == "mesh.wireframe"));

    // Strict Isolation: MeshNode MUST NOT contain material or physics tools!
    assert!(!mesh_tools.iter().any(|t| t.id.starts_with("mat.")));
    assert!(!mesh_tools.iter().any(|t| t.id.starts_with("physics.")));

    // 2. Material Node tools
    let mat_tools = registry.get_tools_for_kind("MaterialNode");
    assert_eq!(mat_tools.len(), 4);
    assert!(mat_tools.iter().any(|t| t.id == "mat.color_picker"));
    assert!(mat_tools.iter().any(|t| t.id == "mat.roughness"));

    // Strict Isolation: MaterialNode MUST NOT contain geometry or physics tools!
    assert!(!mat_tools.iter().any(|t| t.id.starts_with("mesh.")));
    assert!(!mat_tools.iter().any(|t| t.id.starts_with("physics.")));

    // 3. Physics Node tools
    let phys_tools = registry.get_tools_for_kind("PhysicsNode");
    assert_eq!(phys_tools.len(), 3);
    assert!(phys_tools.iter().any(|t| t.id == "physics.collider"));
    assert!(!phys_tools.iter().any(|t| t.id.starts_with("mesh.")));
    assert!(!phys_tools.iter().any(|t| t.id.starts_with("mat.")));

    // 4. Unknown kind returns empty tools
    assert_eq!(registry.get_tools_for_kind("UnknownKind").len(), 0);
}
