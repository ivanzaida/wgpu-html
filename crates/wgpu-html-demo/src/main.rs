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

const DOC: &str = include_str!("../html/flex-grow.html");

/// One font family's worth of system-font paths: regular, bold,
/// italic, bold-italic. An empty path means "this variant isn't on
/// disk for this family"; the loader simply skips it. The table is
/// scanned top-to-bottom — the first row whose `regular` is readable
/// wins. That row's other variants are loaded if present.
const FONT_FAMILIES: &[[&str; 4]] = &[
    // Windows — Segoe UI is the system UI font on modern Windows.
    [
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "C:\\Windows\\Fonts\\segoeuib.ttf",
        "C:\\Windows\\Fonts\\segoeuii.ttf",
        "C:\\Windows\\Fonts\\segoeuiz.ttf",
    ],
    [
        "C:\\Windows\\Fonts\\arial.ttf",
        "C:\\Windows\\Fonts\\arialbd.ttf",
        "C:\\Windows\\Fonts\\ariali.ttf",
        "C:\\Windows\\Fonts\\arialbi.ttf",
    ],
    [
        "C:\\Windows\\Fonts\\calibri.ttf",
        "C:\\Windows\\Fonts\\calibrib.ttf",
        "C:\\Windows\\Fonts\\calibrii.ttf",
        "C:\\Windows\\Fonts\\calibriz.ttf",
    ],
    [
        "C:\\Windows\\Fonts\\tahoma.ttf",
        "C:\\Windows\\Fonts\\tahomabd.ttf",
        "",
        "",
    ],
    // Linux: DejaVu lives in a few different paths across distros.
    [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Oblique.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-BoldOblique.ttf",
    ],
    [
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/TTF/DejaVuSans-Oblique.ttf",
        "/usr/share/fonts/TTF/DejaVuSans-BoldOblique.ttf",
    ],
    [
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans-Oblique.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans-BoldOblique.ttf",
    ],
    // macOS — `.ttc` collections cover all variants in one file, so
    // the bold / italic slots are empty (cosmic-text picks within
    // the collection by `(weight, style)`).
    ["/System/Library/Fonts/Helvetica.ttc", "", "", ""],
    ["/Library/Fonts/Arial.ttf", "", "", ""],
];

/// One demo-font variant the host has loaded.
#[derive(Clone)]
struct DemoFont {
    weight: u16,
    style: FontStyleAxis,
    data: Arc<[u8]>,
}

/// Scan `FONT_FAMILIES` and return every variant found from the
/// first row whose regular file is readable. Each `Arc<[u8]>` is
/// stable across frames so `TextContext::sync_fonts` recognises the
/// faces as already loaded on the second-and-later sync.
fn demo_fonts() -> &'static [DemoFont] {
    static FACES: OnceLock<Vec<DemoFont>> = OnceLock::new();
    FACES.get_or_init(|| {
        for row in FONT_FAMILIES {
            let regular_path = row[0];
            let Ok(reg_bytes) = std::fs::read(regular_path) else {
                continue;
            };
            eprintln!("demo: loaded font family from {regular_path}");
            let mut out = vec![DemoFont {
                weight: 400,
                style: FontStyleAxis::Normal,
                data: Arc::from(reg_bytes.into_boxed_slice()),
            }];
            // (path, weight, style) for the optional 3 variants.
            let variants: [(&str, u16, FontStyleAxis); 3] = [
                (row[1], 700, FontStyleAxis::Normal),
                (row[2], 400, FontStyleAxis::Italic),
                (row[3], 700, FontStyleAxis::Italic),
            ];
            for (path, weight, style) in variants {
                if path.is_empty() {
                    continue;
                }
                if let Ok(bytes) = std::fs::read(path) {
                    eprintln!("demo:   + variant {path} @ {weight} {style:?}");
                    out.push(DemoFont {
                        weight,
                        style,
                        data: Arc::from(bytes.into_boxed_slice()),
                    });
                }
            }
            return out;
        }
        Vec::new()
    })
    .as_slice()
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

                // Parse HTML and register every demo-font variant on
                // the tree under the same family name. The shared
                // `Arc<[u8]>`s returned by `demo_fonts` are stable
                // across frames so the cosmic-text bridge cache stays
                // valid on subsequent syncs.
                let mut tree = wgpu_html::parser::parse(DOC);
                for face in demo_fonts() {
                    tree.register_font(FontFace {
                        family: "DemoSans".into(),
                        weight: face.weight,
                        style: face.style,
                        data: face.data.clone(),
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
    if demo_fonts().is_empty() {
        eprintln!(
            "demo: no system font found at the candidate paths — text \
             will render as zero-size. Edit `FONT_FAMILIES` in main.rs \
             to point at a TTF on your machine."
        );
    }

    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("event loop run");
}
