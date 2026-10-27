Created At: 2026-10-18T17:22:41Z
Completed At: 2026-10-18T17:22:41Z
File Path: `file:///home/abdellah/Univis-project/luma_editor/crates/luma_ui/src/components/node_card.rs`
Total Lines: 425
Total Bytes: 18328
Showing lines 1 to 425
The following code has been modified to include a line number before every line, in the format: <line_number>: <original_line>. Please note that any changes targeting the original code should remove the line number, colon, and leading space.
1: use luma_core::id::{NodeId, PinId};
2: use luma_dataflow::descriptor::NodeBodyStyle;
3: use luma_canvas::nodes::card::NodeCardLayout;
4: use bevy::prelude::*;
5: 
6: #[derive(Component, Clone, Copy, PartialEq, Debug, Default)]
7: pub struct NodeCardVisual {
8:     pub node_id: NodeId,
9:     pub type_color: Color,
10: }
11: 
12: #[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
13: pub struct NodeCardRoot {
14:     pub node_id: NodeId,
15: }
16: 
17: #[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
18: pub struct DiveInButton {
19:     pub node_id: NodeId,
20: }
21: 
22: /// Interactive value display label for parameter nodes (e.g. math.number).
23: #[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
24: pub struct NodeParamValueText {
25:     pub node_id: NodeId,
26: }
27: 
28: /// Interactive step button (- / +) for adjusting parameter values.
29: #[derive(Component, Clone, Copy, PartialEq, Default)]
30: pub struct NodeParamStepButton {
31:     pub node_id: NodeId,
32:     pub delta: f32,
33: }
34: 
35: /// Slot marker for injecting custom node UI widgets into a card's body.
36: #[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
37: pub struct NodeCustomBodySlot {
38:     pub node_id: NodeId,
39: }
40: 
41: /// Marker component for an input slot text label on a node card.
42: #[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
43: pub struct NodeInputValLabel {
44:     pub node_id: NodeId,
45:     pub slot_idx: usize,
46: }
47: 
48: /// Marker component for an output slot text label on a node card.
49: #[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
50: pub struct NodeOutputValLabel {
51:     pub node_id: NodeId,
52:     pub slot_idx: usize,
53: }
54: 
55: /// Marker component for an interactive pin socket circle on a node card.
56: #[derive(Component, Clone, Copy, PartialEq, Eq, Default)]
57: pub struct NodeCardSocketPin {
58:     pub node_id: NodeId,
59:     pub pin_id: Option<PinId>,
60: }
61: 
62: /// Maps data type name to authentic Blender socket type colors.
63: pub fn get_blender_pin_color(type_name: &str) -> Color {
64:     match type_name {
65:         "Float" | "f32" | "Number" => Color::srgb(0.65, 0.65, 0.65),    // Silver / Gray (Float)
66:         "Int" | "Integer" => Color::srgb(0.35, 0.68, 0.78),             // Cyan (Integer)
67:         "Vec2" | "Vec3" | "Vec4" | "Vector" | "Float3" => Color::srgb(0.39, 0.39, 0.78), // Blue/Indigo (Vector)
68:         "Color" | "RGBA" => Color::srgb(0.78, 0.78, 0.16),              // Yellow (Color)
69:         "String" => Color::srgb(0.55, 0.75, 0.95),                      // Light Blue (String)
70:         "Shader" | "Material" => Color::srgb(0.39, 0.78, 0.39),         // Green (Shader/Material)
71:         "Geometry" | "Mesh" => Color::srgb(0.0, 0.84, 0.64),            // Emerald/Teal (Geometry)
72:         "Entity" | "Object" => Color::srgb(0.90, 0.54, 0.18),           // Orange (Object/Entity)
73:         "Asset" | "AssetRef" | "Model" => Color::srgb(1.0, 0.69, 0.12), // Amber (Asset)
74:         "Bool" | "Boolean" => Color::srgb(0.85, 0.40, 0.60),            // Pink (Boolean)
75:         _ => Color::srgb(0.80, 0.80, 0.80),                             // Untyped / Any
76:     }
77: }
78: 
79: /// Spawns a Blender-styled node card using pure bevy_ui with crisp borders,
80: /// colored header strip, input/output socket pins, and parameter steppers.
81: pub fn spawn_bevy_ui_node_card(
82:     commands: &mut Commands,
83:     layout: &NodeCardLayout,
84:     is_selected: bool,
85:     target_camera: Option<Entity>,
86: ) -> Entity {
87:     let mut root_cmd = commands.spawn((
88:         NodeCardRoot { node_id: layout.node_id },
89:         NodeCardVisual {
90:             node_id: layout.node_id,
91:             type_color: layout.header_bg,
92:         },
93:         Node {
94:             position_type: PositionType::Absolute,
95:             left: Val::Px(0.0),
96:             top: Val::Px(0.0),
97:             width: Val::Px(layout.size.x),
98:             height: Val::Px(layout.size.y),
99:             flex_direction: FlexDirection::Column,
100:             border: UiRect::all(Val::Px(if is_selected { 2.0 } else { 1.0 })),
101:             border_radius: BorderRadius::all(Val::Px(8.0)),
102:             overflow: Overflow::visible(),
103:             ..default()
104:         },
105:         BackgroundColor(Color::srgb(0.133, 0.141, 0.157)),
106:         BorderColor::all(if is_selected {
107:             layout.header_bg
108:         } else {
109:             Color::srgba(0.086, 0.090, 0.102, 0.95)
110:         }),
111:         GlobalZIndex(20),
112:     ));
113: 
114:     if let Some(cam) = target_camera {
115:         root_cmd.insert(UiTargetCamera(cam));
116:     }
117: 
118:     root_cmd.with_children(|card| {
119:         // A. Header Bar (32px)
120:         card.spawn((
121:             Node {
122:                 width: Val::Percent(100.0),
123:                 height: Val::Px(32.0),
124:                 flex_direction: FlexDirection::Row,
125:                 align_items: AlignItems::Center,
126:                 justify_content: JustifyContent::SpaceBetween,
127:                 padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
128:                 border_radius: BorderRadius::top(Val::Px(7.0)),
129:                 ..default()
130:             },
131:             BackgroundColor(layout.header_bg),
132:         )).with_children(|header| {
133:             // Left: Category badge + Title
134:             header.spawn(Node {
135:                 flex_direction: FlexDirection::Row,
136:                 align_items: AlignItems::Center,
137:                 column_gap: Val::Px(6.0),
138:                 ..default()
139:             }).with_children(|left| {
140:                 // Category Badge Pill
141:                 left.spawn((
142:                     Node {
143:                         padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
144:                         border_radius: BorderRadius::all(Val::Px(4.0)),
145:                         ..default()
146:                     },
147:                     BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.25)),
148:                 )).with_children(|badge| {
149:                     badge.spawn((
150:                         Text::new(&layout.category_tag),
151:                         TextFont { font_size: FontSize::Px(9.0), ..default() },
152:                         TextColor(Color::WHITE),
153:                     ));
154:                 });
155: 
156:                 // Node Title
157:                 left.spawn((
158:                     Text::new(&layout.title),
159:                     TextFont { font_size: FontSize::Px(12.0), ..default() },
160:                     TextColor(Color::WHITE),
161:                 ));
162:             });
163: 
164:             // Right (if Subgraph): Dive In Button
165:             if layout.kind == "Subgraph" {
166:                 header.spawn((
167:                     DiveInButton { node_id: layout.node_id },
168:                     Button,
169:                     Node {
170:                         padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
171:                         border_radius: BorderRadius::all(Val::Px(4.0)),
172:                         ..default()
173:                     },
174:                     BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
175:                 )).with_children(|btn| {
176:                     btn.spawn((
177:                         Text::new("[Dive In] >"),
178:                         TextFont { font_size: FontSize::Px(9.5), ..default() },
179:                         TextColor(Color::WHITE),
180:                     ));
181:                 });
182:             }
183:         });
184: 
185:         // B. Body Content (Slots & Pins)
186:         card.spawn(Node {
187:             width: Val::Percent(100.0),
188:             flex_grow: 1.0,
189:             flex_direction: FlexDirection::Column,
190:             padding: UiRect::axes(Val::Px(4.0), Val::Px(6.0)),
191:             row_gap: Val::Px(4.0),
192:             ..default()
193:         }).with_children(|body| {
194:             let num_rows = layout.input_pins.len().max(layout.output_pins.len()).max(
195:                 if layout.body_style == NodeBodyStyle::Parameters { 1 } else { 0 }
196:             );
197: 
198:             for row_idx in 0..num_rows {
199:                 body.spawn(Node {
200:                     width: Val::Percent(100.0),
201:                     height: Val::Px(24.0),
202:                     flex_direction: FlexDirection::Row,
203:                     align_items: AlignItems::Center,
204:                     justify_content: JustifyContent::SpaceBetween,
205:                     position_type: PositionType::Relative,
206:                     ..default()
207:                 }).with_children(|row| {
208:                     // 1. Left: Input Pin Dot + Label
209:                     if let Some(in_pin) = layout.input_pins.get(row_idx) {
210:                         let is_connected = layout.is_pin_connected(in_pin.id);
211:                         let color = get_blender_pin_color(&in_pin.data_type.name);
212: 
213:                         row.spawn(Node {
214:                             flex_direction: FlexDirection::Row,
215:                             align_items: AlignItems::Center,
216:                             column_gap: Val::Px(6.0),
217:                             margin: UiRect::left(Val::Px(-9.0)),
218:                             ..default()
219:                         }).with_children(|in_group| {
220:                             // Socket dot
221:                             in_group.spawn((
222:                                 NodeCardSocketPin {
223:                                     node_id: layout.node_id,
224:                                     pin_id: Some(in_pin.id),
225:                                 },
226:                                 Node {
227:                                     width: Val::Px(10.0),
228:                                     height: Val::Px(10.0),
229:                                     border: UiRect::all(Val::Px(if is_connected { 1.0 } else { 2.0 })),
230:                                     border_radius: BorderRadius::all(Val::Px(5.0)),
231:                                     ..default()
232:                                 },
233:                                 BackgroundColor(if is_connected { color } else { Color::srgb(0.133, 0.141, 0.157) }),
234:                                 BorderColor::all(color),
235:                             ));
236: 
237:                             // Label
238:                             let text_val = layout.input_display_texts.get(row_idx).cloned().unwrap_or_else(|| in_pin.name.clone());
239:                             in_group.spawn((
240:                                 NodeInputValLabel {
241:                                     node_id: layout.node_id,
242:                                     slot_idx: row_idx,
243:                                 },
244:                                 Text::new(text_val),
245:                                 TextFont { font_size: FontSize::Px(10.5), ..default() },
246:                                 TextColor(Color::srgb(0.85, 0.88, 0.92)),
247:                             ));
248:                         });
249:                     } else {
250:                         row.spawn(Node::default());
251:                     }
252: 
253:                     // 2. Center: Parameter Stepper [-] [Value] [+]
254:                     if layout.body_style == NodeBodyStyle::Parameters && row_idx == 0 {
255:                         row.spawn(Node {
256:                             flex_direction: FlexDirection::Row,
257:                             align_items: AlignItems::Center,
258:                             column_gap: Val::Px(4.0),
259:                             padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
260:                             border_radius: BorderRadius::all(Val::Px(4.0)),
261:                             ..default()
262:                         }).with_children(|stepper| {
263:                             // [-] button
264:                             stepper.spawn((
265:                                 NodeParamStepButton {
266:                                     node_id: layout.node_id,
267:                                     delta: -1.0,
268:                                 },
269:                                 Button,
270:                                 Node {
271:                                     width: Val::Px(16.0),
272:                                     height: Val::Px(16.0),
273:                                     justify_content: JustifyContent::Center,
274:                                     align_items: AlignItems::Center,
275:                                     border_radius: BorderRadius::all(Val::Px(3.0)),
276:                                     ..default()
277:                                 },
278:                                 BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
279:                             )).with_children(|b| {
280:                                 b.spawn((
281:                                     Text::new("-"),
282:                                     TextFont { font_size: FontSize::Px(11.0), ..default() },
283:                                     TextColor(Color::WHITE),
284:                                 ));
285:                             });
286: 
287:                             // Value text
288:                             stepper.spawn((
289:                                 NodeParamValueText {
290:                                     node_id: layout.node_id,
291:                                 },
292:                                 Text::new("10.0"),
293:                                 TextFont { font_size: FontSize::Px(10.5), ..default() },
294:                                 TextColor(Color::srgb(0.95, 0.85, 0.40)),
295:                             ));
296: 
297:                             // [+] button
298:                             stepper.spawn((
299:                                 NodeParamStepButton {
300:                                     node_id: layout.node_id,
301:                                     delta: 1.0,
302:                                 },
303:                                 Button,
304:                                 Node {
305:                                     width: Val::Px(16.0),
306:                                     height: Val::Px(16.0),
307:                                     justify_content: JustifyContent::Center,
308:                                     align_items: AlignItems::Center,
309:                                     border_radius: BorderRadius::all(Val::Px(3.0)),
310:                                     ..default()
311:                                 },
312:                                 BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
313:                             )).with_children(|b| {
314:                                 b.spawn((
315:                                     Text::new("+"),
316:                                     TextFont { font_size: FontSize::Px(11.0), ..default() },
317:                                     TextColor(Color::WHITE),
318:                                 ));
319:                             });
320:                         });
321:                     }
322: 
323:                     // 3. Right: Output Label + Pin Dot
324:                     if let Some(out_pin) = layout.output_pins.get(row_idx) {
325:                         let is_connected = layout.is_pin_connected(out_pin.id);
326:                         let color = get_blender_pin_color(&out_pin.data_type.name);
327: 
328:                         row.spawn(Node {
329:                             flex_direction: FlexDirection::Row,
330:                             align_items: AlignItems::Center,
331:                             column_gap: Val::Px(6.0),
332:                             margin: UiRect::right(Val::Px(-9.0)),
333:                             ..default()
334:                         }).with_children(|out_group| {
335:                             // Label
336:                             let text_val = layout.output_display_texts.get(row_idx).cloned().unwrap_or_else(|| out_pin.name.clone());
337:                             out_group.spawn((
338:                                 NodeOutputValLabel {
339:                                     node_id: layout.node_id,
340:                                     slot_idx: row_idx,
341:                                 },
342:                                 Text::new(text_val),
343:                                 TextFont { font_size: FontSize::Px(10.5), ..default() },
344:                                 TextColor(Color::srgb(0.85, 0.88, 0.92)),
345:                             ));
346: 
347:                             // Socket dot
348:                             out_group.spawn((
349:                                 NodeCardSocketPin {
350:                                     node_id: layout.node_id,
351:                                     pin_id: Some(out_pin.id),
352:                                 },
353:                                 Node {
354:                                     width: Val::Px(10.0),
355:                                     height: Val::Px(10.0),
356:                                     border: UiRect::all(Val::Px(if is_connected { 1.0 } else { 2.0 })),
357:                                     border_radius: BorderRadius::all(Val::Px(5.0)),
358:                                     ..default()
359:                                 },
360:                                 BackgroundColor(if is_connected { color } else { Color::srgb(0.133, 0.141, 0.157) }),
361:                                 BorderColor::all(color),
362:                             ));
363:                         });
364:                     } else {
365:                         row.spawn(Node::default());
366:                     }
367:                 });
368:             }
369:         });
370:     });
371: 
372:     root_cmd.id()
373: }
374: 
375: /// Compatibility builder bundle for test suites.
376: pub fn node_card_scene(
377:     _font: Handle<Font>,
378:     layout: &NodeCardLayout,
379:     is_selected: bool,
380: ) -> impl Bundle {
381:     (
382:         NodeCardRoot { node_id: layout.node_id },
383:         NodeCardVisual {
384:             node_id: layout.node_id,
385:             type_color: layout.header_bg,
386:         },
387:         Node {
388:             position_type: PositionType::Absolute,
389:             width: Val::Px(layout.size.x),
390:             height: Val::Px(layout.size.y),
391:             border: UiRect::all(Val::Px(if is_selected { 2.0 } else { 1.0 })),
392:             border_radius: BorderRadius::all(Val::Px(8.0)),
393:             ..default()
394:         },
395:         BackgroundColor(Color::srgb(0.133, 0.141, 0.157)),
396:     )
397: }
398: 
399: /// Continuously synchronizes the visual fill and border of socket dots
400: /// when wires are connected or disconnected in the editor graph.
401: pub fn update_node_socket_pins_system(
402:     graph_res: Res<luma_canvas::plugin::EditorGraphResource>,
403:     mut socket_query: Query<(&NodeCardSocketPin, &mut BackgroundColor, &mut BorderColor)>,
404: ) {
405:     if !graph_res.is_changed() {
406:         return;
407:     }
408:     let graph = &graph_res.0;
409:     for (socket, mut bg, mut border) in &mut socket_query {
410:         if let Some(pin_id) = socket.pin_id {
411:             if let Some(pin) = graph.get_pin(pin_id) {
412:                 let is_connected = !graph.get_pin_connections(pin_id).is_empty();
413:                 let color = get_blender_pin_color(&pin.data_type.name);
414:                 if is_connected {
415:                     *bg = BackgroundColor(color);
416:                     *border = BorderColor::all(color);
417:                 } else {
418:                     *bg = BackgroundColor(Color::srgb(0.133, 0.141, 0.157));
419:                     *border = BorderColor::all(color);
420:                 }
421:             }
422:         }
423:     }
424: }
425: 
The above content shows the entire, complete file contents of the requested file.
