use luma_core::graph::GraphStore;
use luma_domain::evaluation::DomainPayloadStore;
use std::path::Path;

pub struct BsnCompiler;

impl BsnCompiler {
    /// Compiles all assembled entities in the graph into a standard Bevy Scene Notation (.bsn) text.
    pub fn compile_scene(
        graph: &GraphStore,
        payloads: &DomainPayloadStore,
    ) -> Result<String, String> {
        let mut bsn_output = String::new();
        bsn_output.push_str("// ------------------------------------------------------------\n");
        bsn_output.push_str("// Generated automatically by Luma Editor (Zero-Runtime Scene)\n");
        bsn_output.push_str("// ------------------------------------------------------------\n\n");
        bsn_output.push_str("(\n");
        bsn_output.push_str("    entities: {\n");

        let mut entity_idx: usize = 0;

        for (&node_id, node) in graph.nodes() {
            if node.kind == "EntityAssembler" {
                if let Ok(entity) = payloads.evaluate_entity(node_id, graph) {
                    bsn_output.push_str(&format!("        {}: (\n", entity_idx));
                    bsn_output.push_str("            components: {\n");

                    // 1. Name Component
                    bsn_output.push_str(&format!(
                        "                \"bevy_core::name::Name\": \"{}\",\n",
                        entity.name
                    ));

                    // 2. Transform Component
                    bsn_output.push_str("                \"bevy_transform::components::transform::Transform\": (\n");
                    bsn_output.push_str(&format!(
                        "                    translation: ({:.3}, {:.3}, {:.3}),\n",
                        entity.translation.x, entity.translation.y, entity.translation.z
                    ));
                    bsn_output.push_str(&format!(
                        "                    rotation: ({:.3}, {:.3}, {:.3}, {:.3}),\n",
                        entity.rotation.x, entity.rotation.y, entity.rotation.z, entity.rotation.w
                    ));
                    bsn_output.push_str(&format!(
                        "                    scale: ({:.3}, {:.3}, {:.3}),\n",
                        entity.scale.x, entity.scale.y, entity.scale.z
                    ));
                    bsn_output.push_str("                ),\n");

                    // 3. Mesh Component (if present)
                    if let Some(mesh) = &entity.mesh_source {
                        bsn_output.push_str(&format!(
                            "                \"bevy_render::mesh::Mesh3d\": \"{}\",\n",
                            mesh
                        ));
                    }

                    // 4. Material Component (if present)
                    if let Some(mat) = &entity.material {
                        bsn_output.push_str("                \"bevy_pbr::pbr_material::MeshMaterial3d\": (\n");
                        bsn_output.push_str(&format!(
                            "                    base_color: ({:.3}, {:.3}, {:.3}, {:.3}),\n",
                            mat.base_color.x, mat.base_color.y, mat.base_color.z, mat.base_color.w
                        ));
                        bsn_output.push_str(&format!(
                            "                    perceptual_roughness: {:.3},\n",
                            mat.perceptual_roughness
                        ));
                        bsn_output.push_str(&format!(
                            "                    metallic: {:.3},\n",
                            mat.metallic
                        ));
                        bsn_output.push_str("                ),\n");
                    }

                    bsn_output.push_str("            },\n");
                    bsn_output.push_str("        ),\n");
                    entity_idx += 1;
                }
            } else if node.kind == "Subgraph" {
                if let Some(inner) = &node.subgraph {
                    let parent_idx = entity_idx;
                    entity_idx += 1;
                    let mut child_indices = Vec::new();
                    let mut inner_entities_output = String::new();

                    for (&inner_id, inner_node) in inner.nodes() {
                        if inner_node.kind == "EntityAssembler" {
                            if let Ok(child_entity) = payloads.evaluate_entity(inner_id, inner) {
                                let child_idx = entity_idx;
                                entity_idx += 1;
                                child_indices.push(child_idx);

                                inner_entities_output.push_str(&format!("        {}: (\n", child_idx));
                                inner_entities_output.push_str("            components: {\n");
                                inner_entities_output.push_str(&format!(
                                    "                \"bevy_core::name::Name\": \"{}\",\n",
                                    child_entity.name
                                ));
                                inner_entities_output.push_str(&format!(
                                    "                \"bevy_hierarchy::components::parent::Parent\": {},\n",
                                    parent_idx
                                ));
                                inner_entities_output.push_str("                \"bevy_transform::components::transform::Transform\": (\n");
                                inner_entities_output.push_str(&format!(
                                    "                    translation: ({:.3}, {:.3}, {:.3}),\n",
                                    child_entity.translation.x, child_entity.translation.y, child_entity.translation.z
                                ));
                                inner_entities_output.push_str(&format!(
                                    "                    rotation: ({:.3}, {:.3}, {:.3}, {:.3}),\n",
                                    child_entity.rotation.x, child_entity.rotation.y, child_entity.rotation.z, child_entity.rotation.w
                                ));
                                inner_entities_output.push_str(&format!(
                                    "                    scale: ({:.3}, {:.3}, {:.3}),\n",
                                    child_entity.scale.x, child_entity.scale.y, child_entity.scale.z
                                ));
                                inner_entities_output.push_str("                ),\n");

                                if let Some(mesh) = &child_entity.mesh_source {
                                    inner_entities_output.push_str(&format!(
                                        "                \"bevy_render::mesh::Mesh3d\": \"{}\",\n",
                                        mesh
                                    ));
                                }
                                if let Some(mat) = &child_entity.material {
                                    inner_entities_output.push_str("                \"bevy_pbr::pbr_material::MeshMaterial3d\": (\n");
                                    inner_entities_output.push_str(&format!(
                                        "                    base_color: ({:.3}, {:.3}, {:.3}, {:.3}),\n",
                                        mat.base_color.x, mat.base_color.y, mat.base_color.z, mat.base_color.w
                                    ));
                                    inner_entities_output.push_str(&format!(
                                        "                    perceptual_roughness: {:.3},\n",
                                        mat.perceptual_roughness
                                    ));
                                    inner_entities_output.push_str(&format!(
                                        "                    metallic: {:.3},\n",
                                        mat.metallic
                                    ));
                                    inner_entities_output.push_str("                ),\n");
                                }
                                inner_entities_output.push_str("            },\n");
                                inner_entities_output.push_str("        ),\n");
                            }
                        }
                    }

                    // Parent Subgraph Entity
                    bsn_output.push_str(&format!("        {}: (\n", parent_idx));
                    bsn_output.push_str("            components: {\n");
                    bsn_output.push_str(&format!(
                        "                \"bevy_core::name::Name\": \"{}\",\n",
                        node.title
                    ));
                    bsn_output.push_str("                \"bevy_transform::components::transform::Transform\": (\n");
                    bsn_output.push_str("                    translation: (0.000, 0.000, 0.000),\n");
                    bsn_output.push_str("                    rotation: (0.000, 0.000, 0.000, 1.000),\n");
                    bsn_output.push_str("                    scale: (1.000, 1.000, 1.000),\n");
                    bsn_output.push_str("                ),\n");
                    if !child_indices.is_empty() {
                        let children_list = child_indices
                            .iter()
                            .map(|idx| idx.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        bsn_output.push_str(&format!(
                            "                \"bevy_hierarchy::components::children::Children\": [{}],\n",
                            children_list
                        ));
                    }
                    bsn_output.push_str("            },\n");
                    bsn_output.push_str("        ),\n");

                    bsn_output.push_str(&inner_entities_output);
                }
            }
        }

        bsn_output.push_str("    },\n");
        bsn_output.push_str(")\n");

        Ok(bsn_output)
    }

    /// Compiles and writes the scene directly to a .bsn file on disk.
    pub fn compile_to_file(
        path: impl AsRef<Path>,
        graph: &GraphStore,
        payloads: &DomainPayloadStore,
    ) -> Result<(), String> {
        let bsn_content = Self::compile_scene(graph, payloads)?;
        std::fs::write(path, bsn_content).map_err(|e| e.to_string())?;
        Ok(())
    }
}
