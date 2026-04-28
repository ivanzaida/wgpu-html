use crate::common::css_enums::{
    AlignContent, AlignItems, AlignSelf, BackgroundClip, BackgroundRepeat, BorderStyle, BoxSizing,
    CssColor, CssImage, CssLength, Cursor, Display, FlexDirection, FlexWrap, FontStyle, FontWeight,
    GridAutoFlow, GridLine, GridTrackSize, JustifyContent, JustifyItems, JustifySelf, Overflow,
    PointerEvents, Position, TextAlign, TextTransform, UserSelect, Visibility, WhiteSpace,
};

#[derive(Debug, Clone, Default)]
pub struct Style {
    pub display: Option<Display>,
    pub position: Option<Position>,
    pub top: Option<CssLength>,
    pub right: Option<CssLength>,
    pub bottom: Option<CssLength>,
    pub left: Option<CssLength>,
    pub width: Option<CssLength>,
    pub height: Option<CssLength>,
    // css property: min-width
    pub min_width: Option<CssLength>,
    // css property: min-height
    pub min_height: Option<CssLength>,
    // css property: max-width
    pub max_width: Option<CssLength>,
    // css property: max-height
    pub max_height: Option<CssLength>,
    pub margin: Option<CssLength>,
    // css property: margin-top
    pub margin_top: Option<CssLength>,
    // css property: margin-right
    pub margin_right: Option<CssLength>,
    // css property: margin-bottom
    pub margin_bottom: Option<CssLength>,
    // css property: margin-left
    pub margin_left: Option<CssLength>,
    pub padding: Option<CssLength>,
    // css property: padding-top
    pub padding_top: Option<CssLength>,
    // css property: padding-right
    pub padding_right: Option<CssLength>,
    // css property: padding-bottom
    pub padding_bottom: Option<CssLength>,
    // css property: padding-left
    pub padding_left: Option<CssLength>,
    pub color: Option<CssColor>,
    pub background: Option<String>,
    // css property: background-color
    pub background_color: Option<CssColor>,
    // css property: background-image
    pub background_image: Option<CssImage>,
    // css property: background-size
    pub background_size: Option<String>,
    // css property: background-position
    pub background_position: Option<String>,
    // css property: background-repeat
    pub background_repeat: Option<BackgroundRepeat>,
    // css property: background-clip
    pub background_clip: Option<BackgroundClip>,
    /// Raw value of the `border` shorthand. Kept for round-tripping;
    /// layout reads the per-side fields below instead.
    pub border: Option<String>,

    // css property: border-top-width / -right- / -bottom- / -left-
    pub border_top_width: Option<CssLength>,
    pub border_right_width: Option<CssLength>,
    pub border_bottom_width: Option<CssLength>,
    pub border_left_width: Option<CssLength>,

    // css property: border-top-style / -right- / -bottom- / -left-
    pub border_top_style: Option<BorderStyle>,
    pub border_right_style: Option<BorderStyle>,
    pub border_bottom_style: Option<BorderStyle>,
    pub border_left_style: Option<BorderStyle>,

    // css property: border-top-color / -right- / -bottom- / -left-
    pub border_top_color: Option<CssColor>,
    pub border_right_color: Option<CssColor>,
    pub border_bottom_color: Option<CssColor>,
    pub border_left_color: Option<CssColor>,

    // css property: border-<corner>-radius — horizontal component (the
    // first value in CSS per-corner `<h> <v>` syntax).
    pub border_top_left_radius: Option<CssLength>,
    pub border_top_right_radius: Option<CssLength>,
    pub border_bottom_right_radius: Option<CssLength>,
    pub border_bottom_left_radius: Option<CssLength>,

