Created At: 2026-10-18T20:18:46Z
Completed At: 2026-10-18T20:18:46Z

The command exited with code 0.
Output:
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


