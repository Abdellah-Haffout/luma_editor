// Luma SDF Multi-Style Connection Wire Shader
// - Supports all wire styles: Bézier (smooth cubic SDF) and Polyline (Orthogonal, Circuit, Stepped, Straight)
// - Razor-sharp, clean, non-neon stroked curve
// - Physical world thickness scaled with camera zoom
// - Automatic node card occlusion (discarding fragments under node cards)
// - Smooth two-pole color gradient (source pin type -> target pin type)
// - Clean traveling photon pulse along the path

import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct LumaConnectionUniforms {
    p0: vec2<f32>,
    p1: vec2<f32>,
    p2: vec2<f32>,
    p3: vec2<f32>,
    src_color: vec4<f32>,
    tgt_color: vec4<f32>,
    width: f32,
    feather: f32,
    pulse_pos: f32,
    pulse_enabled: f32,
    dimmed: f32,
    routing_mode: f32,
    num_points: f32,
    num_occluders: f32,
    poly_points: array<vec4<f32>, 12>,
    occluders: array<vec4<f32>, 16>,
};

@group(1) @binding(0)
var<uniform> u: LumaConnectionUniforms;

fn get_poly_point(idx: u32) -> vec2<f32> {
    let vec_idx = idx / 2u;
    let pair = u.poly_points[vec_idx];
    if (idx % 2u == 0u) {
        return pair.xy;
    } else {
        return pair.zw;
    }
}

// Inigo Quilez: signed distance from point `pos` to quadratic Bezier (A, B, C).
// Returns vec2<f32>(distance, t_parameter).
fn sd_bezier_quadratic(pos: vec2<f32>, A: vec2<f32>, B: vec2<f32>, C: vec2<f32>) -> vec2<f32> {
    let a = B - A;
    let b = A - 2.0 * B + C;
    let c = a * 2.0;
    let d = A - pos;

    let bb = dot(b, b);
    if (bb < 1.0e-6) {
        let ba = C - A;
        let t = clamp(dot(pos - A, ba) / max(dot(ba, ba), 1.0e-6), 0.0, 1.0);
        return vec2<f32>(length((A - pos) + ba * t), t);
    }

    let kk = 1.0 / bb;
    let kx = kk * dot(a, b);
    let ky = kk * (2.0 * dot(a, a) + dot(d, b)) / 3.0;
    let kz = kk * dot(d, a);

    let p = ky - kx * kx;
    let q = kx * (2.0 * kx * kx - 3.0 * ky) + kz;
    let p3 = p * p * p;
    let q2 = q * q;
    let h = q2 + 4.0 * p3;

    var res: f32;
    var best_t: f32;
    if (h >= 0.0) {
        let h_sqrt = sqrt(h);
        let x = (vec2<f32>(h_sqrt, -h_sqrt) - q) * 0.5;
        let uv = sign(x) * pow(abs(x), vec2<f32>(1.0 / 3.0));
        let t = clamp(uv.x + uv.y - kx, 0.0, 1.0);
        let qpt = d + (c + b * t) * t;
        res = dot(qpt, qpt);
        best_t = t;
    } else {
        let z = sqrt(-p);
        let v = acos(q / (p * z * 2.0)) / 3.0;
        let m = cos(v);
        let n = sin(v) * 1.732050808;
        let t3 = clamp(vec3<f32>(m + m, -n - m, n - m) * z - kx, vec3<f32>(0.0), vec3<f32>(1.0));
        let q0 = d + (c + b * t3.x) * t3.x;
        let q1 = d + (c + b * t3.y) * t3.y;
        let q2v = d + (c + b * t3.z) * t3.z;
        let d0 = dot(q0, q0);
        let d1 = dot(q1, q1);
        let d2 = dot(q2v, q2v);
        if (d0 <= d1 && d0 <= d2) {
            res = d0;
            best_t = t3.x;
        } else if (d1 <= d2) {
            res = d1;
            best_t = t3.y;
        } else {
            res = d2;
            best_t = t3.z;
        }
    }

    return vec2<f32>(sqrt(res), best_t);
}

