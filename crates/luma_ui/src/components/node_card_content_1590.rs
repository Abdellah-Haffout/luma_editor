Created At: 2026-10-18T16:31:03Z
Completed At: 2026-10-18T16:31:03Z
File Path: `file:///home/abdellah/Univis-project/luma_editor/crates/luma_ui/src/components/node_card.rs`
Total Lines: 997
Total Bytes: 47145
Showing lines 80 to 160
The following code has been modified to include a line number before every line, in the format: <line_number>: <original_line>. Please note that any changes targeting the original code should remove the line number, colon, and leading space.
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
113:     let get_pin_meta = |pins: &[luma_core::graph::Pin], idx: usize| -> (bool, Option<PinId>, Color, bool) {
114:         if let Some(pin) = pins.get(idx) {
115:             let is_connected = layout.is_pin_connected(pin.id);
116:             let color = get_blender_pin_color(&pin.data_type.name);
117:             (true, Some(pin.id), color, is_connected)
118:         } else {
119:             (false, None, Color::NONE, false)
120:         }
121:     };
122: 
123:     let in_0 = layout.input_display_texts.first().cloned().unwrap_or_default();
124:     let out_0 = layout.output_display_texts.first().cloned().unwrap_or_default();
125:     let (has_in_0, in_pin_0, in_col_0, in_conn_0) = get_pin_meta(&layout.input_pins, 0);
126:     let (has_out_0, out_pin_0, out_col_0, out_conn_0) = get_pin_meta(&layout.output_pins, 0);
127: 
128:     let in_0_sz = if has_in_0 { 10.0 } else { 0.0 };
129:     let in_0_margin = if has_in_0 { USides { left: -5.0, right: 0.0, top: 0.0, bottom: 0.0 } } else { USides::default() };
130:     let in_0_bg = if !has_in_0 { Color::NONE } else if in_conn_0 { in_col_0 } else { Color::srgb(0.133, 0.141, 0.157) };
131:     let in_0_bcol = if has_in_0 { in_col_0 } else { Color::NONE };
132:     let in_0_bw = if !has_in_0 { 0.0 } else if in_conn_0 { 1.0 } else { 2.0 };
133: 
134:     let out_0_sz = if has_out_0 { 10.0 } else { 0.0 };
135:     let out_0_margin = if has_out_0 { USides { left: 0.0, right: -5.0, top: 0.0, bottom: 0.0 } } else { USides::default() };
136:     let out_0_bg = if !has_out_0 { Color::NONE } else if out_conn_0 { out_col_0 } else { Color::srgb(0.133, 0.141, 0.157) };
137:     let out_0_bcol = if has_out_0 { out_col_0 } else { Color::NONE };
138:     let out_0_bw = if !has_out_0 { 0.0 } else if out_conn_0 { 1.0 } else { 2.0 };
139: 
140:     let in_1 = layout.input_display_texts.get(1).cloned().unwrap_or_default();
141:     let out_1 = layout.output_display_texts.get(1).cloned().unwrap_or_default();
142:     let (has_in_1, in_pin_1, in_col_1, in_conn_1) = get_pin_meta(&layout.input_pins, 1);
143:     let (has_out_1, out_pin_1, out_col_1, out_conn_1) = get_pin_meta(&layout.output_pins, 1);
144:     let has_row_1 = has_in_1 || has_out_1;
145:     let row_1_height = if has_row_1 { row_height } else { 0.0 };
146: 
147:     let in_1_sz = if has_in_1 { 10.0 } else { 0.0 };
148:     let in_1_margin = if has_in_1 { USides { left: -5.0, right: 0.0, top: 0.0, bottom: 0.0 } } else { USides::default() };
149:     let in_1_bg = if !has_in_1 { Color::NONE } else if in_conn_1 { in_col_1 } else { Color::srgb(0.133, 0.141, 0.157) };
150:     let in_1_bcol = if has_in_1 { in_col_1 } else { Color::NONE };
151:     let in_1_bw = if !has_in_1 { 0.0 } else if in_conn_1 { 1.0 } else { 2.0 };
152: 
153:     let out_1_sz = if has_out_1 { 10.0 } else { 0.0 };
154:     let out_1_margin = if has_out_1 { USides { left: 0.0, right: -5.0, top: 0.0, bottom: 0.0 } } else { USides::default() };
155:     let out_1_bg = if !has_out_1 { Color::NONE } else if out_conn_1 { out_col_1 } else { Color::srgb(0.133, 0.141, 0.157) };
156:     let out_1_bcol = if has_out_1 { out_col_1 } else { Color::NONE };
157:     let out_1_bw = if !has_out_1 { 0.0 } else if out_conn_1 { 1.0 } else { 2.0 };
158: 
159:     let in_2 = layout.input_display_texts.get(2).cloned().unwrap_or_default();
160:     let out_2 = layout.output_display_texts.get(2).cloned().unwrap_or_default();
The above content does NOT show the entire file contents. If you need to view any lines of the file which were not shown to complete your task, call this tool again to view those lines.
