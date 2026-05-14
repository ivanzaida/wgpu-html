use bumpalo::Bump;
use lui_core::{ArcStr, CssProperty, CssValue};
use rustc_hash::FxHashMap;

/// Computed style for a single node.
///
/// Hot properties (the ~85 that layout reads) get typed fields for O(1) access.
/// Everything else falls through to `extra`. All values are borrowed from either
/// the input stylesheets or a bump arena that holds synthesized values
/// (expanded shorthands, resolved var(), inherited copies).
#[derive(Debug, Default, Clone)]
pub struct ComputedStyle<'a> {
  // ── Display & position ──
  pub display: Option<&'a CssValue>,
  pub position: Option<&'a CssValue>,
  pub top: Option<&'a CssValue>,
  pub right: Option<&'a CssValue>,
  pub bottom: Option<&'a CssValue>,
  pub left: Option<&'a CssValue>,
  pub direction: Option<&'a CssValue>,
  pub writing_mode: Option<&'a CssValue>,
  pub float: Option<&'a CssValue>,
  pub clear: Option<&'a CssValue>,

  // ── Sizing ──
  pub width: Option<&'a CssValue>,
  pub height: Option<&'a CssValue>,
  pub min_width: Option<&'a CssValue>,
  pub min_height: Option<&'a CssValue>,
  pub max_width: Option<&'a CssValue>,
  pub max_height: Option<&'a CssValue>,
  pub box_sizing: Option<&'a CssValue>,
  pub aspect_ratio: Option<&'a CssValue>,

  // ── Margin ──
  pub margin_top: Option<&'a CssValue>,
  pub margin_right: Option<&'a CssValue>,
  pub margin_bottom: Option<&'a CssValue>,
  pub margin_left: Option<&'a CssValue>,

  // ── Padding ──
  pub padding_top: Option<&'a CssValue>,
  pub padding_right: Option<&'a CssValue>,
  pub padding_bottom: Option<&'a CssValue>,
  pub padding_left: Option<&'a CssValue>,

  // ── Border width ──
  pub border_top_width: Option<&'a CssValue>,
  pub border_right_width: Option<&'a CssValue>,
  pub border_bottom_width: Option<&'a CssValue>,
  pub border_left_width: Option<&'a CssValue>,

  // ── Border style ──
  pub border_top_style: Option<&'a CssValue>,
  pub border_right_style: Option<&'a CssValue>,
  pub border_bottom_style: Option<&'a CssValue>,
  pub border_left_style: Option<&'a CssValue>,

  // ── Border color ──
  pub border_top_color: Option<&'a CssValue>,
  pub border_right_color: Option<&'a CssValue>,
  pub border_bottom_color: Option<&'a CssValue>,
  pub border_left_color: Option<&'a CssValue>,

  // ── Border radius ──
  pub border_top_left_radius: Option<&'a CssValue>,
  pub border_top_right_radius: Option<&'a CssValue>,
  pub border_bottom_right_radius: Option<&'a CssValue>,
  pub border_bottom_left_radius: Option<&'a CssValue>,

  // ── Background ──
  pub background_color: Option<&'a CssValue>,
  pub background_image: Option<&'a CssValue>,
  pub background_size: Option<&'a CssValue>,
  pub background_position: Option<&'a CssValue>,
  pub background_repeat: Option<&'a CssValue>,
  pub background_clip: Option<&'a CssValue>,

  // ── Color ──
  pub color: Option<&'a CssValue>,
  pub opacity: Option<&'a CssValue>,
  pub visibility: Option<&'a CssValue>,

  // ── Typography ──
  pub font_family: Option<&'a CssValue>,
  pub font_size: Option<&'a CssValue>,
  pub font_weight: Option<&'a CssValue>,
  pub font_style: Option<&'a CssValue>,
  pub line_height: Option<&'a CssValue>,
  pub text_indent: Option<&'a CssValue>,
  pub letter_spacing: Option<&'a CssValue>,
  pub word_spacing: Option<&'a CssValue>,
  pub text_align: Option<&'a CssValue>,
  pub text_decoration_line: Option<&'a CssValue>,
  pub text_decoration_color: Option<&'a CssValue>,
  pub text_decoration_style: Option<&'a CssValue>,
  pub text_transform: Option<&'a CssValue>,
  pub white_space: Option<&'a CssValue>,
  pub word_break: Option<&'a CssValue>,
  pub overflow_wrap: Option<&'a CssValue>,
  pub text_overflow: Option<&'a CssValue>,
  pub vertical_align: Option<&'a CssValue>,

  // ── Flexbox ──
  pub flex_direction: Option<&'a CssValue>,
  pub flex_wrap: Option<&'a CssValue>,
  pub justify_content: Option<&'a CssValue>,
  pub align_items: Option<&'a CssValue>,
  pub align_content: Option<&'a CssValue>,
  pub align_self: Option<&'a CssValue>,
  pub flex_grow: Option<&'a CssValue>,
  pub flex_shrink: Option<&'a CssValue>,
  pub flex_basis: Option<&'a CssValue>,
  pub order: Option<&'a CssValue>,
  pub row_gap: Option<&'a CssValue>,
  pub column_gap: Option<&'a CssValue>,

  // ── Grid ──
  pub grid_template_columns: Option<&'a CssValue>,
  pub grid_template_rows: Option<&'a CssValue>,
  pub grid_template_areas: Option<&'a CssValue>,
  pub grid_auto_columns: Option<&'a CssValue>,
  pub grid_auto_rows: Option<&'a CssValue>,
  pub grid_auto_flow: Option<&'a CssValue>,
  pub grid_column_start: Option<&'a CssValue>,
  pub grid_column_end: Option<&'a CssValue>,
  pub grid_row_start: Option<&'a CssValue>,
  pub grid_row_end: Option<&'a CssValue>,
  pub justify_items: Option<&'a CssValue>,
  pub justify_self: Option<&'a CssValue>,

  // ── Table ──
  pub border_collapse: Option<&'a CssValue>,
  pub border_spacing: Option<&'a CssValue>,
  pub caption_side: Option<&'a CssValue>,
  pub table_layout: Option<&'a CssValue>,

  // ── Overflow & scroll ──
  pub overflow_x: Option<&'a CssValue>,
  pub overflow_y: Option<&'a CssValue>,
  pub scrollbar_color: Option<&'a CssValue>,
  pub scrollbar_gutter: Option<&'a CssValue>,
  pub scrollbar_width: Option<&'a CssValue>,

  // ── Transform & effects ──
  pub transform: Option<&'a CssValue>,
  pub transform_origin: Option<&'a CssValue>,
  pub box_shadow: Option<&'a CssValue>,
  pub z_index: Option<&'a CssValue>,

  // ── Interaction ──
  pub cursor: Option<&'a CssValue>,
  pub pointer_events: Option<&'a CssValue>,
  pub user_select: Option<&'a CssValue>,
  pub resize: Option<&'a CssValue>,
  pub accent_color: Option<&'a CssValue>,

  // ── List ──
  pub list_style_type: Option<&'a CssValue>,
  pub list_style_position: Option<&'a CssValue>,
  pub list_style_image: Option<&'a CssValue>,

  // ── Content ──
  pub content: Option<&'a CssValue>,

  // ── SVG ──
  pub fill: Option<&'a CssValue>,
  pub fill_opacity: Option<&'a CssValue>,
  pub fill_rule: Option<&'a CssValue>,
  pub stroke: Option<&'a CssValue>,
  pub stroke_width: Option<&'a CssValue>,
  pub stroke_opacity: Option<&'a CssValue>,
  pub stroke_linecap: Option<&'a CssValue>,
  pub stroke_linejoin: Option<&'a CssValue>,
  pub stroke_dasharray: Option<&'a CssValue>,
  pub stroke_dashoffset: Option<&'a CssValue>,

  // ── Cold properties (everything layout doesn't touch) ──
  pub extra: Option<Box<FxHashMap<CssProperty, &'a CssValue>>>,

  // ── Custom properties (always inherited) ──
  pub custom_properties: Option<Box<FxHashMap<ArcStr, &'a CssValue>>>,
}

