//! Textured glyph pipeline. Mirrors `QuadPipeline` in shape, but
//! samples a single `R8Unorm` atlas instead of doing SDF math, and
//! uses a 3-binding bind group (globals uniform, atlas texture, sampler).

use bytemuck::{Pod, Zeroable};

use super::paint::{DisplayList, GlyphQuad};

/// Unit-quad geometry (shared semantics with quad pipeline; kept local
/// so the two pipelines don't accidentally couple).
const UNIT_QUAD: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
const UNIT_QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 1, 3];

/// Per-clip-range uniform block. Mirrors `quad_pipeline::Globals`
/// — vertex stage uses `viewport.xy`, fragment stage uses
/// `clip_*` for the rounded-SDF discard.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Globals {
  viewport: [f32; 4],
  clip_rect: [f32; 4],
  clip_radii_h: [f32; 4],
  clip_radii_v: [f32; 4],
  clip_active: [f32; 4],
}

const CLIP_SLOT_STRIDE: u64 = 256;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct GlyphInstance {
  pos: [f32; 2],
  size: [f32; 2],
  color: [f32; 4],
  uv_min: [f32; 2],
  uv_max: [f32; 2],
  transform: [f32; 4],
  transform_origin: [f32; 2],
  _pad: [f32; 2],
}

impl From<&GlyphQuad> for GlyphInstance {
  fn from(g: &GlyphQuad) -> Self {
    Self {
      pos: [g.rect.x, g.rect.y],
      size: [g.rect.w, g.rect.h],
      color: g.color,
      uv_min: g.uv_min,
      uv_max: g.uv_max,
      transform: g.transform,
      transform_origin: g.transform_origin,
      _pad: [0.0; 2],
    }
  }
}

pub struct GlyphPipeline {
  pipeline: wgpu::RenderPipeline,
  bind_group: wgpu::BindGroup,
  globals_buffer: wgpu::Buffer,
  /// Held to keep the underlying GPU object alive for the bind
  /// group's lifetime. Not accessed directly.
  #[allow(dead_code)]
  sampler: wgpu::Sampler,
  /// `R8Unorm` atlas owned by the pipeline. The CPU-side atlas in
  /// `lui-text` writes into this via `Atlas::upload`.
  atlas_texture: wgpu::Texture,
  atlas_size: u32,
  vertex_buffer: wgpu::Buffer,
  index_buffer: wgpu::Buffer,
  instance_buffer: wgpu::Buffer,
  instance_capacity: u64,
  instance_count: u32,
  /// Mirror of `QuadPipeline::clip_runs` — one entry per
  /// `(scissor, instance_range)` slice produced by the paint-time
  /// clip stack. Kept on the pipeline to avoid threading the
  /// `DisplayList` through the render pass.
  clip_runs: Vec<GlyphClipRun>,
  clip_slots: Vec<Option<GlyphClipRun>>,
  viewport: [u32; 2],
  /// Layout reused at `prepare` time when the globals buffer
  /// grows and the bind group has to be re-built around it.
  bind_group_layout: wgpu::BindGroupLayout,
  /// Capacity of `globals_buffer` in 256-byte clip slots.
  globals_capacity_slots: u64,
}

#[derive(Clone, Copy)]
struct GlyphClipRun {
  rect: [u32; 4],
  instances: (u32, u32),
  slot: u32,
}

