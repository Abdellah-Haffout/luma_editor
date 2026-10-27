use luma_canvas::plugin::{
    EditorCamera, EditorCanvasInputGuard, EditorGraphResource, EditorHistoryResource,
    EditorIdGenResource, EditorSelectionResource, EditorWiringResource,
};
use luma_canvas::wiring::{sample_wire_points, WireRoutingStyle, WiringState};
use luma_core::graph::PinDirection;
use luma_core::history::{ConnectCommand, DisconnectCommand};
use luma_core::id::{ConnectionId, NodeId, PinId};
use luma_canvas::nodes::card::NodeCardLayout;
use crate::bevy_ui_hud::{BevyUiSpawnPalette, BevyUiToast, EditorAccessibilitySettings};
use crate::bevy_ui_settings::{CanvasColorPalette, CanvasPatternStyle, EditorSettingsState};
use crate::components::node_card::{NodeCardRoot, NodeCardSocketPin};
use crate::connection_material::LumaConnectionMaterial;
use crate::plugin::{EditorNavigationResource, EditorUiCamera};
use crate::theme::EditorTheme;
use bevy::prelude::*;
use std::collections::HashMap;

/// Builds a map of exact 2D world-space centers for all rendered `NodeCardSocketPin` entities,
/// accounting for any same-frame node drag delta (`node.position - root_gt.translation()`).
pub fn build_live_pin_positions_map(
    graph: &luma_core::graph::GraphStore,
    card_roots: &Query<(&NodeCardRoot, &GlobalTransform)>,
    socket_pins: &Query<(&NodeCardSocketPin, &GlobalTransform)>,
) -> HashMap<PinId, Vec2> {
    let mut root_positions: HashMap<NodeId, Vec2> = HashMap::with_capacity(card_roots.iter().len());
    for (root, gt) in card_roots.iter() {
        root_positions.insert(root.node_id, gt.translation().truncate());
    }

    let mut map: HashMap<PinId, Vec2> = HashMap::with_capacity(socket_pins.iter().len());
    for (socket, socket_gt) in socket_pins.iter() {
        if let Some(pin_id) = socket.pin_id {
            if let (Some(&root_pos), Some(node)) = (
                root_positions.get(&socket.node_id),
                graph.get_node(socket.node_id),
            ) {
                let local_offset = socket_gt.translation().truncate() - root_pos;
                if local_offset.length_squared() > 1.0 {
                    map.insert(pin_id, node.position + local_offset);
                }
            }
        }
    }
    map
}

/// Computes the exact 2D world-space position of a pin socket directly from the Node definition,
/// with ZERO heap allocations.
#[inline]
pub fn compute_pin_world_position(node: &luma_core::graph::Node, pin_id: PinId) -> Option<Vec2> {
    let card_size = NodeCardLayout::compute_size_fast(
        &node.kind,
        node.size,
        node.inputs.len(),
        node.outputs.len(),
    );
    let half_w = card_size.x * 0.5;
    let half_h = card_size.y * 0.5;
    let header_offset = 34.0;
    let row_height = 26.0;

    if let Some(idx) = node.inputs.iter().position(|&p| p == pin_id) {
        let local_y = header_offset + (idx as f32 * row_height) + 12.0;
        return Some(Vec2::new(node.position.x - half_w, node.position.y + half_h - local_y));
    }

    if let Some(idx) = node.outputs.iter().position(|&p| p == pin_id) {
        let local_y = header_offset + (idx as f32 * row_height) + 12.0;
        return Some(Vec2::new(node.position.x + half_w, node.position.y + half_h - local_y));
    }

    None
}

/// Computes the exact 2D world-space position of a socket dot on a node card.
pub fn pin_world_position(
    node_pos: Vec2,
    _node_size: Vec2,
    pin_id: PinId,
    layout: &NodeCardLayout,
) -> Option<Vec2> {
    let half_w = layout.size.x * 0.5;
    let half_h = layout.size.y * 0.5;
    let header_offset = 34.0;
    let row_height = 26.0;

    if let Some(idx) = layout.input_pins.iter().position(|p| p.id == pin_id) {
        let local_y = header_offset + (idx as f32 * row_height) + 12.0;
        return Some(Vec2::new(node_pos.x - half_w, node_pos.y + half_h - local_y));
    }

    if let Some(idx) = layout.output_pins.iter().position(|p| p.id == pin_id) {
        let local_y = header_offset + (idx as f32 * row_height) + 12.0;
        return Some(Vec2::new(node_pos.x + half_w, node_pos.y + half_h - local_y));
    }

    None
}

