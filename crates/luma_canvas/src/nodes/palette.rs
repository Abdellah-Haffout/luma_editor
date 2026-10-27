use crate::nodes::custom::{builtin_custom_node_templates, CustomNodeContent};
use luma_core::graph::{DataType, GraphStore, Node, Pin, PinCapacity, PinDirection};
use luma_core::history::{AddNodeCommand, HistoryError, HistoryStack};
use luma_core::id::{IdGenerator, NodeId};
use luma_dataflow::descriptor::{NodeBodyStyle, NodeCategory, PinShape};
use luma_dataflow::value::LinearColor;
use bevy::math::Vec2;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NodePinTemplate {
    pub name: String,
    pub direction: PinDirection,
    pub data_type: DataType,
    pub capacity: PinCapacity,
    pub shape: PinShape,
}

#[derive(Debug, Clone)]
pub struct NodeTemplate {
    pub kind: String,
    pub default_title: String,
    pub category: NodeCategory,
    pub accent_color: Option<LinearColor>,
    pub header_bg: Option<LinearColor>,
    pub body_bg: Option<LinearColor>,
    pub corner_radius: f32,
    pub custom_content: CustomNodeContent,
    pub body_style: NodeBodyStyle,
    pub pins: Vec<NodePinTemplate>,
    pub size: Vec2,
}

#[derive(Debug, Default)]
pub struct NodePalette {
    templates: HashMap<String, NodeTemplate>,
}

impl NodePalette {
    pub fn new() -> Self {
        Self::default()
    }

    /// Dynamically constructs templates for all registered nodes in the dataflow registry.
    pub fn from_registry(registry: &luma_dataflow::registry::NodeRegistry) -> Self {
        let mut palette = Self::new();

        for desc in registry.descriptors() {
            let mut pins = Vec::with_capacity(desc.inputs.len() + desc.outputs.len());
            for in_pin in &desc.inputs {
                pins.push(NodePinTemplate {
                    name: in_pin.name.to_string(),
                    direction: PinDirection::Input,
                    data_type: DataType::new(in_pin.data_type.to_string()),
                    capacity: PinCapacity::Single,
                    shape: in_pin.shape,
                });
            }
            for out_pin in &desc.outputs {
                pins.push(NodePinTemplate {
                    name: out_pin.name.to_string(),
                    direction: PinDirection::Output,
                    data_type: DataType::new(out_pin.data_type.to_string()),
                    capacity: PinCapacity::Multiple,
                    shape: out_pin.shape,
                });
            }

            let max_slots = desc.inputs.len().max(desc.outputs.len());
            let card_height = (36.0 + (max_slots as f32) * 24.0 + 10.0).max(56.0);
            let card_width = desc.ui.min_size.map(|s| s[0]).unwrap_or(190.0);
            let size = Vec2::new(card_width, card_height);

            palette.register(NodeTemplate {
                kind: desc.type_id.to_string(),
                default_title: desc.title.to_string(),
                category: desc.ui.category.clone(),
                accent_color: desc.ui.accent_color,
                header_bg: desc.ui.header_bg,
                body_bg: None,
                corner_radius: 6.0,
                custom_content: if desc.ui.body_style == NodeBodyStyle::Parameters {
                    CustomNodeContent::ScalarStepper {
                        default_val: 10.0,
                        step: 1.0,
                    }
                } else {
                    CustomNodeContent::Empty
                },
                body_style: desc.ui.body_style,
                pins,
                size,
            });
        }

        for custom_tmpl in builtin_custom_node_templates() {
            palette.register(custom_tmpl);
        }

        palette
    }

    /// Generates palette containing only custom/application-registered node templates.
    pub fn with_default_templates() -> Self {
        let registry = luma_dataflow::registry::NodeRegistry::new();
        Self::from_registry(&registry)
    }

