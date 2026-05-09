use wgpu_html_layout::LayoutBox;
use wgpu_html_models::common::{
  AlignItems, CssLength, Display, FontWeight,
};
use wgpu_html_style::CascadedNode;
use wgpu_html_ui::{
  el::{self, div},
  style::{self, px, Stylesheet},
  Component, Ctx, El, ShouldRender,
};

use super::lucide_icon::lucide;
use super::store::DevtoolsStore;
use super::theme::Theme;

const ICON_CHEVRON_DOWN: &str = "\u{E06D}";

const BM_MARGIN: &str = "#F6B26B";
const BM_BORDER: &str = "#FBBC04";
const BM_PADDING: &str = "#81C995";
const BM_CONTENT: &str = "#669DF6";
const BM_TEXT: &str = "#202124";

#[derive(Clone)]
pub struct LayoutSectionProps {
  pub store: DevtoolsStore,
}

#[derive(Clone)]
pub enum LayoutSectionMsg {}

pub struct LayoutSection;

impl Component for LayoutSection {
  type Props = LayoutSectionProps;
  type Msg = LayoutSectionMsg;

  fn create(_: &LayoutSectionProps) -> Self { Self }

  fn update(&mut self, msg: LayoutSectionMsg, _: &LayoutSectionProps) -> ShouldRender {
    match msg {}
  }

  fn styles() -> Stylesheet {
    style::sheet([
      style::rule(".section")
        .background_color(Theme::BG_SECONDARY),
      style::rule(".hdr")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .height(px(24))
        .padding_vh(px(0), px(12))
        .gap(px(6))
        .border_top(format!("1px solid {}", Theme::BORDER))
        .border_bottom(format!("1px solid {}", Theme::BORDER)),
      style::rule(".hdr-icon")
        .width(px(11))
        .height(px(11))
        .font_size(px(11))
        .color(Theme::TEXT_SECONDARY)
        .prop("line-height", "11px"),
      style::rule(".hdr-text")
        .font_family("Inter, system-ui, sans-serif")
        .font_size(px(11))
        .font_weight(FontWeight::Weight(600))
        .color(Theme::TEXT_PRIMARY),
      // Box model area
      style::rule(".bm-area")
        .display(Display::Flex)
        .prop("justify-content", "center")
        .padding(px(16)),
      // Band (each colored layer)
      style::rule(".band")
        .display(Display::Flex)
        .prop("flex-direction", "column"),
      style::rule(".band-top")
        .display(Display::Flex)
        .prop("justify-content", "space-between")
        .align_items(AlignItems::Center)
        .padding_vh(px(4), px(8)),
      style::rule(".band-mid")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .flex_grow(1.0)
        .padding_vh(px(0), px(8)),
      style::rule(".band-bot")
        .display(Display::Flex)
        .prop("justify-content", "center")
        .align_items(AlignItems::Center)
        .padding_vh(px(4), px(8)),
      style::rule(".band-inner")
        .flex_grow(1.0)
        .margin_vh(px(0), px(4)),
      style::rule(".bl")
        .font_family("Inter, system-ui, sans-serif")
        .font_size(px(9))
        .font_weight(FontWeight::Weight(600))
        .color(BM_TEXT),
      style::rule(".bv")
        .font_family("monospace")
        .font_size(px(10))
        .color(BM_TEXT)
        .prop("text-align", "center")
        .min_width(px(20)),
      style::rule(".content-box")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .prop("justify-content", "center")
        .flex_grow(1.0)
        .padding(px(8))
        .font_family("monospace")
        .font_size(px(11))
        .font_weight(FontWeight::Weight(600))
        .color(BM_TEXT),
      // Flex info
      style::rule(".fi")
        .display(Display::Flex)
        .prop("flex-direction", "column")
        .gap(px(6))
        .padding(format!("{}px {}px {}px {}px", 8, 12, 12, 12)),
      style::rule(".fi-title")
        .font_family("Inter, system-ui, sans-serif")
        .font_size(px(10))
        .font_weight(FontWeight::Weight(600))
        .color(Theme::TEXT_SECONDARY),
      style::rule(".fi-row")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .gap(px(8)),
      style::rule(".fi-prop")
        .font_family("monospace")
        .font_size(px(11))
        .color(Theme::TEXT_SECONDARY),
      style::rule(".fi-val")
        .font_family("monospace")
        .font_size(px(11))
        .color(Theme::VALUE),
      style::rule(".fi-badge")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .gap(px(2))
        .padding_vh(px(2), px(6))
        .prop("border-radius", "3px")
        .background_color(Theme::BG_TERTIARY),
      style::rule(".fi-unit")
        .font_family("monospace")
        .font_size(px(11))
        .color(Theme::UNIT),
      style::rule(".fi-note")
        .font_family("Inter, system-ui, sans-serif")
        .font_size(px(10))
        .prop("font-style", "italic")
        .color(Theme::TEXT_MUTED),
    ])
    .scoped("layout")
  }

