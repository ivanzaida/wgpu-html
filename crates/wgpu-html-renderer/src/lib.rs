//! M2: wgpu skeleton + solid quad pipeline.
//!
//! Owns the GPU device/queue, a window-bound surface, and a single
//! pipeline that renders a `DisplayList` of colored rectangles.
//! Also exposes a screenshot API: schedule a capture, the next rendered
//! frame is copied into a staging buffer and saved as a PNG.

use std::path::PathBuf;
use std::sync::Arc;

pub use wgpu;
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};

mod glyph_pipeline;
mod image_pipeline;
mod paint;
mod quad_pipeline;
mod screenshot;

pub use glyph_pipeline::GlyphPipeline;
pub use image_pipeline::ImagePipeline;
pub use paint::{
    Color, DisplayCommand, DisplayCommandKind, DisplayList, GlyphQuad, ImageQuad, Quad, Rect,
};
pub use quad_pipeline::QuadPipeline;
pub use screenshot::ScreenshotError;

/// Glyph atlas dimensions (square). The CPU-side atlas in
/// `wgpu-html-text` must be created with the same size so its uploads
/// land in the renderer's GPU texture without scaling.
pub const GLYPH_ATLAS_SIZE: u32 = 2048;

pub struct Renderer {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub clear_color: wgpu::Color,
    /// Non-sRGB equivalent of `surface_config.format`, used as the
    /// view format for the glyph pass. The same physical pixel bytes
    /// are interpreted as raw values (no sRGB decode on read, no
    /// sRGB encode on write), which makes the GPU's linear-space
    /// alpha blend behave like a gamma-space blend — the right thing
    /// for anti-aliased text. Equal to `surface_config.format` when
    /// the surface picked a non-sRGB format already.
    glyph_view_format: wgpu::TextureFormat,
    quads: QuadPipeline,
    images: ImagePipeline,
    glyphs: GlyphPipeline,
    pending_capture: Option<PathBuf>,
}