    /// Groups all registered templates by category name, sorted alphabetically.
    pub fn by_category(&self) -> std::collections::BTreeMap<String, Vec<&NodeTemplate>> {
        let mut map: std::collections::BTreeMap<String, Vec<&NodeTemplate>> =
            std::collections::BTreeMap::new();
        for tmpl in self.templates.values() {
            let group_name = match tmpl.category.primary_group() {
                "Math" => match tmpl.category.sub_group() {
                    Some("Trigonometry") | Some("Interpolation") => "Math: Trig & Interp",
                    _ => "Math: Arithmetic",
                },
                "Vector" => "Vector & Transform",
                "Logic" => "Logic & Comparison",
                other => other,
            };
            map.entry(group_name.to_string()).or_default().push(tmpl);
        }
        for list in map.values_mut() {
            list.sort_by(|a, b| a.default_title.cmp(&b.default_title));
        }
        map
    }

    pub fn register(&mut self, template: NodeTemplate) {
        self.templates.insert(template.kind.clone(), template);
    }

    pub fn get(&self, kind: &str) -> Option<&NodeTemplate> {
        self.templates.get(kind)
    }

    pub fn all(&self) -> impl Iterator<Item = &NodeTemplate> {
        self.templates.values()
    }

    /// Spawns a new node with its default pins from a template at the given canvas position.
    pub fn spawn_node(
        &self,
        kind: &str,
        position: Vec2,
        graph: &mut GraphStore,
        id_gen: &IdGenerator,
        history: &mut HistoryStack,
    ) -> Result<NodeId, HistoryError> {
        let template = self
            .get(kind)
            .ok_or_else(|| HistoryError::Graph(luma_core::graph::GraphError::NodeNotFound(NodeId(0))))?;

        let node_id = id_gen.next_node_id();
        let mut node = Node::new(node_id, &template.default_title, &template.kind, position);
        node.size = template.size;

        if kind == "Subgraph" {
            let mut inner = GraphStore::new();
            let grp_in_id = id_gen.next_node_id();
            let mut grp_in = Node::new(grp_in_id, "Group Input", "GroupInput", Vec2::new(-240.0, 0.0));
            grp_in.size = Vec2::new(160.0, 76.0);
            let in_p_id = id_gen.next_pin_id();
            grp_in.outputs.push(in_p_id);
            inner.add_node(grp_in);
            let _ = inner.add_pin(Pin {
                id: in_p_id,
                node_id: grp_in_id,
                name: "Input".into(),
                direction: PinDirection::Output,
                data_type: DataType::any(),
                capacity: PinCapacity::Multiple,
            });

            let grp_out_id = id_gen.next_node_id();
            let mut grp_out = Node::new(grp_out_id, "Group Output", "GroupOutput", Vec2::new(240.0, 0.0));
            grp_out.size = Vec2::new(160.0, 76.0);
            let out_p_id = id_gen.next_pin_id();
            grp_out.inputs.push(out_p_id);
            inner.add_node(grp_out);
            let _ = inner.add_pin(Pin {
                id: out_p_id,
                node_id: grp_out_id,
                name: "Output".into(),
                direction: PinDirection::Input,
                data_type: DataType::any(),
                capacity: PinCapacity::Single,
            });

            node.subgraph = Some(Box::new(inner));
        }

        history.begin_transaction(format!("Spawn {}", template.default_title));

        // Create the node
        history.apply(Box::new(AddNodeCommand::new(node)), graph)?;

        // Create the pins
        for pin_tmpl in &template.pins {
            let pin_id = id_gen.next_pin_id();
            let pin = Pin {
                id: pin_id,
                node_id,
                name: pin_tmpl.name.clone(),
                direction: pin_tmpl.direction,
                data_type: pin_tmpl.data_type.clone(),
                capacity: pin_tmpl.capacity,
            };
            graph.add_pin(pin)?;
        }

        history.commit_transaction();

        Ok(node_id)
    }
}
