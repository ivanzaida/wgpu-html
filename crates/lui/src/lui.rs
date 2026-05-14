use std::{collections::BTreeMap, path::Path, sync::Arc, time::Instant};

use lui_cascade::{
  cascade::{CascadeContext, InteractionState},
  media::MediaContext,
};
use lui_core::HtmlNode;
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
  base_sheets: Vec<Stylesheet>,
  element_scroll: BTreeMap<Vec<usize>, (f32, f32)>,
  viewport_scroll: (f32, f32),
  cursor_pos: Option<(f32, f32)>,
  cursor_moved: bool,
  hover_path: Option<Vec<usize>>,
  active_path: Option<Vec<usize>>,
  last_click: Option<ClickState>,
  scrollbar_drag: Option<ScrollbarDrag>,
  scrollbar_hover: Option<(Vec<usize>, lui_cascade::cascade::ScrollbarPart)>,
  current_cursor: String,
  pub driver: Box<dyn Driver>,
  pub renderer: Box<dyn RenderBackend>,
}

#[derive(Debug, Clone)]
struct ScrollbarDrag {
  path: Vec<usize>,
  axis: lui_layout::ScrollbarAxis,
  grab_offset: f32,
  track_start: f32,
  track_length: f32,
  thumb_length: f32,
  max_scroll: f32,
  is_viewport: bool,
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
      base_sheets: Vec::new(),
      element_scroll: BTreeMap::new(),
      viewport_scroll: (0.0, 0.0),
      cursor_pos: None,
      cursor_moved: false,
      hover_path: None,
      active_path: None,
      last_click: None,
      scrollbar_drag: None,
      scrollbar_hover: None,
      current_cursor: String::from("auto"),
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
    self.rebuild_stylesheets();
  }

  pub fn doc(&self) -> &HtmlDocument {
    &self.doc
  }
  pub fn doc_mut(&mut self) -> &mut HtmlDocument {
    &mut self.doc
  }

  pub fn set_stylesheets(&mut self, sheets: &[Stylesheet]) {
    self.base_sheets = sheets.to_vec();
    self.rebuild_stylesheets();
  }

  fn rebuild_stylesheets(&mut self) {
    let doc_sheets = extract_style_elements(&self.doc.root);
    let mut all = self.base_sheets.clone();
    all.extend(doc_sheets);
    self.cascade_ctx.set_stylesheets(&all);
  }

  pub fn register_font(&mut self, face: FontFace) -> FontHandle {
    self.text_ctx.register_font(face)
  }

  pub fn current_cursor(&self) -> &str {
    &self.current_cursor
  }

  pub fn set_dpi_scale(&mut self, scale: Option<f32>) {
    self.dpi_scale_override = scale;
  }

  pub fn set_cursor_position(&mut self, x: f32, y: f32) {
    if self.cursor_pos != Some((x, y)) {
      self.cursor_pos = Some((x, y));
      self.cursor_moved = true;
      self.update_scrollbar_drag(x, y);
    }
  }

  pub fn clear_cursor_position(&mut self) {
    self.cursor_pos = None;
    self.cursor_moved = true;
    self.hover_path = None;
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
    let prev_hover = self.hover_path.clone();
    let did_move = self.cursor_moved;
    let list = self.paint(physical_width, physical_height, scale);
    self.dispatch_hover_transitions(&prev_hover, did_move);
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

        let (remaining_x, remaining_y, changed_elements) =
          if let Some(path) = tree.deepest_scrollable_path_at(doc_x, doc_y) {
            let result = tree.scroll_chain(&path, delta_x, delta_y);
            (result.remaining_x, result.remaining_y, result.changed)
          } else {
            (delta_x, delta_y, Vec::new())
          };

        let viewport_change =
          if remaining_x.abs() > 0.001 || remaining_y.abs() > 0.001 {
            let (max_x, max_y) = tree.viewport_scroll_bounds(vw, vh);
            let new_x = (viewport_scroll.0 + remaining_x).clamp(0.0, max_x);
            let new_y = (viewport_scroll.1 + remaining_y).clamp(0.0, max_y);
            let changed = (new_x - viewport_scroll.0).abs() > 0.001
              || (new_y - viewport_scroll.1).abs() > 0.001;
            if changed { Some((new_x, new_y)) } else { None }
          } else {
            None
          };

        WheelOutcome { changed_elements, viewport_change }
      },
    );

    let mut any_changed = false;
    for (path, info) in outcome.changed_elements {
      self.element_scroll.insert(path, (info.scroll_x, info.scroll_y));
      any_changed = true;
    }
    if let Some(new_scroll) = outcome.viewport_change {
      self.viewport_scroll = new_scroll;
      any_changed = true;
    }
    any_changed
  }

  /// Resolve the DOM path under the cursor. One `with_layout` call.
  fn resolve_cursor_target(&mut self, pw: u32, ph: u32, scale: f32) -> Option<Vec<usize>> {
    let (cx, cy) = self.cursor_pos?;
    let vp = self.viewport_scroll;
    let ptr = self.with_layout(pw, ph, scale, |tree, _, _, _, _| {
      tree.hit_test(cx + vp.0, cy + vp.1).map(|n| n as *const _)
    })?;
    crate::dispatch::find_node_path(&self.doc.root, ptr)
  }

  fn fire_mouse_at(
    &mut self,
    path: &[usize],
    event_type: &str,
    button: i16,
    bubbles: bool,
    cancelable: bool,
  ) -> bool {
    let (cx, cy) = self.cursor_pos.unwrap_or((0.0, 0.0));
    let mut event = lui_core::events::DocumentEvent::MouseEvent(lui_core::events::MouseEventInit {
      ui: lui_core::events::UiEventInit {
        base: lui_core::events::EventInit {
          event_type: event_type.into(),
          bubbles,
          cancelable,
          ..Default::default()
        },
        ..Default::default()
      },
      client_x: cx as f64,
      client_y: cy as f64,
      button,
      ..Default::default()
    });
    crate::dispatch::dispatch_event(&mut self.doc.root, path, &mut event);
    event.is_default_prevented()
  }

  fn fire_mouse_event(&mut self, path: &[usize], event_type: &str, button: i16) -> bool {
    self.fire_mouse_at(path, event_type, button, true, true)
  }

  fn dispatch_hover_transitions(&mut self, prev: &Option<Vec<usize>>, did_move: bool) {
    let hover_changed = *prev != self.hover_path;
    if !hover_changed && !did_move {
      return;
    }

    let prev_path = prev.as_deref().unwrap_or(&[]);
    let curr_path = self.hover_path.clone();
    let curr = curr_path.as_deref().unwrap_or(&[]);
    let common = common_prefix_len(prev_path, curr);

    if hover_changed {
      // 1. mouseout on old target (bubbles)
      if !prev_path.is_empty() {
        self.fire_mouse_at(prev_path, "mouseout", 0, true, true);
      }

      // 2. mouseleave on ancestors being left, deepest first (no bubble)
      for depth in (common..prev_path.len()).rev() {
        self.fire_mouse_at(&prev_path[..=depth], "mouseleave", 0, false, false);
      }

      // 3. mouseover on new target (bubbles)
      if !curr.is_empty() {
        self.fire_mouse_at(curr, "mouseover", 0, true, true);
      }

      // 4. mouseenter on ancestors being entered, shallowest first (no bubble)
      for depth in common..curr.len() {
        self.fire_mouse_at(&curr[..=depth], "mouseenter", 0, false, false);
      }
    }

    // 5. mousemove on current target
    if did_move && !curr.is_empty() {
      self.fire_mouse_at(curr, "mousemove", 0, true, true);
    }
  }

  /// Dispatch a mouse event at the current cursor position.
  /// Performs one layout pass for hit-testing.
  pub fn dispatch_mouse_event(
    &mut self,
    pw: u32,
    ph: u32,
    scale: f32,
    event_type: &str,
    button: i16,
  ) -> bool {
    let Some(path) = self.resolve_cursor_target(pw, ph, scale) else {
      return false;
    };
    self.fire_mouse_event(&path, event_type, button)
  }

  pub fn handle_mouse_down(&mut self, pw: u32, ph: u32, scale: f32, button: i16) -> bool {
    if button == 0 {
      if let Some(drag) = self.try_start_scrollbar_drag(pw, ph, scale) {
        self.scrollbar_drag = Some(drag);
        return true;
      }
    }
    let path = self.resolve_cursor_target(pw, ph, scale);
    self.active_path = path.clone();
    match path {
      Some(p) => self.fire_mouse_event(&p, "mousedown", button),
      None => false,
    }
  }

  pub fn handle_mouse_up(&mut self, pw: u32, ph: u32, scale: f32, button: i16) -> bool {
    if button == 0 && self.scrollbar_drag.take().is_some() {
      return true;
    }
    self.active_path = None;
    self.dispatch_mouse_event(pw, ph, scale, "mouseup", button)
  }

  pub fn handle_click(&mut self, pw: u32, ph: u32, scale: f32, button: i16) -> bool {
    self.dispatch_mouse_event(pw, ph, scale, "click", button)
  }

  /// Combined mouseup + click/dblclick/contextmenu with a single hit-test.
  pub fn handle_mouse_release(&mut self, pw: u32, ph: u32, scale: f32, button: i16) -> bool {
    if button == 0 && self.scrollbar_drag.take().is_some() {
      return true;
    }
    self.active_path = None;
    let Some(path) = self.resolve_cursor_target(pw, ph, scale) else {
      return false;
    };
    let up = self.fire_mouse_event(&path, "mouseup", button);

    match button {
      0 => {
        let click = self.fire_mouse_event(&path, "click", button);
        let dbl = self.maybe_fire_dblclick(&path, button);
        if !dbl {
          self.record_click(button);
        }
        up || click || dbl
      }
      2 => {
        let ctx = self.fire_mouse_event(&path, "contextmenu", button);
        up || ctx
      }
      _ => up,
    }
  }

  fn maybe_fire_dblclick(&mut self, path: &[usize], button: i16) -> bool {
    let (cx, cy) = self.cursor_pos.unwrap_or((0.0, 0.0));
    let is_dbl = self.last_click.as_ref().is_some_and(|lc| {
      lc.button == button
        && lc.time.elapsed().as_millis() < DBLCLICK_THRESHOLD_MS
        && (lc.pos.0 - cx).abs() < DBLCLICK_DISTANCE_PX
        && (lc.pos.1 - cy).abs() < DBLCLICK_DISTANCE_PX
    });
    if is_dbl {
      self.last_click = None;
      self.fire_mouse_event(path, "dblclick", button);
      true
    } else {
      false
    }
  }

  fn record_click(&mut self, button: i16) {
    let (cx, cy) = self.cursor_pos.unwrap_or((0.0, 0.0));
    self.last_click = Some(ClickState {
      time: Instant::now(),
      pos: (cx, cy),
      button,
    });
  }

  fn try_start_scrollbar_drag(&mut self, pw: u32, ph: u32, scale: f32) -> Option<ScrollbarDrag> {
    let (cx, cy) = self.cursor_pos?;
    let vp = self.viewport_scroll;

    let hit = self.with_layout(pw, ph, scale, |tree, _, _, vw, vh| {
      let doc_x = cx + vp.0;
      let doc_y = cy + vp.1;

      if let Some(hit) = tree.scrollbar_hit_test(doc_x, doc_y) {
        return Some((hit, false));
      }

      let bar_w = crate::paint::viewport_scrollbar_width(tree);
      if bar_w <= 0.0 { return None; }
      let (max_x, max_y) = tree.viewport_scroll_bounds(vw, vh);
      viewport_scrollbar_hit(cx, cy, vw, vh, bar_w, vp.0, vp.1, max_x, max_y)
        .map(|hit| (hit, true))
    })?;

    let (hit, is_viewport) = hit;
    let mut drag = ScrollbarDrag {
      path: hit.path,
      axis: hit.axis,
      grab_offset: hit.grab_offset,
      track_start: hit.track_start,
      track_length: hit.track_length,
      thumb_length: hit.thumb_length,
      max_scroll: hit.max_scroll,
      is_viewport,
    };

    if !hit.on_thumb {
      self.apply_scrollbar_jump(&drag, cx, cy);
      drag.grab_offset = drag.thumb_length * 0.5;
    }

    Some(drag)
  }

  fn apply_scrollbar_jump(&mut self, drag: &ScrollbarDrag, cx: f32, cy: f32) {
    let vp = self.viewport_scroll;
    let mouse_on_track = match drag.axis {
      lui_layout::ScrollbarAxis::Vertical => cy + if !drag.is_viewport { vp.1 } else { 0.0 } - drag.track_start,
      lui_layout::ScrollbarAxis::Horizontal => cx + if !drag.is_viewport { vp.0 } else { 0.0 } - drag.track_start,
    };
    let travel = (drag.track_length - drag.thumb_length).max(0.001);
    let fraction = ((mouse_on_track - drag.thumb_length * 0.5) / travel).clamp(0.0, 1.0);
    let new_scroll = fraction * drag.max_scroll;

    if drag.is_viewport {
      match drag.axis {
        lui_layout::ScrollbarAxis::Vertical => self.viewport_scroll.1 = new_scroll,
        lui_layout::ScrollbarAxis::Horizontal => self.viewport_scroll.0 = new_scroll,
      }
    } else {
      let prev = self.element_scroll.get(&drag.path).copied().unwrap_or((0.0, 0.0));
      let updated = match drag.axis {
        lui_layout::ScrollbarAxis::Vertical => (prev.0, new_scroll),
        lui_layout::ScrollbarAxis::Horizontal => (new_scroll, prev.1),
      };
      self.element_scroll.insert(drag.path.clone(), updated);
    }
  }

  fn update_scrollbar_drag(&mut self, x: f32, y: f32) {
    let Some(drag) = &self.scrollbar_drag else { return };
    let vp = self.viewport_scroll;

    let mouse_on_track = match drag.axis {
      lui_layout::ScrollbarAxis::Vertical => y + if !drag.is_viewport { vp.1 } else { 0.0 } - drag.track_start,
      lui_layout::ScrollbarAxis::Horizontal => x + if !drag.is_viewport { vp.0 } else { 0.0 } - drag.track_start,
    };

    let travel = (drag.track_length - drag.thumb_length).max(0.001);
    let fraction = ((mouse_on_track - drag.grab_offset) / travel).clamp(0.0, 1.0);
    let new_scroll = fraction * drag.max_scroll;

    let drag = drag.clone();
    if drag.is_viewport {
      match drag.axis {
        lui_layout::ScrollbarAxis::Vertical => self.viewport_scroll.1 = new_scroll,
        lui_layout::ScrollbarAxis::Horizontal => self.viewport_scroll.0 = new_scroll,
      }
    } else {
      let prev = self.element_scroll.get(&drag.path).copied().unwrap_or((0.0, 0.0));
      let updated = match drag.axis {
        lui_layout::ScrollbarAxis::Vertical => (prev.0, new_scroll),
        lui_layout::ScrollbarAxis::Horizontal => (new_scroll, prev.1),
      };
      self.element_scroll.insert(drag.path, updated);
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
    let interaction = InteractionState {
      hover_path: self.hover_path.clone(),
      active_path: self.active_path.clone(),
      scrollbar_hover: self.scrollbar_hover.clone(),
      ..Default::default()
    };
    let styled = self.cascade_ctx.cascade(&self.doc.root, &media, &interaction);
    let mut tree = self.layout_engine.layout(&styled, vw, vh, &mut self.text_ctx);
    apply_element_scroll_state(&mut tree, &self.element_scroll);

    let (max_x, max_y) = tree.viewport_scroll_bounds(vw, vh);
    self.viewport_scroll.0 = self.viewport_scroll.0.clamp(0.0, max_x);
    self.viewport_scroll.1 = self.viewport_scroll.1.clamp(0.0, max_y);

    if self.cursor_moved {
      self.cursor_moved = false;
      if let Some((cx, cy)) = self.cursor_pos {
        let doc_x = cx + self.viewport_scroll.0;
        let doc_y = cy + self.viewport_scroll.1;
        self.hover_path = tree
          .hit_test(doc_x, doc_y)
          .and_then(|n| crate::dispatch::find_node_path(&self.doc.root, n as *const _));
        self.current_cursor = tree.cursor_at(doc_x, doc_y).to_string();

        self.scrollbar_hover = tree.scrollbar_hit_test(doc_x, doc_y).map(|hit| {
          use lui_cascade::cascade::ScrollbarPart;
          let part = if hit.on_thumb { ScrollbarPart::Thumb } else { ScrollbarPart::Track };
          (hit.path, part)
        });
      } else {
        self.hover_path = None;
        self.scrollbar_hover = None;
        self.current_cursor = String::from("auto");
      }
    }

    f(&mut tree, &mut self.text_ctx, effective_scale, vw, vh)
  }
}