// De Casteljau split of cubic Bezier at t = 0.5 into two quadratic Beziers.
fn split_cubic(
    p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>,
) -> array<vec2<f32>, 6> {
    let q0 = mix(p0, p1, 0.5);
    let q1 = mix(p1, p2, 0.5);
    let q2 = mix(p2, p3, 0.5);
    let r0 = mix(q0, q1, 0.5);
    let r1 = mix(q1, q2, 0.5);
    let mid = mix(r0, r1, 0.5);
    return array<vec2<f32>, 6>(
        p0, q0, mid,
        mid, q2, p3,
    );
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let pos = in.uv * in.size;

    // 1. Occlusion test: Discard fragment if it lies inside any node card
    let count = min(u32(u.num_occluders), 16u);
    for (var i = 0u; i < count; i = i + 1u) {
        let r = u.occluders[i];
        if (pos.x >= r.x && pos.x <= r.z && pos.y >= r.y && pos.y <= r.w) {
            discard;
        }
    }

    var d = 1.0e6;
    var global_t = 0.0;

    if (u.routing_mode < 0.5) {
        // Mode 0: Analytic Cubic Bézier
        let split = split_cubic(u.p0, u.p1, u.p2, u.p3);
        let r0 = sd_bezier_quadratic(pos, split[0], split[1], split[2]);
        let r1 = sd_bezier_quadratic(pos, split[3], split[4], split[5]);

        if (r0.x <= r1.x) {
            d = r0.x;
            global_t = r0.y * 0.5;
        } else {
            d = r1.x;
            global_t = 0.5 + r1.y * 0.5;
        }
    } else {
        // Mode 1: Polyline (Orthogonal, Circuit, Stepped, Straight)
        let n_pts = min(u32(u.num_points), 24u);
        if (n_pts >= 2u) {
            let n_segs = n_pts - 1u;
            var min_d = 1.0e6;
            var best_t = 0.0;
            var seg_dists: array<f32, 23>;
            var total_len = 0.0;

            for (var i = 0u; i < n_segs; i = i + 1u) {
                let a = get_poly_point(i);
                let b = get_poly_point(i + 1u);
                let l = length(b - a);
                seg_dists[i] = total_len;
                total_len = total_len + l;
            }

            let safe_total = max(total_len, 1.0e-4);

            for (var i = 0u; i < n_segs; i = i + 1u) {
                let a = get_poly_point(i);
                let b = get_poly_point(i + 1u);
                let pa = pos - a;
                let ba = b - a;
                let ba_sq = dot(ba, ba);
                var h = 0.0;
                if (ba_sq > 1.0e-5) {
                    h = clamp(dot(pa, ba) / ba_sq, 0.0, 1.0);
                }
                let dist = length(pa - ba * h);
                if (dist < min_d) {
                    min_d = dist;
                    let seg_len = length(ba);
                    best_t = (seg_dists[i] + h * seg_len) / safe_total;
                }
            }
            d = min_d;
            global_t = clamp(best_t, 0.0, 1.0);
        }
    }

    // 2. Razor-sharp solid stroke (No fuzzy neon glow!)
    let half_width = u.width * 0.5;
    let edge = d - half_width;
    // Crisp sub-pixel hardware anti-aliasing (0.6px - 1.0px transition)
    let aa = max(fwidth(edge) * 0.75, 0.6);
    let alpha = 1.0 - smoothstep(-aa, aa, edge);

    if (alpha <= 0.01) {
        discard;
    }

    // 3. Two-pole type color gradient
    let base_color = mix(u.src_color, u.tgt_color, global_t);

    // 4. Clean sharp energy pulse along the wire path
    var wire_rgb = base_color.rgb;
    if (u.pulse_enabled > 0.5) {
        let dt = abs(global_t - u.pulse_pos);
        let pulse_len = 0.045;
        if (dt < pulse_len) {
            let s = 1.0 - (dt / pulse_len);
            wire_rgb = mix(wire_rgb, vec3<f32>(1.0, 1.0, 1.0), s * 0.65);
        }
    }

    let dim_mult = mix(1.0, 0.2, u.dimmed);
    return vec4<f32>(wire_rgb, alpha * base_color.a * dim_mult);
}
