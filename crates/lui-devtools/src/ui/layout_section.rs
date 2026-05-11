use lui_layout::LayoutBox;
use lui_models::common::{AlignItems, CssLength, Display, FontWeight, GridTrackSize};
use lui_ui::{
  Component, Ctx, El, ShouldRender,
  el::{self, div},
  style::{self, Stylesheet, px},
};

use super::{lucide_icon::lucide, store::DevtoolsStore, theme::Theme};

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

  fn create(_: &LayoutSectionProps) -> Self {
    Self
  }

  fn update(&mut self, msg: LayoutSectionMsg, _: &LayoutSectionProps) -> ShouldRender {
    match msg {}
  }

  fn styles() -> Stylesheet {
    style::sheet([
      style::rule(".section").background_color(Theme::BG_SECONDARY),
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
      style::rule(".band")
        .display(Display::Flex)
        .prop("flex-direction", "column")
        .width(style::pct(100)),
      style::rule(".band-top")
        .display(Display::Flex)
        .prop("justify-content", "space-between")
        .align_items(AlignItems::Center)
        .padding_vh(px(0), px(8))
        .height(px(20)),
      style::rule(".band-mid")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .flex_grow(1.0)
        .padding_vh(px(0), px(8)),
      style::rule(".band-bot")
        .display(Display::Flex)
        .prop("justify-content", "center")
        .align_items(AlignItems::Center)
        .padding_vh(px(0), px(8))
        .height(px(20)),
      style::rule(".band-inner").flex_grow(1.0).margin_vh(px(0), px(4)),
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
        .height(px(40))
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
      let lb = layout.and_then(|root| box_at_path(root, path));

      if let Some(lb) = lb {
        let extracted = props.store.cascaded.with(|cascaded| {
          let cn = cascaded
            .as_ref()
            .and_then(|c| c.root.as_ref())
            .and_then(|r| r.at_path(path));
          ExtractedStyle::from_cascaded(cn)
        });

        children.push(build_box_model(lb, &extracted, ctx));

        if let Some(fi) = &extracted.flex {
          children.push(build_flex_info(fi, ctx));
        }
        if let Some(gi) = &extracted.grid {
          children.push(build_grid_info(gi, ctx));
        }
        if let Some(ti) = &extracted.table {
          children.push(build_table_info(ti, ctx));
        }
      }
    }

    div().class(ctx.scoped("section")).children(children)
  }
}

// ── Extracted style data (avoids cloning entire CascadedTree) ────────

struct ExtractedFlex {
  direction: String,
  gap: Option<CssLength>,
  row_gap: Option<CssLength>,
  col_gap: Option<CssLength>,
  justify_content: Option<String>,
  align_items: Option<String>,
}

struct ExtractedGrid {
  template_columns: Option<Vec<GridTrackSize>>,
  template_rows: Option<Vec<GridTrackSize>>,
  auto_flow: Option<String>,
  auto_columns: Option<String>,
  auto_rows: Option<String>,
  gap: Option<CssLength>,
  row_gap: Option<CssLength>,
  col_gap: Option<CssLength>,
  justify_items: Option<String>,
  justify_content: Option<String>,
  align_items: Option<String>,
  align_content: Option<String>,
}

struct ExtractedTable {
  display_type: String,
  properties: Vec<(String, String)>,
}

struct ExtractedStyle {
  margin_auto: [bool; 4],
  flex: Option<ExtractedFlex>,
  grid: Option<ExtractedGrid>,
  table: Option<ExtractedTable>,
}

