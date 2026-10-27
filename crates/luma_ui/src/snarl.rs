//! Clean, unified API for defining, styling, and registering custom nodes in one place.
//!
//! Inspired by `egui-snarl`'s philosophy of keeping the core completely node-agnostic while making
//! node definition and custom body UI mounting concise and unified.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use luma_canvas::nodes::card::NodeCardLayout;
use luma_canvas::nodes::custom::{register_custom_node_template, CustomNodeBuilder};
use luma_canvas::nodes::palette::NodePalette;
use luma_canvas::plugin::{
    EditorDataflowResource, EditorGraphResource, EditorHistoryResource, EditorIdGenResource,
    EditorSelectionResource,
};
use luma_core::id::NodeId;
use luma_dataflow::cache::NodeExecutionStatus;
use luma_dataflow::descriptor::{NodeCategory, NodeDescriptor, PinShape};
use luma_dataflow::types::DataType as DfDataType;
use luma_dataflow::value::{LinearColor, PrimitiveValue, Value};
use std::collections::HashSet;
use std::sync::Arc;

use crate::components::node_card::NodeBodyUiRegistry;

/// Describes an input or output pin on a node.
#[derive(Debug, Clone)]
pub struct PinInfo {
    pub name: String,
    pub data_type: DfDataType,
    pub default_value: PrimitiveValue,
    pub shape: PinShape,
}

impl PinInfo {
    pub fn new(name: impl Into<String>, data_type: DfDataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            default_value: PrimitiveValue::None,
            shape: PinShape::Circle,
        }
    }

    pub fn float(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data_type: DfDataType::Float,
            default_value: PrimitiveValue::Float(0.0),
            shape: PinShape::Circle,
        }
    }

    pub fn int(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data_type: DfDataType::Int,
            default_value: PrimitiveValue::Int(0),
            shape: PinShape::Circle,
        }
    }

    pub fn bool(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data_type: DfDataType::Bool,
            default_value: PrimitiveValue::Bool(false),
            shape: PinShape::Circle,
        }
    }

    pub fn string(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data_type: DfDataType::String,
            default_value: PrimitiveValue::String(String::new()),
            shape: PinShape::Circle,
        }
    }

    pub fn vec3(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data_type: DfDataType::Vec3,
            default_value: PrimitiveValue::Vec3(Vec3::ZERO),
            shape: PinShape::Circle,
        }
    }

    pub fn any(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data_type: DfDataType::ANY,
            default_value: PrimitiveValue::None,
            shape: PinShape::Circle,
        }
    }

    pub fn color(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data_type: DfDataType::Color,
            default_value: PrimitiveValue::Color(LinearColor::WHITE),
            shape: PinShape::Circle,
        }
    }

    pub fn custom(name: impl Into<String>, type_name: &'static str) -> Self {
        Self {
            name: name.into(),
            data_type: DfDataType::custom(type_name),
            default_value: PrimitiveValue::None,
            shape: PinShape::Diamond,
        }
    }

    pub fn default_float(mut self, val: f32) -> Self {
        self.default_value = PrimitiveValue::Float(val);
        self
    }

    pub fn default_string(mut self, val: impl Into<String>) -> Self {
        self.default_value = PrimitiveValue::String(val.into());
        self
    }

    pub fn circle(mut self) -> Self {
        self.shape = PinShape::Circle;
        self
    }

    pub fn square(mut self) -> Self {
        self.shape = PinShape::Square;
        self
    }

    pub fn diamond(mut self) -> Self {
        self.shape = PinShape::Diamond;
        self
    }
}

/// Visual styling for a node card's header and frame.
#[derive(Debug, Clone)]
pub struct NodeStyle {
    pub accent_rgb: [f32; 3],
    pub header_rgb: [f32; 3],
    pub body_rgb: [f32; 3],
    pub corner_radius: f32,
    pub min_width: f32,
}

impl Default for NodeStyle {
    fn default() -> Self {
        Self {
            accent_rgb: [0.949, 0.600, 0.180],
            header_rgb: [0.322, 0.212, 0.431],
            body_rgb: [0.188, 0.188, 0.192],
            corner_radius: 8.0,
            min_width: 210.0,
        }
    }
}

impl NodeStyle {
    pub fn header_rgb(r: f32, g: f32, b: f32) -> Self {
        Self {
            header_rgb: [r, g, b],
            accent_rgb: [(r * 1.6).min(1.0), (g * 1.6).min(1.0), (b * 1.6).min(1.0)],
            body_rgb: [0.188, 0.188, 0.192],
            corner_radius: 8.0,
            min_width: 210.0,
        }
    }

