use std::collections::BTreeMap;

use lui_layout_old::LayoutBox;
use lui_renderer_wgpu::{DisplayList, Rect};
use lui_text::TextContext;
use lui_tree::{Node, ScrollOffset, Tree};

fn r(lr: lui_layout_old::Rect) -> Rect {
  Rect::new(lr.x, lr.y, lr.w, lr.h)
}

const OVERLAY_MARGIN: [f32; 4] = [0xF6 as f32 / 255.0, 0xB2 as f32 / 255.0, 0x6B as f32 / 255.0, 0.6];
const OVERLAY_BORDER: [f32; 4] = [0xFB as f32 / 255.0, 0xBC as f32 / 255.0, 0x04 as f32 / 255.0, 0.6];
const OVERLAY_PADDING: [f32; 4] = [0x81 as f32 / 255.0, 0xC9 as f32 / 255.0, 0x95 as f32 / 255.0, 0.6];
const OVERLAY_CONTENT: [f32; 4] = [0x66 as f32 / 255.0, 0x9D as f32 / 255.0, 0xF6 as f32 / 255.0, 0.6];

const TOOLTIP_BG: [f32; 4] = [0.15, 0.15, 0.15, 0.92];
const TOOLTIP_TAG: [f32; 4] = [0.36, 0.69, 0.84, 1.0];
const TOOLTIP_CLASS: [f32; 4] = [0.95, 0.55, 0.35, 1.0];
const TOOLTIP_DIM: [f32; 4] = [0.85, 0.85, 0.85, 1.0];

const TOOLTIP_FONT_SIZE: f32 = 11.0;
const TOOLTIP_PAD_H: f32 = 8.0;
const TOOLTIP_PAD_V: f32 = 4.0;

pub fn paint_inspect_overlay(
  list: &mut DisplayList,
  root: &LayoutBox,
  tree: &Tree,
  text_ctx: &mut TextContext,
  path: &[usize],
  scroll_y: f32,
  scale: f32,
  viewport_w: f32,
  viewport_h: f32,
) {
  let Some(b) = box_at_path(root, path) else {
    return;
  };

  let (element_scroll_x, element_scroll_y) = ancestor_scroll_offset(path, &tree.interaction.scroll_offsets);
  let dx = -element_scroll_x;
  let dy = -scroll_y - element_scroll_y;

  let vis_top = b.margin_rect.y + dy;
  let vis_bottom = vis_top + b.margin_rect.h;
  if vis_bottom < 0.0 || vis_top > viewport_h {
    return;
  }

  let mr = r(b.margin_rect);
  let br = r(b.border_rect);
  let pad = b.border;
  let cr = r(b.content_rect);

  let padding_rect = Rect::new(
    br.x + pad.left,
    br.y + pad.top,
    (br.w - pad.left - pad.right).max(0.0),
    (br.h - pad.top - pad.bottom).max(0.0),
  );

  paint_frame(list, mr, br, OVERLAY_MARGIN, dx, dy);
  paint_frame(list, br, padding_rect, OVERLAY_BORDER, dx, dy);
  paint_frame(list, padding_rect, cr, OVERLAY_PADDING, dx, dy);

  if cr.w > 0.0 && cr.h > 0.0 {
    list.push_quad(Rect::new(cr.x + dx, cr.y + dy, cr.w, cr.h), OVERLAY_CONTENT);
  }

  // Tooltip
  if let Some(node) = node_at_path(tree, path) {
    paint_tooltip(list, text_ctx, node, &br, &mr, dx, dy, scale, viewport_w);
  }
}