  fn view(&self, props: &LayoutSectionProps, ctx: &Ctx<LayoutSectionMsg>) -> El {
    let selected = props.store.selected_path.get();

    let header = div().class(ctx.scoped("hdr")).children([
      lucide(ICON_CHEVRON_DOWN).class(ctx.scoped("hdr-icon")),
      el::span().class(ctx.scoped("hdr-text")).text("Layout"),
    ]);

    let mut children: Vec<El> = vec![header];

    if let Some(path) = &selected {
      let layout = props.store.layout_root();
      let cascaded = props.store.cascaded.get();

      let lb = layout.and_then(|root| box_at_path(root, path));
      let cn = cascaded.as_ref()
        .and_then(|c| c.root.as_ref())
        .and_then(|r| r.at_path(path));

      if let Some(lb) = lb {
        children.push(build_box_model(lb, cn, ctx));

        if let Some(cn) = cn {
          if is_flex_container(&cn.style) {
            children.push(build_flex_info(cn, ctx));
          }
        }
      }
    }

    div().class(ctx.scoped("section")).children(children)
  }
}

// ── Box model ────────────────────────────────────────────────────────

fn build_box_model(
  lb: &LayoutBox,
  cn: Option<&CascadedNode>,
  ctx: &Ctx<LayoutSectionMsg>,
) -> El {
  let mr = &lb.margin_rect;
  let br = &lb.border_rect;
  let cr = &lb.content_rect;
  let bd = &lb.border;

  let m_top = br.y - mr.y;
  let m_right = (mr.x + mr.w) - (br.x + br.w);
  let m_bottom = (mr.y + mr.h) - (br.y + br.h);
  let m_left = br.x - mr.x;

  let p_top = cr.y - (br.y + bd.top);
  let p_right = (br.x + br.w - bd.right) - (cr.x + cr.w);
  let p_bottom = (br.y + br.h - bd.bottom) - (cr.y + cr.h);
  let p_left = cr.x - (br.x + bd.left);

  let m_auto = margin_auto_sides(cn);

  let content_box = div()
    .class(ctx.scoped("content-box"))
    .style(format!("background:{BM_CONTENT}"))
    .text(format!("{} \u{00d7} {}", cr.w.round() as i32, cr.h.round() as i32));

  let padding_band = band(
    "padding", BM_PADDING,
    fmt_val(p_top), fmt_val(p_right), fmt_val(p_bottom), fmt_val(p_left),
    content_box, ctx,
  );

  let border_band = band(
    "border", BM_BORDER,
    fmt_val(bd.top), fmt_val(bd.right), fmt_val(bd.bottom), fmt_val(bd.left),
    padding_band, ctx,
  );

  let margin_band = band(
    "margin", BM_MARGIN,
    if m_auto[0] { "auto".into() } else { fmt_val(m_top) },
    if m_auto[1] { "auto".into() } else { fmt_val(m_right) },
    if m_auto[2] { "auto".into() } else { fmt_val(m_bottom) },
    if m_auto[3] { "auto".into() } else { fmt_val(m_left) },
    border_band, ctx,
  );

  div().class(ctx.scoped("bm-area")).children([margin_band])
}

fn band(
  label: &str,
  bg: &str,
  top: String,
  right: String,
  bottom: String,
  left: String,
  inner: El,
  ctx: &Ctx<LayoutSectionMsg>,
) -> El {
  div()
    .class(ctx.scoped("band"))
    .style(format!("background:{bg}"))
    .children([
      div().class(ctx.scoped("band-top")).children([
        el::span().class(ctx.scoped("bl")).text(label),
        el::span().class(ctx.scoped("bv")).text(top),
      ]),
      div().class(ctx.scoped("band-mid")).children([
        el::span().class(ctx.scoped("bv")).text(left),
        div().class(ctx.scoped("band-inner")).children([inner]),
        el::span().class(ctx.scoped("bv")).text(right),
      ]),
      div().class(ctx.scoped("band-bot")).children([
        el::span().class(ctx.scoped("bv")).text(bottom),
      ]),
    ])
}

