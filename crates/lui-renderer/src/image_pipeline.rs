//! Image pipeline. Draws `<img>` elements as textured quads. Each
//! unique `image_id` gets its own GPU texture (created on first use
//! and cached). The pipeline uses the same clip/scissor scheme as the
//! quad and glyph pipelines.

use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};

use crate::paint::{DisplayList, ImageQuad};

const UNIT_QUAD: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
const UNIT_QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 1, 3];

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
struct ImageInstance {
  pos: [f32; 2],
  size: [f32; 2],
  opacity: [f32; 4],
}

impl From<&ImageQuad> for ImageInstance {
  fn from(q: &ImageQuad) -> Self {
    Self {
      pos: [q.rect.x, q.rect.y],
      size: [q.rect.w, q.rect.h],
      opacity: [q.opacity, 0.0, 0.0, 0.0],
    }
  }
}

/// Cached GPU texture for one image.
struct CachedImage {
  #[allow(dead_code)]
  texture: wgpu::Texture,
  bind_group: wgpu::BindGroup,
}

pub struct ImagePipeline {
  pipeline: wgpu::RenderPipeline,
  sampler: wgpu::Sampler,
  globals_buffer: wgpu::Buffer,
  vertex_buffer: wgpu::Buffer,
  index_buffer: wgpu::Buffer,
  instance_buffer: wgpu::Buffer,
  instance_capacity: u64,
  bind_group_layout: wgpu::BindGroupLayout,
  globals_capacity_slots: u64,
  /// Per-image GPU textures, keyed by `image_id`.
  cache: HashMap<u64, CachedImage>,
  /// Draw commands built during `prepare`.
  draws: Vec<ImageDraw>,
  draws_by_instance: Vec<Option<ImageDraw>>,
  viewport: [u32; 2],
}

#[derive(Clone, Copy)]
struct ImageDraw {
  image_id: u64,
  instance_index: u32,
  scissor: [u32; 4],
  slot: u32,
}

