use crate::selection::SelectionState;
use crate::transform::CanvasTransform;
use crate::wiring::WiringState;
use luma_commands::descriptor::CommandRegistry;
use luma_commands::dispatcher::{CommandContext, CommandDispatcher};
use luma_commands::keybindings::{EditorKey, KeyChord, KeybindingMap};
use luma_core::graph::GraphStore;
use luma_core::history::HistoryStack;
use luma_core::id::IdGenerator;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct EditorGraphResource(pub GraphStore);

#[derive(Resource)]
pub struct EditorHistoryResource(pub HistoryStack);

impl Default for EditorHistoryResource {
    fn default() -> Self {
        Self(HistoryStack::default())
    }
}

#[derive(Resource, Default)]
pub struct EditorIdGenResource(pub IdGenerator);

#[derive(Resource, Default)]
pub struct EditorTransformResource(pub CanvasTransform);

#[derive(Resource, Default)]
pub struct EditorSelectionResource(pub SelectionState);

#[derive(Resource, Default)]
pub struct EditorWiringResource(pub WiringState);

/// Guards the node graph canvas from reacting to mouse or keyboard events
/// when the cursor is over any UI panel/chrome/menu or outside the window,
/// or when a UI modal/popup has active focus.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct EditorCanvasInputGuard {
    pub is_blocked: bool,
}

#[derive(Resource, Default, Clone)]
pub struct EditorDomainPayloadResource(pub luma_domain::evaluation::DomainPayloadStore);

#[derive(Resource)]
pub struct EditorKeybindingsResource(pub KeybindingMap);

impl Default for EditorKeybindingsResource {
    fn default() -> Self {
        Self(KeybindingMap::with_defaults())
    }
}

#[derive(Resource)]
pub struct EditorDispatcherResource(pub CommandDispatcher);

impl Default for EditorDispatcherResource {
    fn default() -> Self {
        Self(CommandDispatcher::new(CommandRegistry::new()))
    }
}

/// Central resource managing the open-closed DataflowEngine and registered node catalog.
#[derive(Resource)]
pub struct EditorDataflowResource(pub luma_dataflow::engine::DataflowEngine);

impl Default for EditorDataflowResource {
    fn default() -> Self {
        let registry = luma_dataflow::registry::NodeRegistry::new();
        Self(luma_dataflow::engine::DataflowEngine::new(registry))
    }
}

/// Manages smooth cubic-out spring easing transitions for camera focus and navigation (~250ms).
#[derive(Resource, Default)]
pub struct CameraAnimationResource {
    pub active: bool,
    pub start_pos: Vec2,
    pub target_pos: Vec2,
    pub start_scale: f32,
    pub target_scale: f32,
    pub elapsed: f32,
    pub duration: f32,
}

impl CameraAnimationResource {
    pub fn animate_to(
        &mut self,
        current_pos: Vec2,
        current_scale: f32,
        target_pos: Vec2,
        target_scale: f32,
    ) {
        self.active = true;
        self.start_pos = current_pos;
        self.target_pos = target_pos;
        self.start_scale = current_scale;
        self.target_scale = target_scale;
        self.elapsed = 0.0;
        self.duration = 0.25; // 250ms smooth spring deceleration
    }

    pub fn cancel(&mut self) {
        self.active = false;
    }
}

/// The main Bevy plugin providing the Luma Editor Canvas systems and resources.
pub struct LumaEditorPlugin;

impl Plugin for LumaEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorGraphResource>()
            .init_resource::<EditorHistoryResource>()
            .init_resource::<EditorIdGenResource>()
            .init_resource::<EditorTransformResource>()
            .init_resource::<EditorSelectionResource>()
            .init_resource::<EditorWiringResource>()
            .init_resource::<EditorCanvasInputGuard>()
            .init_resource::<EditorKeybindingsResource>()
            .init_resource::<EditorDispatcherResource>()
            .init_resource::<CameraAnimationResource>()
            .init_resource::<EditorDomainPayloadResource>()
            .init_resource::<EditorDataflowResource>()
            .add_systems(Update, (canvas_pan_zoom_system, canvas_shortcuts_system));
    }
}

#[derive(Component)]
pub struct EditorCamera;

