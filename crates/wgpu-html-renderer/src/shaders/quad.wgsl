// Solid color quad pipeline.
//
// Vertex buffer 0 (per-vertex, step = Vertex): unit quad corner in [0,1]^2.
// Vertex buffer 1 (per-instance, step = Instance): pos, size in pixels + linear RGBA.
// Bind group 0 / binding 0: viewport size in pixels.

struct Globals {
    viewport: vec2<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;

struct VsIn {
    @location(0) corner: vec2<f32>,
    @location(1) pos:    vec2<f32>,
    @location(2) size:   vec2<f32>,
    @location(3) color:  vec4<f32>,
};

struct VsOut {
    @builtin(position) clip:  vec4<f32>,
    @location(0)       color: vec4<f32>,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
    // Pixel coordinates of this vertex (top-left origin, +Y down).
    let px = in.pos + in.corner * in.size;

    // Convert pixels to clip space: x in [-1, 1], y flipped.
    let ndc = vec2<f32>(
        (px.x / globals.viewport.x) * 2.0 - 1.0,
        1.0 - (px.y / globals.viewport.y) * 2.0,
    );

    var out: VsOut;
    out.clip  = vec4<f32>(ndc, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return in.color;
}