impl ImagePipeline {
  pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("image shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("shaders/image.wgsl").into()),
    });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("image sampler"),
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      mipmap_filter: wgpu::MipmapFilterMode::Nearest,
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      ..Default::default()
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("image bgl"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
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
      label: Some("image pl"),
      bind_group_layouts: &[Some(&bind_group_layout)],
      immediate_size: 0,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("image pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: Some("vs_main"),
        compilation_options: Default::default(),
        buffers: &[
          wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 2]>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
              format: wgpu::VertexFormat::Float32x2,
              offset: 0,
              shader_location: 0,
            }],
          },
          wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ImageInstance>() as u64,
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

    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("image vb"),
      size: std::mem::size_of_val(&UNIT_QUAD) as u64,
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("image ib"),
      size: std::mem::size_of_val(&UNIT_QUAD_INDICES) as u64,
      usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    let init_cap = 16u64;
    let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("image inst"),
      size: init_cap * std::mem::size_of::<ImageInstance>() as u64,
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    let init_slots = 4u64;
    let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("image globals"),
      size: init_slots * CLIP_SLOT_STRIDE,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    Self {
      pipeline,
      sampler,
      globals_buffer,
      vertex_buffer,
      index_buffer,
      instance_buffer,
      instance_capacity: init_cap,
      bind_group_layout,
      globals_capacity_slots: init_slots,
      cache: HashMap::new(),
      draws: Vec::new(),
      draws_by_instance: Vec::new(),
      viewport: [1, 1],
    }
  }

  pub fn upload_static(&self, queue: &wgpu::Queue) {
    queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&UNIT_QUAD));
    queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&UNIT_QUAD_INDICES));
  }

  pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, viewport: [f32; 2], list: &DisplayList) {
    self.draws.clear();
    self.draws_by_instance.clear();
    self.viewport = [viewport[0] as u32, viewport[1] as u32];

    if list.images.is_empty() {
      return;
    }

    // Ensure GPU textures exist for every image_id in the list.
    for img in &list.images {
      if !self.cache.contains_key(&img.image_id) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
          label: Some("img tex"),
          size: wgpu::Extent3d {
            width: img.width,
            height: img.height,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: wgpu::TextureFormat::Rgba8UnormSrgb,
          usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
          view_formats: &[],
        });
        queue.write_texture(
          wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
          },
          &img.data,
          wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * img.width),
            rows_per_image: Some(img.height),
          },
          wgpu::Extent3d {
            width: img.width,
            height: img.height,
            depth_or_array_layers: 1,
          },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
          label: Some("img bg"),
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
              resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
              binding: 2,
              resource: wgpu::BindingResource::Sampler(&self.sampler),
            },
          ],
        });
        self.cache.insert(img.image_id, CachedImage { texture, bind_group });
      }
    }

    // Build instance data and draw commands.
    let mut instances: Vec<ImageInstance> = Vec::with_capacity(list.images.len());
    for img in &list.images {
      instances.push(ImageInstance::from(img));
    }
    self.draws_by_instance.resize(list.images.len(), None);

    // Grow instance buffer if needed.
    let needed = instances.len() as u64;
    if needed > self.instance_capacity {
      let new_cap = needed.next_power_of_two().max(16);
      self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("image inst"),
        size: new_cap * std::mem::size_of::<ImageInstance>() as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      });
      self.instance_capacity = new_cap;
    }
    queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));

    // Globals: one slot per clip range that has images.
    let mut slot_idx = 0u32;
    let mut needed_slots = 0u64;
    for clip in &list.clips {
      if clip.image_start() < clip.image_end() {
        needed_slots += 1;
      }
    }

    if needed_slots > self.globals_capacity_slots {
      let new_cap = needed_slots.next_power_of_two().max(4);
      self.globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("image globals"),
        size: new_cap * CLIP_SLOT_STRIDE,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      });
      self.globals_capacity_slots = new_cap;
      // Rebuild all cached bind groups with the new globals buffer.
      self.rebuild_bind_groups(device);
    }

    for clip in &list.clips {
      let start = clip.image_start();
      let end = clip.image_end();
      if start >= end {
        continue;
      }
      let g = globals_for(viewport, clip);
      queue.write_buffer(
        &self.globals_buffer,
        slot_idx as u64 * CLIP_SLOT_STRIDE,
        bytemuck::bytes_of(&g),
      );
      let scissor = clamp_scissor_rect(clip.rect, self.viewport);
      for i in start..end {
        let draw = ImageDraw {
          image_id: list.images[i as usize].image_id,
          instance_index: i,
          scissor,
          slot: slot_idx,
        };
        self.draws.push(draw);
        self.draws_by_instance[i as usize] = Some(draw);
      }
      slot_idx += 1;
    }
  }

  fn rebuild_bind_groups(&mut self, device: &wgpu::Device) {
    for cached in self.cache.values_mut() {
      let view = cached.texture.create_view(&wgpu::TextureViewDescriptor::default());
      cached.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("img bg"),
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
            resource: wgpu::BindingResource::TextureView(&view),
          },
          wgpu::BindGroupEntry {
            binding: 2,
            resource: wgpu::BindingResource::Sampler(&self.sampler),
          },
        ],
      });
    }
  }

  pub fn record<'p>(&'p self, pass: &mut wgpu::RenderPass<'p>) {
    if self.draws.is_empty() {
      return;
    }
    pass.set_pipeline(&self.pipeline);
    pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

    for draw in &self.draws {
      self.record_prepared_draw(pass, *draw);
    }
  }

  pub fn record_range<'p>(
    &'p self,
    pass: &mut wgpu::RenderPass<'p>,
    _clip_index: u32,
    instances: std::ops::Range<u32>,
  ) {
    if self.draws_by_instance.is_empty() || instances.start >= instances.end {
      return;
    }
    pass.set_pipeline(&self.pipeline);
    pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
    for i in instances {
      let Some(Some(draw)) = self.draws_by_instance.get(i as usize) else {
        continue;
      };
      self.record_prepared_draw(pass, *draw);
    }
  }

  fn record_prepared_draw<'p>(&'p self, pass: &mut wgpu::RenderPass<'p>, draw: ImageDraw) {
    let Some(cached) = self.cache.get(&draw.image_id) else {
      return;
    };
    let offset = draw.slot as u64 * CLIP_SLOT_STRIDE;
    pass.set_bind_group(0, Some(&cached.bind_group), &[offset as u32]);
    let [x, y, w, h] = draw.scissor;
    if w == 0 || h == 0 {
      return;
    }
    pass.set_scissor_rect(x, y, w, h);
    pass.draw_indexed(
      0..UNIT_QUAD_INDICES.len() as u32,
      0,
      draw.instance_index..draw.instance_index + 1,
    );
  }
}

fn globals_for(viewport: [f32; 2], clip: &crate::paint::ClipRange) -> Globals {
  match clip.rect {
    Some(r) => Globals {
      viewport: [viewport[0], viewport[1], 0.0, 0.0],
      clip_rect: [r.x, r.y, r.w, r.h],
      clip_radii_h: clip.radii_h,
      clip_radii_v: clip.radii_v,
      clip_active: if clip.is_rounded() {
        [1.0, 0.0, 0.0, 0.0]
      } else {
        [0.0; 4]
      },
    },
    None => Globals {
      viewport: [viewport[0], viewport[1], 0.0, 0.0],
      clip_rect: [0.0; 4],
      clip_radii_h: [0.0; 4],
      clip_radii_v: [0.0; 4],
      clip_active: [0.0; 4],
    },
  }
}

fn clamp_scissor_rect(rect: Option<crate::paint::Rect>, viewport: [u32; 2]) -> [u32; 4] {
  let vw = viewport[0];
  let vh = viewport[1];
  let Some(r) = rect else {
    return [0, 0, vw, vh];
  };
  let x0 = r.x.max(0.0).round().min(vw as f32) as u32;
  let y0 = r.y.max(0.0).round().min(vh as f32) as u32;
  let x1 = (r.x + r.w).max(0.0).round().min(vw as f32) as u32;
  let y1 = (r.y + r.h).max(0.0).round().min(vh as f32) as u32;
  [x0, y0, x1.saturating_sub(x0), y1.saturating_sub(y0)]
}