fn paint_tooltip(
  list: &mut DisplayList,
  text_ctx: &mut TextContext,
  node: &Node,
  border_rect: &Rect,
  margin_rect: &Rect,
  dx: f32,
  dy: f32,
  scale: f32,
  viewport_w: f32,
) {
  let tag = node.element.tag_name();
  if tag == "#text" {
    return;
  }

  let id = node.element.id();
  let class_list = node.class_list();
  let w_px = (border_rect.w / scale).round() as u32;
  let h_px = (border_rect.h / scale).round() as u32;

  let mut segments: Vec<(&str, [f32; 4])> = Vec::new();

  let tag_str = tag.to_string();
  segments.push((&tag_str, TOOLTIP_TAG));

  let id_str = id.map(|v| format!("#{v}")).unwrap_or_default();
  if !id_str.is_empty() {
    segments.push((&id_str, TOOLTIP_TAG));
  }

  let class_str = if class_list.is_empty() {
    String::new()
  } else {
    format!(
      ".{}",
      class_list.iter().map(|c| c.as_ref()).collect::<Vec<_>>().join(".")
    )
  };
  if !class_str.is_empty() {
    segments.push((&class_str, TOOLTIP_CLASS));
  }

  let dim_str = format!(" {w_px}\u{00d7}{h_px}");
  segments.push((&dim_str, TOOLTIP_DIM));

  let font_size = TOOLTIP_FONT_SIZE * scale;
  let families = ["sans-serif"];
  let font = text_ctx.pick_font(&families, 400, lui_tree::FontStyleAxis::Normal);
  let Some(font_handle) = font else { return };

  // Measure each segment
  let mut total_w = 0.0_f32;
  let mut max_h = 0.0_f32;
  let mut runs = Vec::new();
  for (text, color) in &segments {
    let shaped = text_ctx.shape_and_pack(
      text,
      font_handle,
      font_size,
      font_size * 1.3,
      0.0,
      400,
      lui_tree::FontStyleAxis::Normal,
      None,
      *color,
    );
    if let Some(run) = shaped {
      total_w += run.width;
      if run.height > max_h {
        max_h = run.height;
      }
      runs.push(Some(run));
    } else {
      runs.push(None);
    }
  }

  let pill_w = total_w + TOOLTIP_PAD_H * scale * 2.0;
  let pill_h = max_h + TOOLTIP_PAD_V * scale * 2.0;

  let mut pill_x = margin_rect.x + dx;
  let mut pill_y = margin_rect.y + dy - pill_h - 4.0 * scale;

  if pill_y < 0.0 {
    pill_y = margin_rect.y + margin_rect.h + dy + 4.0 * scale;
  }
  if pill_x + pill_w > viewport_w {
    pill_x = (viewport_w - pill_w).max(0.0);
  }

  let radius = 3.0 * scale;
  list.push_quad_rounded(Rect::new(pill_x, pill_y, pill_w, pill_h), TOOLTIP_BG, [radius; 4]);

  let mut cursor_x = pill_x + TOOLTIP_PAD_H * scale;
  let text_y = pill_y + TOOLTIP_PAD_V * scale;
  for run in &runs {
    if let Some(run) = run {
      for g in &run.glyphs {
        list.push_glyph(
          Rect::new((cursor_x + g.x).round(), (text_y + g.y).round(), g.w, g.h),
          g.color,
          g.uv_min,
          g.uv_max,
        );
      }
      cursor_x += run.width;
    }
  }
}

fn paint_frame(list: &mut DisplayList, outer: Rect, inner: Rect, color: [f32; 4], dx: f32, dy: f32) {
  let top_h = (inner.y - outer.y).max(0.0);
  if top_h > 0.0 {
    list.push_quad(Rect::new(outer.x + dx, outer.y + dy, outer.w, top_h), color);
  }
  let bottom_y = inner.y + inner.h;
  let bottom_h = ((outer.y + outer.h) - bottom_y).max(0.0);
  if bottom_h > 0.0 {
    list.push_quad(Rect::new(outer.x + dx, bottom_y + dy, outer.w, bottom_h), color);
  }
  let mid_y = inner.y;
  let mid_h = inner.h;
  let left_w = (inner.x - outer.x).max(0.0);
  if left_w > 0.0 && mid_h > 0.0 {
    list.push_quad(Rect::new(outer.x + dx, mid_y + dy, left_w, mid_h), color);
  }
  let right_x = inner.x + inner.w;
  let right_w = ((outer.x + outer.w) - right_x).max(0.0);
  if right_w > 0.0 && mid_h > 0.0 {
    list.push_quad(Rect::new(right_x + dx, mid_y + dy, right_w, mid_h), color);
  }
}

pub fn box_at_path<'a>(root: &'a LayoutBox, path: &[usize]) -> Option<&'a LayoutBox> {
  let mut current = root;
  for &idx in path {
    current = current.children.get(idx)?;
  }
  Some(current)
}

fn ancestor_scroll_offset(path: &[usize], scroll_offsets: &BTreeMap<Vec<usize>, ScrollOffset>) -> (f32, f32) {
  let mut total_x = 0.0_f32;
  let mut total_y = 0.0_f32;
  for len in 0..=path.len() {
    let ancestor = &path[..len];
    if let Some(offset) = scroll_offsets.get(ancestor) {
      total_x += offset.x;
      total_y += offset.y;
    }
  }
  (total_x, total_y)
}

fn node_at_path<'a>(tree: &'a Tree, path: &[usize]) -> Option<&'a Node> {
  let root = tree.root.as_ref()?;
  if path.is_empty() {
    return Some(root);
  }
  root.at_path(path)
}