fn apply_element_scroll_state(tree: &mut lui_layout::LayoutTree<'_>, state: &BTreeMap<Vec<usize>, (f32, f32)>) {
  for (path, (sx, sy)) in state {
    tree.set_scroll_at_path(path, *sx, *sy);
  }
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

struct WheelOutcome {
  changed_elements: Vec<(Vec<usize>, lui_layout::ScrollInfo)>,
  viewport_change: Option<(f32, f32)>,
}

fn common_prefix_len(a: &[usize], b: &[usize]) -> usize {
  a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count()
}

const DBLCLICK_THRESHOLD_MS: u128 = 500;
const DBLCLICK_DISTANCE_PX: f32 = 5.0;

struct ClickState {
  time: Instant,
  pos: (f32, f32),
  button: i16,
}

const VIEWPORT_SCROLLBAR_MARGIN: f32 = 2.0;

fn viewport_scrollbar_hit(
  cx: f32,
  cy: f32,
  vw: f32,
  vh: f32,
  bar_w: f32,
  scroll_x: f32,
  scroll_y: f32,
  max_x: f32,
  max_y: f32,
) -> Option<lui_layout::ScrollbarHit> {
  let margin = VIEWPORT_SCROLLBAR_MARGIN;
  let show_y = max_y > 0.5;
  let show_x = max_x > 0.5;

  if show_y {
    let track_x = vw - bar_w - margin;
    let track_y = margin;
    let track_h = vh - margin * 2.0 - if show_x { bar_w } else { 0.0 };

    if cx >= track_x && cx <= track_x + bar_w && cy >= track_y && cy <= track_y + track_h {
      let doc_h = vh + max_y;
      let thumb_h = (track_h * vh / doc_h).clamp(24.0, track_h);
      let travel = (track_h - thumb_h).max(0.0);
      let thumb_y = track_y + travel * (scroll_y / max_y.max(1.0));
      let on_thumb = cy >= thumb_y && cy <= thumb_y + thumb_h;
      return Some(lui_layout::ScrollbarHit {
        path: Vec::new(),
        axis: lui_layout::ScrollbarAxis::Vertical,
        on_thumb,
        grab_offset: if on_thumb { cy - thumb_y } else { thumb_h * 0.5 },
        track_start: track_y,
        track_length: track_h,
        thumb_length: thumb_h,
        max_scroll: max_y,
      });
    }
  }

  if show_x {
    let track_x = margin;
    let track_y = vh - bar_w - margin;
    let track_w = vw - margin * 2.0 - if show_y { bar_w } else { 0.0 };

    if cx >= track_x && cx <= track_x + track_w && cy >= track_y && cy <= track_y + bar_w {
      let doc_w = vw + max_x;
      let thumb_w = (track_w * vw / doc_w).clamp(24.0, track_w);
      let travel = (track_w - thumb_w).max(0.0);
      let thumb_x = track_x + travel * (scroll_x / max_x.max(1.0));
      let on_thumb = cx >= thumb_x && cx <= thumb_x + thumb_w;
      return Some(lui_layout::ScrollbarHit {
        path: Vec::new(),
        axis: lui_layout::ScrollbarAxis::Horizontal,
        on_thumb,
        grab_offset: if on_thumb { cx - thumb_x } else { thumb_w * 0.5 },
        track_start: track_x,
        track_length: track_w,
        thumb_length: thumb_w,
        max_scroll: max_x,
      });
    }
  }

  None
}

fn extract_style_elements(root: &HtmlNode) -> Vec<Stylesheet> {
  let mut sheets = Vec::new();
  collect_style_nodes(root, &mut sheets);
  sheets
}

fn collect_style_nodes(node: &HtmlNode, sheets: &mut Vec<Stylesheet>) {
  if node.element.tag_name() == "style" {
    let css: String = node
      .children
      .iter()
      .filter_map(|c| match &c.element {
        lui_core::HtmlElement::Text(s) => Some(s.as_ref().to_string()),
        _ => None,
      })
      .collect();
    if !css.is_empty() {
      if let Ok(sheet) = lui_parse::parse_stylesheet(&css) {
        sheets.push(sheet);
      }
    }
    return;
  }
  for child in &node.children {
    collect_style_nodes(child, sheets);
  }
}
