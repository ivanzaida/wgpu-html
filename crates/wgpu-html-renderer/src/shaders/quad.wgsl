// Rounded-corner quad pipeline (elliptical, optional ring).
//
// Vertex buffer 0 (per-vertex):  unit quad corner in [0,1]^2.
// Vertex buffer 1 (per-instance): pos / size in pixels, linear RGBA color,
//                                 horizontal corner radii (TL, TR, BR, BL),
//                                 vertical corner radii (same order),
//                                 per-side ring thickness (top, right,
//                                 bottom, left).
// Bind group 0 / binding 0: viewport size in pixels.
//
// Two modes selected by whether any stroke component is > 0:
//   - Filled:    paint the entire (rounded) box with `color`.
//   - Stroked:   paint only the ring between the outer rounded box and
//                an inner rounded box inset on each side by the matching
//                stroke width.
// In both modes a ~1-pixel anti-alias band keeps edges smooth.
//
// The corner zone uses a gradient-corrected ellipse SDF so corners can
// be elliptical (h != v); when h == v it reduces to the usual circular
// case.

struct Globals {
    viewport: vec2<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;

struct VsIn {
    @location(0) corner:  vec2<f32>,
    @location(1) pos:     vec2<f32>,
    @location(2) size:    vec2<f32>,
    @location(3) color:   vec4<f32>,
    @location(4) radii_h: vec4<f32>,  // TL, TR, BR, BL
    @location(5) radii_v: vec4<f32>,  // TL, TR, BR, BL
    @location(6) stroke:  vec4<f32>,  // top, right, bottom, left
};

struct VsOut {
    @builtin(position) clip:      vec4<f32>,
    @location(0)       color:     vec4<f32>,
    /// Pixel offset from the box's centre.
    @location(1)       local:     vec2<f32>,
    @location(2)       half_size: vec2<f32>,
    @location(3)       radii_h:   vec4<f32>,
    @location(4)       radii_v:   vec4<f32>,
    @location(5)       stroke:    vec4<f32>,
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
    out.local     = (in.corner - vec2<f32>(0.5, 0.5)) * in.size;
    out.radii_h   = in.radii_h;
    out.radii_v   = in.radii_v;
    out.stroke    = in.stroke;
    return out;
}

/// Pick the (h, v) radius pair for whichever quadrant `p` lies in.
/// Order: TL, TR, BR, BL (matches CSS `border-radius` longhand order).
fn pick_radius(p: vec2<f32>, rh: vec4<f32>, rv: vec4<f32>) -> vec2<f32> {
    if (p.y < 0.0) {
        if (p.x < 0.0) { return vec2<f32>(rh.x, rv.x); } // TL
        else           { return vec2<f32>(rh.y, rv.y); } // TR
    } else {
        if (p.x < 0.0) { return vec2<f32>(rh.w, rv.w); } // BL
        else           { return vec2<f32>(rh.z, rv.z); } // BR
    }
}

/// SDF of a box with elliptical corners (radius `r = (rx, ry)` per
/// quadrant). `p` is centre-relative pixel coords; `half_size` is the
/// box half-extent.
///
/// - Corner zone (both q components > 0) → gradient-corrected ellipse
///   SDF. Reduces to the exact circular formula `length(q) - r` when
///   rx == ry; smooth approximation otherwise.
/// - Edge band / interior → rectangle distance with the corner radii
///   subtracted on each axis: `max(q.x - rx, q.y - ry)`.
fn sd_rounded_box(p: vec2<f32>, half_size: vec2<f32>, r: vec2<f32>) -> f32 {
    let safe_r = max(r, vec2<f32>(0.0, 0.0));
    let q = abs(p) - half_size + safe_r;

    if (q.x > 0.0 && q.y > 0.0) {
        // Corner zone.
        if (safe_r.x <= 0.0 || safe_r.y <= 0.0) {
            return max(q.x, q.y);
        }
        let pn = q / safe_r;
        let l = length(pn);
        let g = max(length(pn / safe_r), 1e-6);
        // Euclidean distance estimate = (length(pn) - 1) / |grad|, where
        // |grad| = length(pn / r) / length(pn). The factor of `l` in the
        // numerator below absorbs the `length(pn)` in the denominator
        // of |grad|. Reduces to `length(q) - r` for circular corners.
        return (l - 1.0) * l / g;
    }

    // Edge band or interior. The `- safe_r.{x,y}` subtraction undoes the
    // `+ safe_r` we added when computing `q`, which keeps the corner
    // zone consistent with the rounded-box shape.
    return max(q.x - safe_r.x, q.y - safe_r.y);
}

/// Count how many sides have a positive stroke width.
fn nonzero_side_count(s: vec4<f32>) -> i32 {
    var n: i32 = 0;
    if (s.x > 0.0) { n = n + 1; }
    if (s.y > 0.0) { n = n + 1; }
    if (s.z > 0.0) { n = n + 1; }
    if (s.w > 0.0) { n = n + 1; }
    return n;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let outer_r = pick_radius(in.local, in.radii_h, in.radii_v);
    let outer_dist = sd_rounded_box(in.local, in.half_size, outer_r);

    let aa = 0.7;
    let max_stroke = max(max(in.stroke.x, in.stroke.y), max(in.stroke.z, in.stroke.w));

    if (max_stroke <= 0.0) {
        // Filled mode.
        let alpha = clamp(0.5 - outer_dist / aa, 0.0, 1.0);
        if (alpha <= 0.0) { discard; }
        return vec4<f32>(in.color.rgb, in.color.a * alpha);
    }

    let nz = nonzero_side_count(in.stroke);
    var inner_half: vec2<f32>;
    var inner_centre: vec2<f32>;

    if (nz == 1) {
        // One-sided ring: use a uniform inner box (concentric with the
        // outer one) so the painted stroke width stays constant along
        // the curve. The visible region is then trimmed below to the
        // 45° wedge belonging to this side.
        inner_half   = in.half_size - vec2<f32>(max_stroke, max_stroke);
        inner_centre = vec2<f32>(0.0, 0.0);
    } else {
        // Multi-side ring (uniform when all four equal; possibly
        // asymmetric otherwise). Inner box shrinks per-side and shifts
        // when sides are unequal.
        inner_half = vec2<f32>(
            in.half_size.x - 0.5 * (in.stroke.y + in.stroke.w),
            in.half_size.y - 0.5 * (in.stroke.x + in.stroke.z),
        );
        inner_centre = vec2<f32>(
            0.5 * (in.stroke.w - in.stroke.y),
            0.5 * (in.stroke.x - in.stroke.z),
        );
    }

    let inner_r = vec2<f32>(
        max(0.0, outer_r.x - max_stroke),
        max(0.0, outer_r.y - max_stroke),
    );
    let inner_dist = sd_rounded_box(in.local - inner_centre, inner_half, inner_r);
    let dist = max(outer_dist, -inner_dist);

    // For a one-sided ring, restrict to the side's 45° wedge so the
    // adjacent sides don't bleed into this draw call. Wedges are taken
    // in normalised (-1..1) coords so non-square boxes still split into
    // four equal triangles, mitering at the box centre.
    if (nz == 1) {
        let nx = in.local.x / max(in.half_size.x, 1e-6);
        let ny = in.local.y / max(in.half_size.y, 1e-6);
        var inside = false;
        if (in.stroke.x > 0.0 && ny <= -abs(nx)) { inside = true; } // top
        if (in.stroke.y > 0.0 && nx >=  abs(ny)) { inside = true; } // right
        if (in.stroke.z > 0.0 && ny >=  abs(nx)) { inside = true; } // bottom
        if (in.stroke.w > 0.0 && nx <= -abs(ny)) { inside = true; } // left
        if (!inside) { discard; }
    }

    let alpha = clamp(0.5 - dist / aa, 0.0, 1.0);
    if (alpha <= 0.0) {
        discard;
    }
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
