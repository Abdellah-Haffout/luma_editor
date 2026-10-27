use luma_core::id::{ConnectionId, PinId};
use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WireRoutingStyle {
    #[default]
    Orthogonal,
    Bezier,
    Circuit,
    Stepped,
    Straight,
    Showcase,
}

impl WireRoutingStyle {
    pub const ALL: [WireRoutingStyle; 6] = [
        WireRoutingStyle::Orthogonal,
        WireRoutingStyle::Bezier,
        WireRoutingStyle::Circuit,
        WireRoutingStyle::Stepped,
        WireRoutingStyle::Straight,
        WireRoutingStyle::Showcase,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Orthogonal => "Orthogonal (Manhattan 90°)",
            Self::Bezier => "Bézier Spline (Smooth)",
            Self::Circuit => "Circuit PCB (45° Chamfer)",
            Self::Stepped => "Stepped (90° Sharp Step)",
            Self::Straight => "Straight (Direct Linear)",
            Self::Showcase => "Showcase (Mixed Shapes)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Orthogonal => "Clean 90° right-angle routing with rounded filleted corner chamfers.",
            Self::Bezier => "Smooth cubic S-curves with dynamic horizontal tangency.",
            Self::Circuit => "Hardware PCB schematic routing with 45° diagonal angled traces.",
            Self::Stepped => "Direct stepped horizontal and vertical corridor lines.",
            Self::Straight => "Minimal direct point-to-point linear wire connection.",
            Self::Showcase => "Demonstrates all wire styles simultaneously across active graph connections.",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Orthogonal => Self::Bezier,
            Self::Bezier => Self::Circuit,
            Self::Circuit => Self::Stepped,
            Self::Stepped => Self::Straight,
            Self::Straight => Self::Showcase,
            Self::Showcase => Self::Orthogonal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum WiringState {
    #[default]
    Idle,
    DraggingNew {
        source_pin: PinId,
        current_canvas_pos: Vec2,
    },
    Reconnecting {
        connection_id: ConnectionId,
        fixed_pin: PinId,
        current_canvas_pos: Vec2,
    },
}

impl WiringState {
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    pub fn is_active(&self) -> bool {
        !self.is_idle()
    }
}

/// Computes the 4 control points for a cubic Bézier spline representing a node wire.
pub fn calculate_wire_bezier(start: Vec2, end: Vec2) -> (Vec2, Vec2, Vec2, Vec2) {
    let dx = (end.x - start.x).abs() * 0.5;
    let min_tangent = 40.0;
    let tangent_offset = dx.max(min_tangent);

    let p0 = start;
    let p1 = start + Vec2::new(tangent_offset, 0.0);
    let p2 = end - Vec2::new(tangent_offset, 0.0);
    let p3 = end;

    (p0, p1, p2, p3)
}

/// Evaluates a point along a cubic Bézier curve at parameter t in [0.0, 1.0].
pub fn sample_cubic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t = t.clamp(0.0, 1.0);
    let one_minus_t = 1.0 - t;
    let one_minus_t2 = one_minus_t * one_minus_t;
    let one_minus_t3 = one_minus_t2 * one_minus_t;
    let t2 = t * t;
    let t3 = t2 * t;

    p0 * one_minus_t3
        + p1 * (3.0 * one_minus_t2 * t)
        + p2 * (3.0 * one_minus_t * t2)
        + p3 * t3
}

/// Generates a polyline sequence of points approximating the Bézier curve.
pub fn sample_wire_spline_points(start: Vec2, end: Vec2, segments: usize) -> Vec<Vec2> {
    let (p0, p1, p2, p3) = calculate_wire_bezier(start, end);
    let segments = segments.max(2);
    (0..=segments)
        .map(|i| {
            let t = i as f32 / segments as f32;
            sample_cubic_bezier(p0, p1, p2, p3, t)
        })
        .collect()
}

/// Generates direct straight line segment between two pin endpoints.
pub fn sample_straight_wire_points(start: Vec2, end: Vec2) -> Vec<Vec2> {
    vec![start, end]
}

/// Generates sharp 90° stepped routing points.
pub fn sample_stepped_wire_points(start: Vec2, end: Vec2) -> Vec<Vec2> {
    if end.x >= start.x + 16.0 {
        let mid_x = (start.x + end.x) * 0.5;
        vec![
            start,
            Vec2::new(mid_x, start.y),
            Vec2::new(mid_x, end.y),
            end,
        ]
    } else {
        // Target is backwards / to the left; route out and around with clean corridors
        let margin = 32.0;
        let mid_y = (start.y + end.y) * 0.5;
        vec![
            start,
            Vec2::new(start.x + margin, start.y),
            Vec2::new(start.x + margin, mid_y),
            Vec2::new(end.x - margin, mid_y),
            Vec2::new(end.x - margin, end.y),
            end,
        ]
    }
}

/// Generates circuit PCB routing with 45° angled chamfers.
pub fn sample_circuit_wire_points(start: Vec2, end: Vec2, chamfer: f32) -> Vec<Vec2> {
    let dy = end.y - start.y;
    if dy.abs() < 1.0 {
        return vec![start, end];
    }

    if end.x >= start.x + 16.0 {
        let mid_x = (start.x + end.x) * 0.5;
        let c = chamfer
            .min((mid_x - start.x).abs() * 0.8)
            .min(dy.abs() * 0.45);
        vec![
            start,
            Vec2::new(mid_x - c, start.y),
            Vec2::new(mid_x, start.y + dy.signum() * c),
            Vec2::new(mid_x, end.y - dy.signum() * c),
            Vec2::new(mid_x + c, end.y),
            end,
        ]
    } else {
        // Backwards routing with 45° chamfers at all 4 turns
        let margin = 32.0;
        let mid_y = (start.y + end.y) * 0.5;
        let c = chamfer.min(margin * 0.5).min((mid_y - start.y).abs() * 0.45);
        let s1 = (mid_y - start.y).signum();
        let s2 = (end.y - mid_y).signum();

        vec![
            start,
            Vec2::new(start.x + margin - c, start.y),
            Vec2::new(start.x + margin, start.y + s1 * c),
            Vec2::new(start.x + margin, mid_y - s1 * c),
            Vec2::new(start.x + margin - c, mid_y),
            Vec2::new(end.x - margin + c, mid_y),
            Vec2::new(end.x - margin, mid_y + s2 * c),
            Vec2::new(end.x - margin, end.y - s2 * c),
            Vec2::new(end.x - margin + c, end.y),
            end,
        ]
    }
}

/// Generates orthogonal (Manhattan) routing with smooth filleted corners.
pub fn sample_orthogonal_wire_points(start: Vec2, end: Vec2, radius: f32) -> Vec<Vec2> {
    let dy = end.y - start.y;
    if dy.abs() < 1.0 {
        return vec![start, end];
    }

    let arc_steps = 4;
    let mut points = Vec::with_capacity(16);
    points.push(start);

    if end.x >= start.x + 16.0 {
        let mid_x = (start.x + end.x) * 0.5;
        let r = radius
            .min((mid_x - start.x).abs() * 0.8)
            .min(dy.abs() * 0.45);

        // Lead straight into corner 1
        points.push(Vec2::new(mid_x - r, start.y));

        // Corner 1 rounded fillet (from horizontal +X to vertical towards end.y)
        let c1 = Vec2::new(mid_x - r, start.y + dy.signum() * r);
        for step in 0..=arc_steps {
            let t = step as f32 / arc_steps as f32;
            let angle = if dy > 0.0 {
                -std::f32::consts::FRAC_PI_2 + t * std::f32::consts::FRAC_PI_2
            } else {
                std::f32::consts::FRAC_PI_2 - t * std::f32::consts::FRAC_PI_2
            };
            points.push(c1 + Vec2::new(angle.cos() * r, angle.sin() * r));
        }

        // Straight vertical corridor
        points.push(Vec2::new(mid_x, end.y - dy.signum() * r));

        // Corner 2 rounded fillet (from vertical to horizontal +X)
        let c2 = Vec2::new(mid_x + r, end.y - dy.signum() * r);
        for step in 0..=arc_steps {
            let t = step as f32 / arc_steps as f32;
            let angle = if dy > 0.0 {
                std::f32::consts::PI - t * std::f32::consts::FRAC_PI_2
            } else {
                -std::f32::consts::PI + t * std::f32::consts::FRAC_PI_2
            };
            points.push(c2 + Vec2::new(angle.cos() * r, angle.sin() * r));
        }

        points.push(end);
    } else {
        // Backwards routing with rounded fillets
        let margin = 32.0;
        let mid_y = (start.y + end.y) * 0.5;
        let r = radius.min(margin * 0.5).min((mid_y - start.y).abs() * 0.45);
        let s1 = (mid_y - start.y).signum();
        let s2 = (end.y - mid_y).signum();

        // Corner 1: Right to Mid-Y corridor
        points.push(Vec2::new(start.x + margin - r, start.y));
        let c1 = Vec2::new(start.x + margin - r, start.y + s1 * r);
        for step in 0..=arc_steps {
            let t = step as f32 / arc_steps as f32;
            let angle = if s1 > 0.0 {
                -std::f32::consts::FRAC_PI_2 + t * std::f32::consts::FRAC_PI_2
            } else {
                std::f32::consts::FRAC_PI_2 - t * std::f32::consts::FRAC_PI_2
            };
            points.push(c1 + Vec2::new(angle.cos() * r, angle.sin() * r));
        }

        // Corner 2: Turn left towards target column
        points.push(Vec2::new(start.x + margin, mid_y - s1 * r));
        let c2 = Vec2::new(start.x + margin - r, mid_y - s1 * r);
        for step in 0..=arc_steps {
            let t = step as f32 / arc_steps as f32;
            let angle = if s1 > 0.0 {
                0.0 + t * std::f32::consts::FRAC_PI_2
            } else {
                0.0 - t * std::f32::consts::FRAC_PI_2
            };
            points.push(c2 + Vec2::new(angle.cos() * r, angle.sin() * r));
        }

        // Corner 3: Turn vertical towards target row
        points.push(Vec2::new(end.x - margin + r, mid_y));
        let c3 = Vec2::new(end.x - margin + r, mid_y + s2 * r);
        for step in 0..=arc_steps {
            let t = step as f32 / arc_steps as f32;
            let angle = if s2 > 0.0 {
                -std::f32::consts::FRAC_PI_2 - t * std::f32::consts::FRAC_PI_2
            } else {
                std::f32::consts::FRAC_PI_2 + t * std::f32::consts::FRAC_PI_2
            };
            points.push(c3 + Vec2::new(angle.cos() * r, angle.sin() * r));
        }

        // Corner 4: Turn horizontal into target pin
        points.push(Vec2::new(end.x - margin, end.y - s2 * r));
        let c4 = Vec2::new(end.x - margin + r, end.y - s2 * r);
        for step in 0..=arc_steps {
            let t = step as f32 / arc_steps as f32;
            let angle = if s2 > 0.0 {
                std::f32::consts::PI - t * std::f32::consts::FRAC_PI_2
            } else {
                -std::f32::consts::PI + t * std::f32::consts::FRAC_PI_2
            };
            points.push(c4 + Vec2::new(angle.cos() * r, angle.sin() * r));
        }

        points.push(end);
    }

    points
}

/// Unified point sampler supporting all wire routing styles.
pub fn sample_wire_points(start: Vec2, end: Vec2, style: WireRoutingStyle) -> Vec<Vec2> {
    match style {
        WireRoutingStyle::Bezier => sample_wire_spline_points(start, end, 24),
        WireRoutingStyle::Orthogonal => sample_orthogonal_wire_points(start, end, 16.0),
        WireRoutingStyle::Circuit => sample_circuit_wire_points(start, end, 14.0),
        WireRoutingStyle::Stepped => sample_stepped_wire_points(start, end),
        WireRoutingStyle::Straight => sample_straight_wire_points(start, end),
        WireRoutingStyle::Showcase => sample_wire_spline_points(start, end, 24),
    }
}