    // Vertical component for elliptical radii (the second value in
    // `<h> <v>` syntax, or the post-slash list in `border-radius: ... / ...`).
    // None means "same as the horizontal component" — CSS default.
    pub border_top_left_radius_v: Option<CssLength>,
    pub border_top_right_radius_v: Option<CssLength>,
    pub border_bottom_right_radius_v: Option<CssLength>,
    pub border_bottom_left_radius_v: Option<CssLength>,
    // css property: font-family
    pub font_family: Option<String>,
    // css property: font-size
    pub font_size: Option<CssLength>,
    // css property: font-weight
    pub font_weight: Option<FontWeight>,
    // css property: font-style
    pub font_style: Option<FontStyle>,
    // css property: line-height
    pub line_height: Option<CssLength>,
    // css property: letter-spacing
    pub letter_spacing: Option<CssLength>,
    // css property: text-align
    pub text_align: Option<TextAlign>,
    // css property: text-decoration
    pub text_decoration: Option<String>,
    // css property: text-transform
    pub text_transform: Option<TextTransform>,
    // css property: white-space
    pub white_space: Option<WhiteSpace>,
    pub overflow: Option<Overflow>,
    // css property: overflow-x
    pub overflow_x: Option<Overflow>,
    // css property: overflow-y
    pub overflow_y: Option<Overflow>,
    pub opacity: Option<f32>,
    pub visibility: Option<Visibility>,
    // css property: z-index
    pub z_index: Option<i32>,
    // css property: flex-direction
    pub flex_direction: Option<FlexDirection>,
    // css property: flex-wrap
    pub flex_wrap: Option<FlexWrap>,
    // css property: justify-content
    pub justify_content: Option<JustifyContent>,
    // css property: align-items
    pub align_items: Option<AlignItems>,
    // css property: align-content
    pub align_content: Option<AlignContent>,
    // css property: align-self (per-item override of `align-items`)
    pub align_self: Option<AlignSelf>,
    // css property: order (visual order of flex items)
    pub order: Option<i32>,
    pub gap: Option<CssLength>,
    // css property: row-gap
    pub row_gap: Option<CssLength>,
    // css property: column-gap
    pub column_gap: Option<CssLength>,
    pub flex: Option<String>,
    // css property: flex-grow
    pub flex_grow: Option<f32>,
    // css property: flex-shrink
    pub flex_shrink: Option<f32>,
    // css property: flex-basis
    pub flex_basis: Option<CssLength>,
    // CSS Grid track templates. The parser expands `repeat(...)` and
    // resolves keywords / lengths / `<flex>` into a typed track list;
    // layout never re-parses the source text.
    pub grid_template_columns: Option<Vec<GridTrackSize>>,
    pub grid_template_rows: Option<Vec<GridTrackSize>>,
    // Implicit-track sizes for grid items placed beyond the explicit
    // grid (or in implicit rows / columns generated by auto-placement).
    pub grid_auto_columns: Option<GridTrackSize>,
    pub grid_auto_rows: Option<GridTrackSize>,
    // Direction of the auto-placement cursor.
    pub grid_auto_flow: Option<GridAutoFlow>,
    // Per-item placement (longhands populated from the `grid-column` /
    // `grid-row` shorthands at parse time).
    pub grid_column_start: Option<GridLine>,
    pub grid_column_end: Option<GridLine>,
    pub grid_row_start: Option<GridLine>,
    pub grid_row_end: Option<GridLine>,
    /// Raw `grid-column` shorthand, preserved for round-tripping;
    /// layout reads the start/end longhands above.
    pub grid_column: Option<String>,
    /// Raw `grid-row` shorthand; same treatment as `grid_column`.
    pub grid_row: Option<String>,
    // CSS Grid: default inline-axis alignment for items.
    pub justify_items: Option<JustifyItems>,
    // CSS Grid: per-item override of `justify-items`.
    pub justify_self: Option<JustifySelf>,
    pub transform: Option<String>,
    // css property: transform-origin
    pub transform_origin: Option<String>,
    pub transition: Option<String>,
    pub animation: Option<String>,
    pub cursor: Option<Cursor>,
    // css property: pointer-events
    pub pointer_events: Option<PointerEvents>,
    // css property: user-select
    pub user_select: Option<UserSelect>,
    // css property: box-shadow
    pub box_shadow: Option<String>,
    // css property: box-sizing
    pub box_sizing: Option<BoxSizing>,
}
