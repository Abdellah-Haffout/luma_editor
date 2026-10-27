//! Clean, minimal node template and appearance customization infrastructure.

use bevy::math::Vec2;
use luma_core::graph::{DataType, PinCapacity, PinDirection};
use luma_core::id::NodeId;
use luma_dataflow::descriptor::{NodeBodyStyle, NodeCategory, PinShape};
use luma_dataflow::value::LinearColor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::nodes::palette::{NodePinTemplate, NodeTemplate};

/// Inner body layout mode for a node card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum CustomNodeContent {
    /// Clean node with header and input/output sockets only.
    #[default]
    Empty,
    /// Interactive `[-] [value] [+]` numeric stepper inside the node body.
    ScalarStepper {
        default_val: f32,
        step: f32,
    },
    /// Generic custom widget slot of `height` logical pixels populated by the node's `body` callback.
    CustomWidget {
        height: f32,
    },
}

/// Visual appearance settings for a node type or an individual node instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomNodeAppearance {
    pub accent_color: LinearColor,
    pub header_bg: LinearColor,
    pub body_bg: LinearColor,
    pub corner_radius: f32,
}

impl Default for CustomNodeAppearance {
    fn default() -> Self {
        Self {
            accent_color: LinearColor::new(0.949, 0.600, 0.180, 1.0),
            header_bg: LinearColor::new(0.322, 0.212, 0.431, 1.0),
            body_bg: LinearColor::new(0.188, 0.188, 0.192, 1.0),
            corner_radius: 8.0,
        }
    }
}

/// Per-instance visual customization stored for any node on the canvas.
#[derive(Debug, Clone, Default)]
pub struct NodeInstanceCustomization {
    pub accent_color: Option<LinearColor>,
    pub header_bg: Option<LinearColor>,
    pub body_bg: Option<LinearColor>,
    pub corner_radius: Option<f32>,
    pub content: Option<CustomNodeContent>,
}

/// Builder for constructing a `NodeTemplate` with pins, category, appearance, and body slot.
#[derive(Debug, Clone)]
pub struct CustomNodeBuilder {
    kind: String,
    title: String,
    category: NodeCategory,
    appearance: CustomNodeAppearance,
    content: CustomNodeContent,
    pins: Vec<NodePinTemplate>,
    min_width: f32,
}

impl CustomNodeBuilder {
    pub fn new(kind: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            title: title.into(),
            category: NodeCategory::new("Custom").with_tag("NODE"),
            appearance: CustomNodeAppearance::default(),
            content: CustomNodeContent::Empty,
            pins: Vec::new(),
            min_width: 196.0,
        }
    }

    pub fn category(mut self, category_path: &'static str, badge_tag: &'static str) -> Self {
        self.category = NodeCategory::new(category_path).with_tag(badge_tag);
        self
    }

    pub fn accent_rgb(mut self, r: f32, g: f32, b: f32) -> Self {
        self.appearance.accent_color = LinearColor::new(r, g, b, 1.0);
        self
    }

    pub fn header_rgb(mut self, r: f32, g: f32, b: f32) -> Self {
        self.appearance.header_bg = LinearColor::new(r, g, b, 1.0);
        self
    }

    pub fn body_rgb(mut self, r: f32, g: f32, b: f32) -> Self {
        self.appearance.body_bg = LinearColor::new(r, g, b, 1.0);
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.appearance.corner_radius = radius;
        self
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    pub fn content(mut self, content: CustomNodeContent) -> Self {
        self.content = content;
        self
    }

    pub fn custom_widget_slot(mut self, height: f32) -> Self {
        self.content = CustomNodeContent::CustomWidget { height };
        self
    }

    pub fn input(
        mut self,
        name: impl Into<String>,
        data_type: impl Into<String>,
        shape: PinShape,
    ) -> Self {
        self.pins.push(NodePinTemplate {
            name: name.into(),
            direction: PinDirection::Input,
            data_type: DataType::new(data_type.into()),
            capacity: PinCapacity::Single,
            shape,
        });
        self
    }

    pub fn output(
        mut self,
        name: impl Into<String>,
        data_type: impl Into<String>,
        shape: PinShape,
    ) -> Self {
        self.pins.push(NodePinTemplate {
            name: name.into(),
            direction: PinDirection::Output,
            data_type: DataType::new(data_type.into()),
            capacity: PinCapacity::Multiple,
            shape,
        });
        self
    }

    pub fn build(self) -> NodeTemplate {
        let input_count = self
            .pins
            .iter()
            .filter(|p| p.direction == PinDirection::Input)
            .count();
        let output_count = self
            .pins
            .iter()
            .filter(|p| p.direction == PinDirection::Output)
            .count();
        let max_slots = input_count.max(output_count).max(1);
        let extra_body_h = match &self.content {
            CustomNodeContent::Empty => 0.0,
            CustomNodeContent::ScalarStepper { .. } => 26.0,
            CustomNodeContent::CustomWidget { height } => *height + 8.0,
        };
        let body_style = match &self.content {
            CustomNodeContent::Empty => NodeBodyStyle::Empty,
            CustomNodeContent::ScalarStepper { .. } => NodeBodyStyle::Parameters,
            CustomNodeContent::CustomWidget { .. } => NodeBodyStyle::CustomSlot,
        };
        let height = 30.0 + (max_slots as f32 * 24.0) + extra_body_h + 12.0;

        NodeTemplate {
            kind: self.kind,
            default_title: self.title,
            category: self.category,
            accent_color: Some(self.appearance.accent_color),
            header_bg: Some(self.appearance.header_bg),
            body_bg: Some(self.appearance.body_bg),
            corner_radius: self.appearance.corner_radius,
            custom_content: self.content,
            body_style,
            pins: self.pins,
            size: Vec2::new(self.min_width, height.max(64.0)),
        }
    }
}

#[derive(Default)]
pub struct RuntimeNodeCustomizationStore {
    pub custom_templates: Vec<NodeTemplate>,
    pub instance_overrides: HashMap<NodeId, NodeInstanceCustomization>,
}

static CUSTOM_STORE: OnceLock<Mutex<RuntimeNodeCustomizationStore>> = OnceLock::new();

pub fn custom_node_store() -> &'static Mutex<RuntimeNodeCustomizationStore> {
    CUSTOM_STORE.get_or_init(|| Mutex::new(RuntimeNodeCustomizationStore::default()))
}

pub fn register_custom_node_template(template: NodeTemplate) {
    if let Ok(mut store) = custom_node_store().lock() {
        store.custom_templates.retain(|t| t.kind != template.kind);
        store.custom_templates.push(template);
    }
}

pub fn set_node_instance_customization(
    node_id: NodeId,
    update: impl FnOnce(&mut NodeInstanceCustomization),
) {
    if let Ok(mut store) = custom_node_store().lock() {
        let entry = store.instance_overrides.entry(node_id).or_default();
        update(entry);
    }
}

pub fn get_node_instance_customization(node_id: NodeId) -> Option<NodeInstanceCustomization> {
    custom_node_store()
        .lock()
        .ok()
        .and_then(|s| s.instance_overrides.get(&node_id).cloned())
}

pub fn builtin_custom_node_templates() -> Vec<NodeTemplate> {
    if let Ok(store) = custom_node_store().lock() {
        store.custom_templates.clone()
    } else {
        Vec::new()
    }
}
