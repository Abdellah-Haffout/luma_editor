Created At: 2026-10-18T15:15:46Z
Completed At: 2026-10-18T15:15:46Z
File Path: `file:///home/abdellah/Univis-project/luma_editor/src/ui/components/node_card.rs`
Total Lines: 997
Total Bytes: 47151
Showing lines 75 to 120
The following code has been modified to include a line number before every line, in the format: <line_number>: <original_line>. Please note that any changes targeting the original code should remove the line number, colon, and leading space.
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
101:     let height_px = layout.size.y;
102:     let corner_radius = 8.0;
103:     let header_height = 32.0;
104:     let row_height = 26.0;
105: 
106:     let header_bg = layout.header_bg;
107:     let kind_badge = layout.category_tag.clone();
108: 
109:     let pos_x = layout.position.x;
110:     let pos_y = layout.position.y;
111: 
112:     // Helper to extract pin meta (existence, pin_id, color, connection status)
113:     let get_pin_meta = |pins: &[crate::core::graph::Pin], idx: usize| -> (bool, Option<PinId>, Color, bool) {
114:         if let Some(pin) = pins.get(idx) {
115:             let is_connected = layout.is_pin_connected(pin.id);
116:             let color = get_blender_pin_color(&pin.data_type.name);
117:             (true, Some(pin.id), color, is_connected)
118:         } else {
119:             (false, None, Color::NONE, false)
120:         }
The above content does NOT show the entire file contents. If you need to view any lines of the file which were not shown to complete your task, call this tool again to view those lines.
