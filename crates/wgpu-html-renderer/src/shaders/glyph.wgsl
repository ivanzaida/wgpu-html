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

// Per-channel sRGB encode: linear → display-space byte value. Used
// below so the foreground colour lives in the same gamma space as the
// surface bytes the glyph pass blends against.
fn srgb_encode_one(c: f32) -> f32 {
    if (c <= 0.0031308) {
        return c * 12.92;
    }
    return 1.055 * pow(c, 1.0 / 2.4) - 0.055;
}

fn srgb_encode(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(srgb_encode_one(c.x), srgb_encode_one(c.y), srgb_encode_one(c.z));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // The glyph pass renders through a non-sRGB view of the surface,
    // so the GPU's blend step doesn't decode `dst` from sRGB and
    // doesn't sRGB-encode the result on write — both sides see the
    // raw byte values, i.e. the *display-space* representation. To
    // composite text correctly we therefore write the foreground
    // colour in display space too: encode `in.color.rgb` (which
    // arrives as linear from `resolve_color`) into sRGB before the
    // blend. Coverage from the rasteriser is already perceptually
    // weighted in 0..1 and feeds straight into alpha — no curve hack.
    let coverage = textureSample(atlas, atlas_sampler, in.uv).r;
    let rgb_srgb = srgb_encode(in.color.rgb);
    return vec4<f32>(rgb_srgb, in.color.a * coverage);
}