    pub fn accent_rgb(mut self, r: f32, g: f32, b: f32) -> Self {
        self.accent_rgb = [r, g, b];
        self
    }

    pub fn body_rgb(mut self, r: f32, g: f32, b: f32) -> Self {
        self.body_rgb = [r, g, b];
        self
    }

    pub fn width(mut self, min_width: f32) -> Self {
        self.min_width = min_width;
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }
}

/// Context passed to a node's `show_body` method so it can mount any custom Bevy / `bevy_luma`
/// widgets directly inside the node card's body slot.
pub struct NodeBodyContext<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    slot_entity: Entity,
    layout: &'a NodeCardLayout,
    font: Handle<Font>,
}

impl<'a, 'w, 's> NodeBodyContext<'a, 'w, 's> {
    pub fn new(
        commands: &'a mut Commands<'w, 's>,
        slot_entity: Entity,
        layout: &'a NodeCardLayout,
        font: Handle<Font>,
    ) -> Self {
        Self {
            commands,
            slot_entity,
            layout,
            font,
        }
    }

    /// The `NodeId` of this node instance in the graph.
    pub fn node_id(&self) -> NodeId {
        self.layout.node_id
    }

    /// The container `Entity` inside the node card reserved for custom body widgets.
    pub fn slot_entity(&self) -> Entity {
        self.slot_entity
    }

    /// Default UI vector font handle.
    pub fn font(&self) -> Handle<Font> {
        self.font.clone()
    }

    /// Full card layout metadata.
    pub fn layout(&self) -> &NodeCardLayout {
        self.layout
    }

    /// Raw Bevy `Commands` access.
    pub fn commands(&mut self) -> &mut Commands<'w, 's> {
        self.commands
    }

    /// Spawns a child entity directly inside the node's body slot.
    pub fn spawn(&mut self, bundle: impl Bundle) -> EntityCommands<'_> {
        let mut ec = self.commands.spawn(bundle);
        ec.insert(ChildOf(self.slot_entity));
        ec
    }

    /// Attaches an already-spawned widget entity (e.g. from `spawn_text_input`) as a child of the node's body slot.
    pub fn attach(&mut self, entity: Entity) -> EntityCommands<'_> {
        let mut ec = self.commands.entity(entity);
        ec.insert(ChildOf(self.slot_entity));
        ec
    }
}

/// Unified trait for defining a node type (pins, style, evaluation, and optional body UI) in one place.
pub trait LumaNode: Send + Sync + 'static {
    fn kind(&self) -> &'static str;
    fn title(&self) -> &str;

    fn category(&self) -> (&'static str, &'static str) {
        ("Custom", "NODE")
    }

    fn style(&self) -> NodeStyle {
        NodeStyle::header_rgb(0.322, 0.212, 0.431)
    }

    fn inputs(&self) -> Vec<PinInfo> {
        Vec::new()
    }

    fn outputs(&self) -> Vec<PinInfo> {
        Vec::new()
    }

    fn body_height(&self) -> f32 {
        0.0
    }

    fn evaluate(&self, inputs: &[Value], outputs: &mut [Value]) {
        if let (Some(first_in), Some(first_out)) = (inputs.first(), outputs.first_mut()) {
            *first_out = first_in.clone();
        }
    }

    fn show_body(&self, _ctx: &mut NodeBodyContext) {}
}

type EvalCallback = Arc<dyn Fn(&[Value], &mut [Value]) + Send + Sync>;
type BodyCallback = Arc<dyn Fn(&mut NodeBodyContext) + Send + Sync>;

/// Fluent builder for defining a [`LumaNode`] inline without creating a dedicated struct.
#[derive(Clone)]
pub struct NodeDef {
    kind: &'static str,
    title: String,
    category: (&'static str, &'static str),
    style: NodeStyle,
    inputs: Vec<PinInfo>,
    outputs: Vec<PinInfo>,
    body_height: f32,
    eval_fn: Option<EvalCallback>,
    body_fn: Option<BodyCallback>,
}

pub type SnarlNode = NodeDef;

impl NodeDef {
    pub fn new(kind: &'static str, title: impl Into<String>) -> Self {
        Self {
            kind,
            title: title.into(),
            category: ("Custom", "NODE"),
            style: NodeStyle::header_rgb(0.322, 0.212, 0.431),
            inputs: Vec::new(),
            outputs: Vec::new(),
            body_height: 0.0,
            eval_fn: None,
            body_fn: None,
        }
    }

