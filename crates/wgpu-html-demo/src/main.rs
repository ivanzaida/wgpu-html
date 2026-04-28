//! M5 demo (T3): parse HTML, register an external font on the tree,
//! shape text via cosmic-text, render the resulting glyph quads
//! through the renderer's textured pipeline.

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant, SystemTime};

use arboard::Clipboard;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{
    ElementState, KeyEvent, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use wgpu_html::interactivity;
use wgpu_html::layout::LayoutBox;
use wgpu_html::renderer::{DisplayList, FrameOutcome, GLYPH_ATLAS_SIZE, Rect, Renderer};
use wgpu_html_text::TextContext;
use wgpu_html_tree::{FontFace, FontStyleAxis, Modifiers, MouseButton, Tree};

const DEFAULT_DOC: &str = include_str!("../html/flex-browser-like.html");
const DEFAULT_DOC_PATH: &str = "crates/wgpu-html-demo/html/flex-browser-like.html";

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
/// first row whose regular 3file is readable. Each `Arc<[u8]>` is
/// stable across frames so `TextContext::sync_fonts` recognises the
/// faces as already loaded on the second-and-later sync.
fn demo_fonts() -> &'static [DemoFont] {
    static FACES: OnceLock<Vec<DemoFont>> = OnceLock::new();
    FACES
        .get_or_init(|| {
            for row in FONT_FAMILIES {
                let regular_path = row[0];
                let Ok(reg_bytes) = std::fs::read(regular_path) else {
                    continue;
                };
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

#[derive(Debug, Clone, Copy, Default)]
struct StageStats {
    sum_ms: f64,
    max_ms: f64,
}

impl StageStats {
    fn add_sample(&mut self, ms: f64) {
        self.sum_ms += ms;
        self.max_ms = self.max_ms.max(ms);
    }

    fn avg_ms(&self, frames: u64) -> f64 {
        if frames == 0 {
            0.0
        } else {
            self.sum_ms / frames as f64
        }
    }
}

#[derive(Debug, Clone)]
struct ProfileWindow {
    started_at: Instant,
    frames: u64,
    total: StageStats,
    tree: StageStats,
    cascade: StageStats,
    layout: StageStats,
    paint: StageStats,
    layout_paint: StageStats,
    postprocess: StageStats,
    atlas_upload: StageStats,
    render: StageStats,
    quads_sum: u64,
    glyphs_sum: u64,
    images_sum: u64,
    clips_sum: u64,
    hover_moves: u64,
    hover_changed: u64,
    hover_redraw_requests: u64,
    hover_redraw_deferred: u64,
    hover_pointer_move: StageStats,
    hover_frames: u64,
    hover_frame_total: StageStats,
    hover_frame_cascade: StageStats,
    hover_frame_layout: StageStats,
    hover_frame_paint: StageStats,
    hover_frame_layout_paint: StageStats,
    hover_frame_render: StageStats,
}

impl ProfileWindow {
    fn new() -> Self {
        Self {
            started_at: Instant::now(),
            frames: 0,
            total: StageStats::default(),
            tree: StageStats::default(),
            cascade: StageStats::default(),
            layout: StageStats::default(),
            paint: StageStats::default(),
            layout_paint: StageStats::default(),
            postprocess: StageStats::default(),
            atlas_upload: StageStats::default(),
            render: StageStats::default(),
            quads_sum: 0,
            glyphs_sum: 0,
            images_sum: 0,
            clips_sum: 0,
            hover_moves: 0,
            hover_changed: 0,
            hover_redraw_requests: 0,
            hover_redraw_deferred: 0,
            hover_pointer_move: StageStats::default(),
            hover_frames: 0,
            hover_frame_total: StageStats::default(),
            hover_frame_cascade: StageStats::default(),
            hover_frame_layout: StageStats::default(),
            hover_frame_paint: StageStats::default(),
            hover_frame_layout_paint: StageStats::default(),
            hover_frame_render: StageStats::default(),
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn next_deadline(&self) -> Instant {
        self.started_at + Duration::from_secs(1)
    }

    fn is_due(&self) -> bool {
        Instant::now() >= self.next_deadline()
    }

    fn take_line_if_due(&mut self) -> Option<String> {
        if !self.is_due() {
            return None;
        }
        if self.frames == 0 && self.hover_moves == 0 && self.hover_frames == 0 {
            self.reset();
            return None;
        }

        let secs = self.started_at.elapsed().as_secs_f64().max(f64::EPSILON);
        let frames = self.frames;
        let fps = frames as f64 / secs;
        let line = format!(
            "profile: {:.2}s frames={} fps={:.1} total(avg/max)={:.2}/{:.2}ms tree={:.2}/{:.2} cascade={:.2}/{:.2} layout={:.2}/{:.2} paint={:.2}/{:.2} layout+paint={:.2}/{:.2} post={:.2}/{:.2} atlas={:.2}/{:.2} render={:.2}/{:.2} avg_quads={} avg_glyphs={} avg_images={} avg_clips={} hover[moves={} changed={} ptr(avg/max)={:.3}/{:.3}ms redraws={} deferred={} frames={} total(avg/max)={:.2}/{:.2}ms c={:.2}/{:.2} l={:.2}/{:.2} p={:.2}/{:.2} lp={:.2}/{:.2} render={:.2}/{:.2}ms]",
            secs,
            frames,
            fps,
            self.total.avg_ms(frames),
            self.total.max_ms,
            self.tree.avg_ms(frames),
            self.tree.max_ms,
            self.cascade.avg_ms(frames),
            self.cascade.max_ms,
            self.layout.avg_ms(frames),
            self.layout.max_ms,
            self.paint.avg_ms(frames),
            self.paint.max_ms,
            self.layout_paint.avg_ms(frames),
            self.layout_paint.max_ms,
            self.postprocess.avg_ms(frames),
            self.postprocess.max_ms,
            self.atlas_upload.avg_ms(frames),
            self.atlas_upload.max_ms,
            self.render.avg_ms(frames),
            self.render.max_ms,
            if frames == 0 {
                0
            } else {
                self.quads_sum / frames
            },
            if frames == 0 {
                0
            } else {
                self.glyphs_sum / frames
            },
            if frames == 0 {
                0
            } else {
                self.images_sum / frames
            },
            if frames == 0 {
                0
            } else {
                self.clips_sum / frames
            },
            self.hover_moves,
            self.hover_changed,
            self.hover_pointer_move.avg_ms(self.hover_moves),
            self.hover_pointer_move.max_ms,
            self.hover_redraw_requests,
            self.hover_redraw_deferred,
            self.hover_frames,
            self.hover_frame_total.avg_ms(self.hover_frames),
            self.hover_frame_total.max_ms,
            self.hover_frame_cascade.avg_ms(self.hover_frames),
            self.hover_frame_cascade.max_ms,
            self.hover_frame_layout.avg_ms(self.hover_frames),
            self.hover_frame_layout.max_ms,
            self.hover_frame_paint.avg_ms(self.hover_frames),
            self.hover_frame_paint.max_ms,
            self.hover_frame_layout_paint.avg_ms(self.hover_frames),
            self.hover_frame_layout_paint.max_ms,
            self.hover_frame_render.avg_ms(self.hover_frames),
            self.hover_frame_render.max_ms,
        );
        self.reset();
        Some(line)
    }

    fn add_hover_move(&mut self, pointer_move_ms: f64, changed: bool) {
        self.hover_moves += 1;
        self.hover_pointer_move.add_sample(pointer_move_ms);
        if changed {
            self.hover_changed += 1;
        }
    }

    fn mark_hover_redraw_requested(&mut self) {
        self.hover_redraw_requests += 1;
    }

    fn mark_hover_redraw_deferred(&mut self) {
        self.hover_redraw_deferred += 1;
    }

    fn add_hover_frame_detailed(
        &mut self,
        total_ms: f64,
        cascade_ms: f64,
        layout_ms: f64,
        paint_ms: f64,
        layout_paint_ms: f64,
        render_ms: f64,
    ) {
        self.hover_frames += 1;
        self.hover_frame_total.add_sample(total_ms);
        self.hover_frame_cascade.add_sample(cascade_ms);
        self.hover_frame_layout.add_sample(layout_ms);
        self.hover_frame_paint.add_sample(paint_ms);
        self.hover_frame_layout_paint.add_sample(layout_paint_ms);
        self.hover_frame_render.add_sample(render_ms);
    }

    #[allow(clippy::too_many_arguments)]
    fn add_frame(
        &mut self,
        total_ms: f64,
        tree_ms: f64,
        cascade_ms: f64,
        layout_ms: f64,
        paint_ms: f64,
        layout_paint_ms: f64,
        postprocess_ms: f64,
        atlas_upload_ms: f64,
        render_ms: f64,
        quads: usize,
        glyphs: usize,
        images: usize,
        clips: usize,
    ) {
        self.frames += 1;
        self.total.add_sample(total_ms);
        self.tree.add_sample(tree_ms);
        self.cascade.add_sample(cascade_ms);
        self.layout.add_sample(layout_ms);
        self.paint.add_sample(paint_ms);
        self.layout_paint.add_sample(layout_paint_ms);
        self.postprocess.add_sample(postprocess_ms);
        self.atlas_upload.add_sample(atlas_upload_ms);
        self.render.add_sample(render_ms);
        self.quads_sum += quads as u64;
        self.glyphs_sum += glyphs as u64;
        self.images_sum += images as u64;
        self.clips_sum += clips as u64;
    }
}

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    text_ctx: TextContext,
    doc_html: String,
    /// Document tree, parsed once on first redraw and reused across
    /// frames so per-element callbacks survive (re-parsing every
    /// frame would lose them).
    tree: Option<Tree>,
    /// Layout from the previous frame; used to dispatch pointer
    /// events against the geometry the user is actually looking at.
    last_layout: Option<LayoutBox>,
    /// Last cursor position in physical pixels, for translating
    /// `WindowEvent::MouseInput` to a pointer position (winit reports
    /// MouseInput without a position).
    cursor_pos: Option<(f32, f32)>,
    /// Current document viewport scroll in physical pixels. The
    /// layout tree stays in document coordinates; paint translates
    /// display items up by this offset and hit testing adds it back.
    scroll_y: f32,
    /// Active viewport scrollbar thumb drag, if the primary button
    /// started on the thumb or track.
    scrollbar_drag: Option<ScrollbarDrag>,
    /// Counter incremented from the click callback. Demonstrates that
    /// closures capture state correctly.
    click_count: Arc<AtomicUsize>,
    /// Live keyboard modifier state used for shortcut dispatch.
    modifiers: Modifiers,
    /// Whether a hover-state change arrived that hasn't been rendered yet.
    hover_pending: bool,
    /// Timestamp of the last hover-driven `RedrawRequested` delivery.
    /// Used to cap hover redraws to ~60 fps so that a 1000 Hz mouse does
    /// not queue hundreds of full layout+render passes per second.
    last_hover_redraw: Instant,
    /// Emit per-frame timing logs for expensive demo stages.
    profiling_enabled: bool,
    /// Aggregated profiling samples, emitted about once per second.
    profile_window: ProfileWindow,
    /// Whether the next redraw should be attributed to hover-driven work.
    hover_redraw_pending: bool,
}

impl App {
    fn new(doc_html: String, profiling_enabled: bool) -> Self {
        Self {
            window: None,
            renderer: None,
            // CPU atlas size must match the renderer's GPU atlas so
            // uploads land 1:1 without scaling. See
            // `wgpu_html_renderer::GLYPH_ATLAS_SIZE`.
            text_ctx: TextContext::new(GLYPH_ATLAS_SIZE),
            doc_html,
            tree: None,
            last_layout: None,
            cursor_pos: None,
            scroll_y: 0.0,
            scrollbar_drag: None,
            click_count: Arc::new(AtomicUsize::new(0)),
            modifiers: Modifiers::default(),
            hover_pending: false,
            // Initialise to "long ago" so the very first hover fires immediately.
            last_hover_redraw: Instant::now() - Duration::from_secs(1),
            profiling_enabled,
            profile_window: ProfileWindow::new(),
            hover_redraw_pending: false,
        }
    }
}

impl App {
    /// Build the document tree once on the first frame (or first
    /// access). Registers any system-resolved fonts found by
    /// `demo_fonts()` and wires the per-element callbacks.
    ///
    /// Takes the tree slot and counter as separate borrows (rather
    /// than `&mut self`) so it can be called alongside an already-
    /// active `&mut self.renderer` borrow.
    fn ensure_tree_built<'a>(
        tree_slot: &'a mut Option<Tree>,
        doc_html: &str,
        click_count: &Arc<AtomicUsize>,
    ) -> &'a mut Tree {
        if tree_slot.is_none() {
            let mut tree = wgpu_html::parser::parse(doc_html);
            for face in demo_fonts() {
                tree.register_font(FontFace {
                    family: "DemoSans".into(),
                    weight: face.weight,
                    style: face.style,
                    data: face.data.clone(),
                });
            }

            // Wire callbacks via the friendly `get_element_by_id` API.
            // Keep demo callbacks silent so profiling logs stay readable.
            let counter = click_count.clone();
            if let Some(btn) = tree.get_element_by_id("btn") {
                btn.on_click = Some(Arc::new(move |_| {
                    let n = counter.fetch_add(1, Ordering::Relaxed) + 1;
                    let _ = n;
                }));
            }

            if let Some(panel) = tree.get_element_by_id("panel") {
                panel.on_mouse_enter = Some(Arc::new(|_| {}));
                panel.on_mouse_leave = Some(Arc::new(|_| {}));
                panel.on_click = Some(Arc::new(|_| {}));
            }

            *tree_slot = Some(tree);
        }
        tree_slot.as_mut().unwrap()
    }

    fn update_modifier_key(&mut self, key: KeyCode, state: ElementState) {
        let down = state == ElementState::Pressed;
        match key {
            KeyCode::ControlLeft | KeyCode::ControlRight => self.modifiers.ctrl = down,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => self.modifiers.shift = down,
            KeyCode::AltLeft | KeyCode::AltRight => self.modifiers.alt = down,
            KeyCode::SuperLeft | KeyCode::SuperRight => self.modifiers.meta = down,
            _ => {}
        }
    }

    fn copy_selection_to_clipboard(&self) {
        let (Some(tree), Some(layout)) = (self.tree.as_ref(), self.last_layout.as_ref()) else {
            return;
        };
        let Some(text) = wgpu_html::selected_text(tree, layout) else {
            return;
        };
        if text.is_empty() {
            return;
        }
        match Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(err) = clipboard.set_text(text.clone()) {
                    eprintln!("demo: failed to copy selection to clipboard: {err}");
                } else {
                    eprintln!("demo: copied {} chars", text.chars().count());
                }
            }
            Err(err) => eprintln!("demo: clipboard unavailable: {err}"),
        }
    }

    fn toggle_profiling(&mut self) {
        self.profiling_enabled = !self.profiling_enabled;
        self.profile_window.reset();
        self.hover_redraw_pending = false;
        eprintln!(
            "demo: profiling {} (use --profile to enable on startup)",
            if self.profiling_enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
    }
}

fn translate_button(b: WinitMouseButton) -> MouseButton {
    match b {
        WinitMouseButton::Left => MouseButton::Primary,
        WinitMouseButton::Right => MouseButton::Secondary,
        WinitMouseButton::Middle => MouseButton::Middle,
        WinitMouseButton::Back => MouseButton::Other(3),
        WinitMouseButton::Forward => MouseButton::Other(4),
        WinitMouseButton::Other(n) => MouseButton::Other(n.min(255) as u8),
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
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.clone() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                let Some(renderer) = self.renderer.as_mut() else {
                    return;
                };
                renderer.resize(size.width, size.height);
                if let Some(layout) = self.last_layout.as_ref() {
                    self.scroll_y = clamp_scroll_y(self.scroll_y, layout, size.height as f32);
                }
                self.scrollbar_drag = None;
                window.request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(key),
                        repeat,
                        ..
                    },
                ..
            } => {
                self.update_modifier_key(key, state);
                if state == ElementState::Pressed && !repeat {
                    if key == KeyCode::F9 {
                        self.toggle_profiling();
                        return;
                    }
                    if self.modifiers.ctrl {
                        match key {
                            KeyCode::KeyA => {
                                if let (Some(tree), Some(layout)) =
                                    (self.tree.as_mut(), self.last_layout.as_ref())
                                {
                                    if wgpu_html::select_all_text(tree, layout) {
                                        window.request_redraw();
                                    }
                                }
                            }
                            KeyCode::KeyC => self.copy_selection_to_clipboard(),
                            _ => match key {
                                KeyCode::F12 => {
                                    let Some(renderer) = self.renderer.as_mut() else {
                                        return;
                                    };
                                    let path: PathBuf =
                                        format!("screenshot-{}.png", timestamp()).into();
                                    renderer.capture_next_frame_to(path);
                                    window.request_redraw();
                                }
                                KeyCode::Escape => event_loop.exit(),
                                _ => {}
                            },
                        }
                    } else {
                        match key {
                            KeyCode::F12 => {
                                let Some(renderer) = self.renderer.as_mut() else {
                                    return;
                                };
                                let path: PathBuf =
                                    format!("screenshot-{}.png", timestamp()).into();
                                renderer.capture_next_frame_to(path);
                                window.request_redraw();
                            }
                            KeyCode::Escape => event_loop.exit(),
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let pos = physical_to_pos(position);
                self.cursor_pos = Some(pos);
                if let (Some(drag), Some(layout)) =
                    (self.scrollbar_drag.clone(), self.last_layout.as_ref())
                {
                    let size = window.inner_size();
                    match drag.target {
                        ScrollTarget::Viewport => {
                            self.scroll_y = scroll_y_from_thumb_top(
                                pos.1 - drag.grab_offset_y,
                                layout,
                                size.width as f32,
                                size.height as f32,
                            );
                        }
                        ScrollTarget::Element(path) => {
                            if let Some(tree) = self.tree.as_mut() {
                                let doc_pos = viewport_to_document(pos, self.scroll_y);
                                scroll_element_thumb_to(
                                    tree,
                                    layout,
                                    path,
                                    doc_pos.1 - drag.grab_offset_y,
                                );
                            }
                        }
                    }
                    window.request_redraw();
                }
                let doc_pos = viewport_to_document(pos, self.scroll_y);
                if let (Some(tree), Some(layout)) = (self.tree.as_mut(), self.last_layout.as_ref())
                {
                    let hover_t0 = self.profiling_enabled.then(Instant::now);
                    let changed =
                        interactivity::pointer_move(tree, layout, doc_pos, self.modifiers);
                    if let Some(t0) = hover_t0 {
                        self.profile_window
                            .add_hover_move(t0.elapsed().as_secs_f64() * 1000.0, changed);
                    }
                    // Cap hover-driven redraws to ~60 fps.  A 1000 Hz mouse fires
                    // CursorMoved far faster than a full cascade+layout+render can
                    // complete, so without throttling every element boundary crossing
                    // queues a redundant pass.
                    if changed || tree.interaction.selecting_text {
                        const HOVER_FRAME_MS: u64 = 16;
                        if self.last_hover_redraw.elapsed() >= Duration::from_millis(HOVER_FRAME_MS)
                        {
                            if self.profiling_enabled {
                                self.profile_window.mark_hover_redraw_requested();
                            }
                            self.hover_redraw_pending = true;
                            window.request_redraw();
                            self.hover_pending = false;
                        } else {
                            // Defer: about_to_wait will fire one redraw after the
                            // remaining budget expires.
                            if self.profiling_enabled {
                                self.profile_window.mark_hover_redraw_deferred();
                            }
                            self.hover_redraw_pending = true;
                            self.hover_pending = true;
                        }
                    }
                }
            }
            WindowEvent::CursorLeft { .. } => {
                self.cursor_pos = None;
                if let Some(tree) = self.tree.as_mut() {
                    interactivity::pointer_leave(tree, self.modifiers);
                    window.request_redraw();
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let Some(pos) = self.cursor_pos else { return };
                if button == WinitMouseButton::Left {
                    match state {
                        ElementState::Pressed => {
                            if self.start_scrollbar_drag(pos, window.inner_size()) {
                                window.request_redraw();
                                return;
                            }
                        }
                        ElementState::Released => {
                            if self.scrollbar_drag.take().is_some() {
                                window.request_redraw();
                                return;
                            }
                        }
                    }
                }
                let doc_pos = viewport_to_document(pos, self.scroll_y);
                let btn = translate_button(button);
                if let (Some(tree), Some(layout)) = (self.tree.as_mut(), self.last_layout.as_ref())
                {
                    match state {
                        ElementState::Pressed => {
                            interactivity::mouse_down(tree, layout, doc_pos, btn, self.modifiers);
                            window.request_redraw();
                        }
                        ElementState::Released => {
                            interactivity::mouse_up(tree, layout, doc_pos, btn, self.modifiers);
                            window.request_redraw();
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(layout) = self.last_layout.as_ref() {
                    let dy = scroll_delta_to_pixels(delta);
                    if let (Some(tree), Some(pos)) = (self.tree.as_mut(), self.cursor_pos) {
                        let doc_pos = viewport_to_document(pos, self.scroll_y);
                        if scroll_element_at(tree, layout, doc_pos, dy) {
                            interactivity::pointer_move(tree, layout, doc_pos, self.modifiers);
                            window.request_redraw();
                            return;
                        }
                    }
                    self.scroll_y = clamp_scroll_y(
                        self.scroll_y + dy,
                        layout,
                        window.inner_size().height as f32,
                    );
                    if let (Some(tree), Some(pos), Some(layout)) = (
                        self.tree.as_mut(),
                        self.cursor_pos,
                        self.last_layout.as_ref(),
                    ) {
                        let doc_pos = viewport_to_document(pos, self.scroll_y);
                        interactivity::pointer_move(tree, layout, doc_pos, self.modifiers);
                    }
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                let Some(renderer) = self.renderer.as_mut() else {
                    return;
                };
                let size = window.inner_size();

                let profiling = self.profiling_enabled;
                let total_t0 = profiling.then(Instant::now);
                let mut tree_build_ms = 0.0_f64;
                let mut cascade_ms = 0.0_f64;
                let mut layout_ms = 0.0_f64;
                let mut paint_ms = 0.0_f64;
                let mut layout_paint_ms = 0.0_f64;
                let mut postprocess_ms = 0.0_f64;
                let mut atlas_upload_ms = 0.0_f64;
                let mut render_ms = 0.0_f64;

                // Build the tree on first frame; subsequent frames
                // reuse it so callbacks set in `ensure_tree_built`
                // persist.
                let tree_t0 = profiling.then(Instant::now);
                let tree_ref =
                    App::ensure_tree_built(&mut self.tree, &self.doc_html, &self.click_count);
                if let Some(t0) = tree_t0 {
                    tree_build_ms = t0.elapsed().as_secs_f64() * 1000.0;
                }

                let (mut list, layout) = if profiling {
                    let (list, layout, timings) = wgpu_html::paint_tree_returning_layout_profiled(
                        tree_ref,
                        &mut self.text_ctx,
                        size.width as f32,
                        size.height as f32,
                        1.0, // T3: fixed scale; T7 honours `scale_factor_changed`.
                    );
                    cascade_ms = timings.cascade_ms;
                    layout_ms = timings.layout_ms;
                    paint_ms = timings.paint_ms;
                    layout_paint_ms = layout_ms + paint_ms;
                    (list, layout)
                } else {
                    wgpu_html::paint_tree_returning_layout(
                        tree_ref,
                        &mut self.text_ctx,
                        size.width as f32,
                        size.height as f32,
                        1.0, // T3: fixed scale; T7 honours `scale_factor_changed`.
                    )
                };

                let postprocess_t0 = profiling.then(Instant::now);
                if let Some(layout) = layout.as_ref() {
                    self.scroll_y = clamp_scroll_y(self.scroll_y, layout, size.height as f32);
                    translate_display_list_y(&mut list, -self.scroll_y);
                    paint_viewport_scrollbar(
                        &mut list,
                        layout,
                        size.width as f32,
                        size.height as f32,
                        self.scroll_y,
                    );
                } else {
                    self.scroll_y = 0.0;
                }
                self.last_layout = layout;
                if let Some(t0) = postprocess_t0 {
                    postprocess_ms = t0.elapsed().as_secs_f64() * 1000.0;
                }

                // Push any newly-rasterised glyph rasters into the
                // renderer's GPU atlas before the draw.
                let atlas_upload_t0 = profiling.then(Instant::now);
                self.text_ctx
                    .atlas
                    .upload(&renderer.queue, renderer.glyph_atlas_texture());
                if let Some(t0) = atlas_upload_t0 {
                    atlas_upload_ms = t0.elapsed().as_secs_f64() * 1000.0;
                }

                let render_t0 = profiling.then(Instant::now);
                match renderer.render(&list) {
                    FrameOutcome::Presented | FrameOutcome::Skipped => {}
                    FrameOutcome::Reconfigure => {
                        renderer.resize(size.width, size.height);
                    }
                }
                if let Some(t0) = render_t0 {
                    render_ms = t0.elapsed().as_secs_f64() * 1000.0;
                }

                if let Some(t0) = total_t0 {
                    let total_ms = t0.elapsed().as_secs_f64() * 1000.0;
                    self.profile_window.add_frame(
                        total_ms,
                        tree_build_ms,
                        cascade_ms,
                        layout_ms,
                        paint_ms,
                        layout_paint_ms,
                        postprocess_ms,
                        atlas_upload_ms,
                        render_ms,
                        list.quads.len(),
                        list.glyphs.len(),
                        list.images.len(),
                        list.clips.len(),
                    );
                    if self.hover_redraw_pending {
                        self.profile_window.add_hover_frame_detailed(
                            total_ms,
                            cascade_ms,
                            layout_ms,
                            paint_ms,
                            layout_paint_ms,
                            render_ms,
                        );
                    }
                }
                self.hover_redraw_pending = false;

                // Record when this frame finished so the hover throttle knows
                // how much of the 16 ms budget remains.
                self.last_hover_redraw = Instant::now();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.profiling_enabled {
            if let Some(line) = self.profile_window.take_line_if_due() {
                eprintln!("{line}");
            }
        }

        let mut next_deadline: Option<Instant> = if self.profiling_enabled {
            Some(self.profile_window.next_deadline())
        } else {
            None
        };

        if self.hover_pending {
            let elapsed = self.last_hover_redraw.elapsed();
            let budget = Duration::from_millis(16);
            if elapsed >= budget {
                // Frame budget exhausted — fire the deferred hover redraw now.
                self.hover_pending = false;
                if let Some(window) = self.window.as_ref() {
                    if self.profiling_enabled {
                        self.profile_window.mark_hover_redraw_requested();
                    }
                    self.hover_redraw_pending = true;
                    window.request_redraw();
                }
            } else {
                let hover_deadline = Instant::now() + (budget - elapsed);
                next_deadline = Some(match next_deadline {
                    Some(existing) => existing.min(hover_deadline),
                    None => hover_deadline,
                });
            }
        }

        if let Some(deadline) = next_deadline {
            event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
        }
    }
}

#[derive(Debug, Clone)]
struct ScrollbarDrag {
    target: ScrollTarget,
    grab_offset_y: f32,
}

#[derive(Debug, Clone)]
enum ScrollTarget {
    Viewport,
    Element(Vec<usize>),
}

#[derive(Debug, Clone, Copy)]
struct ScrollbarGeometry {
    track: Rect,
    thumb: Rect,
    max_scroll: f32,
    travel: f32,
}

impl App {
    fn start_scrollbar_drag(&mut self, pos: (f32, f32), size: PhysicalSize<u32>) -> bool {
        let Some(layout) = self.last_layout.as_ref() else {
            return false;
        };
        if let Some(tree) = self.tree.as_mut() {
            let doc_pos = viewport_to_document(pos, self.scroll_y);
            if let Some((path, geom)) = deepest_element_scrollbar_at(
                layout,
                doc_pos,
                &tree.interaction.scroll_offsets_y,
                &mut Vec::new(),
            ) {
                if rect_contains(geom.thumb, doc_pos) {
                    self.scrollbar_drag = Some(ScrollbarDrag {
                        target: ScrollTarget::Element(path),
                        grab_offset_y: doc_pos.1 - geom.thumb.y,
                    });
                    return true;
                }
                if rect_contains(geom.track, doc_pos) {
                    let thumb_top = doc_pos.1 - geom.thumb.h * 0.5;
                    scroll_element_thumb_to(tree, layout, path.clone(), thumb_top);
                    if let Some(box_) = layout.box_at_path(&path) {
                        let new_scroll = tree
                            .interaction
                            .scroll_offsets_y
                            .get(&path)
                            .copied()
                            .unwrap_or(0.0);
                        if let Some(updated) = element_scrollbar_geometry(box_, new_scroll) {
                            self.scrollbar_drag = Some(ScrollbarDrag {
                                target: ScrollTarget::Element(path),
                                grab_offset_y: doc_pos.1 - updated.thumb.y,
                            });
                        }
                    }
                    return true;
                }
            }
        }
        let Some(geom) =
            scrollbar_geometry(layout, size.width as f32, size.height as f32, self.scroll_y)
        else {
            return false;
        };
        if rect_contains(geom.thumb, pos) {
            self.scrollbar_drag = Some(ScrollbarDrag {
                target: ScrollTarget::Viewport,
                grab_offset_y: pos.1 - geom.thumb.y,
            });
            return true;
        }
        if rect_contains(geom.track, pos) {
            let thumb_top = pos.1 - geom.thumb.h * 0.5;
            self.scroll_y =
                scroll_y_from_thumb_top(thumb_top, layout, size.width as f32, size.height as f32);
            if let Some(updated) =
                scrollbar_geometry(layout, size.width as f32, size.height as f32, self.scroll_y)
            {
                self.scrollbar_drag = Some(ScrollbarDrag {
                    target: ScrollTarget::Viewport,
                    grab_offset_y: pos.1 - updated.thumb.y,
                });
            }
            return true;
        }
        false
    }
}

/// Convert a `PhysicalPosition<f64>` from winit into the engine's
/// `(f32, f32)` physical-pixel pair.
fn physical_to_pos(p: PhysicalPosition<f64>) -> (f32, f32) {
    (p.x as f32, p.y as f32)
}

fn viewport_to_document(pos: (f32, f32), scroll_y: f32) -> (f32, f32) {
    (pos.0, pos.1 + scroll_y)
}

fn scroll_delta_to_pixels(delta: MouseScrollDelta) -> f32 {
    match delta {
        MouseScrollDelta::LineDelta(_, y) => -y * 48.0,
        MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
    }
}

fn clamp_scroll_y(scroll_y: f32, layout: &LayoutBox, viewport_h: f32) -> f32 {
    scroll_y.clamp(0.0, max_scroll_y(layout, viewport_h))
}

fn max_scroll_y(layout: &LayoutBox, viewport_h: f32) -> f32 {
    (document_bottom(layout) - viewport_h).max(0.0)
}

fn scrollbar_geometry(
    layout: &LayoutBox,
    viewport_w: f32,
    viewport_h: f32,
    scroll_y: f32,
) -> Option<ScrollbarGeometry> {
    let doc_h = document_bottom(layout).max(viewport_h);
    if doc_h <= viewport_h + 0.5 || viewport_w < 12.0 || viewport_h <= 0.0 {
        return None;
    }

    let track_w = 10.0;
    let margin = 2.0;
    let track = Rect::new(
        viewport_w - track_w - margin,
        margin,
        track_w,
        viewport_h - margin * 2.0,
    );
    let thumb_h = (track.h * viewport_h / doc_h).clamp(24.0, track.h);
    let max_scroll = max_scroll_y(layout, viewport_h);
    let travel = (track.h - thumb_h).max(0.0);
    let thumb_y = track.y + travel * (scroll_y / max_scroll.max(1.0));
    let thumb = Rect::new(track.x + 1.0, thumb_y, track.w - 2.0, thumb_h);

    Some(ScrollbarGeometry {
        track,
        thumb,
        max_scroll,
        travel,
    })
}

fn scroll_y_from_thumb_top(
    thumb_top: f32,
    layout: &LayoutBox,
    viewport_w: f32,
    viewport_h: f32,
) -> f32 {
    let Some(geom) = scrollbar_geometry(layout, viewport_w, viewport_h, 0.0) else {
        return 0.0;
    };
    if geom.travel <= 0.0 {
        return 0.0;
    }
    let t = ((thumb_top - geom.track.y) / geom.travel).clamp(0.0, 1.0);
    t * geom.max_scroll
}

fn rect_contains(rect: Rect, pos: (f32, f32)) -> bool {
    pos.0 >= rect.x && pos.0 < rect.x + rect.w && pos.1 >= rect.y && pos.1 < rect.y + rect.h
}

fn scroll_element_at(
    tree: &mut Tree,
    layout: &LayoutBox,
    doc_pos: (f32, f32),
    delta_y: f32,
) -> bool {
    let Some(path) = deepest_scrollable_path_at(
        layout,
        doc_pos,
        &tree.interaction.scroll_offsets_y,
        &mut Vec::new(),
    ) else {
        return false;
    };
    let Some(box_) = layout.box_at_path(&path) else {
        return false;
    };
    let max_scroll = max_element_scroll_y(box_);
    if max_scroll <= 0.0 {
        return false;
    }

    let old = tree
        .interaction
        .scroll_offsets_y
        .get(&path)
        .copied()
        .unwrap_or(0.0)
        .clamp(0.0, max_scroll);
    let new = (old + delta_y).clamp(0.0, max_scroll);
    if (new - old).abs() <= 0.5 {
        return false;
    }

    if new <= 0.0 {
        tree.interaction.scroll_offsets_y.remove(&path);
    } else {
        tree.interaction.scroll_offsets_y.insert(path, new);
    }
    true
}

fn scroll_element_thumb_to(tree: &mut Tree, layout: &LayoutBox, path: Vec<usize>, thumb_top: f32) {
    let Some(box_) = layout.box_at_path(&path) else {
        return;
    };
    let Some(geom) = element_scrollbar_geometry(box_, 0.0) else {
        return;
    };
    if geom.travel <= 0.0 {
        return;
    }
    let t = ((thumb_top - geom.track.y) / geom.travel).clamp(0.0, 1.0);
    let scroll_y = t * geom.max_scroll;
    if scroll_y <= 0.0 {
        tree.interaction.scroll_offsets_y.remove(&path);
    } else {
        tree.interaction.scroll_offsets_y.insert(path, scroll_y);
    }
}

fn deepest_element_scrollbar_at(
    b: &LayoutBox,
    pos: (f32, f32),
    offsets: &std::collections::BTreeMap<Vec<usize>, f32>,
    path: &mut Vec<usize>,
) -> Option<(Vec<usize>, ScrollbarGeometry)> {
    let own_scroll = offsets
        .get(path)
        .copied()
        .unwrap_or(0.0)
        .clamp(0.0, max_element_scroll_y(b));
    let child_pos = (pos.0, pos.1 + own_scroll);
    for (i, child) in b.children.iter().enumerate().rev() {
        if !child.border_rect.contains(child_pos.0, child_pos.1) {
            continue;
        }
        path.push(i);
        if let Some(found) = deepest_element_scrollbar_at(child, child_pos, offsets, path) {
            path.pop();
            return Some(found);
        }
        path.pop();
    }

    let geom = element_scrollbar_geometry(b, own_scroll)?;
    (rect_contains(geom.track, pos) || rect_contains(geom.thumb, pos)).then(|| (path.clone(), geom))
}

fn deepest_scrollable_path_at(
    b: &LayoutBox,
    pos: (f32, f32),
    offsets: &std::collections::BTreeMap<Vec<usize>, f32>,
    path: &mut Vec<usize>,
) -> Option<Vec<usize>> {
    let own_scroll = offsets
        .get(path)
        .copied()
        .unwrap_or(0.0)
        .clamp(0.0, max_element_scroll_y(b));
    let child_pos = (pos.0, pos.1 + own_scroll);
    for (i, child) in b.children.iter().enumerate().rev() {
        if !child.border_rect.contains(child_pos.0, child_pos.1) {
            continue;
        }
        path.push(i);
        if let Some(found) = deepest_scrollable_path_at(child, child_pos, offsets, path) {
            path.pop();
            return Some(found);
        }
        path.pop();
    }

    let pad = element_padding_box(b);
    if matches!(
        b.overflow.y,
        wgpu_html::models::common::css_enums::Overflow::Scroll
            | wgpu_html::models::common::css_enums::Overflow::Auto
    ) && max_element_scroll_y(b) > 0.0
        && rect_contains(pad, pos)
    {
        return Some(path.clone());
    }
    None
}

fn element_scrollbar_geometry(b: &LayoutBox, scroll_y: f32) -> Option<ScrollbarGeometry> {
    if !matches!(
        b.overflow.y,
        wgpu_html::models::common::css_enums::Overflow::Scroll
            | wgpu_html::models::common::css_enums::Overflow::Auto
    ) {
        return None;
    }
    let pad = element_padding_box(b);
    if pad.w <= 0.0 || pad.h <= 0.0 {
        return None;
    }
    let scroll_h = scrollable_content_height(b).max(pad.h);
    let max_scroll = (scroll_h - pad.h).max(0.0);
    if max_scroll <= 0.0 {
        return None;
    }
    let track_w = 10.0_f32.min(pad.w);
    let track = Rect::new(pad.x + pad.w - track_w, pad.y, track_w, pad.h);
    let thumb_h = (pad.h * pad.h / scroll_h).clamp(18.0_f32.min(pad.h), pad.h);
    let travel = (pad.h - thumb_h).max(0.0);
    let thumb_y = track.y + travel * (scroll_y.clamp(0.0, max_scroll) / max_scroll.max(1.0));
    let thumb = Rect::new(
        track.x + 2.0,
        thumb_y + 2.0,
        (track.w - 4.0).max(1.0),
        (thumb_h - 4.0).max(1.0),
    );
    Some(ScrollbarGeometry {
        track,
        thumb,
        max_scroll,
        travel,
    })
}

fn max_element_scroll_y(b: &LayoutBox) -> f32 {
    (scrollable_content_height(b) - element_padding_box(b).h).max(0.0)
}

fn element_padding_box(b: &LayoutBox) -> Rect {
    Rect::new(
        b.border_rect.x + b.border.left,
        b.border_rect.y + b.border.top,
        (b.border_rect.w - b.border.horizontal()).max(0.0),
        (b.border_rect.h - b.border.vertical()).max(0.0),
    )
}

fn scrollable_content_height(b: &LayoutBox) -> f32 {
    let pad = element_padding_box(b);
    let mut bottom = pad.y + pad.h;
    for child in &b.children {
        bottom = bottom.max(element_subtree_bottom(child));
    }
    (bottom - pad.y).max(0.0)
}

fn element_subtree_bottom(b: &LayoutBox) -> f32 {
    let mut bottom = b.margin_rect.y + b.margin_rect.h;
    for child in &b.children {
        bottom = bottom.max(element_subtree_bottom(child));
    }
    bottom
}

fn document_bottom(b: &LayoutBox) -> f32 {
    b.children
        .iter()
        .map(document_bottom)
        .fold(b.margin_rect.y + b.margin_rect.h, f32::max)
}

fn translate_display_list_y(list: &mut DisplayList, dy: f32) {
    for quad in &mut list.quads {
        quad.rect.y += dy;
    }
    for image in &mut list.images {
        image.rect.y += dy;
    }
    for glyph in &mut list.glyphs {
        glyph.rect.y += dy;
    }
    for clip in &mut list.clips {
        if let Some(rect) = clip.rect.as_mut() {
            rect.y += dy;
        }
    }
}

fn paint_viewport_scrollbar(
    list: &mut DisplayList,
    layout: &LayoutBox,
    viewport_w: f32,
    viewport_h: f32,
    scroll_y: f32,
) {
    let Some(geom) = scrollbar_geometry(layout, viewport_w, viewport_h, scroll_y) else {
        return;
    };

    list.push_clip(None, [0.0; 4], [0.0; 4]);
    list.push_quad(geom.track, [0.0, 0.0, 0.0, 0.18]);
    list.push_quad(geom.thumb, [0.0, 0.0, 0.0, 0.55]);
    list.finalize();
}

/// Seconds since the Unix epoch, used as a unique-ish screenshot filename.
fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn print_usage(program: &str) {
    println!("Usage: {program} [--profile] [HTML_FILE]");
    println!();
    println!("If HTML_FILE is omitted, the built-in demo document is used:");
    println!("  {DEFAULT_DOC_PATH}");
    println!();
    println!("Options:");
    println!("  --profile   enable per-frame profiling logs at startup");
    println!();
    println!("Examples:");
    println!("  {program}");
    println!("  {program} --profile");
    println!("  {program} crates/wgpu-html-demo/html/flex-browser-like.html");
    println!("  {program} --profile crates/wgpu-html-demo/html/events-test.html");
}

fn resolve_doc_from_args() -> Result<(String, String, bool), ExitCode> {
    let mut args = env::args_os();
    let program = args
        .next()
        .map(|arg| arg.to_string_lossy().into_owned())
        .unwrap_or_else(|| "wgpu-html-demo".to_owned());

    let mut profiling_enabled = false;
    let mut doc_arg: Option<std::ffi::OsString> = None;

    for arg in args {
        let text = arg.to_string_lossy();
        match text.as_ref() {
            "-h" | "--help" => {
                print_usage(&program);
                return Err(ExitCode::SUCCESS);
            }
            "--profile" => profiling_enabled = true,
            _ if text.starts_with('-') => {
                eprintln!("demo: unknown flag: {text}\n");
                print_usage(&program);
                return Err(ExitCode::FAILURE);
            }
            _ => {
                if let Some(extra) = doc_arg.replace(arg) {
                    eprintln!(
                        "demo: unexpected extra argument: {}\n",
                        extra.to_string_lossy()
                    );
                    print_usage(&program);
                    return Err(ExitCode::FAILURE);
                }
            }
        }
    }

    let Some(doc_arg) = doc_arg else {
        return Ok((
            DEFAULT_DOC.to_owned(),
            format!("embedded default ({DEFAULT_DOC_PATH})"),
            profiling_enabled,
        ));
    };

    let path = PathBuf::from(doc_arg);
    let html = match std::fs::read_to_string(&path) {
        Ok(html) => html,
        Err(err) => {
            eprintln!(
                "demo: failed to read HTML document '{}': {err}",
                path.display()
            );
            return Err(ExitCode::FAILURE);
        }
    };

    Ok((html, path.display().to_string(), profiling_enabled))
}

fn main() -> ExitCode {
    println!("wgpu-html demo:");
    println!("  F12  →  save current frame as screenshot-<unix>.png");
    println!("  F9   →  toggle frame profiling logs");
    println!("  Esc  →  quit");
    let (doc_html, doc_source, profiling_enabled) = match resolve_doc_from_args() {
        Ok(doc) => doc,
        Err(code) => return code,
    };
    println!("  doc  →  {doc_source}");
    if demo_fonts().is_empty() {
        eprintln!(
            "demo: no system font found at the candidate paths — text \
             will render as zero-size. Edit `FONT_FAMILIES` in main.rs \
             to point at a TTF on your machine."
        );
    }

    let event_loop = EventLoop::new().expect("event loop");
    // Event-driven redraws: avoid running full layout/paint/render when idle.
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    if profiling_enabled {
        eprintln!("demo: profiling enabled via --profile");
    }
    let mut app = App::new(doc_html, profiling_enabled);
    event_loop.run_app(&mut app).expect("event loop run");
    ExitCode::SUCCESS
}
