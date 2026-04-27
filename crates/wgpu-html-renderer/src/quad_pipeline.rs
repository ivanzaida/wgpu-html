//! Solid-color instanced quad pipeline.

use bytemuck::{Pod, Zeroable};

use crate::paint::{DisplayList, Quad};

/// 4 corners of the unit quad: (0,0), (1,0), (0,1), (1,1).
const UNIT_QUAD: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
/// Two triangles: TL-TR-BL, BL-TR-BR.
const UNIT_QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 1, 3];

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Globals {
    viewport: [f32; 2],
    _pad: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct QuadInstance {
    pos: [f32; 2],
    size: [f32; 2],
    color: [f32; 4],
}

impl From<&Quad> for QuadInstance {
    fn from(q: &Quad) -> Self {
        Self {
            pos: [q.rect.x, q.rect.y],
            size: [q.rect.w, q.rect.h],
            color: q.color,
        }
    }
}

pub struct QuadPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    globals_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_capacity: u64,
    instance_count: u32,
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
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<Globals>() as u64
                    ),
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
                    // Per-instance: pos, size, color.
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
            label: Some("quad globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("quad bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
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
            bind_group,
            globals_buffer,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            instance_capacity: initial_instances,
            instance_count: 0,
        }
    }

    /// Upload static unit-quad geometry. Call once after creation.
    pub fn upload_static(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&UNIT_QUAD));
        queue.write_buffer(
            &self.index_buffer,
            0,
            bytemuck::cast_slice(&UNIT_QUAD_INDICES),
        );
    }

    /// Update viewport + instance data for this frame.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport: [f32; 2],
        list: &DisplayList,
    ) {
        let globals = Globals {
            viewport,
            _pad: [0.0; 2],
        };
        queue.write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&globals));

        let count = list.quads.len() as u32;
        self.instance_count = count;
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
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..6, 0, 0..self.instance_count);
    }
}