/// System for panning and zooming the infinite canvas using mouse and touchpad inputs via the 2D Camera.
pub fn canvas_pan_zoom_system(
    time: Res<Time>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    _motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    input_guard: Res<EditorCanvasInputGuard>,
    windows: Query<&Window>,
    mut camera_query: Query<(&mut Transform, &mut Projection), With<EditorCamera>>,
    mut transform: ResMut<EditorTransformResource>,
    mut anim: ResMut<CameraAnimationResource>,
    mut last_cursor_pos: Local<Option<Vec2>>,
) {
    let window = match windows.iter().next() {
        Some(w) => w,
        None => return,
    };

    let (mut cam_transform, mut projection) = match camera_query.iter_mut().next() {
        Some(c) => c,
        None => return,
    };

    let proj = match *projection {
        Projection::Orthographic(ref mut p) => p,
        _ => return,
    };

    // 1. Pan: Middle mouse OR Right mouse OR Space + Left mouse (1:1 cursor delta tracking)
    let is_panning = mouse_buttons.pressed(MouseButton::Middle)
        || mouse_buttons.pressed(MouseButton::Right)
        || (keys.pressed(KeyCode::Space) && mouse_buttons.pressed(MouseButton::Left));

    let cursor_pos = window.cursor_position();
    let is_blocked = input_guard.is_blocked
        || cursor_pos.map_or(true, |pos| {
            pos.y <= 68.0 || pos.y >= (window.height() - 44.0)
        });

    let has_scroll = !is_blocked && scroll.delta.y.abs() > 0.0;
    let ctrl = !is_blocked && (keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight));

    if (!is_blocked && is_panning) || has_scroll || (ctrl && (keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::Digit0))) {
        anim.cancel();
    }

    if !is_blocked && is_panning {
        if let Some(current_cursor) = cursor_pos {
            if let Some(last_pos) = *last_cursor_pos {
                let delta = current_cursor - last_pos;
                let world_delta = Vec2::new(-delta.x * proj.scale, delta.y * proj.scale);
                cam_transform.translation += world_delta.extend(0.0);
            }
            *last_cursor_pos = Some(current_cursor);
        } else {
            *last_cursor_pos = None;
        }
    } else {
        *last_cursor_pos = None;
    }

    // 2. Real Camera Zoom: Mouse wheel scroll scales the camera's orthographic projection
    if has_scroll {
        let cursor_screen = window.cursor_position().unwrap_or_else(|| {
            Vec2::new(window.width() * 0.5, window.height() * 0.5)
        });

        let half_win = Vec2::new(window.width() * 0.5, window.height() * 0.5);
        let cursor_view_offset = Vec2::new(cursor_screen.x - half_win.x, half_win.y - cursor_screen.y);

        let old_scale = proj.scale;
        let delta_y = scroll.delta.y.signum() * scroll.delta.y.abs().min(2.5);
        let zoom_step = 1.0 - (delta_y * 0.05).clamp(-0.12, 0.12);
        let new_scale = (old_scale * zoom_step).clamp(0.25, 4.0);

        if (new_scale - old_scale).abs() > f32::EPSILON {
            let cam_shift = cursor_view_offset * (old_scale - new_scale);
            cam_transform.translation += cam_shift.extend(0.0);
            proj.scale = new_scale;
        }
    }

    // 3. Hotkey Zoom: Ctrl + Plus, Ctrl + Minus, Ctrl + 0 (Reset)
    if ctrl {
        if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
            proj.scale = (proj.scale * 0.85).clamp(0.25, 4.0);
        }
        if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
            proj.scale = (proj.scale * 1.15).clamp(0.25, 4.0);
        }
        if keys.just_pressed(KeyCode::Digit0) || keys.just_pressed(KeyCode::Numpad0) {
            proj.scale = 1.0;
            cam_transform.translation = Vec3::ZERO;
        }
    }

    // 4. Spring Easing Camera Animation (~250ms)
    if anim.active {
        anim.elapsed += time.delta_secs();
        let t = (anim.elapsed / anim.duration).clamp(0.0, 1.0);
        // Cubic Out Easing: 1.0 - (1.0 - t).powi(3)
        let factor = 1.0 - (1.0 - t).powi(3);

        let cur_pos = anim.start_pos.lerp(anim.target_pos, factor);
        let cur_scale = anim.start_scale + (anim.target_scale - anim.start_scale) * factor;

        cam_transform.translation.x = cur_pos.x;
        cam_transform.translation.y = cur_pos.y;
        proj.scale = cur_scale;

        if t >= 1.0 {
            anim.active = false;
        }
    }

    // Sync telemetry
    transform.0.pan = Vec2::new(cam_transform.translation.x, cam_transform.translation.y);
    transform.0.zoom = 1.0 / proj.scale;
}

/// System for capturing hotkeys and routing them through the CommandDispatcher.
pub fn canvas_shortcuts_system(
    keys: Res<ButtonInput<KeyCode>>,
    input_guard: Res<EditorCanvasInputGuard>,
    keybindings: Res<EditorKeybindingsResource>,
    mut dispatcher: ResMut<EditorDispatcherResource>,
    mut graph_res: ResMut<EditorGraphResource>,
    mut history_res: ResMut<EditorHistoryResource>,
    mut payload_res: ResMut<EditorDomainPayloadResource>,
    mut dataflow_res: ResMut<EditorDataflowResource>,
) {
    if input_guard.is_blocked {
        return;
    }

    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let alt = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);

    for &key_code in keys.get_just_pressed() {
        let editor_key = EditorKey::from(key_code);
        let chord = KeyChord {
            key: editor_key,
            ctrl,
            shift,
            alt,
        };

        if let Some(command_id) = keybindings.0.get_command(&chord) {
            let mut ctx = CommandContext {
                graph: &mut graph_res.0,
                history: &mut history_res.0,
                payloads: &mut payload_res.0,
                dataflow: &mut dataflow_res.0,
            };
            let _ = dispatcher.0.dispatch(command_id, &mut ctx);
        }
    }
}