impl Renderer {
    /// Create a renderer bound to the given window-like surface target.
    /// The window is held via `Arc`; the renderer keeps it alive for the
    /// lifetime of the surface.
    pub async fn new<W>(window: Arc<W>, width: u32, height: u32) -> Self
    where
        W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static,
    {
        let mut idesc = wgpu::InstanceDescriptor::new_without_display_handle();
        idesc.backends = wgpu::Backends::PRIMARY;
        let instance = wgpu::Instance::new(idesc);

        let surface = instance
            .create_surface(window)
            .expect("failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("no suitable GPU adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("wgpu-html device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults()
                    .using_resolution(adapter.limits()),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("failed to acquire device");

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        // `glyph_view_format` is the same texture interpreted without
        // sRGB encoding; if `format` is already non-sRGB the call
        // returns it unchanged. We always add it to `view_formats` so
        // the surface texture is created allowing both views.
        let glyph_view_format = format.remove_srgb_suffix();
        let mut extra_view_formats: Vec<wgpu::TextureFormat> = Vec::new();
        if glyph_view_format != format {
            extra_view_formats.push(glyph_view_format);
        }

        let surface_config = wgpu::SurfaceConfiguration {
            // COPY_SRC is required so we can read the surface texture back
            // for screenshots.
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: extra_view_formats,
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let quads = QuadPipeline::new(&device, format);
        quads.upload_static(&queue);
        // Glyph pipeline targets the *non-sRGB* view of the surface,
        // so its blend equation runs on already-encoded byte values.
        let images = ImagePipeline::new(&device, format);
        images.upload_static(&queue);
        let glyphs = GlyphPipeline::new(&device, glyph_view_format, GLYPH_ATLAS_SIZE);
        glyphs.upload_static(&queue);

        Self {
            instance,
            adapter,
            device,
            queue,
            surface,
            surface_config,
            clear_color: wgpu::Color::WHITE,
            glyph_view_format,
            quads,
            images,
            glyphs,
            pending_capture: None,
        }
    }

    /// Borrow the glyph atlas's GPU texture so the host's CPU-side
    /// atlas can upload pending glyph rasters into it via
    /// `wgpu-html-text::Atlas::upload`.
    pub fn glyph_atlas_texture(&self) -> &wgpu::Texture {
        self.glyphs.atlas_texture()
    }

    /// Schedule the next rendered frame to be saved to `path` as a PNG.
    /// Capture happens on the next call to [`Renderer::render`].
    pub fn capture_next_frame_to(&mut self, path: impl Into<PathBuf>) {
        self.pending_capture = Some(path.into());
    }

    /// Render `list` into a freshly-allocated off-screen texture
    /// sized `width × height` and write it to `path` as a PNG.
    ///
    /// This is independent of the on-screen surface — coordinates in
    /// `list` are taken at face value (so to capture a particular
    /// document region, translate the list with
    /// [`paint::DisplayList::translated`] first and pass the
    /// region's pixel size). Because the off-screen target is
    /// allocated at the requested size and not constrained by the
    /// surface, regions partially or fully outside the visible
    /// viewport are captured at full fidelity.
    ///
    /// Reuses the renderer's pipelines, so per-frame buffers
    /// (instances, globals, scissors) are re-prepared for `list`
    /// here and `render`'s next call will re-prepare with its own
    /// list — calling `capture_to` between frames is safe.
    ///
    /// Errors propagate from the staging-buffer mapping or PNG
    /// encoder; the off-screen texture is always submitted and
    /// dropped before this returns.
    pub fn capture_to(
        &mut self,
        list: &DisplayList,
        width: u32,
        height: u32,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), ScreenshotError> {
        let width = width.max(1);
        let height = height.max(1);
        let format = self.surface_config.format;
        let glyph_view_format = self.glyph_view_format;

        let mut view_formats = Vec::new();
        if glyph_view_format != format {
            view_formats.push(glyph_view_format);
        }
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen capture target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &view_formats,
        });

        let srgb_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let glyph_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("offscreen glyph view"),
            format: Some(glyph_view_format),
            ..Default::default()
        });