macro_rules! property_field_map {
    ($($prop:ident => $field:ident),* $(,)?) => {
        impl<'a> ComputedStyle<'a> {
            pub fn set(&mut self, prop: &CssProperty, value: &'a CssValue) {
                match prop {
                    $(CssProperty::$prop => self.$field = Some(value),)*
                    _ => {
                        self.extra
                            .get_or_insert_with(Default::default)
                            .insert(prop.clone(), value);
                    }
                }
            }

            pub fn clear(&mut self, prop: &CssProperty) {
                match prop {
                    $(CssProperty::$prop => self.$field = None,)*
                    _ => {
                        if let Some(m) = &mut self.extra { m.remove(prop); }
                    }
                }
            }

            pub fn get(&self, prop: &CssProperty) -> Option<&'a CssValue> {
                match prop {
                    $(CssProperty::$prop => self.$field,)*
                    _ => self.extra.as_ref().and_then(|m| m.get(prop).copied()),
                }
            }

            pub fn has(&self, prop: &CssProperty) -> bool {
                match prop {
                    $(CssProperty::$prop => self.$field.is_some(),)*
                    _ => self.extra.as_ref().is_some_and(|m| m.contains_key(prop)),
                }
            }

            /// Deep-copy all values into a different arena, producing a
            /// `ComputedStyle` with the target lifetime.
            pub fn clone_into<'b>(&self, arena: &'b Bump) -> ComputedStyle<'b> {
                let mut out = ComputedStyle::default();
                $(
                    if let Some(val) = self.$field {
                        out.$field = Some(arena.alloc(val.clone()));
                    }
                )*

                if let Some(ref extra) = self.extra {
                    let mut new_extra = FxHashMap::default();
                    for (prop, val) in extra.iter() {
                        new_extra.insert(prop.clone(), &*arena.alloc((*val).clone()));
                    }
                    out.extra = Some(Box::new(new_extra));
                }

                if let Some(ref cp) = self.custom_properties {
                    let mut new_cp = FxHashMap::default();
                    for (name, val) in cp.iter() {
                        new_cp.insert(name.clone(), &*arena.alloc((*val).clone()));
                    }
                    out.custom_properties = Some(Box::new(new_cp));
                }

                out
            }

            pub fn inherit_from(&mut self, parent: &ComputedStyle<'a>) {
                $(
                    if self.$field.is_none() && CssProperty::$prop.inherited() {
                        self.$field = parent.$field;
                    }
                )*

                if let Some(parent_extra) = &parent.extra {
                    for (prop, value) in parent_extra.iter() {
                        if prop.clone().inherited() {
                            let child_extra = self.extra.get_or_insert_with(Default::default);
                            child_extra.entry(prop.clone()).or_insert(*value);
                        }
                    }
                }

                if let Some(parent_cp) = &parent.custom_properties {
                    let child_cp = self.custom_properties.get_or_insert_with(Default::default);
                    for (name, value) in parent_cp.iter() {
                        child_cp.entry(name.clone()).or_insert(*value);
                    }
                }
            }
        }
    };
}

