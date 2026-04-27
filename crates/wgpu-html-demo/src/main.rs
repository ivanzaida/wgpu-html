//! M5 demo (T3): parse HTML, register an external font on the tree,
//! shape text via cosmic-text, render the resulting glyph quads
//! through the renderer's textured pipeline.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::SystemTime;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use wgpu_html::renderer::{FrameOutcome, Renderer, GLYPH_ATLAS_SIZE};
use wgpu_html_text::TextContext;
use wgpu_html_tree::{FontFace, FontStyleAxis};

const DOC: &str = include_str!("../html/hello-text.html");

/// Search a few common system-font locations for a TTF the host can
/// register. T3 only needs *some* font to demonstrate shaping; the
/// constraint we care about is that fonts go through the tree, not
/// where the bytes come from. Hosts that ship their own asset can
/// replace this with `include_bytes!(...)`.
fn load_demo_font_bytes() -> Option<Vec<u8>> {
    let candidates = [
        // Windows
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "C:\\Windows\\Fonts\\arial.ttf",
        "C:\\Windows\\Fonts\\calibri.ttf",
        "C:\\Windows\\Fonts\\tahoma.ttf",
        // Linux (covers most distros)
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        // macOS
        "/System/Library/Fonts/Helvetica.ttc",
        "/Library/Fonts/Arial.ttf",
    ];
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            eprintln!("demo: loaded font from {path}");
            return Some(bytes);
        }
    }
    None
}

/// Stable shared `Arc<[u8]>` for the demo font — re-using the same
/// `Arc` across frames means `TextContext::sync_fonts` recognises the
/// font as already loaded (`Arc::as_ptr` cache key) and skips
/// re-ingestion.
fn demo_font_data() -> Option<Arc<[u8]>> {
    static DATA: OnceLock<Option<Arc<[u8]>>> = OnceLock::new();
    DATA.get_or_init(|| {
        load_demo_font_bytes()
            .map(|v| Arc::from(v.into_boxed_slice()))
    })
    .clone()
}

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    text_ctx: TextContext,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            renderer: None,
            // CPU atlas size must match the renderer's GPU atlas so
            // uploads land 1:1 without scaling. See
            // `wgpu_html_renderer::GLYPH_ATLAS_SIZE`.
            text_ctx: TextContext::new(GLYPH_ATLAS_SIZE),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = Window::default_attributes()
            .with_title("wgpu-html — T3: text rendering")
            .with_inner_size(PhysicalSize::new(1280u32, 720u32));
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("failed to create window"),
        );

        let size = window.inner_size();
        let renderer = pollster::block_on(Renderer::new(window.clone(), size.width, size.height));

        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        let (Some(window), Some(renderer)) = (self.window.as_ref(), self.renderer.as_mut()) else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                renderer.resize(size.width, size.height);
                window.request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(key),
                        repeat: false,
                        ..
                    },
                ..
            } => match key {
                KeyCode::F12 => {
                    let path: PathBuf = format!("screenshot-{}.png", timestamp()).into();
                    renderer.capture_next_frame_to(path);
                    window.request_redraw();
                }
                KeyCode::Escape => event_loop.exit(),
                _ => {}
            },
            WindowEvent::RedrawRequested => {
                let size = window.inner_size();

                // Parse HTML and register the demo font on the tree.
                // The shared `Arc` returned by `demo_font_data` keeps
                // the cosmic-text bridge cache valid across frames.
                let mut tree = wgpu_html::parser::parse(DOC);
                if let Some(data) = demo_font_data() {
                    tree.register_font(FontFace {
                        family: "DemoSans".into(),
                        weight: 400,
                        style: FontStyleAxis::Normal,
                        data,
                    });
                }

                let list = wgpu_html::paint_tree_with_text(
                    &tree,
                    &mut self.text_ctx,
                    size.width as f32,
                    size.height as f32,
                    1.0, // T3: fixed scale; T7 honours `scale_factor_changed`.
                );

                // Push any newly-rasterised glyph rasters into the
                // renderer's GPU atlas before the draw.
                self.text_ctx
                    .atlas
                    .upload(&renderer.queue, renderer.glyph_atlas_texture());

                match renderer.render(&list) {
                    FrameOutcome::Presented | FrameOutcome::Skipped => {}
                    FrameOutcome::Reconfigure => {
                        renderer.resize(size.width, size.height);
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

/// Seconds since the Unix epoch, used as a unique-ish screenshot filename.
fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn main() {
    println!("wgpu-html demo:");
    println!("  F12  →  save current frame as screenshot-<unix>.png");
    println!("  Esc  →  quit");
    if demo_font_data().is_none() {
        eprintln!(
            "demo: no system font found at the candidate paths — text \
             will render as zero-size. Edit `load_demo_font_bytes` in \
             main.rs to point at a TTF on your machine."
        );
    }

    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("event loop run");
}
