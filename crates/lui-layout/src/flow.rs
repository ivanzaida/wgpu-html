//! Inline layout: horizontal flow with line breaking.

use bumpalo::Bump;
use lui_core::Rect;
use lui_parse::HtmlNode;

use crate::{
  box_tree::{BoxKind, LayoutBox},
  context::LayoutContext,
  geometry::Point,
  sides, sizes,
  text::TextContext,
};

fn css_str(v: Option<&lui_core::CssValue>) -> &str {
  match v {
    Some(lui_core::CssValue::String(s)) | Some(lui_core::CssValue::Unknown(s)) => s.as_ref(),
    _ => "",
  }
}

/// CSS `white-space: normal` collapsing: newlines/tabs → space, runs of
/// spaces → single space, strip leading and trailing whitespace.
pub fn collapse_whitespace(text: &str) -> String {
  let mut out = String::with_capacity(text.len());
  let mut prev_space = true; // treat start as "after space" to strip leading
  for ch in text.chars() {
    if ch == '\n' || ch == '\r' || ch == '\t' || ch == ' ' {
      if !prev_space {
        out.push(' ');
        prev_space = true;
      }
    } else {
      out.push(ch);
      prev_space = false;
    }
  }
  if out.ends_with(' ') {
    out.pop();
  }
  out
}

/// Layout inline content with line wrapping.
pub fn layout_inline<'a>(
  b: &mut LayoutBox<'a>,
  ctx: &LayoutContext,
  pos: Point,
  text_ctx: &mut TextContext,
  rects: &mut Vec<(&'a HtmlNode, Rect)>,
  cache: &crate::incremental::CacheView,
  bump: &'a Bump,
) {
  if let Some(text) = get_text(b.node) {
    layout_text_node(b, ctx, pos, &text, text_ctx);
  } else {
    layout_inline_container(b, ctx, pos, text_ctx, rects, cache, bump);
  }
}

fn layout_text_node(b: &mut LayoutBox, ctx: &LayoutContext, pos: Point, text: &str, text_ctx: &mut TextContext) {
  let style = lui_glyph::text_style_from_cascade(b.style);
  let white_space = css_str(b.style.white_space);
  let word_break = css_str(b.style.word_break);
  let overflow_wrap = css_str(b.style.overflow_wrap);
  let text_transform = css_str(b.style.text_transform);
  let max_width = ctx.containing_width;

  let transformed: Option<String> = match text_transform {
    "uppercase" => Some(text.to_uppercase()),
    "lowercase" => Some(text.to_lowercase()),
    "capitalize" => Some(
      text
        .split_whitespace()
        .map(|w| {
          let mut c = w.chars();
          match c.next() {
            Some(f) => f.to_uppercase().to_string() + c.as_str(),
            None => String::new(),
          }
        })
        .collect::<Vec<_>>()
        .join(" "),
    ),
    _ => None,
  };
  let text = transformed.as_deref().unwrap_or(text);

  let should_collapse = !matches!(white_space, "pre" | "pre-wrap" | "nowrap");
  let collapsed: Option<String> = if should_collapse {
    Some(collapse_whitespace(text))
  } else {
    None
  };
  let text = collapsed.as_deref().unwrap_or(text);

  if text.is_empty() {
    b.content.x = pos.x;
    b.content.y = pos.y;
    b.content.width = 0.0;
    b.content.height = 0.0;
    return;
  }

  let ls = style.letter_spacing;
  let ws = style.word_spacing;
  let adjust_width = |base_width: f32, txt: &str| -> f32 {
    let mut w = base_width;
    if ls.abs() > 0.001 {
      let chars = txt.chars().count().saturating_sub(1);
      w += ls * chars as f32;
    }
    if ws.abs() > 0.001 {
      let spaces = txt.chars().filter(|c| *c == ' ').count();
      w += ws * spaces as f32;
    }
    w
  };

  let no_wrap = matches!(white_space, "nowrap" | "pre");
  let preserve_newlines = matches!(white_space, "pre" | "pre-wrap" | "pre-line");
  let break_chars = word_break == "break-all" || overflow_wrap == "break-word";

  // Handle explicit newlines for pre-wrap/pre-line
  if preserve_newlines && text.contains('\n') {
    let mut total_height = 0.0_f32;
    let mut max_line_width = 0.0_f32;
    let mut first_ascent = None;

    for segment in text.split('\n') {
      let segment = if white_space == "pre-line" {
        segment.split_whitespace().collect::<Vec<_>>().join(" ")
      } else {
        segment.to_string()
      };
      if segment.is_empty() {
        total_height += style.line_height;
        continue;
      }

      let can_wrap = matches!(white_space, "pre-wrap" | "pre-line");
      if can_wrap && max_width > 0.0 {
        let lines = text_ctx.break_into_lines(&segment, &style, max_width);
        for line in &lines {
          max_line_width = max_line_width.max(line.width);
          total_height += line.height;
        }
        if first_ascent.is_none() && !lines.is_empty() {
          let run = text_ctx.shape(&segment, &style);
          first_ascent = Some(run.ascent);
        }
      } else {
        let run = text_ctx.shape(&segment, &style);
        max_line_width = max_line_width.max(run.width);
        total_height += run.height;
        if first_ascent.is_none() {
          first_ascent = Some(run.ascent);
        }
      }
    }

    b.content.x = pos.x;
    b.content.y = pos.y;
    b.content.width = adjust_width(max_line_width, text);
    b.content.height = total_height;
    b.baseline = first_ascent;
    return;
  }

  if !no_wrap && max_width > 0.0 && text.len() > 1 {
    let effective_text = if break_chars {
      break_long_words(text, &style, max_width, text_ctx)
    } else {
      text.to_string()
    };

    let lines = text_ctx.break_into_lines(&effective_text, &style, max_width);
    if lines.is_empty() {
      b.content.x = pos.x;
      b.content.y = pos.y;
      b.content.width = 0.0;
      b.content.height = 0.0;
      return;
    }
    let mut total_height = 0.0_f32;
    let mut max_line_width = 0.0_f32;
    for line in &lines {
      max_line_width = max_line_width.max(line.width);
      total_height += line.height;
    }
    b.content.x = pos.x;
    b.content.y = pos.y;
    b.content.width = adjust_width(max_line_width, text);
    b.content.height = total_height;
    let first_line_run = text_ctx.shape(text, &style);
    b.baseline = Some(first_line_run.ascent);
  } else {
    let run = text_ctx.shape(text, &style);
    b.content.x = pos.x;
    b.content.y = pos.y;
    b.content.width = adjust_width(run.width, text);
    b.content.height = run.height;
    b.baseline = Some(run.ascent);
  }
}