    pub fn category(mut self, category: &'static str, tag: &'static str) -> Self {
        self.category = (category, tag);
        self
    }

    pub fn style(mut self, style: NodeStyle) -> Self {
        self.style = style;
        self
    }

    pub fn header_rgb(mut self, r: f32, g: f32, b: f32) -> Self {
        self.style.header_rgb = [r, g, b];
        self.style.accent_rgb = [(r * 1.6).min(1.0), (g * 1.6).min(1.0), (b * 1.6).min(1.0)];
        self
    }

    pub fn accent_rgb(mut self, r: f32, g: f32, b: f32) -> Self {
        self.style.accent_rgb = [r, g, b];
        self
    }

    pub fn width(mut self, min_width: f32) -> Self {
        self.style.min_width = min_width;
        self
    }

    pub fn input(mut self, pin: PinInfo) -> Self {
        self.inputs.push(pin);
        self
    }

    pub fn output(mut self, pin: PinInfo) -> Self {
        self.outputs.push(pin);
        self
    }

    pub fn eval<F>(mut self, f: F) -> Self
    where
        F: Fn(&[Value], &mut [Value]) + Send + Sync + 'static,
    {
        self.eval_fn = Some(Arc::new(f));
        self
    }

    pub fn body<F>(mut self, height: f32, f: F) -> Self
    where
        F: Fn(&mut NodeBodyContext) + Send + Sync + 'static,
    {
        self.body_height = height;
        self.body_fn = Some(Arc::new(f));
        self
    }
}

impl LumaNode for NodeDef {
    fn kind(&self) -> &'static str {
        self.kind
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn category(&self) -> (&'static str, &'static str) {
        self.category
    }

    fn style(&self) -> NodeStyle {
        self.style.clone()
    }

    fn inputs(&self) -> Vec<PinInfo> {
        self.inputs.clone()
    }

    fn outputs(&self) -> Vec<PinInfo> {
        self.outputs.clone()
    }

    fn body_height(&self) -> f32 {
        self.body_height
    }

    fn evaluate(&self, inputs: &[Value], outputs: &mut [Value]) {
        if let Some(eval) = &self.eval_fn {
            eval(inputs, outputs);
        } else if let (Some(first_in), Some(first_out)) = (inputs.first(), outputs.first_mut()) {
            *first_out = first_in.clone();
        }
    }

    fn show_body(&self, ctx: &mut NodeBodyContext) {
        if let Some(body) = &self.body_fn {
            body(ctx);
        }
    }
}

#[derive(Resource, Default)]
pub struct PendingSnarlNodes {
    pub nodes: Vec<Arc<dyn LumaNode>>,
}

/// Global inventory entry for compile-time discovered nodes.
pub struct RegisteredLumaNode {
    pub constructor: fn() -> Box<dyn LumaNode>,
}

inventory::collect!(RegisteredLumaNode);

/// Global inventory entry for compile-time discovered Bevy systems attached to nodes.
pub struct RegisteredLumaSystem {
    pub register: fn(&mut App),
}

inventory::collect!(RegisteredLumaSystem);

/// Extension trait on Bevy's [`App`] to register [`LumaNode`]s in a single call.
pub trait LumaSnarlAppExt {
    fn register_node(&mut self, node: impl LumaNode) -> &mut Self;
    fn add_snarl_node(&mut self, node: impl LumaNode) -> &mut Self {
        self.register_node(node)
    }
}

impl LumaSnarlAppExt for App {
    fn register_node(&mut self, node: impl LumaNode) -> &mut Self {
        self.init_resource::<PendingSnarlNodes>();
        let arc_node: Arc<dyn LumaNode> = Arc::new(node);
        register_node_template_now(arc_node.as_ref());

        self.world_mut()
            .resource_scope::<PendingSnarlNodes, _>(|world, mut pending| {
                if world.contains_resource::<EditorDataflowResource>()
                    && world.contains_resource::<NodeBodyUiRegistry>()
                {
                    apply_node_to_world(world, arc_node.clone());
                } else {
                    pending.nodes.push(arc_node);
                }
            });
        self
    }
}