/// Helper to clip 1D line segments against an occluding interval in-place without heap allocations.
fn clip_segments_in_place(
    current: &mut Vec<(f32, f32)>,
    scratch: &mut Vec<(f32, f32)>,
    occluder_min: f32,
    occluder_max: f32,
) {
    scratch.clear();
    for &(start, end) in current.iter() {
        if end <= occluder_min || start >= occluder_max {
            scratch.push((start, end));
        } else {
            if start < occluder_min {
                scratch.push((start, occluder_min));
            }
            if end > occluder_max {
                scratch.push((occluder_max, end));
            }
        }
    }
    std::mem::swap(current, scratch);
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PortShape {
    Circle,        // Asset (circle)
    Diamond,       // Mesh (diamond)
    RoundedSquare, // Material (rounded square)
    Hexagon,       // Entity (hexagon)
}

#[allow(dead_code)]
pub fn get_pin_type_info(type_name: &str) -> (Color, PortShape) {
    let color = crate::components::node_card::get_blender_pin_color(type_name);
    (color, PortShape::Circle)
}

fn lerp_color(c0: Color, c1: Color, t: f32) -> Color {
    let s0 = c0.to_srgba();
    let s1 = c1.to_srgba();
    Color::srgba(
        s0.red + (s1.red - s0.red) * t,
        s0.green + (s1.green - s0.green) * t,
        s0.blue + (s1.blue - s0.blue) * t,
        s0.alpha + (s1.alpha - s0.alpha) * t,
    )
}

#[allow(dead_code)]
fn draw_diamond_path(gizmos: &mut Gizmos, center: Vec2, radius: f32, color: Color) {
    let p0 = Vec2::new(center.x, center.y + radius);
    let p1 = Vec2::new(center.x + radius, center.y);
    let p2 = Vec2::new(center.x, center.y - radius);
    let p3 = Vec2::new(center.x - radius, center.y);
    gizmos.linestrip_2d([p0, p1, p2, p3, p0], color);
}

#[allow(dead_code)]
fn draw_diamond_port(
    gizmos: &mut Gizmos,
    center: Vec2,
    is_connected: bool,
    color: Color,
    pulse: f32,
    reduced_motion: bool,
) {
    let card_bg = EditorTheme::BG_CARD_BOTTOM;
    let s = color.to_srgba();
    let p = if reduced_motion { 1.0 } else { pulse };

    if is_connected {
        // Soft glow (8px radius, 60% opacity)
        let mut gr = 8.0;
        while gr >= 6.8 {
            let t = (gr - 6.8) / 1.2;
            draw_diamond_path(
                gizmos,
                center,
                gr,
                Color::srgba(s.red, s.green, s.blue, 0.60 * (1.0 - t * 0.5)),
            );
            gr -= 0.3;
        }
        // Filled with type color
        let mut r = 7.0;
        while r >= 0.3 {
            draw_diamond_path(gizmos, center, r, color);
            r -= 0.5;
        }
    } else {
        let r_outer = 7.0 * p;
        let r_inner = 5.0 * p;
        // Inside filled with card background
        let mut r = r_inner;
        while r >= 0.3 {
            draw_diamond_path(gizmos, center, r, card_bg);
            r -= 0.5;
        }
        // 2px ring in full-brightness type color
        let mut r = r_outer;
        while r >= r_inner {
            draw_diamond_path(gizmos, center, r, color);
            r -= 0.4;
        }
    }
}

#[allow(dead_code)]
fn draw_rounded_square_path(gizmos: &mut Gizmos, center: Vec2, half: f32, cr: f32, color: Color) {
    let r = cr.min(half * 0.5);
    let mut pts = Vec::with_capacity(21);
    let corners = [
        (center + Vec2::new(half - r, half - r), 0.0, std::f32::consts::FRAC_PI_2),
        (center + Vec2::new(-(half - r), half - r), std::f32::consts::FRAC_PI_2, std::f32::consts::PI),
        (center + Vec2::new(-(half - r), -(half - r)), std::f32::consts::PI, 3.0 * std::f32::consts::FRAC_PI_2),
        (center + Vec2::new(half - r, -(half - r)), 3.0 * std::f32::consts::FRAC_PI_2, std::f32::consts::TAU),
    ];
    for (c, a0, a1) in corners {
        for s in 0..=4 {
            let angle = a0 + (a1 - a0) * (s as f32 / 4.0);
            pts.push(c + Vec2::new(angle.cos(), angle.sin()) * r);
        }
    }
    pts.push(pts[0]);
    gizmos.linestrip_2d(pts, color);
}

#[allow(dead_code)]
fn draw_rounded_square_port(
    gizmos: &mut Gizmos,
    center: Vec2,
    is_connected: bool,
    color: Color,
    pulse: f32,
    reduced_motion: bool,
) {
    let card_bg = EditorTheme::BG_CARD_BOTTOM;
    let s = color.to_srgba();
    let p = if reduced_motion { 1.0 } else { pulse };

    if is_connected {
        // Soft glow (8px radius, 60% opacity)
        let mut gr = 8.0;
        while gr >= 6.8 {
            let t = (gr - 6.8) / 1.2;
            draw_rounded_square_path(
                gizmos,
                center,
                gr,
                3.0,
                Color::srgba(s.red, s.green, s.blue, 0.60 * (1.0 - t * 0.5)),
            );
            gr -= 0.3;
        }
        // Filled with type color
        let mut sz = 7.0;
        while sz >= 0.3 {
            let cr = 3.0 * (sz / 7.0);
            draw_rounded_square_path(gizmos, center, sz, cr, color);
            sz -= 0.5;
        }
    } else {
        let sz_outer = 7.0 * p;
        let sz_inner = 5.0 * p;
        // Inside filled with card background
        let mut sz = sz_inner;
        while sz >= 0.3 {
            let cr = 3.0 * (sz / 7.0);
            draw_rounded_square_path(gizmos, center, sz, cr, card_bg);
            sz -= 0.5;
        }
        // 2px ring in full-brightness type color
        let mut sz = sz_outer;
        while sz >= sz_inner {
            let cr = 3.0 * (sz / sz_outer);
            draw_rounded_square_path(gizmos, center, sz, cr, color);
            sz -= 0.4;
        }
    }
}

#[allow(dead_code)]
fn draw_hexagon_path(gizmos: &mut Gizmos, center: Vec2, radius: f32, color: Color) {
    let mut pts = [Vec2::ZERO; 7];
    for i in 0..6 {
        let angle = i as f32 * std::f32::consts::FRAC_PI_3;
        pts[i] = center + Vec2::new(angle.cos(), angle.sin()) * radius;
    }
    pts[6] = pts[0];
    gizmos.linestrip_2d(pts, color);
}

#[allow(dead_code)]
fn draw_hexagon_port(
    gizmos: &mut Gizmos,
    center: Vec2,
    is_connected: bool,
    color: Color,
    pulse: f32,
    reduced_motion: bool,
) {
    let card_bg = EditorTheme::BG_CARD_BOTTOM;
    let s = color.to_srgba();
    let p = if reduced_motion { 1.0 } else { pulse };

    if is_connected {
        // Soft glow (8px radius, 60% opacity)
        let mut gr = 8.0;
        while gr >= 6.8 {
            let t = (gr - 6.8) / 1.2;
            draw_hexagon_path(
                gizmos,
                center,
                gr,
                Color::srgba(s.red, s.green, s.blue, 0.60 * (1.0 - t * 0.5)),
            );
            gr -= 0.3;
        }
        // Filled with type color
        let mut r = 7.0;
        while r >= 0.3 {
            draw_hexagon_path(gizmos, center, r, color);
            r -= 0.5;
        }
    } else {
        let r_outer = 7.0 * p;
        let r_inner = 5.0 * p;
        // Inside filled with card background
        let mut r = r_inner;
        while r >= 0.3 {
            draw_hexagon_path(gizmos, center, r, card_bg);
            r -= 0.5;
        }
        // 2px ring in full-brightness type color (emerald entity ring is as bright as the others)
        let mut r = r_outer;
        while r >= r_inner {
            draw_hexagon_path(gizmos, center, r, color);
            r -= 0.4;
        }
    }
}

#[allow(dead_code)]
fn draw_circle_port(
    gizmos: &mut Gizmos,
    center: Vec2,
    is_connected: bool,
    color: Color,
    pulse: f32,
    reduced_motion: bool,
) {
    let card_bg = EditorTheme::BG_CARD_BOTTOM;
    let s = color.to_srgba();
    let p = if reduced_motion { 1.0 } else { pulse };

    if is_connected {
        // Soft glow (8px radius, 60% opacity)
        let mut gr = 8.0;
        while gr >= 6.8 {
            let t = (gr - 6.8) / 1.2;
            gizmos.circle_2d(
                Isometry2d::from_translation(center),
                gr,
                Color::srgba(s.red, s.green, s.blue, 0.60 * (1.0 - t * 0.5)),
            );
            gr -= 0.3;
        }
        // Filled with type color
        let mut r = 7.0;
        while r >= 0.3 {
            gizmos.circle_2d(Isometry2d::from_translation(center), r, color);
            r -= 0.5;
        }
    } else {
        let r_outer = 7.0 * p;
        let r_inner = 5.0 * p;
        // Inside filled with card background
        let mut r = r_inner;
        while r >= 0.3 {
            gizmos.circle_2d(Isometry2d::from_translation(center), r, card_bg);
            r -= 0.5;
        }
        // 2px ring in full-brightness type color
        let mut r = r_outer;
        while r >= r_inner {
            gizmos.circle_2d(Isometry2d::from_translation(center), r, color);
            r -= 0.4;
        }
    }
}

/// Renders real visual thumbnails inside node card viewports (3D wireframe mesh, PBR sphere, etc.)
#[allow(dead_code)]
fn draw_node_visual_previews(
    gizmos: &mut Gizmos,
    node: &luma_core::graph::Node,
    time_secs: f32,
    reduced_motion: bool,
) {
    let card_h = if matches!(node.kind.as_str(), "AssetLoader" | "MeshNode" | "MaterialNode" | "EntityAssembler") {
        188.0
    } else {
        node.size.y
    };
    let half_h = card_h * 0.5;

    match node.kind.as_str() {
        "AssetLoader" => {
            // Graphic preview centered alone in preview container (72px box)
            let center = Vec2::new(node.position.x, node.position.y + half_h - 116.0);

            // Rendered 3D wireframe character/asset mesh (rotating smoothly)
            let rot = if reduced_motion { 0.8 } else { time_secs * 0.6 };
            let cos_r = rot.cos();
            let sin_r = rot.sin();

            let vertices_3d = [
                Vec3::new(0.0, 16.0, 0.0),
                Vec3::new(12.0 * cos_r - 0.0 * sin_r, 0.0, 12.0 * sin_r + 0.0 * cos_r),
                Vec3::new(0.0 * cos_r - 12.0 * sin_r, 0.0, 0.0 * sin_r + 12.0 * cos_r),
                Vec3::new(-12.0 * cos_r - 0.0 * sin_r, 0.0, -12.0 * sin_r + 0.0 * cos_r),
                Vec3::new(0.0 * cos_r + 12.0 * sin_r, 0.0, 0.0 * sin_r - 12.0 * cos_r),
                Vec3::new(0.0, -16.0, 0.0),
            ];

            let project = |v: Vec3| -> Vec2 {
                let iso_x = (v.x - v.z) * 0.866;
                let iso_y = v.y + (v.x + v.z) * 0.4;
                center + Vec2::new(iso_x, iso_y)
            };

            let wire_col = EditorTheme::PIN_ASSET;
            let edges = [
                (0, 1), (0, 2), (0, 3), (0, 4),
                (1, 2), (2, 3), (3, 4), (4, 1),
                (5, 1), (5, 2), (5, 3), (5, 4),
            ];

            for (i0, i1) in edges {
                let p0 = project(vertices_3d[i0]);
                let p1 = project(vertices_3d[i1]);
                gizmos.line_2d(p0, p1, wire_col);
            }
        }

        "MeshNode" => {
            // Graphic preview centered alone in preview container (72px box)
            let center = Vec2::new(node.position.x, node.position.y + half_h - 116.0);

            // Rendered 3D wireframe cube
            let rot = if reduced_motion { 0.6 } else { time_secs * 0.5 };
            let cos_r = rot.cos();
            let sin_r = rot.sin();
            let s = 10.5;

            let cube_corners = [
                Vec3::new(-s, -s, -s), Vec3::new(s, -s, -s),
                Vec3::new(s, s, -s),   Vec3::new(-s, s, -s),
                Vec3::new(-s, -s, s),  Vec3::new(s, -s, s),
                Vec3::new(s, s, s),    Vec3::new(-s, s, s),
            ];

            let rot_corners: Vec<Vec3> = cube_corners.iter().map(|v| {
                Vec3::new(
                    v.x * cos_r - v.z * sin_r,
                    v.y,
                    v.x * sin_r + v.z * cos_r,
                )
            }).collect();

            let project = |v: Vec3| -> Vec2 {
                let iso_x = (v.x - v.z) * 0.866;
                let iso_y = v.y + (v.x + v.z) * 0.35;
                center + Vec2::new(iso_x, iso_y)
            };

            let wire_col = EditorTheme::PIN_MESH;
            let cube_edges = [
                (0, 1), (1, 2), (2, 3), (3, 0),
                (4, 5), (5, 6), (6, 7), (7, 4),
                (0, 4), (1, 5), (2, 6), (3, 7),
            ];

            for (i0, i1) in cube_edges {
                let p0 = project(rot_corners[i0]);
                let p1 = project(rot_corners[i1]);
                gizmos.line_2d(p0, p1, wire_col);
            }
        }

        "MaterialNode" => {
            // Rendered PBR sphere centered alone with 16px clear space on top and bottom (> 12px)
            let center = Vec2::new(node.position.x, node.position.y + half_h - 116.0);
            let sphere_r = 20.0;

            // Concentric shaded rings for 3D sphere depth
            let base_col = EditorTheme::PIN_MATERIAL.to_srgba();
            let dark_tint = Color::srgba(base_col.red * 0.30, base_col.green * 0.30, base_col.blue * 0.30, 1.0);
            let mid_tint = EditorTheme::PIN_MATERIAL;

            let mut r = sphere_r;
            while r >= 1.0 {
                let t = r / sphere_r;
                let ring_c = lerp_color(mid_tint, dark_tint, t);
                gizmos.circle_2d(Isometry2d::from_translation(center), r, ring_c);
                r -= 0.6;
            }

            // Metallic reflection rim on outer circumference
            gizmos.circle_2d(
                Isometry2d::from_translation(center),
                sphere_r,
                Color::srgba(1.0, 0.70, 0.85, 0.85),
            );

            // Specular highlight spot (roughness 0.50 = tight highlight with soft falloff)
            let spec_center = center + Vec2::new(-5.5, 5.5);
            // Specular halo
            gizmos.circle_2d(
                Isometry2d::from_translation(spec_center),
                5.5,
                Color::srgba(1.0, 1.0, 1.0, 0.35),
            );
            // Specular core spot
            let mut sr = 2.6;
            while sr >= 0.3 {
                gizmos.circle_2d(Isometry2d::from_translation(spec_center), sr, Color::WHITE);
                sr -= 0.5;
            }
        }

        "EntityAssembler" => {
            // Rendered entity indicator centered alone
            let center = Vec2::new(node.position.x, node.position.y + half_h - 116.0);
            let hex_r = 18.0;
            draw_hexagon_path(gizmos, center, hex_r, EditorTheme::PIN_ENTITY);
            gizmos.circle_2d(Isometry2d::from_translation(center), 5.0, EditorTheme::PIN_ENTITY);
        }

        _ => {}
    }
}

/// System that draws infinite canvas grid and focus bounding boxes directly in 2D World Space.
/// Handled natively by Bevy's 2D Camera so elements automatically scale and pan with zero CPU recalculations.
pub fn draw_canvas_wires_system(
    mut gizmos: Gizmos,
    mut clear_color: Option<ResMut<ClearColor>>,
    graph_res: Res<EditorGraphResource>,
    _wiring_res: Res<EditorWiringResource>,
    selection_res: Res<EditorSelectionResource>,
    _nav_res: Res<EditorNavigationResource>,
    _access_res: Res<EditorAccessibilitySettings>,
    settings_res: Option<Res<EditorSettingsState>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform, &Projection), With<EditorCamera>>,
    _card_roots: Query<(&NodeCardRoot, &GlobalTransform)>,
    _socket_pins: Query<(&NodeCardSocketPin, &GlobalTransform)>,
    _time: Res<Time>,
) {
    let graph = &graph_res.0;
    let show_grid = settings_res.as_ref().map_or(true, |s| s.show_grid);
    let canvas_pattern = settings_res
        .as_ref()
        .map_or(CanvasPatternStyle::Dots, |s| s.canvas_pattern);
    let canvas_palette = settings_res
        .as_ref()
        .map_or(CanvasColorPalette::DarkSlate, |s| s.canvas_palette);

    // Dynamic background color sync with selected palette
    if let Some(ref mut cc) = clear_color {
        let desired_bg = canvas_palette.canvas_bg();
        if cc.0 != desired_bg {
            cc.0 = desired_bg;
        }
    }

    // Collect 2D bounding boxes of all node cards so grid lines/dots/crosses are never drawn over node bodies
    let node_rects: Vec<Rect> = graph
        .nodes()
        .values()
        .map(|n| {
            let sz = NodeCardLayout::compute_size_fast(&n.kind, n.size, n.inputs.len(), n.outputs.len());
            let half = sz * 0.5 + Vec2::splat(1.5);
            Rect {
                min: n.position - half,
                max: n.position + half,
            }
        })
        .collect();

    // 1. Draw infinite 2D canvas pattern matching camera viewport (clipped around node cards)
    if show_grid && canvas_pattern != CanvasPatternStyle::None {
        if let (Some(window), Some((_camera, cam_gt, projection))) =
            (windows.iter().next(), camera_query.iter().next())
        {
            let scale = match projection {
                Projection::Orthographic(proj) => proj.scale,
                _ => 1.0,
            };
            let cam_pos = cam_gt.translation().truncate();
            let half_w = window.width() * 0.5 * scale;
            let half_h = window.height() * 0.5 * scale;

            let min_x = cam_pos.x - half_w;
            let max_x = cam_pos.x + half_w;
            let min_y = cam_pos.y - half_h;
            let max_y = cam_pos.y + half_h;

            let major_step = 100.0;
            let mut segments: Vec<(f32, f32)> = Vec::with_capacity(8);
            let mut scratch: Vec<(f32, f32)> = Vec::with_capacity(8);

            match canvas_pattern {
                CanvasPatternStyle::Grid => {
                    let minor_step = 20.0;

                    // Minor grid lines (every 20px, rendered when zoomed in enough: scale < 2.0)
                    if scale < 2.0 {
                        let start_x = (min_x / minor_step).floor() * minor_step;
                        let mut x = start_x;
                        let minor_color = canvas_palette.pattern_secondary();
                        while x <= max_x {
                            if (x % major_step).abs() > 0.1 && ((x % major_step) - major_step).abs() > 0.1 {
                                segments.clear();
                                segments.push((min_y, max_y));
                                for r in &node_rects {
                                    if x >= r.min.x && x <= r.max.x {
                                        clip_segments_in_place(&mut segments, &mut scratch, r.min.y, r.max.y);
                                    }
                                }
                                for &(sy, ey) in &segments {
                                    gizmos.line_2d(Vec2::new(x, sy), Vec2::new(x, ey), minor_color);
                                }
                            }
                            x += minor_step;
                        }

                        let start_y = (min_y / minor_step).floor() * minor_step;
                        let mut y = start_y;
                        while y <= max_y {
                            if (y % major_step).abs() > 0.1 && ((y % major_step) - major_step).abs() > 0.1 {
                                segments.clear();
                                segments.push((min_x, max_x));
                                for r in &node_rects {
                                    if y >= r.min.y && y <= r.max.y {
                                        clip_segments_in_place(&mut segments, &mut scratch, r.min.x, r.max.x);
                                    }
                                }
                                for &(sx, ex) in &segments {
                                    gizmos.line_2d(Vec2::new(sx, y), Vec2::new(ex, y), minor_color);
                                }
                            }
                            y += minor_step;
                        }
                    }

                    // Major grid lines (every 100px)
                    let start_major_x = (min_x / major_step).floor() * major_step;
                    let mut x = start_major_x;
                    let major_col = canvas_palette.pattern_primary();
                    while x <= max_x {
                        segments.clear();
                        segments.push((min_y, max_y));
                        for r in &node_rects {
                            if x >= r.min.x && x <= r.max.x {
                                clip_segments_in_place(&mut segments, &mut scratch, r.min.y, r.max.y);
                            }
                        }
                        for &(sy, ey) in &segments {
                            gizmos.line_2d(Vec2::new(x, sy), Vec2::new(x, ey), major_col);
                        }
                        x += major_step;
                    }

                    let start_major_y = (min_y / major_step).floor() * major_step;
                    let mut y = start_major_y;
                    while y <= max_y {
                        segments.clear();
                        segments.push((min_x, max_x));
                        for r in &node_rects {
                            if y >= r.min.y && y <= r.max.y {
                                clip_segments_in_place(&mut segments, &mut scratch, r.min.x, r.max.x);
                            }
                        }
                        for &(sx, ex) in &segments {
                            gizmos.line_2d(Vec2::new(sx, y), Vec2::new(ex, y), major_col);
                        }
                        y += major_step;
                    }
                }

                CanvasPatternStyle::Dots => {
                    let step = if scale > 1.8 {
                        100.0
                    } else if scale > 0.85 {
                        50.0
                    } else {
                        25.0
                    };

                    let start_x = (min_x / step).floor() * step;
                    let start_y = (min_y / step).floor() * step;
                    let major_color = canvas_palette.pattern_primary();
                    let minor_color = canvas_palette.pattern_secondary();

                    let mut x = start_x;
                    while x <= max_x {
                        let is_major_x = (x % 100.0).abs() < 0.1 || ((x % 100.0) - 100.0).abs() < 0.1;
                        let mut y = start_y;
                        while y <= max_y {
                            let is_major_y = (y % 100.0).abs() < 0.1 || ((y % 100.0) - 100.0).abs() < 0.1;
                            let is_major = is_major_x && is_major_y;

                            let pt = Vec2::new(x, y);
                            let occluded = node_rects.iter().any(|r| r.contains(pt));
                            if !occluded {
                                if is_major {
                                    // Faint, crisp pin-point dot (radius 0.95px, low distraction)
                                    gizmos.circle_2d(Isometry2d::from_translation(pt), 0.95, major_color);
                                } else if scale < 1.25 {
                                    // Sub-pixel faint guide dot (radius 0.65px)
                                    gizmos.circle_2d(Isometry2d::from_translation(pt), 0.65, minor_color);
                                }
                            }
                            y += step;
                        }
                        x += step;
                    }
                }

                CanvasPatternStyle::Crosses => {
                    let step = if scale > 1.8 {
                        100.0
                    } else if scale > 0.85 {
                        50.0
                    } else {
                        25.0
                    };

                    let start_x = (min_x / step).floor() * step;
                    let start_y = (min_y / step).floor() * step;
                    let major_color = canvas_palette.pattern_primary();
                    let minor_color = canvas_palette.pattern_secondary();

                    let mut x = start_x;
                    while x <= max_x {
                        let is_major_x = (x % 100.0).abs() < 0.1 || ((x % 100.0) - 100.0).abs() < 0.1;
                        let mut y = start_y;
                        while y <= max_y {
                            let is_major_y = (y % 100.0).abs() < 0.1 || ((y % 100.0) - 100.0).abs() < 0.1;
                            let is_major = is_major_x && is_major_y;

                            let pt = Vec2::new(x, y);
                            let occluded = node_rects.iter().any(|r| r.contains(pt));
                            if !occluded {
                                if is_major {
                                    let arm = 2.5;
                                    gizmos.line_2d(Vec2::new(x - arm, y), Vec2::new(x + arm, y), major_color);
                                    gizmos.line_2d(Vec2::new(x, y - arm), Vec2::new(x, y + arm), major_color);
                                } else if scale < 1.25 {
                                    let arm = 1.2;
                                    gizmos.line_2d(Vec2::new(x - arm, y), Vec2::new(x + arm, y), minor_color);
                                    gizmos.line_2d(Vec2::new(x, y - arm), Vec2::new(x, y + arm), minor_color);
                                }
                            }
                            y += step;
                        }
                        x += step;
                    }
                }

                CanvasPatternStyle::None => {}
            }

            // Radial vignette: darkening the canvas edges smoothly by 40%
            let max_r = (half_w * half_w + half_h * half_h).sqrt();
            let inner_r = half_w.min(half_h) * 0.55;
            let ring_step = 24.0 * scale;
            let mut vr = inner_r;
            while vr <= max_r + ring_step {
                let t = ((vr - inner_r) / (max_r - inner_r)).clamp(0.0, 1.0);
                let alpha = 0.40 * (t * t);
                if alpha > 0.005 {
                    gizmos.circle_2d(
                        Isometry2d::from_translation(cam_pos),
                        vr,
                        Color::srgba(0.0, 0.0, 0.0, alpha),
                    );
                }
                vr += ring_step;
            }
        }
    }

    // 2. Draw active marquee selection box on canvas
    if let Some(rect) = selection_res.0.box_selection {
        let center = rect.center();
        let size = rect.size();
        if size.x > 1.0 && size.y > 1.0 {
            gizmos.rect_2d(
                Isometry2d::from_translation(center),
                size,
                EditorTheme::BORDER_FOCUSED,
            );
        }
    }

    // Note: Connection wires and ghost dragging wires are rendered via the GPU SDF shader (connection.wesl)
    // inside `update_luma_connections_system` and `update_luma_ghost_connection_system`.
}

