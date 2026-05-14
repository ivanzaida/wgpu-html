use std::{collections::BTreeMap, path::Path, sync::Arc};

use lui_cascade::{
  cascade::{CascadeContext, InteractionState},
  media::MediaContext,
};
use lui_glyph::{FontFace, FontHandle, TextContext};
use lui_layout::engine::LayoutEngine;
use lui_parse::{HtmlDocument, Stylesheet};

use crate::{
  Driver, RenderBackend, RenderError, WindowHandle,
  display_list::{DisplayList, FrameOutcome},
};

pub struct Lui {
  pub doc: HtmlDocument,
  pub(crate) text_ctx: TextContext,
  cascade_ctx: CascadeContext,
  layout_engine: LayoutEngine,
  dpi_scale_override: Option<f32>,
  element_scroll: BTreeMap<Vec<usize>, (f32, f32)>,
  viewport_scroll: (f32, f32),
  cursor_pos: Option<(f32, f32)>,
  pub driver: Box<dyn Driver>,
  pub renderer: Box<dyn RenderBackend>,
}

impl Lui {
  pub fn new(driver: Box<dyn Driver>, renderer: Box<dyn RenderBackend>) -> Self {
    #[allow(unused_mut)]
    let mut s = Self {
      doc: lui_parse::parse("<html><body></body></html>"),
      text_ctx: TextContext::new(),
      cascade_ctx: CascadeContext::new(),
      layout_engine: LayoutEngine::new(),
      dpi_scale_override: None,
      element_scroll: BTreeMap::new(),
      viewport_scroll: (0.0, 0.0),
      cursor_pos: None,
      driver,
      renderer,
    };

    #[cfg(feature = "ua_whatwg")]
    {
      use std::sync::LazyLock;
      static UA_SHEET: &str = include_str!("../ua/ua_whatwg.css");
      static PARSED_UA_SHEET: LazyLock<Stylesheet> =
        LazyLock::new(|| lui_parse::parse_stylesheet(UA_SHEET).unwrap_or_default());
      s.set_stylesheets(&[PARSED_UA_SHEET.clone()])
    }
    s
  }

  pub fn set_html(&mut self, html: &str) {
    self.doc = lui_parse::parse(html);
  }

  pub fn doc(&self) -> &HtmlDocument {
    &self.doc
  }
  pub fn doc_mut(&mut self) -> &mut HtmlDocument {
    &mut self.doc
  }

  pub fn set_stylesheets(&mut self, sheets: &[Stylesheet]) {
    self.cascade_ctx.set_stylesheets(sheets);
  }

  pub fn register_font(&mut self, face: FontFace) -> FontHandle {
    self.text_ctx.register_font(face)
  }

  pub fn set_dpi_scale(&mut self, scale: Option<f32>) {
    self.dpi_scale_override = scale;
  }

  pub fn set_cursor_position(&mut self, x: f32, y: f32) {
    self.cursor_pos = Some((x, y));
  }

  pub fn clear_cursor_position(&mut self) {
    self.cursor_pos = None;
  }

  pub fn run(mut self) {
    let driver = std::mem::replace(&mut self.driver, Box::new(crate::NullDriver));
    driver.run(self);
  }

  /// Called by the driver when the window is ready.
  pub fn init_renderer(&mut self, window: Arc<dyn WindowHandle>, width: u32, height: u32) {
    self.renderer.init(window, width, height);
  }

  pub fn render_frame(&mut self, physical_width: u32, physical_height: u32, scale: f32) -> FrameOutcome {
    let list = self.paint(physical_width, physical_height, scale);
    self.flush_atlas();
    self.renderer.render(&list)
  }

  pub fn screenshot_to(
    &mut self,
    physical_width: u32,
    physical_height: u32,
    scale: f32,
    path: impl AsRef<Path>,
  ) -> Result<(), RenderError> {
    let list = self.paint(physical_width, physical_height, scale);
    self.flush_atlas();
    self
      .renderer
      .capture_to(&list, physical_width, physical_height, path.as_ref())
  }

  fn paint(&mut self, pw: u32, ph: u32, scale: f32) -> DisplayList {
    let viewport_scroll = self.viewport_scroll;
    self.with_layout(pw, ph, scale, |tree, text_ctx, effective_scale, vw, vh| {
      let mut list = crate::paint::paint_scaled(tree, text_ctx, effective_scale);
      translate_display_list(&mut list, -viewport_scroll.0, -viewport_scroll.1);
      crate::paint::paint_viewport_scrollbars(&mut list, tree, vw, vh, viewport_scroll.0, viewport_scroll.1);
      list.finalize();
      list.dpi_scale = effective_scale;
      list
    })
  }

