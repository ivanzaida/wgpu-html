// Glyph pipeline: instanced textured quads. The atlas is a single-
// channel `R8Unorm` mask; each glyph quad samples it and multiplies by
// the per-instance color (RGB + alpha). Premultiplied-alpha blending is
// done by the pipeline blend state.

struct Globals {
    viewport: vec2<f32>,
    _pad: vec2<f32>,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var atlas: texture_2d<f32>;
@group(0) @binding(2) var atlas_sampler: sampler;

struct VsIn {
    @location(0) corner: vec2<f32>, // unit-quad corner in [0,1]
    @location(1) pos: vec2<f32>,    // top-left in physical pixels
    @location(2) size: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) uv_min: vec2<f32>,
    @location(5) uv_max: vec2<f32>,
}

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VsIn) -> VsOut {
    let world = in.pos + in.corner * in.size;
    let ndc_x = (world.x / globals.viewport.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (world.y / globals.viewport.y) * 2.0;
    let uv = mix(in.uv_min, in.uv_max, in.corner);

    var out: VsOut;
    out.clip = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.color = in.color;
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // Coverage from cosmic-text / swash is perceptually-weighted (the
    // rasteriser's "looks like X% ink" estimate). Blending it raw on
    // an sRGB surface — where the GPU blends in linear space then
    // sRGB-encodes for display — washes out anti-aliased strokes: a
    // pixel marked 50%-covered ends up rendered as ~74%-gray on
    // screen instead of the ~50% the rasteriser asked for, and small
    // text reads gray.
    //
    // Lift the curve so that after the linear blend + sRGB encode the
    // visual weight matches what gamma-space blending would have
    // produced. Exponent 1/1.43 is the empirical "reduced gamma" fit
    // Skia and most UI text engines use for a single-channel alpha
    // mask path.
    let raw = textureSample(atlas, atlas_sampler, in.uv).r;
    let coverage = pow(raw, 1.0 / 1.43);
    return vec4<f32>(in.color.rgb, in.color.a * coverage);
}
