Created At: 2026-10-18T16:53:04Z
Completed At: 2026-10-18T16:53:04Z
File Path: `file:///home/abdellah/Univis-project/luma_editor/crates/luma_ui/src/components/node_card.rs`
Total Lines: 426
Total Bytes: 18371
Showing lines 375 to 400
The following code has been modified to include a line number before every line, in the format: <line_number>: <original_line>. Please note that any changes targeting the original code should remove the line number, colon, and leading space.
375: 
376: /// Compatibility builder bundle for test suites.
377: pub fn node_card_scene(
378:     _font: Handle<Font>,
379:     layout: &NodeCardLayout,
380:     is_selected: bool,
381: ) -> impl Bundle {
382:     (
383:         NodeCardRoot { node_id: layout.node_id },
384:         NodeCardVisual {
385:             node_id: layout.node_id,
386:             type_color: layout.header_bg,
387:         },
388:         Node {
389:             position_type: PositionType::Absolute,
390:             width: Val::Px(layout.size.x),
391:             height: Val::Px(layout.size.y),
392:             border: UiRect::all(Val::Px(if is_selected { 2.0 } else { 1.0 })),
393:             border_radius: BorderRadius::all(Val::Px(8.0)),
394:             ..default()
395:         },
396:         BackgroundColor(Color::srgb(0.133, 0.141, 0.157)),
397:     )
398: }
399: 
400: /// Continuously synchronizes the visual fill and border of socket dots
The above content does NOT show the entire file contents. If you need to view any lines of the file which were not shown to complete your task, call this tool again to view those lines.
