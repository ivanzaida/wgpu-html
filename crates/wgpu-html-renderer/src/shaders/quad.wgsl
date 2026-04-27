// Rounded-corner quad pipeline.
//
// Vertex buffer 0 (per-vertex):  unit quad corner in [0,1]^2.
// Vertex buffer 1 (per-instance): pos / size in pixels, linear RGBA color,
//                                 per-corner radii (TL, TR, BR, BL), and
//                                 per-side ring thickness (top, right,
//                                 bottom, left).
// Bind group 0 / binding 0: viewport size in pixels.
//
// Two modes selected by whether any stroke component is > 0:
//   - Filled:    paint the entire (rounded) box with `color`.
//   - Stroked:   paint only the ring between the outer rounded box and
//                an inner rounded box inset on each side by the matching
//                stroke width. The inner radius is clamped to >= 0.
// In both modes a ~1-pixel anti-alias band keeps edges smooth.

struct Globals {
    viewport: vec2<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;

struct VsIn {
    @location(0) corner: vec2<f32>,
    @location(1) pos:    vec2<f32>,
    @location(2) size:   vec2<f32>,
    @location(3) color:  vec4<f32>,
    @location(4) radii:  vec4<f32>,  // TL, TR, BR, BL
    @location(5) stroke: vec4<f32>,  // top, right, bottom, left
};

struct VsOut {
    @builtin(position) clip:      vec4<f32>,
    @location(0)       color:     vec4<f32>,
    /// Pixel offset from the box's centre (so abs(local) <= half_size).
    @location(1)       local:     vec2<f32>,
    @location(2)       half_size: vec2<f32>,
    @location(3)       radii:     vec4<f32>,
    @location(4)       stroke:    vec4<f32>,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
    let px = in.pos + in.corner * in.size;
    let ndc = vec2<f32>(
        (px.x / globals.viewport.x) * 2.0 - 1.0,
        1.0 - (px.y / globals.viewport.y) * 2.0,
    );

    var out: VsOut;
    out.clip      = vec4<f32>(ndc, 0.0, 1.0);
    out.color     = in.color;
    out.half_size = in.size * 0.5;
    // local = pixel offset from centre. corner=(0,0)→-half, (1,1)→+half.
    out.local     = (in.corner - vec2<f32>(0.5, 0.5)) * in.size;
    out.radii     = in.radii;
    out.stroke    = in.stroke;
    return out;
}

/// Pick the radius for whichever quadrant `p` lies in.
/// `radii` order: TL, TR, BR, BL. y < 0 means upper half (top-left coords).
fn pick_radius(p: vec2<f32>, radii: vec4<f32>) -> f32 {
    if (p.y < 0.0) {
        if (p.x < 0.0) { return radii.x; } // TL
        else           { return radii.y; } // TR
    } else {
        if (p.x < 0.0) { return radii.w; } // BL
        else           { return radii.z; } // BR
    }
}

/// Signed distance from `p` (centre-relative) to a rounded box of
/// half-extent `half_size` and radius `r` for this quadrant. Negative
/// inside, zero on the edge, positive outside.
fn sd_rounded_box(p: vec2<f32>, half_size: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - half_size + vec2<f32>(r, r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0, 0.0))) - r;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let outer_r = pick_radius(in.local, in.radii);
    let outer_dist = sd_rounded_box(in.local, in.half_size, outer_r);

    let aa = 0.7;
    let max_stroke = max(max(in.stroke.x, in.stroke.y), max(in.stroke.z, in.stroke.w));

    var dist: f32;
    if (max_stroke <= 0.0) {
        // Filled mode.
        dist = outer_dist;
    } else {
        // Ring mode. Build the inner rounded box: each side is inset by
        // its stroke thickness, so the inner half-size shrinks by half
        // the sum of opposite strokes, and the centre shifts when the
        // strokes are asymmetric.
        let inner_half = vec2<f32>(
            in.half_size.x - 0.5 * (in.stroke.y + in.stroke.w),
            in.half_size.y - 0.5 * (in.stroke.x + in.stroke.z),
        );
        let inner_centre = vec2<f32>(
            0.5 * (in.stroke.w - in.stroke.y),
            0.5 * (in.stroke.x - in.stroke.z),
        );
        // Inner radius = outer minus the larger of the adjacent strokes,
        // clamped at 0. Using `max_stroke` is the safe conservative
        // choice when sides differ.
        let inner_r = max(0.0, outer_r - max_stroke);
        let inner_dist = sd_rounded_box(in.local - inner_centre, inner_half, inner_r);
        // Ring distance: outside if not in outer OR if inside inner.
        dist = max(outer_dist, -inner_dist);
    }

    let alpha = clamp(0.5 - dist / aa, 0.0, 1.0);
    if (alpha <= 0.0) {
        discard;
    }
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
