//! Solid-color instanced quad pipeline.

use bytemuck::{Pod, Zeroable};

use super::paint::{DisplayList, Quad};

/// 4 corners of the unit quad: (0,0), (1,0), (0,1), (1,1).
const UNIT_QUAD: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
/// Two triangles: TL-TR-BL, BL-TR-BR.
const UNIT_QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 1, 3];

/// Per-clip-range uniform block. The viewport sits at the top
/// (vertex shader needs it for NDC conversion), then the active
/// rounded clip in fragment-shader-friendly form. `clip_active.x`
/// is `1.0` when this range carries any non-zero corner radius —
/// the fragment shader skips the SDF discard otherwise.
///
/// All-vec4 layout dodges WGSL std140 alignment headaches.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Globals {
  /// `[viewport_w, viewport_h, _, _]`.
  viewport: [f32; 4],
  /// `[pos.x, pos.y, size.w, size.h]` of the rounded clip rect.
  clip_rect: [f32; 4],
  /// Horizontal corner radii (TL, TR, BR, BL) at the rect's corners.
  clip_radii_h: [f32; 4],
  /// Vertical corner radii (TL, TR, BR, BL).
  clip_radii_v: [f32; 4],
  /// `[active, _, _, _]`. `1.0` when the fragment shader should
  /// run the rounded SDF discard; `0.0` for plain rectangular or
  /// no clip (the rectangular scissor handles those upstream).
  clip_active: [f32; 4],
}

const CLIP_SLOT_STRIDE: u64 = 256;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct QuadInstance {
  pos: [f32; 2],
  size: [f32; 2],
  color: [f32; 4],
  /// Horizontal radii per corner (TL, TR, BR, BL).
  radii_h: [f32; 4],
  /// Vertical radii per corner (TL, TR, BR, BL).
  radii_v: [f32; 4],
  /// Per-side ring thickness in pixels: top, right, bottom, left. All
  /// zero → filled mode.
  stroke: [f32; 4],
  /// Stroke pattern: (kind, dash, gap, _). `kind == 0` (solid) is the
  /// default and ignores the rest.
  pattern: [f32; 4],
  /// 2x2 rotation/scale matrix [a, b, c, d]. Identity = [1, 0, 0, 1].
  transform: [f32; 4],
  /// Transform origin relative to rect top-left, in pixels.
  transform_origin: [f32; 2],
  /// Shadow blur sigma. 0 = sharp edge.
  shadow_sigma: f32,
  _pad: f32,
}

impl From<&Quad> for QuadInstance {
  fn from(q: &Quad) -> Self {
    Self {
      pos: [q.rect.x, q.rect.y],
      size: [q.rect.w, q.rect.h],
      color: q.color,
      radii_h: q.radii_h,
      radii_v: q.radii_v,
      stroke: q.stroke,
      pattern: q.pattern,
      transform: q.transform,
      transform_origin: q.transform_origin,
      shadow_sigma: q.shadow_sigma,
      _pad: 0.0,
    }
  }
}

pub struct QuadPipeline {
  pipeline: wgpu::RenderPipeline,
  bind_group_layout: wgpu::BindGroupLayout,
  bind_group: wgpu::BindGroup,
  globals_buffer: wgpu::Buffer,
  /// Current capacity of `globals_buffer` in 256-byte slots.
  globals_capacity_slots: u64,
  vertex_buffer: wgpu::Buffer,
  index_buffer: wgpu::Buffer,
  instance_buffer: wgpu::Buffer,
  instance_capacity: u64,
  instance_count: u32,
  /// One scissor-tagged run per `(rect, instance_range)` slice of
  /// the prepared instance buffer. Built from `DisplayList::clips`
  /// at `prepare` time and walked in `record` to issue one
  /// `set_scissor_rect` + `draw_indexed` per entry.
  clip_runs: Vec<ClipRun>,
  clip_slots: Vec<Option<ClipRun>>,
  /// Cached viewport extents so `record` can clamp out-of-bounds
  /// scissor rects (wgpu panics on out-of-bounds values) and emit
  /// a full-viewport scissor for `rect == None`.
  viewport: [u32; 2],
}