/// Sibling UI marker for a connection wire rendered via `LumaConnectionMaterial` (connection.wesl).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LumaConnectionView {
    pub connection_id: ConnectionId,
}

/// Sibling UI marker for the interactive drag-to-connect ghost wire rendered via `LumaConnectionMaterial`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LumaGhostConnectionView;

/// Converts a Bevy `Color` to a uniform-friendly `Vec4` RGBA.
#[inline]
pub fn color_to_vec4(c: Color) -> Vec4 {
    let s = c.to_srgba();
    Vec4::new(s.red, s.green, s.blue, s.alpha)
}

/// Automatically spawns and despawns UI nodes carrying `LumaConnectionMaterial` to match active graph connections.
pub fn sync_luma_connections_system(
    mut commands: Commands,
    graph_res: Res<EditorGraphResource>,
    mut materials: ResMut<Assets<LumaConnectionMaterial>>,
    existing_wires: Query<(Entity, &LumaConnectionView)>,
    ui_cam_query: Query<Entity, With<EditorUiCamera>>,
) {
    let graph = &graph_res.0;
    let ui_cam = ui_cam_query.iter().next();

    let mut live_wire_map: HashMap<ConnectionId, Entity> = HashMap::with_capacity(existing_wires.iter().len());
    for (entity, view) in existing_wires.iter() {
        if graph.connections().contains_key(&view.connection_id) {
            live_wire_map.insert(view.connection_id, entity);
        } else {
            commands.entity(entity).despawn();
        }
    }

    for &conn_id in graph.connections().keys() {
        if !live_wire_map.contains_key(&conn_id) {
            let material = materials.add(LumaConnectionMaterial::default());
            let mut wire_cmd = commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                GlobalZIndex(2),
                MaterialNode(material),
                Pickable::IGNORE,
                LumaConnectionView { connection_id: conn_id },
            ));
            if let Some(cam) = ui_cam {
                wire_cmd.insert(UiTargetCamera(cam));
            }
        }
    }
}