        let viewport = [width as f32, height as f32];
        self.quads
            .prepare(&self.device, &self.queue, viewport, list);
        self.images
            .prepare(&self.device, &self.queue, viewport, list);
        self.glyphs
            .prepare(&self.device, &self.queue, viewport, list);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("offscreen capture encoder"),
            });

        // Initial clear matches the on-screen `render` path so the
        // captured image has the same background fill.
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("offscreen clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &srgb_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            let _ = &mut pass;
        }

        self.record_ordered_commands(list, &mut encoder, &srgb_view, &glyph_view);

        let staging =
            screenshot::begin_capture(&self.device, &mut encoder, &texture, width, height);
        self.queue.submit(Some(encoder.finish()));

        screenshot::finish_capture(&self.device, staging, width, height, format, path.as_ref())
    }

    /// Capture a rectangular region of the document described by
    /// `list` and write it to `path`. The output image is exactly
    /// `region.w × region.h` (rounded up to integer pixels). Equivalent
    /// to `capture_to(&list.translated(-region.x, -region.y), w, h, path)`.
    ///
    /// Works for regions outside the on-screen viewport because the
    /// off-screen render target is sized to the region — the
    /// renderer's pipelines paint every command in `list` against it
    /// regardless of where it falls relative to the visible window.
    pub fn capture_rect_to(
        &mut self,
        list: &DisplayList,
        region: paint::Rect,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), ScreenshotError> {
        let width = region.w.max(1.0).ceil() as u32;
        let height = region.h.max(1.0).ceil() as u32;
        let translated = list.translated(-region.x, -region.y);
        self.capture_to(&translated, width, height, path)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    /// Render one frame from a display list.
    pub fn render(&mut self, list: &DisplayList) -> FrameOutcome {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                return FrameOutcome::Reconfigure;
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => return FrameOutcome::Skipped,
        };

        // Two views over the same surface texture: the default sRGB
        // view (matching the surface format) for the quad pass, and a
        // non-sRGB unorm view for the glyph pass. Same memory; the
        // glyph pass treats the bytes as raw so the alpha blend runs
        // in display space.
        let srgb_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let glyph_view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("glyph non-srgb view"),
            format: Some(self.glyph_view_format),
            ..Default::default()
        });

        let viewport = [
            self.surface_config.width as f32,
            self.surface_config.height as f32,
        ];
        self.quads
            .prepare(&self.device, &self.queue, viewport, list);
        self.images
            .prepare(&self.device, &self.queue, viewport, list);
        self.glyphs
            .prepare(&self.device, &self.queue, viewport, list);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("wgpu-html frame encoder"),
            });

        // Initial clear. Actual drawing below follows DisplayList
        // command order, switching render passes only when the command
        // type needs a different pipeline/view.
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &srgb_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            let _ = &mut pass;
        }

        self.record_ordered_commands(list, &mut encoder, &srgb_view, &glyph_view);

        // If a capture was requested, append a texture-to-buffer copy to
        // the same encoder so it sees the just-rendered surface texture.
        let capture_target = self.pending_capture.take();
        let staging = capture_target.as_ref().map(|_| {
            screenshot::begin_capture(
                &self.device,
                &mut encoder,
                &frame.texture,
                self.surface_config.width,
                self.surface_config.height,
            )
        });

        self.queue.submit(Some(encoder.finish()));

        if let (Some(stg), Some(path)) = (staging, capture_target) {
            if let Err(e) = screenshot::finish_capture(
                &self.device,
                stg,
                self.surface_config.width,
                self.surface_config.height,
                self.surface_config.format,
                &path,
            ) {
                eprintln!("screenshot failed: {e}");
            } else {
                eprintln!("saved screenshot to {}", path.display());
            }
        }

        frame.present();
        FrameOutcome::Presented
    }

    fn record_ordered_commands(
        &self,
        list: &DisplayList,
        encoder: &mut wgpu::CommandEncoder,
        srgb_view: &wgpu::TextureView,
        glyph_view: &wgpu::TextureView,
    ) {
        if list.commands.is_empty() {
            self.record_legacy_batches(encoder, srgb_view, glyph_view);
            return;
        }

        let mut cursor = 0usize;
        while cursor < list.commands.len() {
            let first = list.commands[cursor];
            let mut end = cursor + 1;
            while end < list.commands.len() {
                let prev = list.commands[end - 1];
                let next = list.commands[end];
                if next.kind != first.kind
                    || next.clip_index != first.clip_index
                    || next.index != prev.index + 1
                {
                    break;
                }
                end += 1;
            }

            let instances = first.index..list.commands[end - 1].index + 1;
            match first.kind {
                DisplayCommandKind::Quad => {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("ordered quad pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: srgb_view,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    self.quads
                        .record_range(&mut pass, first.clip_index, instances);
                }
                DisplayCommandKind::Image => {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("ordered image pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: srgb_view,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    self.images
                        .record_range(&mut pass, first.clip_index, instances);
                }
                DisplayCommandKind::Glyph => {
                    // Glyphs use the non-sRGB view so alpha blending
                    // happens in display space, matching the previous
                    // dedicated glyph pass.
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("ordered glyph pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: glyph_view,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    self.glyphs
                        .record_range(&mut pass, first.clip_index, instances);
                }
            }

            cursor = end;
        }
    }

    fn record_legacy_batches(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        srgb_view: &wgpu::TextureView,
        glyph_view: &wgpu::TextureView,
    ) {
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("legacy quad pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: srgb_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.quads.record(&mut pass);
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("legacy image pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: srgb_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.images.record(&mut pass);
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("legacy glyph pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: glyph_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.glyphs.record(&mut pass);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FrameOutcome {
    Presented,
    /// Surface lost or outdated; caller should resize/reconfigure.
    Reconfigure,
    /// Skipped (timeout/occluded/validation); just try again next frame.
    Skipped,
}
