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
mod paint;
mod quad_pipeline;
mod screenshot;

pub use glyph_pipeline::GlyphPipeline;
pub use paint::{Color, DisplayList, GlyphQuad, Quad, Rect};
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
    quads: QuadPipeline,
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

        let surface_config = wgpu::SurfaceConfiguration {
            // COPY_SRC is required so we can read the surface texture back
            // for screenshots.
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let quads = QuadPipeline::new(&device, format);
        quads.upload_static(&queue);
        let glyphs = GlyphPipeline::new(&device, format, GLYPH_ATLAS_SIZE);
        glyphs.upload_static(&queue);

        Self {
            instance,
            adapter,
            device,
            queue,
            surface,
            surface_config,
            clear_color: wgpu::Color::WHITE,
            quads,
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
                return FrameOutcome::Reconfigure
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => return FrameOutcome::Skipped,
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let viewport = [
            self.surface_config.width as f32,
            self.surface_config.height as f32,
        ];
        self.quads
            .prepare(&self.device, &self.queue, viewport, list);
        self.glyphs
            .prepare(&self.device, &self.queue, viewport, list);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("wgpu-html frame encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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
            self.quads.record(&mut pass);
            self.glyphs.record(&mut pass);
        }

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
}

#[derive(Debug, Clone, Copy)]
pub enum FrameOutcome {
    Presented,
    /// Surface lost or outdated; caller should resize/reconfigure.
    Reconfigure,
    /// Skipped (timeout/occluded/validation); just try again next frame.
    Skipped,
}
