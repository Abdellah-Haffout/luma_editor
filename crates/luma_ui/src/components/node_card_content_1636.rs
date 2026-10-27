Created At: 2026-10-18T16:37:12Z
Completed At: 2026-10-18T16:37:12Z
File Path: `file:///home/abdellah/Univis-project/luma_editor/crates/luma_ui/src/components/node_card.rs`
Total Lines: 997
Total Bytes: 47145
Showing lines 1 to 100
The following code has been modified to include a line number before every line, in the format: <line_number>: <original_line>. Please note that any changes targeting the original code should remove the line number, colon, and leading space.
1: use luma_core::id::{NodeId, PinId};
2: use luma_dataflow::descriptor::NodeBodyStyle;
3: use luma_canvas::nodes::card::NodeCardLayout;
4: use crate::theme::EditorTheme;
5: use bevy::prelude::*;
6: use bevy_luma::prelude::*;
7: 
8: #[derive(Component, Clone, Copy, PartialEq, Debug, Default)]
9: pub struct NodeCardVisual {
10:     pub node_id: NodeId,
11:     pub type_color: Color,
12: }
13: 
14: #[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
15: pub struct NodeCardRoot {
16:     pub node_id: NodeId,
17: }
18: 
19: #[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
20: pub struct DiveInButton {
21:     pub node_id: NodeId,
22: }
23: 
24: /// Interactive value display label for parameter nodes (e.g. math.number).
25: #[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
26: pub struct NodeParamValueText {
27:     pub node_id: NodeId,
28: }
29: 
30: /// Interactive step button (- / +) for adjusting parameter values.
31: #[derive(Component, Clone, Copy, PartialEq, Default)]
32: pub struct NodeParamStepButton {
33:     pub node_id: NodeId,
34:     pub delta: f32,
35: }
36: 
37: /// Slot marker for injecting custom node UI widgets into a card's body.
38: #[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
39: pub struct NodeCustomBodySlot {
40:     pub node_id: NodeId,
41: }
42: 
43: /// Marker component for an input slot text label on a node card.
44: #[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
45: pub struct NodeInputValLabel {
46:     pub node_id: NodeId,
47:     pub slot_idx: usize,
48: }
49: 
50: /// Marker component for an output slot text label on a node card.
51: #[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
52: pub struct NodeOutputValLabel {
53:     pub node_id: NodeId,
54:     pub slot_idx: usize,
55: }
56: 
57: /// Marker component for an interactive pin socket circle on a node card.
58: #[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
59: pub struct NodeCardSocketPin {
60:     pub node_id: NodeId,
61:     pub pin_id: Option<PinId>,
62: }
63: 
64: /// Maps data type name to authentic Blender socket type colors.
65: pub fn get_blender_pin_color(type_name: &str) -> Color {
66:     match type_name {
67:         "Float" | "f32" | "Number" => Color::srgb(0.65, 0.65, 0.65),    // Silver / Gray (Float)
68:         "Int" | "Integer" => Color::srgb(0.35, 0.68, 0.78),             // Cyan (Integer)
69:         "Vec2" | "Vec3" | "Vec4" | "Vector" | "Float3" => Color::srgb(0.39, 0.39, 0.78), // Blue/Indigo (Vector)
70:         "Color" | "RGBA" => Color::srgb(0.78, 0.78, 0.16),              // Yellow (Color)
71:         "String" => Color::srgb(0.55, 0.75, 0.95),                      // Light Blue (String)
72:         "Shader" | "Material" => Color::srgb(0.39, 0.78, 0.39),         // Green (Shader/Material)
73:         "Geometry" | "Mesh" => Color::srgb(0.0, 0.84, 0.64),            // Emerald/Teal (Geometry)
74:         "Entity" | "Object" => Color::srgb(0.90, 0.54, 0.18),           // Orange (Object/Entity)
75:         "Asset" | "AssetRef" | "Model" => Color::srgb(1.0, 0.69, 0.12), // Amber (Asset)
76:         "Bool" | "Boolean" => Color::srgb(0.85, 0.40, 0.60),            // Pink (Boolean)
77:         _ => Color::srgb(0.80, 0.80, 0.80),                             // Untyped / Any
78:     }
79: }
80: 
81: /// Spawns a physical in-world 2D node card (`URootUi::world_2d`) using BSN with Blender aesthetics.
82: /// 
83: /// Refinements:
84: /// - Pure Blender aesthetic: sleek dark charcoal container, rounded 8px corners.
85: /// - Rounded colored header with category tint, semibold white title, and indicator dot.
86: /// - Ports/Sockets implemented 100% in bevy_luma (ZERO Gizmos):
87: ///   - 10px circular dots positioned directly on card boundaries (half inside, half outside).
88: ///   - Solid fill in type color when connected.
89: ///   - 2px colored ring with dark card interior when unconnected.
90: /// - Compact 3-column layout (Inputs | Center Parameter Slot | Outputs).
91: pub fn node_card_scene(
92:     font: Handle<Font>,
93:     layout: &NodeCardLayout,
94:     _is_selected: bool,
95: ) -> impl Scene {
96:     let node_id = layout.node_id;
97:     let title = layout.title.clone();
98:     let is_subgraph = layout.kind == "Subgraph";
99: 
100:     let width_px = layout.size.x;
The above content does NOT show the entire file contents. If you need to view any lines of the file which were not shown to complete your task, call this tool again to view those lines.