property_field_map! {
    Display => display,
    Position => position,
    Top => top,
    Right => right,
    Bottom => bottom,
    Left => left,
    Direction => direction,
    WritingMode => writing_mode,
    Float => float,
    Clear => clear,

    Width => width,
    Height => height,
    MinWidth => min_width,
    MinHeight => min_height,
    MaxWidth => max_width,
    MaxHeight => max_height,
    BoxSizing => box_sizing,
    AspectRatio => aspect_ratio,

    MarginTop => margin_top,
    MarginRight => margin_right,
    MarginBottom => margin_bottom,
    MarginLeft => margin_left,

    PaddingTop => padding_top,
    PaddingRight => padding_right,
    PaddingBottom => padding_bottom,
    PaddingLeft => padding_left,

    BorderTopWidth => border_top_width,
    BorderRightWidth => border_right_width,
    BorderBottomWidth => border_bottom_width,
    BorderLeftWidth => border_left_width,

    BorderTopStyle => border_top_style,
    BorderRightStyle => border_right_style,
    BorderBottomStyle => border_bottom_style,
    BorderLeftStyle => border_left_style,

    BorderTopColor => border_top_color,
    BorderRightColor => border_right_color,
    BorderBottomColor => border_bottom_color,
    BorderLeftColor => border_left_color,

    BorderTopLeftRadius => border_top_left_radius,
    BorderTopRightRadius => border_top_right_radius,
    BorderBottomRightRadius => border_bottom_right_radius,
    BorderBottomLeftRadius => border_bottom_left_radius,

    BackgroundColor => background_color,
    BackgroundImage => background_image,
    BackgroundSize => background_size,
    BackgroundPosition => background_position,
    BackgroundRepeat => background_repeat,
    BackgroundClip => background_clip,

    Color => color,
    Opacity => opacity,
    Visibility => visibility,

    FontFamily => font_family,
    FontSize => font_size,
    FontWeight => font_weight,
    FontStyle => font_style,
    LineHeight => line_height,
    TextIndent => text_indent,
    LetterSpacing => letter_spacing,
    WordSpacing => word_spacing,
    TextAlign => text_align,
    TextDecorationLine => text_decoration_line,
    TextDecorationColor => text_decoration_color,
    TextDecorationStyle => text_decoration_style,
    TextTransform => text_transform,
    WhiteSpace => white_space,
    WordBreak => word_break,
    OverflowWrap => overflow_wrap,
    TextOverflow => text_overflow,
    VerticalAlign => vertical_align,

    FlexDirection => flex_direction,
    FlexWrap => flex_wrap,
    JustifyContent => justify_content,
    AlignItems => align_items,
    AlignContent => align_content,
    AlignSelf => align_self,
    FlexGrow => flex_grow,
    FlexShrink => flex_shrink,
    FlexBasis => flex_basis,
    Order => order,
    RowGap => row_gap,
    ColumnGap => column_gap,

    GridTemplateColumns => grid_template_columns,
    GridTemplateRows => grid_template_rows,
    GridTemplateAreas => grid_template_areas,
    GridAutoColumns => grid_auto_columns,
    GridAutoRows => grid_auto_rows,
    GridAutoFlow => grid_auto_flow,
    GridColumnStart => grid_column_start,
    GridColumnEnd => grid_column_end,
    GridRowStart => grid_row_start,
    GridRowEnd => grid_row_end,
    JustifyItems => justify_items,
    JustifySelf => justify_self,

    BorderCollapse => border_collapse,
    BorderSpacing => border_spacing,
    CaptionSide => caption_side,
    TableLayout => table_layout,

    OverflowX => overflow_x,
    OverflowY => overflow_y,
    ScrollbarColor => scrollbar_color,
    ScrollbarGutter => scrollbar_gutter,
    ScrollbarWidth => scrollbar_width,

    Transform => transform,
    TransformOrigin => transform_origin,
    BoxShadow => box_shadow,
    ZIndex => z_index,

    Cursor => cursor,
    PointerEvents => pointer_events,
    UserSelect => user_select,
    Resize => resize,
    AccentColor => accent_color,

    ListStyleType => list_style_type,
    ListStylePosition => list_style_position,
    ListStyleImage => list_style_image,

    Content => content,

    Fill => fill,
    FillOpacity => fill_opacity,
    FillRule => fill_rule,
    Stroke => stroke,
    StrokeWidth => stroke_width,
    StrokeOpacity => stroke_opacity,
    StrokeLinecap => stroke_linecap,
    StrokeLinejoin => stroke_linejoin,
    StrokeDasharray => stroke_dasharray,
    StrokeDashoffset => stroke_dashoffset,
}

/// Allocate a `CssValue` in the bump arena and return a reference with the
/// arena's lifetime. Use this for synthesized values (expanded shorthands,
/// resolved var(), inherited copies).
pub fn alloc_value(arena: &Bump, value: CssValue) -> &CssValue {
  arena.alloc(value)
}
