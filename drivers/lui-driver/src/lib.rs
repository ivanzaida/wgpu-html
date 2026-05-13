//! V2 driver runtime — owns the full pipeline from HTML to GPU.
//!
//! ```text
//! HTML → parse → cascade → layout → paint → DisplayList → RenderBackend
//! ```
//!
//! The `Runtime` struct owns all persistent state (cascade context, layout
//! engine, text context). Platform integration crates (winit, egui, bevy)
//! own a `Runtime` and call `render_frame()` each frame.

use lui_cascade::cascade::{CascadeContext, InteractionState};
use lui_cascade::media::MediaContext;
use lui_glyph::TextContext;
use lui_layout::engine::LayoutEngine;
use lui_parse::HtmlDocument;
pub use lui_display_list::{DisplayList, FrameOutcome};
pub use lui_render_api::RenderBackend;

/// Minimal trait for platform windows.
pub trait Driver {
    fn inner_size(&self) -> (u32, u32);
    fn scale_factor(&self) -> f64;
    fn request_redraw(&self);
}

/// Owns the full v2 pipeline. Generic over the platform driver and render backend.
pub struct Runtime<D: Driver, B: RenderBackend> {
    pub driver: D,
    pub renderer: B,
    pub text_ctx: TextContext,
    cascade_ctx: CascadeContext,
    layout_engine: LayoutEngine,
    frame_count: u64,
}

impl<D: Driver, B: RenderBackend> Runtime<D, B> {
    pub fn new(driver: D, renderer: B) -> Self {
        Self {
            driver,
            renderer,
            text_ctx: TextContext::new(),
            cascade_ctx: CascadeContext::new(),
            layout_engine: LayoutEngine::new(),
            frame_count: 0,
        }
    }

    /// Set the stylesheets used for cascade. Call once at startup or when CSS changes.
    pub fn set_stylesheets(&mut self, sheets: &[lui_parse::Stylesheet]) {
        self.cascade_ctx.set_stylesheets(sheets);
    }

    /// Run the full pipeline: cascade → layout → paint → render.
    /// Returns the frame outcome from the render backend.
    pub fn render_frame(&mut self, doc: &HtmlDocument) -> FrameOutcome {
        let (w, h) = self.driver.inner_size();
        let vw = w as f32;
        let vh = h as f32;

        let media = MediaContext::default();
        let interaction = InteractionState::default();
        let styled = self.cascade_ctx.cascade(&doc.root, &media, &interaction);

        let tree = self.layout_engine.layout(&styled, vw, vh, &mut self.text_ctx);

        if self.frame_count == 0 {
            fn dump_tree(b: &lui_layout::LayoutBox, depth: usize) {
                let indent = "  ".repeat(depth);
                eprintln!("{}{:?} tag={} x={:.0} y={:.0} w={:.0} h={:.0} ov={:?}/{:?}",
                    indent, b.kind, b.node.element.tag_name(),
                    b.content.x, b.content.y, b.content.width, b.content.height,
                    b.overflow_x, b.overflow_y);
                for c in &b.children {
                    dump_tree(c, depth + 1);
                }
            }
            eprintln!("[lui-driver] layout tree:");
            dump_tree(&tree.root, 0);
        }
        let list = lui_paint::paint(&tree, &mut self.text_ctx);

        self.text_ctx.flush_dirty(|rect, data| {
            self.renderer.upload_atlas_region(rect.x, rect.y, rect.w, rect.h, data);
        });

        if self.frame_count == 0 {
            eprintln!("[lui-driver] frame 0: {} quads, {} glyphs, {} images, {} clips",
                list.quads.len(), list.glyphs.len(), list.images.len(), list.clips.len());
            for (i, q) in list.quads.iter().enumerate().take(15) {
                eprintln!("  quad[{}]: x={:.0} y={:.0} w={:.0} h={:.0} color=[{:.2},{:.2},{:.2},{:.2}]",
                    i, q.rect.x, q.rect.y, q.rect.w, q.rect.h,
                    q.color[0], q.color[1], q.color[2], q.color[3]);
            }
            for (i, c) in list.clips.iter().enumerate().take(5) {
                eprintln!("  clip[{}]: rect={:?}", i, c.rect);
            }
        }
        self.frame_count += 1;

        self.renderer.render(&list)
    }

    /// Paint without rendering — useful for testing or headless use.
    pub fn paint_frame(&mut self, doc: &HtmlDocument, vw: f32, vh: f32) -> DisplayList {
        let media = MediaContext::default();
        let interaction = InteractionState::default();
        let styled = self.cascade_ctx.cascade(&doc.root, &media, &interaction);
        let tree = self.layout_engine.layout(&styled, vw, vh, &mut self.text_ctx);
        lui_paint::paint(&tree, &mut self.text_ctx)
    }
}