impl ExtractedStyle {
  fn from_cascaded(cn: Option<&lui_style::CascadedNode>) -> Self {
    let Some(cn) = cn else {
      return Self {
        margin_auto: [false; 4],
        flex: None,
        grid: None,
        table: None,
      };
    };
    let s = &cn.style;
    let is_auto = |specific: &Option<CssLength>, shorthand: &Option<CssLength>| -> bool {
      match specific {
        Some(CssLength::Auto) => true,
        Some(_) => false,
        None => matches!(shorthand, Some(CssLength::Auto)),
      }
    };
    let margin_auto = [
      is_auto(&s.margin_top, &s.margin),
      is_auto(&s.margin_right, &s.margin),
      is_auto(&s.margin_bottom, &s.margin),
      is_auto(&s.margin_left, &s.margin),
    ];
    let is_flex = matches!(s.display, Some(Display::Flex) | Some(Display::InlineFlex));
    let flex = if is_flex {
      Some(ExtractedFlex {
        direction: s
          .flex_direction
          .as_ref()
          .map(|d| d.as_css_str().to_string())
          .unwrap_or_default(),
        gap: s.gap.clone(),
        row_gap: s.row_gap.clone(),
        col_gap: s.column_gap.clone(),
        justify_content: s.justify_content.as_ref().map(|v| v.as_css_str().to_string()),
        align_items: s.align_items.as_ref().map(|v| v.as_css_str().to_string()),
      })
    } else {
      None
    };
    let is_grid = matches!(s.display, Some(Display::Grid) | Some(Display::InlineGrid));
    let grid = if is_grid {
      Some(ExtractedGrid {
        template_columns: s.grid_template_columns.clone(),
        template_rows: s.grid_template_rows.clone(),
        auto_flow: s.grid_auto_flow.as_ref().map(|v| v.as_css_str().to_string()),
        auto_columns: s.grid_auto_columns.as_ref().map(|v| format!("{v}")),
        auto_rows: s.grid_auto_rows.as_ref().map(|v| format!("{v}")),
        gap: s.gap.clone(),
        row_gap: s.row_gap.clone(),
        col_gap: s.column_gap.clone(),
        justify_items: s.justify_items.as_ref().map(|v| v.as_css_str().to_string()),
        justify_content: s.justify_content.as_ref().map(|v| v.as_css_str().to_string()),
        align_items: s.align_items.as_ref().map(|v| v.as_css_str().to_string()),
        align_content: s.align_content.as_ref().map(|v| v.as_css_str().to_string()),
      })
    } else {
      None
    };

    let is_table = matches!(
      s.display,
      Some(Display::Table)
        | Some(Display::TableCaption)
        | Some(Display::TableHeaderGroup)
        | Some(Display::TableRowGroup)
        | Some(Display::TableFooterGroup)
        | Some(Display::TableRow)
        | Some(Display::TableCell)
        | Some(Display::TableColumn)
        | Some(Display::TableColumnGroup)
    );
    let table = if is_table {
      let display_type = s
        .display
        .as_ref()
        .map(|d| d.as_css_str().to_string())
        .unwrap_or_default();
      let table_props: &[&str] = &[
        "border-collapse",
        "border-spacing",
        "table-layout",
        "caption-side",
        "empty-cells",
        "vertical-align",
      ];
      let properties: Vec<(String, String)> = table_props
        .iter()
        .filter_map(|&key| s.deferred_longhands.get(key).map(|v| (key.to_string(), v.to_string())))
        .collect();
      Some(ExtractedTable {
        display_type,
        properties,
      })
    } else {
      None
    };

    Self {
      margin_auto,
      flex,
      grid,
      table,
    }
  }
}

// ── Box model ────────────────────────────────────────────────────────

fn build_box_model(lb: &LayoutBox, style: &ExtractedStyle, ctx: &Ctx<LayoutSectionMsg>) -> El {
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

  let m = &style.margin_auto;

  let content_box = div()
    .class(ctx.scoped("content-box"))
    .style(format!("background:{BM_CONTENT}"))
    .text(format!("{} \u{00d7} {}", cr.w.round() as i32, cr.h.round() as i32));

  let padding_band = band(
    "padding",
    BM_PADDING,
    fmt_val(p_top),
    fmt_val(p_right),
    fmt_val(p_bottom),
    fmt_val(p_left),
    content_box,
    ctx,
  );

  let border_band = band(
    "border",
    BM_BORDER,
    fmt_val(bd.top),
    fmt_val(bd.right),
    fmt_val(bd.bottom),
    fmt_val(bd.left),
    padding_band,
    ctx,
  );

  let margin_band = band(
    "margin",
    BM_MARGIN,
    if m[0] { "auto".into() } else { fmt_val(m_top) },
    if m[1] { "auto".into() } else { fmt_val(m_right) },
    if m[2] { "auto".into() } else { fmt_val(m_bottom) },
    if m[3] { "auto".into() } else { fmt_val(m_left) },
    border_band,
    ctx,
  );

  div()
    .class(ctx.scoped("bm-area"))
    .children([div().style("width:340px").children([margin_band])])
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
      div()
        .class(ctx.scoped("band-bot"))
        .children([el::span().class(ctx.scoped("bv")).text(bottom)]),
    ])
}

fn fmt_val(v: f32) -> String {
  let r = v.round() as i32;
  r.to_string()
}