fn fmt_val(v: f32) -> String {
  let r = v.round() as i32;
  r.to_string()
}

fn margin_auto_sides(cn: Option<&CascadedNode>) -> [bool; 4] {
  let Some(cn) = cn else { return [false; 4] };
  let s = &cn.style;
  let is_auto = |specific: &Option<CssLength>, shorthand: &Option<CssLength>| -> bool {
    match specific {
      Some(CssLength::Auto) => true,
      Some(_) => false,
      None => matches!(shorthand, Some(CssLength::Auto)),
    }
  };
  [
    is_auto(&s.margin_top, &s.margin),
    is_auto(&s.margin_right, &s.margin),
    is_auto(&s.margin_bottom, &s.margin),
    is_auto(&s.margin_left, &s.margin),
  ]
}

// ── Flex info ────────────────────────────────────────────────────────

fn is_flex_container(s: &wgpu_html_models::Style) -> bool {
  matches!(s.display, Some(Display::Flex) | Some(Display::InlineFlex))
}

fn build_flex_info(cn: &CascadedNode, ctx: &Ctx<LayoutSectionMsg>) -> El {
  let s = &cn.style;
  let mut rows: Vec<El> = Vec::new();
  rows.push(el::span().class(ctx.scoped("fi-title")).text("Flex Container"));

  if let Some(dir) = &s.flex_direction {
    rows.push(fi_row("flex-direction:", dir.as_css_str(), ctx));
  }

  let gap = s.gap.as_ref().or(s.row_gap.as_ref()).or(s.column_gap.as_ref());
  if let Some(gap_val) = gap {
    let row_gap = s.row_gap.as_ref().or(s.gap.as_ref());
    let col_gap = s.column_gap.as_ref().or(s.gap.as_ref());
    let same = match (row_gap, col_gap) {
      (Some(a), Some(b)) => format!("{a}") == format!("{b}"),
      (None, None) => true,
      _ => false,
    };
    rows.push(fi_gap_row(gap_val, same, ctx));
  }

  if let Some(jc) = &s.justify_content {
    rows.push(fi_row("justify-content:", jc.as_css_str(), ctx));
  }

  if let Some(ai) = &s.align_items {
    rows.push(fi_row("align-items:", ai.as_css_str(), ctx));
  }

  div().class(ctx.scoped("fi")).children(rows)
}

fn fi_row(prop: &str, val: &str, ctx: &Ctx<LayoutSectionMsg>) -> El {
  div().class(ctx.scoped("fi-row")).children([
    el::span().class(ctx.scoped("fi-prop")).text(prop),
    el::span().class(ctx.scoped("fi-val")).text(val),
  ])
}

fn fi_gap_row(gap: &CssLength, same: bool, ctx: &Ctx<LayoutSectionMsg>) -> El {
  let (num, unit) = match gap {
    CssLength::Px(v) => (format!("{}", v.round() as i32), "px"),
    CssLength::Em(v) => (format!("{v}"), "em"),
    CssLength::Rem(v) => (format!("{v}"), "rem"),
    CssLength::Percent(v) => (format!("{v}"), "%"),
    other => (format!("{other}"), ""),
  };

  let mut children: Vec<El> = vec![
    el::span().class(ctx.scoped("fi-prop")).text("gap:"),
    div().class(ctx.scoped("fi-badge")).children([
      el::span().class(ctx.scoped("fi-val")).text(num),
      el::span().class(ctx.scoped("fi-unit")).text(unit),
    ]),
  ];

  if same {
    children.push(el::span().class(ctx.scoped("fi-note")).text("(row & column)"));
  }

  div().class(ctx.scoped("fi-row")).children(children)
}

// ── Helpers ──────────────────────────────────────────────────────────

fn box_at_path<'a>(root: &'a LayoutBox, path: &[usize]) -> Option<&'a LayoutBox> {
  let mut current = root;
  for &idx in path {
    current = current.children.get(idx)?;
  }
  Some(current)
}