  fn flush_atlas(&mut self) {
    self.text_ctx.flush_dirty(|rect, data| {
      self.renderer.upload_atlas_region(rect.x, rect.y, rect.w, rect.h, data);
    });
  }

  pub fn handle_wheel(
    &mut self,
    physical_width: u32,
    physical_height: u32,
    scale: f32,
    delta_x: f32,
    delta_y: f32,
  ) -> bool {
    let Some((cursor_x, cursor_y)) = self.cursor_pos else {
      return false;
    };
    let viewport_scroll = self.viewport_scroll;
    let outcome = self.with_layout(
      physical_width,
      physical_height,
      scale,
      |tree, _text_ctx, _effective_scale, vw, vh| {
        let doc_x = cursor_x + viewport_scroll.0;
        let doc_y = cursor_y + viewport_scroll.1;

        if let Some(path) = tree.deepest_scrollable_path_at(doc_x, doc_y) {
          let changed = tree.scroll_by_at_path(&path, delta_x, delta_y);
          let scroll = if changed {
            find_scroll_box_at_path(&tree.root, &path).and_then(|scroll_box| scroll_box.scroll)
          } else {
            None
          };
          return WheelOutcome::Element { changed, path, scroll };
        }

        let (max_x, max_y) = tree.viewport_scroll_bounds(vw, vh);
        let new_x = (viewport_scroll.0 + delta_x).clamp(0.0, max_x);
        let new_y = (viewport_scroll.1 + delta_y).clamp(0.0, max_y);
        WheelOutcome::Viewport {
          new_scroll: (new_x, new_y),
          changed: (new_x - viewport_scroll.0).abs() > 0.001 || (new_y - viewport_scroll.1).abs() > 0.001,
        }
      },
    );

    match outcome {
      WheelOutcome::Element { changed, path, scroll } => {
        if changed && let Some(info) = scroll {
          self.element_scroll.insert(path, (info.scroll_x, info.scroll_y));
        }
        changed
      }
      WheelOutcome::Viewport { new_scroll, changed } => {
        self.viewport_scroll = new_scroll;
        changed
      }
    }
  }

  fn with_layout<T>(
    &mut self,
    pw: u32,
    ph: u32,
    scale: f32,
    f: impl for<'a> FnOnce(&mut lui_layout::LayoutTree<'a>, &mut TextContext, f32, f32, f32) -> T,
  ) -> T {
    let effective_scale = self.dpi_scale_override.unwrap_or(scale);
    let vw = pw as f32 / effective_scale;
    let vh = ph as f32 / effective_scale;

    let media = MediaContext {
      viewport_width: vw,
      viewport_height: vh,
      dpi: 96.0 * effective_scale,
      ..MediaContext::default()
    };
    let interaction = InteractionState::default();
    let styled = self.cascade_ctx.cascade(&self.doc.root, &media, &interaction);
    let mut tree = self.layout_engine.layout(&styled, vw, vh, &mut self.text_ctx);
    apply_element_scroll_state(&mut tree, &self.element_scroll);

    let (max_x, max_y) = tree.viewport_scroll_bounds(vw, vh);
    self.viewport_scroll.0 = self.viewport_scroll.0.clamp(0.0, max_x);
    self.viewport_scroll.1 = self.viewport_scroll.1.clamp(0.0, max_y);

    f(&mut tree, &mut self.text_ctx, effective_scale, vw, vh)
  }
}

fn apply_element_scroll_state(tree: &mut lui_layout::LayoutTree<'_>, state: &BTreeMap<Vec<usize>, (f32, f32)>) {
  for (path, (sx, sy)) in state {
    tree.set_scroll_at_path(path, *sx, *sy);
  }
}

fn find_scroll_box_at_path<'a>(
  mut current: &'a lui_layout::LayoutBox<'a>,
  path: &[usize],
) -> Option<&'a lui_layout::LayoutBox<'a>> {
  for &idx in path {
    current = current.children.get(idx)?;
  }
  Some(current)
}

fn translate_display_list(list: &mut DisplayList, dx: f32, dy: f32) {
  for quad in &mut list.quads {
    quad.rect.x += dx;
    quad.rect.y += dy;
  }
  for image in &mut list.images {
    image.rect.x += dx;
    image.rect.y += dy;
  }
  for glyph in &mut list.glyphs {
    glyph.rect.x += dx;
    glyph.rect.y += dy;
  }
  for clip in &mut list.clips {
    if let Some(rect) = clip.rect.as_mut() {
      rect.x += dx;
      rect.y += dy;
    }
  }
}

enum WheelOutcome {
  Element {
    changed: bool,
    path: Vec<usize>,
    scroll: Option<lui_layout::ScrollInfo>,
  },
  Viewport {
    new_scroll: (f32, f32),
    changed: bool,
  },
}