// ── Flex info ────────────────────────────────────────────────────────

fn build_flex_info(fi: &ExtractedFlex, ctx: &Ctx<LayoutSectionMsg>) -> El {
  let mut rows: Vec<El> = Vec::new();
  rows.push(el::span().class(ctx.scoped("fi-title")).text("Flex Container"));

  if !fi.direction.is_empty() {
    rows.push(fi_row("flex-direction:", &fi.direction, ctx));
  }

  let gap = fi.gap.as_ref().or(fi.row_gap.as_ref()).or(fi.col_gap.as_ref());
  if let Some(gap_val) = gap {
    let rg = fi.row_gap.as_ref().or(fi.gap.as_ref());
    let cg = fi.col_gap.as_ref().or(fi.gap.as_ref());
    let same = match (rg, cg) {
      (Some(a), Some(b)) => format!("{a}") == format!("{b}"),
      (None, None) => true,
      _ => false,
    };
    rows.push(fi_gap_row(gap_val, same, ctx));
  }

  if let Some(jc) = &fi.justify_content {
    rows.push(fi_row("justify-content:", jc, ctx));
  }

  if let Some(ai) = &fi.align_items {
    rows.push(fi_row("align-items:", ai, ctx));
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

// ── Grid info ───────────────────────────────────────────────────────

fn build_grid_info(gi: &ExtractedGrid, ctx: &Ctx<LayoutSectionMsg>) -> El {
  let mut rows: Vec<El> = Vec::new();
  rows.push(el::span().class(ctx.scoped("fi-title")).text("Grid Container"));

  if let Some(cols) = &gi.template_columns {
    let val = fmt_track_list(cols);
    rows.push(fi_row("grid-template-columns:", &val, ctx));
  }
  if let Some(template_rows) = &gi.template_rows {
    let val = fmt_track_list(template_rows);
    rows.push(fi_row("grid-template-rows:", &val, ctx));
  }
  if let Some(flow) = &gi.auto_flow {
    rows.push(fi_row("grid-auto-flow:", flow, ctx));
  }
  if let Some(ac) = &gi.auto_columns {
    rows.push(fi_row("grid-auto-columns:", ac, ctx));
  }
  if let Some(ar) = &gi.auto_rows {
    rows.push(fi_row("grid-auto-rows:", ar, ctx));
  }

  let gap = gi.gap.as_ref().or(gi.row_gap.as_ref()).or(gi.col_gap.as_ref());
  if let Some(gap_val) = gap {
    let rg = gi.row_gap.as_ref().or(gi.gap.as_ref());
    let cg = gi.col_gap.as_ref().or(gi.gap.as_ref());
    let same = match (rg, cg) {
      (Some(a), Some(b)) => format!("{a}") == format!("{b}"),
      (None, None) => true,
      _ => false,
    };
    rows.push(fi_gap_row(gap_val, same, ctx));
  }

  if let Some(ji) = &gi.justify_items {
    rows.push(fi_row("justify-items:", ji, ctx));
  }
  if let Some(jc) = &gi.justify_content {
    rows.push(fi_row("justify-content:", jc, ctx));
  }
  if let Some(ai) = &gi.align_items {
    rows.push(fi_row("align-items:", ai, ctx));
  }
  if let Some(ac) = &gi.align_content {
    rows.push(fi_row("align-content:", ac, ctx));
  }

  div().class(ctx.scoped("fi")).children(rows)
}

fn fmt_track_list(tracks: &[GridTrackSize]) -> String {
  tracks.iter().map(|t| format!("{t}")).collect::<Vec<_>>().join(" ")
}

// ── Table info ──────────────────────────────────────────────────────

fn build_table_info(ti: &ExtractedTable, ctx: &Ctx<LayoutSectionMsg>) -> El {
  let mut rows: Vec<El> = Vec::new();
  rows.push(el::span().class(ctx.scoped("fi-title")).text("Table Layout"));

  rows.push(fi_row("display:", &ti.display_type, ctx));

  for (prop, val) in &ti.properties {
    rows.push(fi_row(&format!("{prop}:"), val, ctx));
  }

  div().class(ctx.scoped("fi")).children(rows)
}

// ── Helpers ──────────────────────────────────────────────────────────

fn box_at_path<'a>(root: &'a LayoutBox, path: &[usize]) -> Option<&'a LayoutBox> {
  let mut current = root;
  for &idx in path {
    current = current.children.get(idx)?;
  }
  Some(current)
}