/// Updates cubic Bézier endpoints and uniforms (`p0`..`p3`, gradient colors, pulse, dimming, occlusion)
/// on each connection's GPU material based on camera projection and pin world positions.
pub fn update_luma_connections_system(
    graph_res: Res<EditorGraphResource>,
    nav_res: Res<EditorNavigationResource>,
    access_res: Res<EditorAccessibilitySettings>,
    settings_res: Option<Res<EditorSettingsState>>,
    camera_query: Query<(&Camera, &GlobalTransform, &Projection), With<EditorCamera>>,
    card_roots: Query<(&NodeCardRoot, &GlobalTransform)>,
    socket_pins: Query<(&NodeCardSocketPin, &GlobalTransform)>,
    mut materials: ResMut<Assets<LumaConnectionMaterial>>,
    wire_views: Query<(&LumaConnectionView, &MaterialNode<LumaConnectionMaterial>)>,
    time: Res<Time>,
) {
    let graph = &graph_res.0;
    let Some((camera, cam_gt, projection)) = camera_query.iter().next() else {
        return;
    };

    let scale = match projection {
        Projection::Orthographic(proj) => proj.scale,
        _ => 1.0,
    };
    // Thickness scales dynamically with camera zoom: thinner when zoomed out, thicker when zoomed in
    let wire_screen_width = (2.4 / scale).clamp(1.0, 32.0);

    // Compute bounding boxes of all node cards in viewport pixel coordinates for GPU occlusion discard
    let mut occluders = [Vec4::ZERO; 16];
    let mut num_occluders = 0.0;
    for (i, node) in graph.nodes().values().enumerate().take(16) {
        let sz = NodeCardLayout::compute_size_fast(&node.kind, node.size, node.inputs.len(), node.outputs.len());
        let half = sz * 0.5;
        let p_top_left = Vec3::new(node.position.x - half.x, node.position.y + half.y, 0.0);
        let p_bottom_right = Vec3::new(node.position.x + half.x, node.position.y - half.y, 0.0);
        if let (Ok(tl), Ok(br)) = (
            camera.world_to_viewport(cam_gt, p_top_left),
            camera.world_to_viewport(cam_gt, p_bottom_right),
        ) {
            occluders[i] = Vec4::new(tl.x.min(br.x), tl.y.min(br.y), tl.x.max(br.x), tl.y.max(br.y));
            num_occluders += 1.0;
        }
    }

    let live_pins = build_live_pin_positions_map(graph, &card_roots, &socket_pins);
    let wire_pulses = settings_res.as_ref().map_or(true, |s| s.wire_pulses) && !access_res.reduced_motion;
    let global_wire_style = settings_res.as_ref().map_or(WireRoutingStyle::Orthogonal, |s| s.wire_style);
    let pulse_cycle = (time.elapsed_secs() * 0.45) % 1.0;
    let focused_id = nav_res.0.focused_node_id();

    for (conn_idx, (view, mat_handle)) in wire_views.iter().enumerate() {
        let Some(conn) = graph.connections().get(&view.connection_id) else {
            continue;
        };
        let Some(source_node) = graph.get_node(conn.source_node) else {
            continue;
        };
        let Some(target_node) = graph.get_node(conn.target_node) else {
            continue;
        };

        let Some(start_world) = live_pins
            .get(&conn.source_pin)
            .copied()
            .or_else(|| compute_pin_world_position(source_node, conn.source_pin))
        else {
            continue;
        };

        let Some(end_world) = live_pins
            .get(&conn.target_pin)
            .copied()
            .or_else(|| compute_pin_world_position(target_node, conn.target_pin))
        else {
            continue;
        };

        let Ok(start_screen) = camera.world_to_viewport(cam_gt, start_world.extend(0.0)) else {
            continue;
        };
        let Ok(end_screen) = camera.world_to_viewport(cam_gt, end_world.extend(0.0)) else {
            continue;
        };

        // Source and target type colors for the smooth two-pole gradient
        let (src_color, _) = if let Some(pin) = graph.get_pin(conn.source_pin) {
            get_pin_type_info(&pin.data_type.name)
        } else {
            (EditorTheme::ACCENT_PRIMARY, PortShape::Circle)
        };
        let (tgt_color, _) = if let Some(pin) = graph.get_pin(conn.target_pin) {
            get_pin_type_info(&pin.data_type.name)
        } else {
            (EditorTheme::ACCENT_PRIMARY, PortShape::Circle)
        };

        let is_dimmed = if let Some(fid) = focused_id {
            conn.source_node != fid && conn.target_node != fid
        } else {
            false
        };

        let Some(mut mat) = materials.get_mut(&mat_handle.0) else {
            continue;
        };

        let style = match global_wire_style {
            WireRoutingStyle::Showcase => match conn_idx % 5 {
                0 => WireRoutingStyle::Orthogonal,
                1 => WireRoutingStyle::Bezier,
                2 => WireRoutingStyle::Circuit,
                3 => WireRoutingStyle::Stepped,
                _ => WireRoutingStyle::Straight,
            },
            s => s,
        };

        if style == WireRoutingStyle::Bezier {
            let dx = (end_screen.x - start_screen.x).abs().max(40.0);
            mat.p0 = start_screen;
            mat.p1 = start_screen + Vec2::new(dx * 0.5, 0.0);
            mat.p2 = end_screen - Vec2::new(dx * 0.5, 0.0);
            mat.p3 = end_screen;
            mat.routing_mode = 0.0;
            mat.num_points = 0.0;
        } else {
            let pts_world = sample_wire_points(start_world, end_world, style);
            let mut poly_points = [Vec4::ZERO; 12];
            let num_pts = pts_world.len().min(24);
            for (idx, pt) in pts_world.iter().take(24).enumerate() {
                if let Ok(screen_pt) = camera.world_to_viewport(cam_gt, pt.extend(0.0)) {
                    let vec_idx = idx / 2;
                    if idx % 2 == 0 {
                        poly_points[vec_idx].x = screen_pt.x;
                        poly_points[vec_idx].y = screen_pt.y;
                    } else {
                        poly_points[vec_idx].z = screen_pt.x;
                        poly_points[vec_idx].w = screen_pt.y;
                    }
                }
            }
            mat.routing_mode = 1.0;
            mat.num_points = num_pts as f32;
            mat.poly_points = poly_points;
        }

        mat.src_color = color_to_vec4(src_color);
        mat.tgt_color = color_to_vec4(tgt_color);
        mat.width = wire_screen_width;
        mat.feather = 1.0;
        mat.pulse_pos = pulse_cycle;
        mat.pulse_enabled = if wire_pulses { 1.0 } else { 0.0 };
        mat.dimmed = if is_dimmed { 1.0 } else { 0.0 };
        mat.num_occluders = num_occluders;
        mat.occluders = occluders;
    }
}