impl GlyphPipeline {
  pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat, atlas_size: u32) -> Self {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("glyph shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("shaders/glyph.wgsl").into()),
    });

    let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("glyph atlas"),
      size: wgpu::Extent3d {
        width: atlas_size,
        height: atlas_size,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::R8Unorm,
      usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
      view_formats: &[],
    });
    let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("glyph atlas sampler"),
      mag_filter: wgpu::FilterMode::Nearest,
      min_filter: wgpu::FilterMode::Nearest,
      mipmap_filter: wgpu::MipmapFilterMode::Nearest,
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      ..Default::default()
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("glyph bgl"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          // Fragment now reads the clip data too.
          visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            // Dynamic offset → one slot per clip range.
            has_dynamic_offset: true,
            min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<Globals>() as u64),
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 2,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
      ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("glyph pl"),
      bind_group_layouts: &[Some(&bind_group_layout)],
      immediate_size: 0,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("glyph pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: Some("vs_main"),
        compilation_options: Default::default(),
        buffers: &[
          // Per-vertex: unit-quad corner.
          wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 2]>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
              format: wgpu::VertexFormat::Float32x2,
              offset: 0,
              shader_location: 0,
            }],
          },
          // Per-instance: pos, size, color, uv_min, uv_max.
          wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GlyphInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
              wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 1,
              },
              wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 8,
                shader_location: 2,
              },
              wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 16,
                shader_location: 3,
              },
              wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 32,
                shader_location: 4,
              },
              wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 40,
                shader_location: 5,
              },
              wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 48,
                shader_location: 6,
              },
              wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 64,
                shader_location: 7,
              },
            ],
          },
        ],
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: Some("fs_main"),
        compilation_options: Default::default(),
        targets: &[Some(wgpu::ColorTargetState {
          format: surface_format,
          blend: Some(wgpu::BlendState::ALPHA_BLENDING),
          write_mask: wgpu::ColorWrites::ALL,
        })],
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: None,
        unclipped_depth: false,
        polygon_mode: wgpu::PolygonMode::Fill,
        conservative: false,
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      multiview_mask: None,
      cache: None,
    });

    let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("glyph globals"),
      // One slot up-front; `prepare` grows the buffer to fit
      // the current frame's clip-range count.
      size: CLIP_SLOT_STRIDE,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("glyph bg"),
      layout: &bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          // Dynamic-offset window of one `Globals` worth.
          resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &globals_buffer,
            offset: 0,
            size: wgpu::BufferSize::new(std::mem::size_of::<Globals>() as u64),
          }),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::TextureView(&atlas_view),
        },
        wgpu::BindGroupEntry {
          binding: 2,
          resource: wgpu::BindingResource::Sampler(&sampler),
        },
      ],
    });

    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("glyph vb"),
      size: std::mem::size_of_val(&UNIT_QUAD) as u64,
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("glyph ib"),
      size: std::mem::size_of_val(&UNIT_QUAD_INDICES) as u64,
      usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    let initial_instances: u64 = 256;
    let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("glyph instances"),
      size: initial_instances * std::mem::size_of::<GlyphInstance>() as u64,
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    Self {
      pipeline,
      bind_group,
      globals_buffer,
      sampler,
      atlas_texture,
      atlas_size,
      vertex_buffer,
      index_buffer,
      instance_buffer,
      instance_capacity: initial_instances,
      instance_count: 0,
      clip_runs: Vec::new(),
      clip_slots: Vec::new(),
      viewport: [0, 0],
      bind_group_layout,
      globals_capacity_slots: 1,
    }
  }

  pub fn upload_static(&self, queue: &wgpu::Queue) {
    queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&UNIT_QUAD));
    queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&UNIT_QUAD_INDICES));
  }

  pub fn atlas_size(&self) -> u32 {
    self.atlas_size
  }

  /// Borrow the atlas texture so the host can upload glyph rasters
  /// into it (e.g. `lui-text::Atlas::upload`).
  pub fn atlas_texture(&self) -> &wgpu::Texture {
    &self.atlas_texture
  }

  pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, viewport: [f32; 2], list: &DisplayList) {
    let count = list.glyphs.len() as u32;
    self.instance_count = count;
    let s = list.dpi_scale.max(1.0);
    self.viewport = [
      (viewport[0] * s).max(0.0).round() as u32,
      (viewport[1] * s).max(0.0).round() as u32,
    ];

    // Build per-clip-range globals + clip runs. Empty ranges are
    // skipped; an all-empty list still gets one default slot so
    // record() can bind something.
    self.clip_runs.clear();
    self.clip_slots.clear();
    self.clip_slots.resize(list.clips.len(), None);
    let mut globals_blocks: Vec<Globals> = Vec::new();
    for (clip_index, clip) in list.clips.iter().enumerate() {
      if clip.glyph_range.0 == clip.glyph_range.1 {
        continue;
      }
      let rect = clamp_scissor_rect(clip.rect, self.viewport, s);
      let slot = globals_blocks.len() as u32;
      globals_blocks.push(globals_for(viewport, clip));
      let run = GlyphClipRun {
        rect,
        instances: clip.glyph_range,
        slot,
      };
      self.clip_runs.push(run);
      self.clip_slots[clip_index] = Some(run);
    }
    if globals_blocks.is_empty() {
      globals_blocks.push(globals_for(
        viewport,
        &super::paint::ClipRange {
          rect: None,
          radii_h: [0.0; 4],
          radii_v: [0.0; 4],
          quad_range: (0, 0),
          image_range: (0, 0),
          glyph_range: (0, 0),
        },
      ));
    }

    let needed_slots = globals_blocks.len() as u64;
    if needed_slots > self.globals_capacity_slots {
      let new_cap = needed_slots.next_power_of_two().max(1);
      self.globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("glyph globals"),
        size: new_cap * CLIP_SLOT_STRIDE,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      });
      // Re-create the bind group around the new buffer; the
      // texture / sampler bindings are unchanged but
      // `wgpu::BindGroup` is monolithic so we rebuild the whole
      // thing.
      let atlas_view = self.atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
      self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("glyph bg"),
        layout: &self.bind_group_layout,
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
              buffer: &self.globals_buffer,
              offset: 0,
              size: wgpu::BufferSize::new(std::mem::size_of::<Globals>() as u64),
            }),
          },
          wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::TextureView(&atlas_view),
          },
          wgpu::BindGroupEntry {
            binding: 2,
            resource: wgpu::BindingResource::Sampler(&self.sampler),
          },
        ],
      });
      self.globals_capacity_slots = new_cap;
    }

    for (i, block) in globals_blocks.iter().enumerate() {
      let offset = (i as u64) * CLIP_SLOT_STRIDE;
      queue.write_buffer(&self.globals_buffer, offset, bytemuck::bytes_of(block));
    }

    if count == 0 {
      return;
    }

    let needed = list.glyphs.len() as u64;
    if needed > self.instance_capacity {
      let new_cap = needed.next_power_of_two();
      self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("glyph instances"),
        size: new_cap * std::mem::size_of::<GlyphInstance>() as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      });
      self.instance_capacity = new_cap;
    }

    let instances: Vec<GlyphInstance> = list.glyphs.iter().map(GlyphInstance::from).collect();
    queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
  }

  pub fn record<'p>(&'p self, pass: &mut wgpu::RenderPass<'p>) {
    if self.instance_count == 0 {
      return;
    }
    pass.set_pipeline(&self.pipeline);
    pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
    pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

    if self.clip_runs.is_empty() {
      // Bind slot 0 unconditionally — `prepare` always populates
      // it.
      pass.set_bind_group(0, &self.bind_group, &[0]);
      pass.draw_indexed(0..6, 0, 0..self.instance_count);
      return;
    }
    for run in &self.clip_runs {
      self.record_prepared_range(pass, *run, run.instances.0..run.instances.1);
    }
  }

  pub fn record_range<'p>(&'p self, pass: &mut wgpu::RenderPass<'p>, clip_index: u32, instances: std::ops::Range<u32>) {
    if self.instance_count == 0 || instances.start >= instances.end {
      return;
    }
    let Some(Some(run)) = self.clip_slots.get(clip_index as usize) else {
      return;
    };
    pass.set_pipeline(&self.pipeline);
    pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
    pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    self.record_prepared_range(pass, *run, instances);
  }

  fn record_prepared_range<'p>(
    &'p self,
    pass: &mut wgpu::RenderPass<'p>,
    run: GlyphClipRun,
    instances: std::ops::Range<u32>,
  ) {
    let offset = (run.slot as u32) * (CLIP_SLOT_STRIDE as u32);
    pass.set_bind_group(0, &self.bind_group, &[offset]);
    pass.set_scissor_rect(run.rect[0], run.rect[1], run.rect[2], run.rect[3]);
    pass.draw_indexed(0..6, 0, instances);
  }
}