fn register_node_template_now(node: &dyn LumaNode) {
    let (cat_name, cat_tag) = node.category();
    let style = node.style();
    let body_h = node.body_height();

    let mut builder = CustomNodeBuilder::new(node.kind(), node.title())
        .category(cat_name, cat_tag)
        .accent_rgb(style.accent_rgb[0], style.accent_rgb[1], style.accent_rgb[2])
        .header_rgb(style.header_rgb[0], style.header_rgb[1], style.header_rgb[2])
        .body_rgb(style.body_rgb[0], style.body_rgb[1], style.body_rgb[2])
        .corner_radius(style.corner_radius)
        .min_width(style.min_width);

    if body_h > 0.0 {
        builder = builder.custom_widget_slot(body_h);
    }

    for pin in node.inputs() {
        builder = builder.input(pin.name, pin.data_type.to_string(), pin.shape);
    }
    for pin in node.outputs() {
        builder = builder.output(pin.name, pin.data_type.to_string(), pin.shape);
    }

    register_custom_node_template(builder.build());
}

fn apply_node_to_world(world: &mut World, arc_node: Arc<dyn LumaNode>) {
    let kind = arc_node.kind();
    let title = arc_node.title().to_string();
    let (cat_name, cat_tag) = arc_node.category();
    let style = arc_node.style();
    let inputs = arc_node.inputs();
    let outputs = arc_node.outputs();
    let body_h = arc_node.body_height();

    let eval_node = arc_node.clone();
    let mut desc = NodeDescriptor::new_sync(kind, title, cat_name, move |in_vals, out_vals| {
        eval_node.evaluate(in_vals, out_vals);
        Ok(())
    })
    .with_category(NodeCategory::new(cat_name).with_tag(cat_tag))
    .with_accent_color(LinearColor::new(
        style.accent_rgb[0],
        style.accent_rgb[1],
        style.accent_rgb[2],
        1.0,
    ));

    for pin in inputs {
        desc = desc.with_input(pin.name, pin.data_type, pin.default_value);
    }
    for pin in outputs {
        desc = desc.with_output(pin.name, pin.data_type);
    }

    if let Some(mut df_res) = world.get_resource_mut::<EditorDataflowResource>() {
        df_res.0.registry.register(desc);
    }

    if body_h > 0.0 {
        let ui_node = arc_node;
        if let Some(mut ui_reg) = world.get_resource_mut::<NodeBodyUiRegistry>() {
            ui_reg.register(kind, move |commands, slot_entity, layout, font| {
                let mut ctx = NodeBodyContext::new(commands, slot_entity, layout, font);
                ui_node.show_body(&mut ctx);
            });
        }
    }
}

/// Flushes any nodes registered before `LumaEditorPlugin`/`LumaUiPlugin` were initialized,
/// and auto-discovers all nodes submitted via `#[luma_node]` or `inventory`.
pub fn setup_snarl_infrastructure_system(world: &mut World) {
    // 1. Auto-discover all compile-time nodes registered via inventory (#[luma_node])
    for reg in inventory::iter::<RegisteredLumaNode> {
        let instance = (reg.constructor)();
        register_node_template_now(instance.as_ref());
        apply_node_to_world(world, Arc::from(instance));
    }

    // 2. Also flush any runtime-registered nodes
    let pending: Vec<Arc<dyn LumaNode>> = world
        .get_resource_mut::<PendingSnarlNodes>()
        .map(|mut p| std::mem::take(&mut p.nodes))
        .unwrap_or_default();

    for node in pending {
        apply_node_to_world(world, node);
    }
}

/// Unified `SystemParam` for spawning nodes, connecting pins, reading inputs, and pushing outputs.
#[derive(SystemParam)]
pub struct LumaGraph<'w> {
    pub graph: ResMut<'w, EditorGraphResource>,
    pub dataflow: ResMut<'w, EditorDataflowResource>,
    pub history: ResMut<'w, EditorHistoryResource>,
    pub selection: ResMut<'w, EditorSelectionResource>,
    pub id_gen: Res<'w, EditorIdGenResource>,
}

pub type LumaSnarl<'w> = LumaGraph<'w>;

impl<'w> LumaGraph<'w> {
    /// Spawns a node of `kind` at `position` on the canvas.
    pub fn spawn(&mut self, kind: &str, position: Vec2) -> NodeId {
        let palette = NodePalette::from_registry(&self.dataflow.0.registry);
        let node_id = palette
            .spawn_node(
                kind,
                position,
                &mut self.graph.0,
                &self.id_gen.0,
                &mut self.history.0,
            )
            .unwrap_or_else(|e| panic!("Failed to spawn node '{}': {:?}", kind, e));
        let _ = self
            .dataflow
            .0
            .pull_node(&self.graph.0, node_id, &HashSet::new());
        node_id
    }