/// Renders the interactive live dragging wire using `connection.wesl` during connect-drag gestures.
pub fn update_luma_ghost_connection_system(
    mut commands: Commands,
    graph_res: Res<EditorGraphResource>,
    wiring_res: Res<EditorWiringResource>,
    settings_res: Option<Res<EditorSettingsState>>,
    camera_query: Query<(&Camera, &GlobalTransform, &Projection), With<EditorCamera>>,
    card_roots: Query<(&NodeCardRoot, &GlobalTransform)>,
    socket_pins: Query<(&NodeCardSocketPin, &GlobalTransform)>,
    mut materials: ResMut<Assets<LumaConnectionMaterial>>,
    ghost_views: Query<(Entity, &MaterialNode<LumaConnectionMaterial>), With<LumaGhostConnectionView>>,
    ui_cam_query: Query<Entity, With<EditorUiCamera>>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    let graph = &graph_res.0;
    let ui_cam = ui_cam_query.iter().next();
    let Some((camera, cam_gt, projection)) = camera_query.iter().next() else {
        return;
    };

    let scale = match projection {
        Projection::Orthographic(proj) => proj.scale,
        _ => 1.0,
    };
    let wire_screen_width = (2.4 / scale).clamp(1.0, 32.0);

    let mut occluders = [Vec4::ZERO; 16];
    let mut num_occluders = 0.0;
    for (i, node) in graph.nodes().values().enumerate().take(16) {
        let sz = NodeCardLayout::compute_size_fast(&node.kind, node.size, node.inputs.len(), node.outputs.len());
        let half = sz * 0.5;
        let p_top_left = Vec3::new(node.position.x - half.x, node.position.y + half.y, 0.0);
        let p_bottom_right = Vec3::new(node.position.x + half.x, node.position.y - half.y, 0.0);
        if let (Ok(tl), Ok(br)) = (
            camera.world_to_viewport(cam_gt, p_top_left),
            camera.world_to_viewport(cam_gt, p_bottom_right),
        ) {
            occluders[i] = Vec4::new(tl.x.min(br.x), tl.y.min(br.y), tl.x.max(br.x), tl.y.max(br.y));
            num_occluders += 1.0;
        }
    }

    if let WiringState::DraggingNew {
        source_pin,
        current_canvas_pos,
    } = &wiring_res.0
    {
        let live_pins = build_live_pin_positions_map(graph, &card_roots, &socket_pins);
        let Some(pin) = graph.get_pin(*source_pin) else {
            return;
        };
        let Some(source_node) = graph.get_node(pin.node_id) else {
            return;
        };
        let Some(start_world) = live_pins
            .get(source_pin)
            .copied()
            .or_else(|| compute_pin_world_position(source_node, *source_pin))
        else {
            return;
        };

        // Snapping magnet effect
        let mut snap_pos = *current_canvas_pos;
        let mut snapped_pin_id = None;
        let magnet_radius = 42.0;

        for (&node_id, node) in graph.nodes() {
            if node_id == pin.node_id {
                continue;
            }
            for &candidate_pin_id in &node.inputs {
                if let Some(candidate_world) = live_pins
                    .get(&candidate_pin_id)
                    .copied()
                    .or_else(|| compute_pin_world_position(node, candidate_pin_id))
                {
                    if (*current_canvas_pos - candidate_world).length() <= magnet_radius {
                        snap_pos = candidate_world;
                        snapped_pin_id = Some((candidate_pin_id, candidate_world));
                        break;
                    }
                }
            }
        }

        let Ok(start_screen) = camera.world_to_viewport(cam_gt, start_world.extend(0.0)) else {
            return;
        };
        let Ok(end_screen) = camera.world_to_viewport(cam_gt, snap_pos.extend(0.0)) else {
            return;
        };

        let (src_color, _) = get_pin_type_info(&pin.data_type.name);
        let tgt_color = if let Some((cand_id, snap_coord)) = snapped_pin_id {
            // Ripple indicator on snapped socket
            let ripple_r = 7.0 + (time.elapsed_secs() * 6.0).sin().abs() * 3.5;
            gizmos.circle_2d(
                Isometry2d::from_translation(snap_coord),
                ripple_r,
                EditorTheme::ACCENT_SUCCESS,
            );

            if let Some(cand_pin) = graph.get_pin(cand_id) {
                get_pin_type_info(&cand_pin.data_type.name).0
            } else {
                EditorTheme::ACCENT_SUCCESS
            }
        } else {
            Color::srgba(0.85, 0.85, 0.9, 0.75)
        };

        let global_wire_style = settings_res.as_ref().map_or(WireRoutingStyle::Orthogonal, |s| s.wire_style);
        let drag_style = match global_wire_style {
            WireRoutingStyle::Showcase => WireRoutingStyle::Orthogonal,
            s => s,
        };

        let p0 = start_screen;
        let mut p1 = start_screen;
        let mut p2 = end_screen;
        let p3 = end_screen;
        let routing_mode;
        let num_points;
        let mut poly_points = [Vec4::ZERO; 12];

        if drag_style == WireRoutingStyle::Bezier {
            let dx = (end_screen.x - start_screen.x).abs().max(40.0);
            p1 = start_screen + Vec2::new(dx * 0.5, 0.0);
            p2 = end_screen - Vec2::new(dx * 0.5, 0.0);
            routing_mode = 0.0;
            num_points = 0.0;
        } else {
            let pts_world = sample_wire_points(start_world, snap_pos, drag_style);
            let num_pts = pts_world.len().min(24);
            for (idx, pt) in pts_world.iter().take(24).enumerate() {
                if let Ok(screen_pt) = camera.world_to_viewport(cam_gt, pt.extend(0.0)) {
                    let vec_idx = idx / 2;
                    if idx % 2 == 0 {
                        poly_points[vec_idx].x = screen_pt.x;
                        poly_points[vec_idx].y = screen_pt.y;
                    } else {
                        poly_points[vec_idx].z = screen_pt.x;
                        poly_points[vec_idx].w = screen_pt.y;
                    }
                }
            }
            routing_mode = 1.0;
            num_points = num_pts as f32;
        }

        if let Some((_, mat_handle)) = ghost_views.iter().next() {
            if let Some(mut mat) = materials.get_mut(&mat_handle.0) {
                mat.p0 = p0;
                mat.p1 = p1;
                mat.p2 = p2;
                mat.p3 = p3;
                mat.src_color = color_to_vec4(src_color);
                mat.tgt_color = color_to_vec4(tgt_color);
                mat.width = wire_screen_width;
                mat.feather = 1.0;
                mat.pulse_pos = (time.elapsed_secs() * 1.2) % 1.0;
                mat.pulse_enabled = 1.0;
                mat.dimmed = 0.0;
                mat.routing_mode = routing_mode;
                mat.num_points = num_points;
                mat.poly_points = poly_points;
                mat.num_occluders = num_occluders;
                mat.occluders = occluders;
            }
        } else {
            let material = materials.add(LumaConnectionMaterial {
                p0,
                p1,
                p2,
                p3,
                src_color: color_to_vec4(src_color),
                tgt_color: color_to_vec4(tgt_color),
                width: wire_screen_width,
                feather: 1.0,
                pulse_pos: 0.0,
                pulse_enabled: 1.0,
                dimmed: 0.0,
                routing_mode,
                num_points,
                num_occluders,
                poly_points,
                occluders,
            });

            let mut ghost_cmd = commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                GlobalZIndex(3),
                MaterialNode(material),
                Pickable::IGNORE,
                LumaGhostConnectionView,
            ));
            if let Some(cam) = ui_cam {
                ghost_cmd.insert(UiTargetCamera(cam));
            }
        }
    } else {
        for (entity, _) in ghost_views.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// Handles interactive drag-and-drop wire creation between card pin sockets in world space.
/// Also supports right-clicking a pin to quickly sever all its connections with Undo support.
pub fn interactive_pin_wiring_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    input_guard: Res<EditorCanvasInputGuard>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform, &Projection), With<EditorCamera>>,
    card_roots: Query<(&NodeCardRoot, &GlobalTransform)>,
    socket_pins: Query<(&NodeCardSocketPin, &GlobalTransform)>,
    mut graph_res: ResMut<EditorGraphResource>,
    mut wiring_res: ResMut<EditorWiringResource>,
    mut history_res: ResMut<EditorHistoryResource>,
    id_gen: Res<EditorIdGenResource>,
    mut toast_query: Query<(&mut BevyUiToast, &mut Node), Without<BevyUiSpawnPalette>>,
) {
    if keys.pressed(KeyCode::Space) {
        return;
    }

    if input_guard.is_blocked {
        if mouse_buttons.just_released(MouseButton::Left) {
            wiring_res.0 = WiringState::Idle;
        }
        return;
    }

    let window = match windows.iter().next() {
        Some(w) => w,
        None => return,
    };

    let cursor_screen = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    // Ignore clicks in screen chrome (Top Navbar or Bottom Status Bar)
    if cursor_screen.y <= 36.0 || cursor_screen.y >= (window.height() - 24.0) {
        return;
    }

    let (camera, cam_gt, projection) = match camera_query.iter().next() {
        Some(c) => c,
        None => return,
    };

    let scale = match projection {
        Projection::Orthographic(proj) => proj.scale,
        _ => 1.0,
    };

    let cursor_world = match camera.viewport_to_world_2d(cam_gt, cursor_screen).ok() {
        Some(p) => p,
        None => return,
    };

    let hit_radius = (16.0f32 * scale).max(12.0f32);
    let live_pins = build_live_pin_positions_map(&graph_res.0, &card_roots, &socket_pins);

    // Helper to hit-test any pin socket at the current world position
    let find_pin_at_cursor = |world_pos: Vec2| -> Option<(PinId, NodeId, PinDirection)> {
        for (&node_id, node) in graph_res.0.nodes() {
            for &pin_id in &node.inputs {
                if let Some(pin_world) = live_pins
                    .get(&pin_id)
                    .copied()
                    .or_else(|| compute_pin_world_position(node, pin_id))
                {
                    if (world_pos - pin_world).length() <= hit_radius {
                        return Some((pin_id, node_id, PinDirection::Input));
                    }
                }
            }
            for &pin_id in &node.outputs {
                if let Some(pin_world) = live_pins
                    .get(&pin_id)
                    .copied()
                    .or_else(|| compute_pin_world_position(node, pin_id))
                {
                    if (world_pos - pin_world).length() <= hit_radius {
                        return Some((pin_id, node_id, PinDirection::Output));
                    }
                }
            }
        }
        None
    };

    // 1. Right-click on a pin: disconnect all wires attached to it
    if mouse_buttons.just_pressed(MouseButton::Right) {
        if let Some((pin_id, _, _)) = find_pin_at_cursor(cursor_world) {
            let conn_ids = graph_res.0.get_pin_connections(pin_id).to_vec();
            for conn_id in conn_ids {
                let cmd = Box::new(DisconnectCommand::new(conn_id));
                let _ = history_res.0.apply(cmd, &mut graph_res.0);
            }
            return;
        }
    }

    // 2. Left-click on a pin: start dragging new wire
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some((pin_id, _, _)) = find_pin_at_cursor(cursor_world) {
            wiring_res.0 = WiringState::DraggingNew {
                source_pin: pin_id,
                current_canvas_pos: cursor_world,
            };
        }
    }

    // 3. Left-drag: update current dragging wire end position
    if mouse_buttons.pressed(MouseButton::Left) {
        if let WiringState::DraggingNew {
            ref mut current_canvas_pos,
            ..
        } = wiring_res.0
        {
            *current_canvas_pos = cursor_world;
        }
    }

    // 4. Left-release: finalize wire connection if dropped onto a valid socket
    if mouse_buttons.just_released(MouseButton::Left) {
        if let WiringState::DraggingNew { source_pin, .. } = wiring_res.0 {
            if let Some((target_pin, _, _)) = find_pin_at_cursor(cursor_world) {
                if target_pin != source_pin {
                    let src = graph_res.0.get_pin(source_pin).cloned();
                    let tgt = graph_res.0.get_pin(target_pin).cloned();

                    if let (Some(src_p), Some(tgt_p)) = (src, tgt) {
                        let connection_pair = if src_p.direction == PinDirection::Output
                            && tgt_p.direction == PinDirection::Input
                        {
                            Some((src_p.id, tgt_p.id))
                        } else if src_p.direction == PinDirection::Input
                            && tgt_p.direction == PinDirection::Output
                        {
                            Some((tgt_p.id, src_p.id))
                        } else {
                            None
                        };

                        if let Some((out_pin, in_pin)) = connection_pair {
                            match graph_res.0.validate_connection(out_pin, in_pin) {
                                Ok(()) => {
                                    let conn_id = id_gen.0.next_connection_id();
                                    let cmd = Box::new(ConnectCommand::new(conn_id, out_pin, in_pin));
                                    let _ = history_res.0.apply(cmd, &mut graph_res.0);
                                }
                                Err(_) => {
                                    for (mut toast, mut t_node) in &mut toast_query {
                                        t_node.display = Display::Flex;
                                        toast.timer.reset();
                                    }
                                }
                            }
                        }
                    }
                }
            }
            wiring_res.0 = WiringState::Idle;
        }
    }
}
