//! M5 demo (T3): parse HTML, register an external font on the tree,
//! shape text via cosmic-text, render the resulting glyph quads
//! through the renderer's textured pipeline.

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::SystemTime;

use arboard::Clipboard;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{
    ElementState, KeyEvent, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, EventLoop};
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
}

impl App {
    fn new(doc_html: String) -> Self {
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
            // The button increments a shared counter; the panel logs
            // hover transitions and clicks.
            let counter = click_count.clone();
            if let Some(btn) = tree.get_element_by_id("btn") {
                btn.on_click = Some(Arc::new(move |ev| {
                    let n = counter.fetch_add(1, Ordering::Relaxed) + 1;
                    eprintln!("[btn] click #{n} at {:?}", ev.pos);
                }));
            } else {
                eprintln!("demo: no element with id=\"btn\"");
            }

            if let Some(panel) = tree.get_element_by_id("panel") {
                panel.on_mouse_enter = Some(Arc::new(|ev| {
                    eprintln!("[panel] mouse enter at {:?}", ev.pos);
                }));
                panel.on_mouse_leave = Some(Arc::new(|ev| {
                    eprintln!("[panel] mouse leave at {:?}", ev.pos);
                }));
                panel.on_click = Some(Arc::new(|ev| {
                    eprintln!("[panel] click at {:?}", ev.pos);
                }));
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
                if let (Some(drag), Some(layout)) = (self.scrollbar_drag, self.last_layout.as_ref())
                {
                    let size = window.inner_size();
                    self.scroll_y = scroll_y_from_thumb_top(
                        pos.1 - drag.grab_offset_y,
                        layout,
                        size.width as f32,
                        size.height as f32,
                    );
                    window.request_redraw();
                }
                let doc_pos = viewport_to_document(pos, self.scroll_y);
                if let (Some(tree), Some(layout)) = (self.tree.as_mut(), self.last_layout.as_ref())
                {
                    interactivity::pointer_move(tree, layout, doc_pos, self.modifiers);
                }
            }
            WindowEvent::CursorLeft { .. } => {
                self.cursor_pos = None;
                if let Some(tree) = self.tree.as_mut() {
                    interactivity::pointer_leave(tree, self.modifiers);
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
                        }
                        ElementState::Released => {
                            interactivity::mouse_up(tree, layout, doc_pos, btn, self.modifiers);
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(layout) = self.last_layout.as_ref() {
                    let dy = scroll_delta_to_pixels(delta);
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

                // Build the tree on first frame; subsequent frames
                // reuse it so callbacks set in `ensure_tree_built`
                // persist.
                let tree_ref =
                    App::ensure_tree_built(&mut self.tree, &self.doc_html, &self.click_count);
                let (mut list, layout) = wgpu_html::paint_tree_returning_layout(
                    tree_ref,
                    &mut self.text_ctx,
                    size.width as f32,
                    size.height as f32,
                    1.0, // T3: fixed scale; T7 honours `scale_factor_changed`.
                );
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

#[derive(Debug, Clone, Copy)]
struct ScrollbarDrag {
    grab_offset_y: f32,
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
        let Some(geom) =
            scrollbar_geometry(layout, size.width as f32, size.height as f32, self.scroll_y)
        else {
            return false;
        };
        if rect_contains(geom.thumb, pos) {
            self.scrollbar_drag = Some(ScrollbarDrag {
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
    println!("Usage: {program} [HTML_FILE]");
    println!();
    println!("If HTML_FILE is omitted, the built-in demo document is used:");
    println!("  {DEFAULT_DOC_PATH}");
    println!();
    println!("Examples:");
    println!("  {program}");
    println!("  {program} crates/wgpu-html-demo/html/flex-browser-like.html");
}

fn resolve_doc_from_args() -> Result<(String, String), ExitCode> {
    let mut args = env::args_os();
    let program = args
        .next()
        .map(|arg| arg.to_string_lossy().into_owned())
        .unwrap_or_else(|| "wgpu-html-demo".to_owned());

    let Some(doc_arg) = args.next() else {
        return Ok((
            DEFAULT_DOC.to_owned(),
            format!("embedded default ({DEFAULT_DOC_PATH})"),
        ));
    };

    let arg = doc_arg.to_string_lossy();
    if arg == "-h" || arg == "--help" {
        print_usage(&program);
        return Err(ExitCode::SUCCESS);
    }

    if let Some(extra) = args.next() {
        eprintln!(
            "demo: unexpected extra argument: {}\n",
            extra.to_string_lossy()
        );
        print_usage(&program);
        return Err(ExitCode::FAILURE);
    }

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

    Ok((html, path.display().to_string()))
}

fn main() -> ExitCode {
    println!("wgpu-html demo:");
    println!("  F12  →  save current frame as screenshot-<unix>.png");
    println!("  Esc  →  quit");
    let (doc_html, doc_source) = match resolve_doc_from_args() {
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
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::new(doc_html);
    event_loop.run_app(&mut app).expect("event loop run");
    ExitCode::SUCCESS
}