fn break_long_words(
  text: &str,
  style: &lui_glyph::TextStyle,
  max_width: f32,
  text_ctx: &mut crate::text::TextContext,
) -> String {
  let mut result = String::with_capacity(text.len() + 10);
  for word in text.split(' ') {
    if !result.is_empty() {
      result.push(' ');
    }
    let run = text_ctx.shape(word, style);
    if run.width > max_width && word.len() > 1 {
      let mut current = String::new();
      for ch in word.chars() {
        current.push(ch);
        let w = text_ctx.shape(&current, style).width;
        if w > max_width && current.len() > 1 {
          current.pop();
          result.push_str(&current);
          result.push(' ');
          current.clear();
          current.push(ch);
        }
      }
      result.push_str(&current);
    } else {
      result.push_str(word);
    }
  }
  result
}

fn layout_inline_container<'a>(
  b: &mut LayoutBox<'a>,
  ctx: &LayoutContext,
  pos: Point,
  text_ctx: &mut TextContext,
  rects: &mut Vec<(&'a HtmlNode, Rect)>,
  cache: &crate::incremental::CacheView,
  bump: &'a Bump,
) {
  let is_anon = matches!(b.kind, BoxKind::AnonymousBlock | BoxKind::AnonymousInline);
  if !is_anon {
    let margin = sides::resolve_margin_against(b.style, ctx.containing_width);
    let border = sides::resolve_border(b.style);
    let padding = sides::resolve_padding_against(b.style, ctx.containing_width);
    b.margin = margin.edges;
    b.border = border;
    b.padding = padding;
  }

  let frame_left = b.margin.left + b.border.left + b.padding.left;

  b.content.x = pos.x + frame_left;
  b.content.y = pos.y + b.margin.top + b.border.top + b.padding.top;

  let max_width = ctx.containing_width;
  let white_space = css_str(b.style.white_space);
  let no_wrap = matches!(white_space, "nowrap" | "pre");
  let css_line_height = sizes::resolve_length(b.style.line_height, 0.0).unwrap_or(0.0);
  let text_indent = sizes::resolve_length(b.style.text_indent, ctx.containing_width).unwrap_or(0.0);
  let mut cursor_x = text_indent;
  let mut cursor_y = 0.0_f32;
  let mut line_height = css_line_height;
  let mut max_line_width = 0.0_f32;
  let mut line_start_idx: usize = 0;

  for idx in 0..b.children.len() {
    let child = &mut b.children[idx];
    let is_inline_block = matches!(
      child.kind,
      BoxKind::InlineBlock | BoxKind::InlineFlex | BoxKind::InlineGrid
    );

    if is_inline_block {
      let placeholder = LayoutBox::new(BoxKind::InlineBlock, child.node, child.style, bump);
      let old = std::mem::replace(child, placeholder);
      let result = crate::engine::layout_node(
        old,
        ctx,
        Point::new(b.content.x + cursor_x, b.content.y + cursor_y),
        text_ctx,
        rects,
        cache,
        bump,
      );
      *child = result;
    } else {
      layout_inline(
        child,
        ctx,
        Point::new(b.content.x + cursor_x, b.content.y + cursor_y),
        text_ctx,
        rects,
        cache,
        bump,
      );
    }

    let child_w = child.outer_width();
    let child_h = child.outer_height();

    if !no_wrap && cursor_x > 0.0 && max_width > 0.0 && cursor_x + child_w > max_width {
      apply_vertical_align(&mut b.children[line_start_idx..idx], line_height);
      max_line_width = max_line_width.max(cursor_x);
      cursor_y += line_height;
      line_height = css_line_height;
      line_start_idx = idx;
      let child = &mut b.children[idx];
      if is_inline_block {
        let placeholder = LayoutBox::new(BoxKind::InlineBlock, child.node, child.style, bump);
        let old = std::mem::replace(child, placeholder);
        let result = crate::engine::layout_node(
          old,
          ctx,
          Point::new(b.content.x, b.content.y + cursor_y),
          text_ctx,
          rects,
          cache,
          bump,
        );
        *child = result;
      } else {
        layout_inline(
          child,
          ctx,
          Point::new(b.content.x, b.content.y + cursor_y),
          text_ctx,
          rects,
          cache,
          bump,
        );
      }
      let child_w_new = child.outer_width();
      let child_h_new = child.outer_height();
      cursor_x = child_w_new;
      line_height = line_height.max(child_h_new);
    } else {
      cursor_x += child_w;
      line_height = line_height.max(child_h);
    }
  }
  apply_vertical_align(&mut b.children[line_start_idx..], line_height);
  max_line_width = max_line_width.max(cursor_x);

  let is_anon = matches!(b.kind, BoxKind::AnonymousBlock | BoxKind::AnonymousInline);
  let explicit_w = if is_anon {
    None
  } else {
    sizes::resolve_length(b.style.width, ctx.containing_width)
  };
  let explicit_h = if is_anon {
    None
  } else {
    sizes::resolve_length(b.style.height, ctx.containing_height)
  };
  b.content.width = explicit_w.unwrap_or(max_line_width);
  b.content.height = explicit_h.unwrap_or(cursor_y + line_height);

  if css_str(b.style.direction) == "rtl" {
    let container_w = b.content.width;
    for child in b.children.iter_mut() {
      let child_left = child.content.x - b.content.x;
      let child_w = child.outer_width();
      let new_left = container_w - child_left - child_w;
      let dx = b.content.x + new_left - (child.content.x - child.padding.left - child.border.left - child.margin.left);
      if dx.abs() > 0.001 {
        crate::positioned::translate_recursive_pub(child, dx, 0.0);
      }
    }
  }
}

fn apply_vertical_align(children: &mut [LayoutBox], line_height: f32) {
  for child in children.iter_mut() {
    let valign = css_str(child.style.vertical_align);
    let child_h = child.outer_height();
    let dy = match valign {
      "top" => 0.0,
      "bottom" => line_height - child_h,
      "middle" => (line_height - child_h) / 2.0,
      _ => 0.0, // baseline (default) — no adjustment
    };
    if dy.abs() > 0.001 {
      child.content.y += dy;
      for gc in &mut child.children {
        crate::positioned::translate_recursive_pub(gc, 0.0, dy);
      }
    }
  }
}

fn get_text(node: &lui_core::HtmlNode) -> Option<String> {
  if let lui_core::HtmlElement::Text(t) = &node.element {
    Some(t.to_string())
  } else {
    None
  }
}
