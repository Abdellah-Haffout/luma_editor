    let type_color = match layout.kind.as_str() {
        "AssetLoader" => EditorTheme::PIN_ASSET,
        "MeshNode" => EditorTheme::PIN_MESH,
        "MaterialNode" => EditorTheme::PIN_MATERIAL,
        "EntityAssembler" => EditorTheme::PIN_ENTITY,
        "Subgraph" => Color::srgb(0.68, 0.42, 0.95),
        _ => layout.accent_color,
    };

    bsn! {
        NodeCardRoot { node_id }
        NodeCardVisual { node_id, type_color }