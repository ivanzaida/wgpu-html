use bumpalo::Bump;
use lui_cascade::ComputedStyle;
use lui_layout::sides;

use crate::helpers::*;

#[test]
fn resolve_margin_with_px_and_auto() {
  let arena = Bump::new();
  let mut style = ComputedStyle::default();
  style.margin_top = Some(arena.alloc(px(10.0)));
  style.margin_right = Some(arena.alloc(auto()));
  style.margin_bottom = Some(arena.alloc(px(20.0)));
  style.margin_left = Some(arena.alloc(px(5.0)));

  let result = sides::resolve_margin(&style);

  assert_eq!(result.edges.top, 10.0, "margin-top should be 10px");
  assert_eq!(result.edges.right, 0.0, "margin-right auto resolves to 0 with mask");
  assert_eq!(result.edges.bottom, 20.0, "margin-bottom should be 20px");
  assert_eq!(result.edges.left, 5.0, "margin-left should be 5px");
  // auto_mask bits: bit 0=top, 1=right, 2=bottom, 3=left
  assert_eq!(result.auto_mask, 0b0010, "only right (bit 1) should be auto");
}

#[test]
fn resolve_margin_all_zero_when_no_margin_set() {
  let style = ComputedStyle::default();
  let result = sides::resolve_margin(&style);
  assert_eq!(result.edges.top, 0.0);
  assert_eq!(result.edges.right, 0.0);
  assert_eq!(result.edges.bottom, 0.0);
  assert_eq!(result.edges.left, 0.0);
  assert_eq!(result.auto_mask, 0, "no auto margins");
}

#[test]
fn resolve_margin_all_auto_sets_mask_correctly() {
  let arena = Bump::new();
  let mut style = ComputedStyle::default();
  style.margin_top = Some(arena.alloc(auto()));
  style.margin_right = Some(arena.alloc(auto()));
  style.margin_bottom = Some(arena.alloc(auto()));
  style.margin_left = Some(arena.alloc(auto()));

  let result = sides::resolve_margin(&style);
  // All edges resolve to 0.0
  assert_eq!(result.edges.top, 0.0);
  assert_eq!(result.edges.right, 0.0);
  assert_eq!(result.edges.bottom, 0.0);
  assert_eq!(result.edges.left, 0.0);
  assert_eq!(result.auto_mask, 0b1111, "all four sides should be auto");
}

#[test]
fn resolve_margin_number_zero() {
  let arena = Bump::new();
  let mut style = ComputedStyle::default();
  style.margin_top = Some(arena.alloc(num(0.0)));
  let result = sides::resolve_margin(&style);
  assert_eq!(result.edges.top, 0.0, "Number(0) resolves to 0");
  assert_eq!(result.auto_mask, 0, "Number is not auto");
}

#[test]
fn resolve_border_with_single_edge_set() {
  let arena = Bump::new();
  let mut style = ComputedStyle::default();
  style.border_top_width = Some(arena.alloc(px(2.0)));

  let border = sides::resolve_border(&style);
  assert_eq!(border.top, 2.0, "border-top should be 2px");
  assert_eq!(border.right, 0.0, "border-right should default to 0");
  assert_eq!(border.bottom, 0.0, "border-bottom should default to 0");
  assert_eq!(border.left, 0.0, "border-left should default to 0");
}

#[test]
fn resolve_border_all_four_edges() {
  let arena = Bump::new();
  let mut style = ComputedStyle::default();
  style.border_top_width = Some(arena.alloc(px(1.0)));
  style.border_right_width = Some(arena.alloc(px(2.0)));
  style.border_bottom_width = Some(arena.alloc(px(3.0)));
  style.border_left_width = Some(arena.alloc(px(4.0)));

  let border = sides::resolve_border(&style);
  assert_eq!(border.top, 1.0);
  assert_eq!(border.right, 2.0);
  assert_eq!(border.bottom, 3.0);
  assert_eq!(border.left, 4.0);
}

#[test]
fn resolve_border_with_number_zero() {
  let arena = Bump::new();
  let mut style = ComputedStyle::default();
  style.border_top_width = Some(arena.alloc(num(0.0)));
  let border = sides::resolve_border(&style);
  assert_eq!(border.top, 0.0, "Number(0) resolves to 0");
}

#[test]
fn resolve_padding_all_four_set() {
  let arena = Bump::new();
  let mut style = ComputedStyle::default();
  style.padding_top = Some(arena.alloc(px(8.0)));
  style.padding_right = Some(arena.alloc(px(8.0)));
  style.padding_bottom = Some(arena.alloc(px(8.0)));
  style.padding_left = Some(arena.alloc(px(8.0)));

  let padding = sides::resolve_padding(&style);
  assert_eq!(padding.top, 8.0);
  assert_eq!(padding.right, 8.0);
  assert_eq!(padding.bottom, 8.0);
  assert_eq!(padding.left, 8.0);
}

#[test]
fn resolve_padding_different_per_side() {
  let arena = Bump::new();
  let mut style = ComputedStyle::default();
  style.padding_top = Some(arena.alloc(px(1.0)));
  style.padding_right = Some(arena.alloc(px(2.0)));
  style.padding_bottom = Some(arena.alloc(px(3.0)));
  style.padding_left = Some(arena.alloc(px(4.0)));

  let padding = sides::resolve_padding(&style);
  assert_eq!(padding.top, 1.0);
  assert_eq!(padding.right, 2.0);
  assert_eq!(padding.bottom, 3.0);
  assert_eq!(padding.left, 4.0);
}

#[test]
fn resolve_padding_defaults_to_zero() {
  let style = ComputedStyle::default();
  let padding = sides::resolve_padding(&style);
  assert_eq!(padding.top, 0.0);
  assert_eq!(padding.right, 0.0);
  assert_eq!(padding.bottom, 0.0);
  assert_eq!(padding.left, 0.0);
}
