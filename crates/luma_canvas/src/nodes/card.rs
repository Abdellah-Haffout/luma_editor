use crate::nodes::custom::{get_node_instance_customization, CustomNodeContent};
use crate::nodes::palette::NodePalette;
use luma_core::graph::{GraphStore, Pin, PinDirection};
use luma_core::id::{NodeId, PinId};
use luma_dataflow::descriptor::NodeBodyStyle;
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeCardComponent {
    pub node_id: NodeId,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinSocketComponent {
    pub pin_id: PinId,
    pub node_id: NodeId,
    pub direction: PinDirection,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiveInButtonComponent {
    pub node_id: NodeId,
}

/// Representation of a node card's visual layout parameters.
/// Declarative, open-closed architecture modeled after UnivisEditor.
#[derive(Debug, Clone)]
pub struct NodeCardLayout {
    pub node_id: NodeId,
    pub title: String,
    pub kind: String,
    pub position: Vec2,
    pub size: Vec2,
    pub input_pins: Vec<Pin>,
    pub output_pins: Vec<Pin>,
    pub accent_color: Color,
    pub header_bg: Color,
    pub body_bg: Color,
    pub corner_radius: f32,
    pub category_tag: String,
    pub body_style: NodeBodyStyle,
    pub custom_content: CustomNodeContent,
    pub number_value: Option<f32>,
    pub input_display_texts: Vec<String>,
    pub output_display_texts: Vec<String>,
    pub connected_pins: std::collections::HashSet<PinId>,
}

impl NodeCardLayout {
    pub fn is_pin_connected(&self, pin_id: PinId) -> bool {
        self.connected_pins.contains(&pin_id)
    }

    pub fn total_connections(&self) -> usize {
        self.connected_pins.len()
    }

    /// Maps a node kind and category tag to an authentic Blender Node Editor header bar color.
    #[inline]
    pub fn blender_header_color(kind: &str, category_tag: &str) -> Color {
        if kind == "math.number"
            || kind == "math.integer"
            || kind.starts_with("math.constant")
            || matches!(category_tag, "NUM" | "INT" | "CONST" | "ASSET" | "INPUT" | "LINK")
        {
            // Blender Input Node Header (Crimson / Rose Red #83314A)
            Color::srgb(0.514, 0.192, 0.290)
        } else if category_tag == "COLOR" || category_tag == "IMAGE" {
            // Blender Color / Image Node Header (Warm Ochre #7A5821)
            Color::srgb(0.478, 0.345, 0.129)
        } else if kind.starts_with("shader.") || matches!(category_tag, "SHADER" | "MAT") {
            // Blender Shader Node Header (Emerald Green #2B6644)
            Color::srgb(0.169, 0.400, 0.267)
        } else if kind.starts_with("scene.")
            || matches!(category_tag, "MESH" | "ENTITY" | "CAMERA" | "LIGHT" | "GEOM")
        {
            // Blender Geometry Node Header (Deep Teal #1F665A)
            Color::srgb(0.122, 0.400, 0.353)
        } else if kind.starts_with("math.vector") || category_tag == "VEC" {
            // Blender Vector Node Header (Indigo / Purple #4B3B80)
            Color::srgb(0.294, 0.231, 0.502)
        } else if kind == "Subgraph" || category_tag == "GROUP" {
            // Blender Group Node Header (Olive Green #3B572E)
            Color::srgb(0.231, 0.341, 0.180)
        } else if kind.starts_with("custom.")
            || kind.starts_with("signal.")
            || matches!(category_tag, "CUSTOM" | "PROBE" | "SIGNAL" | "ANIM" | "VIEW")
        {
            // Blender Layout / Custom Node Header (Plum / Violet #52366E)
            Color::srgb(0.322, 0.212, 0.431)
        } else {
            // Blender Converter / Math Node Header (Steel Blue #29567A)
            Color::srgb(0.161, 0.337, 0.478)
        }
    }

    /// Returns true if this node is a mathematical operation node that displays
    /// a dedicated Blender-style live received-inputs & computed-output readout box.
    #[inline]
    pub fn has_math_value_readout(kind: &str) -> bool {
        kind.starts_with("math.") && !matches!(kind, "math.number" | "math.integer")
    }

    /// Computes the exact 2D visual size of a node card without allocating any Vecs or HashSets.
    #[inline]
    pub fn compute_size_fast(kind: &str, node_size: Vec2, input_count: usize, output_count: usize) -> Vec2 {
        let body_style = if let Some(tmpl) = NodePalette::with_default_templates().get(kind) {
            tmpl.body_style
        } else if let Some(desc) = crate::nodes::stdlib::stdlib_registry().get(kind) {
            desc.ui.body_style
        } else {
            NodeBodyStyle::Empty
        };

        let header_height = 28.0;
        let row_height = 26.0;
        let max_pins = input_count.max(output_count).max(1);
        let rows_height = max_pins as f32 * row_height;
        let readout_h = if Self::has_math_value_readout(kind) {
            36.0
        } else {
            0.0
        };

        let custom_content = if let Some(tmpl) = NodePalette::with_default_templates().get(kind) {
            tmpl.custom_content.clone()
        } else {
            CustomNodeContent::Empty
        };
        let extra_custom_h = match &custom_content {
            CustomNodeContent::Empty | CustomNodeContent::ScalarStepper { .. } => 0.0,
            CustomNodeContent::CustomWidget { height } => *height + 8.0,
        };

        match body_style {
            NodeBodyStyle::Empty => {
                let min_w = if readout_h > 0.0 { 206.0f32 } else { 196.0f32 };
                let w = node_size.x.max(min_w);
                let h = header_height + 6.0 + rows_height + readout_h + 8.0;
                Vec2::new(w, h)
            }
            NodeBodyStyle::Parameters => {
                let w = node_size.x.max(216.0f32);
                let h = header_height + 6.0 + rows_height.max(28.0) + readout_h + 8.0;
                Vec2::new(w, h)
            }
            NodeBodyStyle::CustomSlot => {
                let w = node_size.x.max(220.0f32);
                let h = header_height + 6.0 + rows_height + extra_custom_h + readout_h + 10.0;
                Vec2::new(w, h)
            }
        }
    }

    pub fn from_graph(node_id: NodeId, graph: &GraphStore) -> Option<Self> {
        Self::from_graph_with_registry(node_id, graph, None)
    }

    pub fn from_graph_with_registry(
        node_id: NodeId,
        graph: &GraphStore,
        registry: Option<&luma_dataflow::registry::NodeRegistry>,
    ) -> Option<Self> {
        let node = graph.get_node(node_id)?;

        let mut input_pins = Vec::new();
        let mut input_display_texts = Vec::new();
        for &pin_id in &node.inputs {
            if let Some(pin) = graph.get_pin(pin_id) {
                input_pins.push(pin.clone());
                input_display_texts.push(pin.name.clone());
            }
        }

        let mut output_pins = Vec::new();
        let mut output_display_texts = Vec::new();
        for &pin_id in &node.outputs {
            if let Some(pin) = graph.get_pin(pin_id) {
                output_pins.push(pin.clone());
                output_display_texts.push(pin.name.clone());
            }
        }

        let mut connected_pins = std::collections::HashSet::new();
        for pin in input_pins.iter().chain(output_pins.iter()) {
            if !graph.get_pin_connections(pin.id).is_empty() {
                connected_pins.insert(pin.id);
            }
        }

        // Declarative lookup from active NodeRegistry + NodePalette + runtime overrides
        let palette = match registry {
            Some(reg) => NodePalette::from_registry(reg),
            None => NodePalette::with_default_templates(),
        };
        let tmpl_opt = palette.get(&node.kind);
        let desc = registry
            .and_then(|reg| reg.get(&node.kind))
            .or_else(|| crate::nodes::stdlib::stdlib_registry().get(&node.kind));

        // Blender Node Editor default body background (#303030)
        let blender_body_bg = Color::srgb(0.188, 0.188, 0.192);
        // Blender active selection highlight orange (#F2992E)
        let blender_select_accent = Color::srgb(0.949, 0.600, 0.180);

        let (mut accent_color, mut header_bg, mut body_bg, mut corner_radius, category_tag, mut body_style, mut custom_content) =
            if let Some(tmpl) = tmpl_opt {
                let tag = tmpl.category.tag.to_string();
                let accent = tmpl
                    .accent_color
                    .map(|c| Color::srgba(c.r, c.g, c.b, c.a))
                    .unwrap_or(blender_select_accent);
                let header = tmpl
                    .header_bg
                    .map(|c| Color::srgba(c.r, c.g, c.b, c.a))
                    .unwrap_or_else(|| Self::blender_header_color(&node.kind, &tag));
                let body = tmpl
                    .body_bg
                    .map(|c| Color::srgba(c.r, c.g, c.b, c.a))
                    .unwrap_or(blender_body_bg);
                (
                    accent,
                    header,
                    body,
                    8.0,
                    tag,
                    tmpl.body_style,
                    tmpl.custom_content.clone(),
                )
            } else if let Some(desc) = desc {
                let tag = desc.ui.category.tag.to_string();
                let accent = desc
                    .ui
                    .accent_color
                    .map(|c| Color::srgba(c.r, c.g, c.b, c.a))
                    .unwrap_or(blender_select_accent);
                let header = Self::blender_header_color(&node.kind, &tag);
                (
                    accent,
                    header,
                    blender_body_bg,
                    8.0,
                    tag,
                    desc.ui.body_style,
                    CustomNodeContent::Empty,
                )
            } else {
                let tag = "NODE".to_string();
                let header = Self::blender_header_color(&node.kind, &tag);
                (
                    blender_select_accent,
                    header,
                    blender_body_bg,
                    8.0,
                    tag,
                    NodeBodyStyle::Empty,
                    CustomNodeContent::Empty,
                )
            };

        if let Some(ov) = get_node_instance_customization(node_id) {
            if let Some(c) = ov.accent_color {
                accent_color = Color::srgba(c.r, c.g, c.b, c.a);
            }
            if let Some(c) = ov.header_bg {
                header_bg = Color::srgba(c.r, c.g, c.b, c.a);
            }
            if let Some(c) = ov.body_bg {
                body_bg = Color::srgba(c.r, c.g, c.b, c.a);
            }
            if let Some(r) = ov.corner_radius {
                corner_radius = r;
            }
            if let Some(content) = ov.content {
                body_style = match &content {
                    CustomNodeContent::Empty => NodeBodyStyle::Empty,
                    CustomNodeContent::ScalarStepper { .. } => NodeBodyStyle::Parameters,
                    _ => NodeBodyStyle::CustomSlot,
                };
                custom_content = content;
            }
        }

        // Determine if node has an inline number value to display
        let number_value = if node.kind == "math.number" {
            Some(10.0)
        } else if node.kind == "math.integer" {
            Some(0.0)
        } else if let CustomNodeContent::ScalarStepper { default_val, .. } = custom_content {
            Some(default_val)
        } else {
            None
        };

        let header_height = 28.0;
        let row_height = 26.0;
        let max_pins = input_pins.len().max(output_pins.len()).max(1);
        let rows_height = max_pins as f32 * row_height;
        let readout_h = if Self::has_math_value_readout(&node.kind) {
            36.0
        } else {
            0.0
        };
        let extra_custom_h = match &custom_content {
            CustomNodeContent::Empty | CustomNodeContent::ScalarStepper { .. } => 0.0,
            CustomNodeContent::CustomWidget { height } => *height + 8.0,
        };

        let size = match body_style {
            NodeBodyStyle::Empty => {
                let min_w = if readout_h > 0.0 { 206.0f32 } else { 196.0f32 };
                let w = node.size.x.max(min_w);
                let h = header_height + 6.0 + rows_height + readout_h + 8.0;
                Vec2::new(w, h)
            }
            NodeBodyStyle::Parameters => {
                let w = node.size.x.max(216.0f32);
                let h = header_height + 6.0 + rows_height.max(28.0) + readout_h + 8.0;
                Vec2::new(w, h)
            }
            NodeBodyStyle::CustomSlot => {
                let w = node.size.x.max(220.0f32);
                let h = header_height + 6.0 + rows_height + extra_custom_h + readout_h + 10.0;
                Vec2::new(w, h)
            }
        };

        Some(Self {
            node_id,
            title: node.title.clone(),
            kind: node.kind.clone(),
            position: node.position,
            size,
            input_pins,
            output_pins,
            accent_color,
            header_bg,
            body_bg,
            corner_radius,
            category_tag,
            body_style,
            custom_content,
            number_value,
            input_display_texts,
            output_display_texts,
            connected_pins,
        })
    }


    /// Calculate the relative local position of a pin socket on the card.
    pub fn pin_local_position(&self, pin_id: PinId) -> Option<Vec2> {
        let header_offset = 34.0;
        let y_spacing = 26.0;

        // Inputs placed exactly on left edge
        if let Some(idx) = self.input_pins.iter().position(|p| p.id == pin_id) {
            let y = header_offset + (idx as f32 * y_spacing) + 12.0;
            return Some(Vec2::new(0.0, y));
        }

        // Outputs placed exactly on right edge
        if let Some(idx) = self.output_pins.iter().position(|p| p.id == pin_id) {
            let y = header_offset + (idx as f32 * y_spacing) + 12.0;
            return Some(Vec2::new(self.size.x, y));
        }

        None
    }
}