/// Same per-clip-range Globals constructor as the quad pipeline.
fn globals_for(viewport: [f32; 2], clip: &super::paint::ClipRange) -> Globals {
  let (rect, active) = match (clip.rect, clip.is_rounded()) {
    (Some(r), true) => ([r.x, r.y, r.w, r.h], 1.0),
    _ => ([0.0; 4], 0.0),
  };
  Globals {
    viewport: [viewport[0], viewport[1], 0.0, 0.0],
    clip_rect: rect,
    clip_radii_h: clip.radii_h,
    clip_radii_v: clip.radii_v,
    clip_active: [active, 0.0, 0.0, 0.0],
  }
}

/// Same scissor clamp as the quad pipeline; duplicated rather than
/// shared to keep the pipelines decoupled (no cross-module helper
/// crate exists yet).
fn clamp_scissor_rect(rect: Option<super::paint::Rect>, viewport: [u32; 2], scale: f32) -> [u32; 4] {
  let vw = viewport[0];
  let vh = viewport[1];
  let Some(r) = rect else {
    return [0, 0, vw, vh];
  };
  let x0 = (r.x * scale).max(0.0).round().min(vw as f32) as u32;
  let y0 = (r.y * scale).max(0.0).round().min(vh as f32) as u32;
  let x1 = ((r.x + r.w) * scale).max(0.0).round().min(vw as f32) as u32;
  let y1 = ((r.y + r.h) * scale).max(0.0).round().min(vh as f32) as u32;
  [x0, y0, x1.saturating_sub(x0), y1.saturating_sub(y0)]
}
