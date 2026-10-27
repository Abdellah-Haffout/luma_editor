Created At: 2026-10-18T14:58:16Z
Completed At: 2026-10-18T14:58:16Z

The command exited with code 0.
Output:
<truncated 228 lines>
/home/abdellah/Univis-project/luma_editor/src/canvas/selection.rs:36:    pub fn select_connection(&mut self, id: ConnectionId, additive: bool) {
/home/abdellah/Univis-project/luma_editor/src/canvas/selection.rs:43:    pub fn clear(&mut self) {
/home/abdellah/Univis-project/luma_editor/src/canvas/selection.rs:49:    pub fn is_node_selected(&self, id: NodeId) -> bool {
/home/abdellah/Univis-project/luma_editor/src/canvas/selection.rs:53:    pub fn is_connection_selected(&self, id: ConnectionId) -> bool {
/home/abdellah/Univis-project/luma_editor/src/canvas/selection.rs:57:    pub fn is_empty(&self) -> bool {
/home/abdellah/Univis-project/luma_editor/src/canvas/selection.rs:61:    pub fn selected_nodes_count(&self) -> usize {
/home/abdellah/Univis-project/luma_editor/src/canvas/plugin.rs:83:    pub fn animate_to(
/home/abdellah/Univis-project/luma_editor/src/canvas/plugin.rs:99:    pub fn cancel(&mut self) {
/home/abdellah/Univis-project/luma_editor/src/canvas/plugin.rs:128:pub fn canvas_pan_zoom_system(
/home/abdellah/Univis-project/luma_editor/src/canvas/plugin.rs:242:pub fn canvas_shortcuts_system(
/home/abdellah/Univis-project/luma_editor/src/canvas/transform.rs:24:    pub fn new(pan: Vec2, zoom: f32) -> Self {
/home/abdellah/Univis-project/luma_editor/src/canvas/transform.rs:33:    pub fn screen_to_canvas(&self, screen_pos: Vec2) -> Vec2 {
/home/abdellah/Univis-project/luma_editor/src/canvas/transform.rs:38:    pub fn canvas_to_screen(&self, canvas_pos: Vec2) -> Vec2 {
/home/abdellah/Univis-project/luma_editor/src/canvas/transform.rs:43:    pub fn pan_by(&mut self, screen_delta: Vec2) {
/home/abdellah/Univis-project/luma_editor/src/canvas/transform.rs:49:    pub fn zoom_at(&mut self, screen_pivot: Vec2, zoom_factor: f32) {
/home/abdellah/Univis-project/luma_editor/src/canvas/transform.rs:62:    pub fn reset(&mut self) {
/home/abdellah/Univis-project/luma_editor/src/canvas/wiring.rs:20:    pub fn is_idle(&self) -> bool {
/home/abdellah/Univis-project/luma_editor/src/canvas/wiring.rs:24:    pub fn is_active(&self) -> bool {
/home/abdellah/Univis-project/luma_editor/src/canvas/wiring.rs:31:pub fn calculate_wire_bezier(start: Vec2, end: Vec2) -> (Vec2, Vec2, Vec2, Vec2) {
/home/abdellah/Univis-project/luma_editor/src/canvas/wiring.rs:45:pub fn sample_cubic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
/home/abdellah/Univis-project/luma_editor/src/canvas/wiring.rs:60:pub fn sample_wire_spline_points(start: Vec2, end: Vec2, segments: usize) -> Vec<Vec2> {
/home/abdellah/Univis-project/luma_editor/src/scene/bsn_export.rs:9:    pub fn compile_scene(
/home/abdellah/Univis-project/luma_editor/src/scene/bsn_export.rs:185:    pub fn compile_to_file(
/home/abdellah/Univis-project/luma_editor/src/scene/auto_bake.rs:17:    pub fn save_and_bake(
/home/abdellah/Univis-project/luma_editor/src/scene/luma_file.rs:43:    pub fn new(
/home/abdellah/Univis-project/luma_editor/src/scene/luma_file.rs:81:    pub fn to_domain_store(&self) -> DomainPayloadStore {
/home/abdellah/Univis-project/luma_editor/src/scene/luma_file.rs:98:    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), String> {
/home/abdellah/Univis-project/luma_editor/src/scene/luma_file.rs:105:    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
/home/abdellah/Univis-project/luma_editor/src/ui/bevy_ui_inspector.rs:168:pub fn setup_bevy_ui_inspector(
/home/abdellah/Univis-project/luma_editor/src/ui/bevy_ui_inspector.rs:732:pub fn update_bevy_ui_inspector_system(
/home/abdellah/Univis-project/luma_editor/src/ui/bevy_ui_inspector.rs:919:pub fn bevy_ui_inspector_actions_system(
/home/abdellah/Univis-project/luma_editor/src/ui/plugin.rs:130:pub fn setup_luma_screen_ui_system(
/home/abdellah/Univis-project/luma_editor/src/ui/plugin.rs:150:pub fn update_luma_chrome_telemetry_system(
/home/abdellah/Univis-project/luma_editor/src/ui/plugin.rs:198:pub fn luma_chrome_actions_system(
/home/abdellah/Univis-project/luma_editor/src/ui/plugin.rs:258:pub fn sync_node_cards_system(
/home/abdellah/Univis-project/luma_editor/src/ui/plugin.rs:303:pub fn update_node_cards_position_system(
/home/abdellah/Univis-project/luma_editor/src/ui/plugin.rs:352:pub fn node_drag_system(
/home/abdellah/Univis-project/luma_editor/src/ui/plugin.rs:535:pub fn delete_selected_system(
/home/abdellah/Univis-project/luma_editor/src/ui/plugin.rs:556:pub fn ui_focus_navigation_system(
/home/abdellah/Univis-project/luma_editor/src/ui/plugin.rs:630:pub fn subgraph_grouping_shortcuts_system(
/home/abdellah/Univis-project/luma_editor/src/ui/plugin.rs:700:pub fn node_param_step_system(
/home/abdellah/Univis-project/luma_editor/src/ui/plugin.rs:751:pub fn update_node_io_values_system(
/home/abdellah/Univis-project/luma_editor/src/ui/bevy_ui_hud.rs:66:pub fn setup_bevy_ui_hud(
/home/abdellah/Univis-project/luma_editor/src/ui/bevy_ui_hud.rs:545:pub fn bevy_ui_toast_system(
/home/abdellah/Univis-project/luma_editor/src/ui/bevy_ui_hud.rs:560:pub fn bevy_ui_quick_spawn_interaction_system(
/home/abdellah/Univis-project/luma_editor/src/ui/bevy_ui_hud.rs:630:pub fn toggle_bevy_ui_quick_spawn_system(
/home/abdellah/Univis-project/luma_editor/src/ui/bevy_ui_hud.rs:679:pub fn bevy_ui_floating_toolbar_system(
/home/abdellah/Univis-project/luma_editor/src/ui/components/status_bar.rs:19:pub fn status_bar_scene(
/home/abdellah/Univis-project/luma_editor/src/ui/components/focus_toolbar.rs:14:pub fn focus_toolbar_dock_scene(
/home/abdellah/Univis-project/luma_editor/src/ui/components/node_card.rs:65:pub fn get_blender_pin_color(type_name: &str) -> Color {
/home/abdellah/Univis-project/luma_editor/src/ui/components/node_card.rs:91:pub fn node_card_scene(
/home/abdellah/Univis-project/luma_editor/src/ui/components/node_card.rs:971:pub fn update_node_socket_pins_system(
/home/abdellah/Univis-project/luma_editor/src/ui/components/header.rs:27:pub fn top_navbar_scene(
/home/abdellah/Univis-project/luma_editor/src/ui/menu_bar.rs:30:    pub fn label(&self) -> &'static str {
/home/abdellah/Univis-project/luma_editor/src/ui/menu_bar.rs:86:pub fn spawn_menu_triggers(
/home/abdellah/Univis-project/luma_editor/src/ui/menu_bar.rs:127:pub fn setup_bevy_ui_menu_dropdowns(
/home/abdellah/Univis-project/luma_editor/src/ui/menu_bar.rs:267:pub fn bevy_ui_menu_toggle_system(
/home/abdellah/Univis-project/luma_editor/src/ui/menu_bar.rs:298:pub fn bevy_ui_menu_visibility_system(
/home/abdellah/Univis-project/luma_editor/src/ui/menu_bar.rs:316:pub fn bevy_ui_menu_item_hover_system(
/home/abdellah/Univis-project/luma_editor/src/ui/menu_bar.rs:332:pub fn bevy_ui_menu_trigger_hover_system() {
/home/abdellah/Univis-project/luma_editor/src/ui/menu_bar.rs:337:pub fn bevy_ui_menu_dismiss_system(
/home/abdellah/Univis-project/luma_editor/src/ui/menu_bar.rs:358:pub fn bevy_ui_menu_actions_system(
/home/abdellah/Univis-project/luma_editor/src/ui/bevy_ui_chrome.rs:58:pub fn setup_bevy_ui_chrome(
/home/abdellah/Univis-project/luma_editor/src/ui/bevy_ui_chrome.rs:526:pub fn update_bevy_ui_chrome_telemetry_system(
/home/abdellah/Univis-project/luma_editor/src/ui/bevy_ui_chrome.rs:580:pub fn bevy_ui_chrome_actions_system(
/home/abdellah/Univis-project/luma_editor/src/ui/bevy_ui_chrome.rs:665:pub fn bevy_ui_chrome_hover_system(
/home/abdellah/Univis-project/luma_editor/src/ui/bevy_ui_chrome.rs:690:pub fn ensure_ui_camera_target_system(
/home/abdellah/Univis-project/luma_editor/src/ui/wires.rs:18:pub fn compute_pin_world_position(node: &crate::core::graph::Node, pin_id: PinId) -> Option<Vec2> {
/home/abdellah/Univis-project/luma_editor/src/ui/wires.rs:44:pub fn pin_world_position(
/home/abdellah/Univis-project/luma_editor/src/ui/wires.rs:101:pub fn get_pin_type_info(type_name: &str) -> (Color, PortShape) {
/home/abdellah/Univis-project/luma_editor/src/ui/wires.rs:513:pub fn draw_canvas_wires_system(
/home/abdellah/Univis-project/luma_editor/src/ui/wires.rs:846:pub fn interactive_pin_wiring_system(
/home/abdellah/Univis-project/luma_editor/src/ui/layout.rs:16:pub fn editor_root_scene(