/// Pre-resolved per-run scissor, instance range, and dynamic-offset
/// slot index into `globals_buffer`. `rect` is already clamped to
/// viewport bounds (so `set_scissor_rect` can be called directly); a
/// full-viewport rect signals "no rectangular clip" upstream.
#[derive(Clone, Copy)]
struct ClipRun {
  rect: [u32; 4], // x, y, w, h in physical pixels
  instances: (u32, u32),
  slot: u32,
}

impl QuadPipeline {
  pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("quad shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("shaders/quad.wgsl").into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("quad bgl"),
      entries: &[wgpu::BindGroupLayoutEntry {
        binding: 0,
        // Vertex needs the viewport for NDC mapping; fragment
        // needs the clip data for the rounded-SDF discard.
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          // Dynamic offset → one shared buffer with one
          // slot per clip range, addressed at draw time.
          has_dynamic_offset: true,
          min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<Globals>() as u64),
        },
        count: None,
      }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("quad pl"),
      bind_group_layouts: &[Some(&bind_group_layout)],
      immediate_size: 0,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("quad pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: Some("vs_main"),
        compilation_options: Default::default(),
        buffers: &[
          // Per-vertex: unit corner.
          wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 2]>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
              format: wgpu::VertexFormat::Float32x2,
              offset: 0,
              shader_location: 0,
            }],
          },
          // Per-instance: pos, size, color, radii_h, radii_v, stroke widths.
          wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<QuadInstance>() as u64,
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
                format: wgpu::VertexFormat::Float32x4,
                offset: 32,
                shader_location: 4,
              },
              wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 48,
                shader_location: 5,
              },
              wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 64,
                shader_location: 6,
              },
              wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 80,
                shader_location: 7,
              },
              wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 96,
                shader_location: 8,
              },
              wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 112,
                shader_location: 9,
              },
              wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32,
                offset: 120,
                shader_location: 10,
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

    // One slot up-front; `prepare` grows the buffer to fit the
    // current frame's clip-range count.
    let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("quad globals"),
      size: CLIP_SLOT_STRIDE,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("quad bg"),
      layout: &bind_group_layout,
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        // The dynamic-offset binding always sees a single
        // `Globals`-sized window into the buffer.
        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
          buffer: &globals_buffer,
          offset: 0,
          size: wgpu::BufferSize::new(std::mem::size_of::<Globals>() as u64),
        }),
      }],
    });

    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("quad vb"),
      size: std::mem::size_of_val(&UNIT_QUAD) as u64,
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("quad ib"),
      size: std::mem::size_of_val(&UNIT_QUAD_INDICES) as u64,
      usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    // Instance buffer starts with a small capacity; grows on demand.
    let initial_instances: u64 = 64;
    let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("quad instances"),
      size: initial_instances * std::mem::size_of::<QuadInstance>() as u64,
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    Self {
      pipeline,
      bind_group_layout,
      bind_group,
      globals_buffer,
      globals_capacity_slots: 1,
      vertex_buffer,
      index_buffer,
      instance_buffer,
      instance_capacity: initial_instances,
      instance_count: 0,
      clip_runs: Vec::new(),
      clip_slots: Vec::new(),
      viewport: [0, 0],
    }
  }

  /// Upload static unit-quad geometry. Call once after creation.
  pub fn upload_static(&self, queue: &wgpu::Queue) {
    queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&UNIT_QUAD));
    queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&UNIT_QUAD_INDICES));
  }

  /// Update viewport + instance + per-clip-range globals for this
  /// frame.
  pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, viewport: [f32; 2], list: &DisplayList) {
    let count = list.quads.len() as u32;
    self.instance_count = count;
    let s = list.dpi_scale.max(1.0);
    self.viewport = [
      (viewport[0] * s).max(0.0).round() as u32,
      (viewport[1] * s).max(0.0).round() as u32,
    ];

    // Build the per-clip-range globals + clip runs. We allocate
    // one slot per *non-empty* clip range; a list with no clipping
    // emits a single slot covering everything.
    self.clip_runs.clear();
    self.clip_slots.clear();
    self.clip_slots.resize(list.clips.len(), None);
    let mut globals_blocks: Vec<Globals> = Vec::new();
    for (clip_index, clip) in list.clips.iter().enumerate() {
      if clip.quad_range.0 == clip.quad_range.1 {
        continue;
      }
      let rect = clamp_scissor_rect(clip.rect, self.viewport, s);
      let slot = globals_blocks.len() as u32;
      globals_blocks.push(globals_for(viewport, clip));
      let run = ClipRun {
        rect,
        instances: clip.quad_range,
        slot,
      };
      self.clip_runs.push(run);
      self.clip_slots[clip_index] = Some(run);
    }
    if globals_blocks.is_empty() {
      // No quads at all → still need one slot for any future
      // record() call to see the right viewport.
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

    // Grow the globals buffer if needed and re-bind. The buffer
    // holds `n_slots × CLIP_SLOT_STRIDE` bytes; each slot starts
    // with a `Globals` block followed by padding.
    let needed_slots = globals_blocks.len() as u64;
    if needed_slots > self.globals_capacity_slots {
      let new_cap = needed_slots.next_power_of_two().max(1);
      self.globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("quad globals"),
        size: new_cap * CLIP_SLOT_STRIDE,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      });
      self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("quad bg"),
        layout: &self.bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &self.globals_buffer,
            offset: 0,
            size: wgpu::BufferSize::new(std::mem::size_of::<Globals>() as u64),
          }),
        }],
      });
      self.globals_capacity_slots = new_cap;
    }

    // Write each `Globals` block at its slot's offset.
    for (i, block) in globals_blocks.iter().enumerate() {
      let offset = (i as u64) * CLIP_SLOT_STRIDE;
      queue.write_buffer(&self.globals_buffer, offset, bytemuck::bytes_of(block));
    }

    if count == 0 {
      return;
    }

    let needed = list.quads.len() as u64;
    if needed > self.instance_capacity {
      let new_cap = needed.next_power_of_two();
      self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("quad instances"),
        size: new_cap * std::mem::size_of::<QuadInstance>() as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      });
      self.instance_capacity = new_cap;
    }

    let instances: Vec<QuadInstance> = list.quads.iter().map(QuadInstance::from).collect();
    queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
  }

  /// Record draw calls into an existing render pass.
  pub fn record<'p>(&'p self, pass: &mut wgpu::RenderPass<'p>) {
    if self.instance_count == 0 {
      return;
    }
    pass.set_pipeline(&self.pipeline);
    pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
    pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

    if self.clip_runs.is_empty() {
      // Fast path for display lists that didn't carry any
      // explicit clip ranges (legacy producers). Bind slot 0
      // unconditionally — `prepare` always populates it.
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
    run: ClipRun,
    instances: std::ops::Range<u32>,
  ) {
    let offset = (run.slot as u32) * (CLIP_SLOT_STRIDE as u32);
    pass.set_bind_group(0, &self.bind_group, &[offset]);
    pass.set_scissor_rect(run.rect[0], run.rect[1], run.rect[2], run.rect[3]);
    pass.draw_indexed(0..6, 0, instances);
  }
}

/// Convert a paint-side `Option<Rect>` into integer scissor
/// coordinates clamped to the viewport. `None` (no scissor) becomes
/// the full viewport rect, the wgpu equivalent of "no clip".
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

/// Build the per-clip-range `Globals` block from the active clip and
/// the current viewport. `clip_active.x` is `1.0` when the clip
/// carries any non-zero corner radius — that's what tells the
/// fragment shader to do the rounded-SDF discard.
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
