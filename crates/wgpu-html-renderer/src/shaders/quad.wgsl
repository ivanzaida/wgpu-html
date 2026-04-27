// Rounded-corner quad pipeline.
//
// Vertex buffer 0 (per-vertex):  unit quad corner in [0,1]^2.
// Vertex buffer 1 (per-instance): pos / size in pixels, linear RGBA color,
//                                 per-corner radii (TL, TR, BR, BL).
// Bind group 0 / binding 0: viewport size in pixels.
//
// The fragment shader computes a signed distance from the rounded
// rectangle and alpha-blends a 1-pixel anti-alias band so edges stay
// smooth at any radius. A quad with all-zero radii degenerates to a
// sharp axis-aligned rectangle (alpha == 1 everywhere inside).

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
};

struct VsOut {
    @builtin(position) clip:      vec4<f32>,
    @location(0)       color:     vec4<f32>,
    /// Pixel offset from the box's centre (so abs(local) <= half_size).
    @location(1)       local:     vec2<f32>,
    @location(2)       half_size: vec2<f32>,
    @location(3)       radii:     vec4<f32>,
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
    let r = pick_radius(in.local, in.radii);
    let dist = sd_rounded_box(in.local, in.half_size, r);

    // Anti-alias band: ~1 pixel wide. fwidth would be ideal but is not
    // available without `derivative_uniformity`; a fixed 0.7 covers the
    // typical AA region well at integer scales.
    let aa = 0.7;
    let alpha = clamp(0.5 - dist / aa, 0.0, 1.0);
    if (alpha <= 0.0) {
        discard;
    }
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