    /// Connects `out_idx` of `from_node` to `in_idx` of `to_node`.
    pub fn connect(&mut self, from_node: NodeId, out_idx: usize, to_node: NodeId, in_idx: usize) {
        let out_pin = self
            .graph
            .0
            .get_node(from_node)
            .and_then(|n| n.outputs.get(out_idx).copied());
        let in_pin = self
            .graph
            .0
            .get_node(to_node)
            .and_then(|n| n.inputs.get(in_idx).copied());

        if let (Some(op), Some(ip)) = (out_pin, in_pin) {
            let _ = self
                .graph
                .0
                .connect(self.id_gen.0.next_connection_id(), op, ip);
            self.dataflow.0.push_dirty(&self.graph.0, from_node);
            let _ = self
                .dataflow
                .0
                .pull_node(&self.graph.0, to_node, &HashSet::new());
        }
    }

    /// Directly sets an output pin's value on `node_id` (e.g. from a custom interactive widget)
    /// and re-evaluates all connected downstream nodes.
    pub fn set_output(&mut self, node_id: NodeId, out_idx: usize, value: Value) {
        let num_outputs = self
            .graph
            .0
            .get_node(node_id)
            .map(|n| n.outputs.len().max(out_idx + 1))
            .unwrap_or(out_idx + 1);

        let cache_entry = self.dataflow.0.cache.get_or_create(node_id);
        if cache_entry.cached_outputs.len() < num_outputs {
            cache_entry.cached_outputs.resize(num_outputs, Value::none());
        }
        cache_entry.cached_outputs[out_idx] = value.clone();
        cache_entry.status = NodeExecutionStatus::Clean;

        let downstream = self.graph.0.downstream_dependents(node_id);
        let pinned = HashSet::new();
        for &dep_id in &downstream {
            self.dataflow.0.cache.mark_dirty(dep_id);
        }
        for dep_id in downstream {
            let _ = self.dataflow.0.pull_node(&self.graph.0, dep_id, &pinned);
            let src_cache = self.dataflow.0.cache.get_or_create(node_id);
            if src_cache.cached_outputs.len() <= out_idx {
                src_cache.cached_outputs.resize(out_idx + 1, Value::none());
            }
            src_cache.cached_outputs[out_idx] = value.clone();
            src_cache.status = NodeExecutionStatus::Clean;
        }
    }

    /// Reads the resolved incoming `Value` on `in_idx` of `node_id`.
    pub fn input_value(&self, node_id: NodeId, in_idx: usize) -> Value {
        self.dataflow
            .0
            .resolve_input_value(&self.graph.0, node_id, in_idx)
    }

    /// Reads the resolved incoming `String` on `in_idx` of `node_id`.
    pub fn input_string(&self, node_id: NodeId, in_idx: usize) -> String {
        match self.input_value(node_id, in_idx) {
            Value::Primitive(PrimitiveValue::String(s)) => s,
            _ => String::new(),
        }
    }

    /// Reads the resolved incoming `LinearColor` on `in_idx` of `node_id`.
    pub fn input_color(&self, node_id: NodeId, in_idx: usize) -> Option<LinearColor> {
        match self.input_value(node_id, in_idx) {
            Value::Primitive(PrimitiveValue::Color(c)) => Some(c),
            _ => None,
        }
    }

    /// Directly sets an output resource of type `T` on `node_id` and triggers downstream dependents.
    pub fn set_output_resource<T: Send + Sync + 'static>(
        &mut self,
        node_id: NodeId,
        out_idx: usize,
        type_name: &'static str,
        resource: T,
    ) {
        let val = Value::new_resource(
            DfDataType::custom(type_name),
            std::mem::size_of::<T>(),
            resource,
        );
        self.set_output(node_id, out_idx, val);
    }

    /// Directly sets an output color on `node_id` and triggers downstream dependents.
    pub fn set_output_color(&mut self, node_id: NodeId, out_idx: usize, color: LinearColor) {
        self.set_output(node_id, out_idx, Value::color(color));
    }

    /// Sets a float parameter value on `in_idx` of `node`.
    pub fn set_float(&mut self, node: NodeId, in_idx: usize, value: f32) {
        if let Some(pin_id) = self
            .graph
            .0
            .get_node(node)
            .and_then(|n| n.inputs.get(in_idx).copied())
        {
            self.dataflow
                .0
                .set_pin_value(&self.graph.0, pin_id, PrimitiveValue::Float(value));
            let _ = self
                .dataflow
                .0
                .pull_node(&self.graph.0, node, &HashSet::new());
        }
    }

    /// Selects `node` in the editor.
    pub fn select(&mut self, node: NodeId) {
        self.selection.0.select_node(node, false);
    }
}
