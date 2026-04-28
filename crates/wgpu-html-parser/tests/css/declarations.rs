//! parse_inline_style: per-property / per-value-type coverage.

use wgpu_html_models::Style;
use wgpu_html_models::common::css_enums::{
    AlignItems, BackgroundRepeat, BorderStyle, BoxSizing, CssColor, CssImage, CssLength, Cursor,
    Display, FlexDirection, FlexWrap, FontStyle, FontWeight, JustifyContent, Overflow,
    PointerEvents, Position, TextAlign, TextTransform, UserSelect, Visibility, WhiteSpace,
};
use wgpu_html_parser::parse_inline_style;

fn s(css: &str) -> Style {
    parse_inline_style(css)
}

// --------------------------------------------------------------------------
// Length values
// --------------------------------------------------------------------------

#[test]
fn length_px() {
    let style = s("width: 42px;");
    assert!(matches!(style.width, Some(CssLength::Px(v)) if v == 42.0));
}

#[test]
fn length_px_decimal() {
    let style = s("width: 12.5px;");
    assert!(matches!(style.width, Some(CssLength::Px(v)) if (v - 12.5).abs() < 1e-6));
}

#[test]
fn length_negative_px() {
    let style = s("margin-top: -8px;");
    assert!(matches!(style.margin_top, Some(CssLength::Px(v)) if v == -8.0));
}

#[test]
fn length_percent() {
    let style = s("width: 50%;");
    assert!(matches!(style.width, Some(CssLength::Percent(v)) if v == 50.0));
}

#[test]
fn length_em() {
    let style = s("font-size: 1.5em;");
    assert!(matches!(style.font_size, Some(CssLength::Em(v)) if (v - 1.5).abs() < 1e-6));
}

#[test]
fn length_rem() {
    let style = s("font-size: 2rem;");
    assert!(matches!(style.font_size, Some(CssLength::Rem(v)) if v == 2.0));
}

#[test]
fn length_vw_vh() {
    assert!(matches!(s("width: 100vw;").width, Some(CssLength::Vw(v)) if v == 100.0));
    assert!(matches!(s("height: 50vh;").height, Some(CssLength::Vh(v)) if v == 50.0));
}

#[test]
fn length_vmin_vmax() {
    assert!(matches!(s("width: 25vmin;").width, Some(CssLength::Vmin(v)) if v == 25.0));
    assert!(matches!(s("width: 25vmax;").width, Some(CssLength::Vmax(v)) if v == 25.0));
}

#[test]
fn length_zero_no_unit() {
    let style = s("margin: 0;");
    assert!(matches!(style.margin, Some(CssLength::Zero)));
}

#[test]
fn length_auto() {
    let style = s("width: auto;");
    assert!(matches!(style.width, Some(CssLength::Auto)));
}

#[test]
fn length_calc_preserved_as_math_tree() {
    let style = s("width: calc(100% - 20px);");
    assert!(matches!(style.width, Some(CssLength::Calc(_))));
}

#[test]
fn length_min_max_clamp_parse() {
    assert!(matches!(
        s("width: min(100%, 420px);").width,
        Some(CssLength::Min(_))
    ));
    assert!(matches!(
        s("width: max(12px, 2em);").width,
        Some(CssLength::Max(_))
    ));
    assert!(matches!(
        s("font-size: clamp(12px, 2vw, 24px);").font_size,
        Some(CssLength::Clamp { .. })
    ));
}

#[test]
fn length_math_functions_parse_inside_calc() {
    let style = s("width: calc(pow(2, 3) * 10px);");
    assert!(matches!(style.width, Some(CssLength::Calc(_))));
}

// --------------------------------------------------------------------------
// Color values
// --------------------------------------------------------------------------

#[test]
fn color_named() {
    let style = s("color: red;");
    assert!(matches!(style.color, Some(CssColor::Named(ref n)) if n == "red"));
}

#[test]
fn color_hex_long() {
    let style = s("background-color: #ff0080;");
    assert!(matches!(
        style.background_color,
        Some(CssColor::Hex(ref s)) if s == "#ff0080"
    ));
}

#[test]
fn color_hex_short() {
    let style = s("color: #f0a;");
    assert!(matches!(style.color, Some(CssColor::Hex(ref s)) if s == "#f0a"));
}

#[test]
fn color_rgb() {
    let style = s("color: rgb(10, 20, 30);");
    assert!(matches!(style.color, Some(CssColor::Rgb(10, 20, 30))));
}

#[test]
fn color_rgba_with_alpha() {
    let style = s("color: rgba(255, 128, 64, 0.5);");
    let CssColor::Rgba(r, g, b, a) = style.color.unwrap() else {
        panic!("expected rgba")
    };
    assert_eq!((r, g, b), (255, 128, 64));
    assert!((a - 0.5).abs() < 1e-6);
}

#[test]
fn color_rgb_modern_space_syntax_with_percent_alpha() {
    let style = s("color: rgb(255 128 64 / 50%);");
    let CssColor::Rgba(r, g, b, a) = style.color.unwrap() else {
        panic!("expected rgba")
    };
    assert_eq!((r, g, b), (255, 128, 64));
    assert!((a - 0.5).abs() < 1e-6);
}

#[test]
fn color_hsl() {
    let style = s("color: hsl(120, 100%, 50%);");
    let CssColor::Hsl(h, sat, l) = style.color.unwrap() else {
        panic!("expected hsl")
    };
    assert_eq!((h as i32, sat as i32, l as i32), (120, 100, 50));
}

#[test]
fn color_hsla() {
    let style = s("color: hsla(0, 100%, 50%, 0.25);");
    assert!(matches!(style.color, Some(CssColor::Hsla(_, _, _, _))));
}

#[test]
fn color_modern_functions_are_preserved() {
    let style = s("color: oklch(60% 0.2 30);");
    assert!(matches!(style.color, Some(CssColor::Function(ref f)) if f == "oklch(60% 0.2 30)"));
}

#[test]
fn color_transparent() {
    let style = s("background-color: transparent;");
    assert!(matches!(
        style.background_color,
        Some(CssColor::Transparent)
    ));
}

#[test]
fn color_currentcolor() {
    let style = s("background-color: currentcolor;");
    assert!(matches!(
        style.background_color,
        Some(CssColor::CurrentColor)
    ));
}

// --------------------------------------------------------------------------
// Margin / padding shorthand
// --------------------------------------------------------------------------

#[test]
fn margin_shorthand_one_value() {
    let style = s("margin: 8px;");
    assert!(matches!(style.margin_top,    Some(CssLength::Px(v)) if v == 8.0));
    assert!(matches!(style.margin_right,  Some(CssLength::Px(v)) if v == 8.0));
    assert!(matches!(style.margin_bottom, Some(CssLength::Px(v)) if v == 8.0));
    assert!(matches!(style.margin_left,   Some(CssLength::Px(v)) if v == 8.0));
}

#[test]
fn margin_shorthand_two_values() {
    // top/bottom = 4, left/right = 8
    let style = s("margin: 4px 8px;");
    assert!(matches!(style.margin_top,    Some(CssLength::Px(v)) if v == 4.0));
    assert!(matches!(style.margin_bottom, Some(CssLength::Px(v)) if v == 4.0));
    assert!(matches!(style.margin_right,  Some(CssLength::Px(v)) if v == 8.0));
    assert!(matches!(style.margin_left,   Some(CssLength::Px(v)) if v == 8.0));
}

#[test]
fn margin_shorthand_three_values() {
    // top, left/right, bottom
    let style = s("margin: 1px 2px 3px;");
    assert!(matches!(style.margin_top,    Some(CssLength::Px(v)) if v == 1.0));
    assert!(matches!(style.margin_right,  Some(CssLength::Px(v)) if v == 2.0));
    assert!(matches!(style.margin_left,   Some(CssLength::Px(v)) if v == 2.0));
    assert!(matches!(style.margin_bottom, Some(CssLength::Px(v)) if v == 3.0));
}

#[test]
fn margin_shorthand_four_values() {
    let style = s("margin: 1px 2px 3px 4px;");
    assert!(matches!(style.margin_top,    Some(CssLength::Px(v)) if v == 1.0));
    assert!(matches!(style.margin_right,  Some(CssLength::Px(v)) if v == 2.0));
    assert!(matches!(style.margin_bottom, Some(CssLength::Px(v)) if v == 3.0));
    assert!(matches!(style.margin_left,   Some(CssLength::Px(v)) if v == 4.0));
}

#[test]
fn padding_shorthand_one_value_sets_all_sides() {
    let style = s("padding: 6px;");
    for v in [
        &style.padding_top,
        &style.padding_right,
        &style.padding_bottom,
        &style.padding_left,
    ] {
        assert!(matches!(v, Some(CssLength::Px(p)) if *p == 6.0));
    }
}

#[test]
fn margin_per_side_overrides_shorthand() {
    // CSS source-order semantics: per-side after shorthand wins.
    let style = s("margin: 8px; margin-top: 16px;");
    assert!(matches!(style.margin_top,    Some(CssLength::Px(v)) if v == 16.0));
    assert!(matches!(style.margin_bottom, Some(CssLength::Px(v)) if v == 8.0));
}

// --------------------------------------------------------------------------
// Box / position properties
// --------------------------------------------------------------------------

#[test]
fn width_and_height() {
    let style = s("width: 100px; height: 200px;");
    assert!(matches!(style.width,  Some(CssLength::Px(v)) if v == 100.0));
    assert!(matches!(style.height, Some(CssLength::Px(v)) if v == 200.0));
}

#[test]
fn min_max_constraints() {
    let style = s("min-width: 10px; min-height: 20px; max-width: 30px; max-height: 40px;");
    assert!(matches!(style.min_width,  Some(CssLength::Px(v)) if v == 10.0));
    assert!(matches!(style.min_height, Some(CssLength::Px(v)) if v == 20.0));
    assert!(matches!(style.max_width,  Some(CssLength::Px(v)) if v == 30.0));
    assert!(matches!(style.max_height, Some(CssLength::Px(v)) if v == 40.0));
}

#[test]
fn position_offsets() {
    let style = s("top: 1px; right: 2px; bottom: 3px; left: 4px;");
    assert!(matches!(style.top,    Some(CssLength::Px(v)) if v == 1.0));
    assert!(matches!(style.right,  Some(CssLength::Px(v)) if v == 2.0));
    assert!(matches!(style.bottom, Some(CssLength::Px(v)) if v == 3.0));
    assert!(matches!(style.left,   Some(CssLength::Px(v)) if v == 4.0));
}

// --------------------------------------------------------------------------
// Display / position keywords
// --------------------------------------------------------------------------

#[test]
fn display_keywords() {
    assert!(matches!(s("display: block;").display, Some(Display::Block)));
    assert!(matches!(
        s("display: inline;").display,
        Some(Display::Inline)
    ));
    assert!(matches!(
        s("display: inline-block;").display,
        Some(Display::InlineBlock)
    ));
    assert!(matches!(s("display: flex;").display, Some(Display::Flex)));
    assert!(matches!(s("display: grid;").display, Some(Display::Grid)));
    assert!(matches!(s("display: none;").display, Some(Display::None)));
}

#[test]
fn position_keywords() {
    assert!(matches!(
        s("position: static;").position,
        Some(Position::Static)
    ));
    assert!(matches!(
        s("position: relative;").position,
        Some(Position::Relative)
    ));
    assert!(matches!(
        s("position: absolute;").position,
        Some(Position::Absolute)
    ));
    assert!(matches!(
        s("position: fixed;").position,
        Some(Position::Fixed)
    ));
    assert!(matches!(
        s("position: sticky;").position,
        Some(Position::Sticky)
    ));
}

// --------------------------------------------------------------------------
// Flex
// --------------------------------------------------------------------------

#[test]
fn flex_direction_values() {
    assert!(matches!(
        s("flex-direction: row;").flex_direction,
        Some(FlexDirection::Row)
    ));
    assert!(matches!(
        s("flex-direction: row-reverse;").flex_direction,
        Some(FlexDirection::RowReverse)
    ));
    assert!(matches!(
        s("flex-direction: column;").flex_direction,
        Some(FlexDirection::Column)
    ));
    assert!(matches!(
        s("flex-direction: column-reverse;").flex_direction,
        Some(FlexDirection::ColumnReverse)
    ));
}

#[test]
fn flex_wrap_values() {
    assert!(matches!(
        s("flex-wrap: nowrap;").flex_wrap,
        Some(FlexWrap::Nowrap)
    ));
    assert!(matches!(
        s("flex-wrap: wrap;").flex_wrap,
        Some(FlexWrap::Wrap)
    ));
    assert!(matches!(
        s("flex-wrap: wrap-reverse;").flex_wrap,
        Some(FlexWrap::WrapReverse)
    ));
}

#[test]
fn justify_content_values() {
    assert!(matches!(
        s("justify-content: center;").justify_content,
        Some(JustifyContent::Center)
    ));
    assert!(matches!(
        s("justify-content: space-between;").justify_content,
        Some(JustifyContent::SpaceBetween)
    ));
    assert!(matches!(
        s("justify-content: space-around;").justify_content,
        Some(JustifyContent::SpaceAround)
    ));
    assert!(matches!(
        s("justify-content: space-evenly;").justify_content,
        Some(JustifyContent::SpaceEvenly)
    ));
    assert!(matches!(
        s("justify-content: flex-end;").justify_content,
        Some(JustifyContent::FlexEnd)
    ));
}

#[test]
fn align_items_values() {
    assert!(matches!(
        s("align-items: stretch;").align_items,
        Some(AlignItems::Stretch)
    ));
    assert!(matches!(
        s("align-items: center;").align_items,
        Some(AlignItems::Center)
    ));
    assert!(matches!(
        s("align-items: baseline;").align_items,
        Some(AlignItems::Baseline)
    ));
}

#[test]
fn gap_props() {
    assert!(matches!(s("gap: 8px;").gap,               Some(CssLength::Px(v)) if v == 8.0));
    assert!(matches!(s("row-gap: 4px;").row_gap,       Some(CssLength::Px(v)) if v == 4.0));
    assert!(matches!(s("column-gap: 12px;").column_gap, Some(CssLength::Px(v)) if v == 12.0));
}

// --------------------------------------------------------------------------
// Typography
// --------------------------------------------------------------------------

#[test]
fn font_size() {
    assert!(matches!(s("font-size: 14px;").font_size, Some(CssLength::Px(v)) if v == 14.0));
}

#[test]
fn font_weight_keywords_and_numeric() {
    assert!(matches!(
        s("font-weight: normal;").font_weight,
        Some(FontWeight::Normal)
    ));
    assert!(matches!(
        s("font-weight: bold;").font_weight,
        Some(FontWeight::Bold)
    ));
    assert!(matches!(
        s("font-weight: bolder;").font_weight,
        Some(FontWeight::Bolder)
    ));
    assert!(matches!(
        s("font-weight: lighter;").font_weight,
        Some(FontWeight::Lighter)
    ));
    assert!(matches!(
        s("font-weight: 700;").font_weight,
        Some(FontWeight::Weight(700))
    ));
}

#[test]
fn font_style_keywords() {
    assert!(matches!(
        s("font-style: normal;").font_style,
        Some(FontStyle::Normal)
    ));
    assert!(matches!(
        s("font-style: italic;").font_style,
        Some(FontStyle::Italic)
    ));
    assert!(matches!(
        s("font-style: oblique;").font_style,
        Some(FontStyle::Oblique)
    ));
}

#[test]
fn text_align_keywords() {
    assert!(matches!(
        s("text-align: left;").text_align,
        Some(TextAlign::Left)
    ));
    assert!(matches!(
        s("text-align: center;").text_align,
        Some(TextAlign::Center)
    ));
    assert!(matches!(
        s("text-align: right;").text_align,
        Some(TextAlign::Right)
    ));
    assert!(matches!(
        s("text-align: justify;").text_align,
        Some(TextAlign::Justify)
    ));
}

#[test]
fn text_transform_keywords() {
    assert!(matches!(
        s("text-transform: uppercase;").text_transform,
        Some(TextTransform::Uppercase)
    ));
    assert!(matches!(
        s("text-transform: lowercase;").text_transform,
        Some(TextTransform::Lowercase)
    ));
    assert!(matches!(
        s("text-transform: capitalize;").text_transform,
        Some(TextTransform::Capitalize)
    ));
    assert!(matches!(
        s("text-transform: none;").text_transform,
        Some(TextTransform::None)
    ));
}

#[test]
fn white_space_keywords() {
    assert!(matches!(
        s("white-space: nowrap;").white_space,
        Some(WhiteSpace::Nowrap)
    ));
    assert!(matches!(
        s("white-space: pre;").white_space,
        Some(WhiteSpace::Pre)
    ));
    assert!(matches!(
        s("white-space: pre-wrap;").white_space,
        Some(WhiteSpace::PreWrap)
    ));
    assert!(matches!(
        s("white-space: pre-line;").white_space,
        Some(WhiteSpace::PreLine)
    ));
}

#[test]
fn line_height_length() {
    assert!(
        matches!(s("line-height: 1.5em;").line_height, Some(CssLength::Em(v)) if (v - 1.5).abs() < 1e-6)
    );
}

// --------------------------------------------------------------------------
// Overflow / visibility / opacity
// --------------------------------------------------------------------------

#[test]
fn overflow_keywords() {
    let hidden = s("overflow: hidden;");
    assert!(matches!(hidden.overflow, Some(Overflow::Hidden)));
    assert!(matches!(hidden.overflow_x, Some(Overflow::Hidden)));
    assert!(matches!(hidden.overflow_y, Some(Overflow::Hidden)));

    assert!(matches!(
        s("overflow: auto;").overflow,
        Some(Overflow::Auto)
    ));
    assert!(matches!(
        s("overflow: scroll;").overflow,
        Some(Overflow::Scroll)
    ));
}

#[test]
fn overflow_shorthand_accepts_two_axes() {
    let style = s("overflow: hidden visible;");
    assert!(matches!(style.overflow_x, Some(Overflow::Hidden)));
    assert!(matches!(style.overflow_y, Some(Overflow::Visible)));
}

#[test]
fn overflow_axis_independent() {
    let style = s("overflow-x: scroll; overflow-y: hidden;");
    assert!(matches!(style.overflow_x, Some(Overflow::Scroll)));
    assert!(matches!(style.overflow_y, Some(Overflow::Hidden)));
}

#[test]
fn visibility_keywords() {
    assert!(matches!(
        s("visibility: hidden;").visibility,
        Some(Visibility::Hidden)
    ));
    assert!(matches!(
        s("visibility: visible;").visibility,
        Some(Visibility::Visible)
    ));
    assert!(matches!(
        s("visibility: collapse;").visibility,
        Some(Visibility::Collapse)
    ));
}

#[test]
fn opacity_value() {
    let style = s("opacity: 0.5;");
    let v = style.opacity.expect("opacity");
    assert!((v - 0.5).abs() < 1e-6);
}

#[test]
fn z_index_integer() {
    assert_eq!(s("z-index: 7;").z_index, Some(7));
    assert_eq!(s("z-index: -1;").z_index, Some(-1));
}

// --------------------------------------------------------------------------
// Background / border / box
// --------------------------------------------------------------------------

#[test]
fn background_repeat_keywords() {
    assert!(matches!(
        s("background-repeat: no-repeat;").background_repeat,
        Some(BackgroundRepeat::NoRepeat)
    ));
    assert!(matches!(
        s("background-repeat: repeat-x;").background_repeat,
        Some(BackgroundRepeat::RepeatX)
    ));
}

#[test]
fn background_image_url_parses_to_typed_image() {
    let style = s("background-image: url('assets/bg.png');");
    assert!(matches!(
        style.background_image,
        Some(CssImage::Url(ref url)) if url == "assets/bg.png"
    ));
}

#[test]
fn background_image_function_is_preserved() {
    let style = s("background-image: linear-gradient(red, blue);");
    assert!(matches!(
        style.background_image,
        Some(CssImage::Function(ref f)) if f == "linear-gradient(red, blue)"
    ));
}

#[test]
fn background_image_none_clears_value() {
    let style = s("background-image: none;");
    assert!(style.background_image.is_none());
}

#[test]
fn border_style_keyword_fans_to_all_sides() {
    let style = s("border-style: solid;");
    assert!(matches!(style.border_top_style, Some(BorderStyle::Solid)));
    assert!(matches!(style.border_right_style, Some(BorderStyle::Solid)));
    assert!(matches!(
        style.border_bottom_style,
        Some(BorderStyle::Solid)
    ));
    assert!(matches!(style.border_left_style, Some(BorderStyle::Solid)));
}

#[test]
fn border_radius_one_value_fans_to_all_corners() {
    let style = s("border-radius: 8px;");
    assert!(matches!(style.border_top_left_radius,     Some(CssLength::Px(v)) if v == 8.0));
    assert!(matches!(style.border_top_right_radius,    Some(CssLength::Px(v)) if v == 8.0));
    assert!(matches!(style.border_bottom_right_radius, Some(CssLength::Px(v)) if v == 8.0));
    assert!(matches!(style.border_bottom_left_radius,  Some(CssLength::Px(v)) if v == 8.0));
}

#[test]
fn border_radius_four_values_per_corner() {
    // Order: TL TR BR BL
    let style = s("border-radius: 1px 2px 3px 4px;");
    assert!(matches!(style.border_top_left_radius,     Some(CssLength::Px(v)) if v == 1.0));
    assert!(matches!(style.border_top_right_radius,    Some(CssLength::Px(v)) if v == 2.0));
    assert!(matches!(style.border_bottom_right_radius, Some(CssLength::Px(v)) if v == 3.0));
    assert!(matches!(style.border_bottom_left_radius,  Some(CssLength::Px(v)) if v == 4.0));
}

#[test]
fn border_shorthand_fans_to_all_sides() {
    // `border: <width> <style> <color>` sets all 12 per-side fields.
    let style = s("border: 2px solid red;");
    assert!(style.border.is_some()); // raw kept for round-tripping
    for w in [
        &style.border_top_width,
        &style.border_right_width,
        &style.border_bottom_width,
        &style.border_left_width,
    ] {
        assert!(matches!(w, Some(CssLength::Px(v)) if *v == 2.0));
    }
    for s in [
        &style.border_top_style,
        &style.border_right_style,
        &style.border_bottom_style,
        &style.border_left_style,
    ] {
        assert!(matches!(s, Some(BorderStyle::Solid)));
    }
    for c in [
        &style.border_top_color,
        &style.border_right_color,
        &style.border_bottom_color,
        &style.border_left_color,
    ] {
        assert!(matches!(c, Some(CssColor::Named(n)) if n == "red"));
    }
}

#[test]
fn border_shorthand_token_order_does_not_matter() {
    let style = s("border: dashed #00f 4px;");
    assert!(matches!(style.border_top_width, Some(CssLength::Px(v)) if v == 4.0));
    assert!(matches!(style.border_top_style, Some(BorderStyle::Dashed)));
    assert!(matches!(style.border_top_color, Some(CssColor::Hex(ref s)) if s == "#00f"));
}

#[test]
fn border_shorthand_partial_only_fills_present_pieces() {
    let style = s("border: 1px solid;");
    assert!(matches!(style.border_top_width, Some(CssLength::Px(v)) if v == 1.0));
    assert!(matches!(style.border_top_style, Some(BorderStyle::Solid)));
    assert!(style.border_top_color.is_none());
}

#[test]
fn per_side_longhand_only_fills_that_side() {
    let style = s("border-top: 4px dashed blue;");
    assert!(matches!(style.border_top_width, Some(CssLength::Px(v)) if v == 4.0));
    assert!(matches!(style.border_top_style, Some(BorderStyle::Dashed)));
    assert!(style.border_top_color.is_some());
    // Other sides untouched.
    assert!(style.border_right_width.is_none());
    assert!(style.border_bottom_color.is_none());
}

#[test]
fn per_side_longhand_overrides_general_shorthand() {
    // Source order: shorthand fills all sides, then per-side longhand
    // overrides just that side.
    let style = s("border: 2px solid red; border-top: 4px dashed blue;");
    assert!(matches!(style.border_top_width, Some(CssLength::Px(v)) if v == 4.0));
    assert!(matches!(style.border_top_style, Some(BorderStyle::Dashed)));
    // Other sides retain the shorthand values.
    assert!(matches!(style.border_right_width, Some(CssLength::Px(v)) if v == 2.0));
    assert!(matches!(
        style.border_bottom_style,
        Some(BorderStyle::Solid)
    ));
}

#[test]
fn border_width_box_shorthand_two_values() {
    // 2 values: vertical, horizontal.
    let style = s("border-width: 4px 8px;");
    assert!(matches!(style.border_top_width,    Some(CssLength::Px(v)) if v == 4.0));
    assert!(matches!(style.border_bottom_width, Some(CssLength::Px(v)) if v == 4.0));
    assert!(matches!(style.border_right_width,  Some(CssLength::Px(v)) if v == 8.0));
    assert!(matches!(style.border_left_width,   Some(CssLength::Px(v)) if v == 8.0));
}

#[test]
fn border_width_box_shorthand_four_values() {
    // T R B L
    let style = s("border-width: 1px 2px 3px 4px;");
    assert!(matches!(style.border_top_width,    Some(CssLength::Px(v)) if v == 1.0));
    assert!(matches!(style.border_right_width,  Some(CssLength::Px(v)) if v == 2.0));
    assert!(matches!(style.border_bottom_width, Some(CssLength::Px(v)) if v == 3.0));
    assert!(matches!(style.border_left_width,   Some(CssLength::Px(v)) if v == 4.0));
}

#[test]
fn border_color_box_shorthand_four_values() {
    let style = s("border-color: red green blue gold;");
    assert!(matches!(style.border_top_color,    Some(CssColor::Named(ref n)) if n == "red"));
    assert!(matches!(style.border_right_color,  Some(CssColor::Named(ref n)) if n == "green"));
    assert!(matches!(style.border_bottom_color, Some(CssColor::Named(ref n)) if n == "blue"));
    assert!(matches!(style.border_left_color,   Some(CssColor::Named(ref n)) if n == "gold"));
}

#[test]
fn border_style_box_shorthand_two_values() {
    let style = s("border-style: solid dashed;");
    assert!(matches!(style.border_top_style, Some(BorderStyle::Solid)));
    assert!(matches!(
        style.border_bottom_style,
        Some(BorderStyle::Solid)
    ));
    assert!(matches!(
        style.border_right_style,
        Some(BorderStyle::Dashed)
    ));
    assert!(matches!(style.border_left_style, Some(BorderStyle::Dashed)));
}

#[test]
fn explicit_border_pieces_parse() {
    let style = s("border-top-width: 2px;
         border-top-style: dashed;
         border-top-color: blue;");
    assert!(matches!(style.border_top_width, Some(CssLength::Px(v)) if v == 2.0));
    assert!(matches!(style.border_top_style, Some(BorderStyle::Dashed)));
    assert!(style.border_top_color.is_some());
}

#[test]
fn per_corner_radius_explicit_pieces_parse() {
    let style = s("border-top-left-radius: 1px;
         border-top-right-radius: 2px;
         border-bottom-right-radius: 3px;
         border-bottom-left-radius: 4px;");
    assert!(matches!(style.border_top_left_radius,     Some(CssLength::Px(v)) if v == 1.0));
    assert!(matches!(style.border_top_right_radius,    Some(CssLength::Px(v)) if v == 2.0));
    assert!(matches!(style.border_bottom_right_radius, Some(CssLength::Px(v)) if v == 3.0));
    assert!(matches!(style.border_bottom_left_radius,  Some(CssLength::Px(v)) if v == 4.0));
}

#[test]
fn box_sizing_keywords() {
    assert!(matches!(
        s("box-sizing: border-box;").box_sizing,
        Some(BoxSizing::BorderBox)
    ));
    assert!(matches!(
        s("box-sizing: content-box;").box_sizing,
        Some(BoxSizing::ContentBox)
    ));
}

// --------------------------------------------------------------------------
// Cursor / pointer-events / user-select
// --------------------------------------------------------------------------

#[test]
fn cursor_keywords() {
    assert!(matches!(
        s("cursor: pointer;").cursor,
        Some(Cursor::Pointer)
    ));
    assert!(matches!(
        s("cursor: default;").cursor,
        Some(Cursor::Default)
    ));
    assert!(matches!(
        s("cursor: not-allowed;").cursor,
        Some(Cursor::NotAllowed)
    ));
    assert!(matches!(s("cursor: text;").cursor, Some(Cursor::Text)));
}

#[test]
fn pointer_events_keywords() {
    assert!(matches!(
        s("pointer-events: none;").pointer_events,
        Some(PointerEvents::None)
    ));
    assert!(matches!(
        s("pointer-events: auto;").pointer_events,
        Some(PointerEvents::Auto)
    ));
}

#[test]
fn user_select_keywords() {
    assert!(matches!(
        s("user-select: none;").user_select,
        Some(UserSelect::None)
    ));
    assert!(matches!(
        s("user-select: text;").user_select,
        Some(UserSelect::Text)
    ));
    assert!(matches!(
        s("user-select: all;").user_select,
        Some(UserSelect::All)
    ));
}

// --------------------------------------------------------------------------
// Robustness / quirks
// --------------------------------------------------------------------------

#[test]
fn empty_input_yields_default() {
    let style = parse_inline_style("");
    let default = Style::default();
    // Just check a few fields are still None.
    assert!(style.color.is_none());
    assert!(style.width.is_none());
    let _ = default;
}

#[test]
fn whitespace_around_separators_ok() {
    let style = parse_inline_style("  color :  red  ;  width  :  10px  ;");
    assert!(style.color.is_some());
    assert!(matches!(style.width, Some(CssLength::Px(v)) if v == 10.0));
}

#[test]
fn missing_trailing_semicolon_ok() {
    let style = parse_inline_style("color: red");
    assert!(style.color.is_some());
}

#[test]
fn unknown_property_is_ignored_others_apply() {
    let style = parse_inline_style("frobnicate: 7; color: red;");
    assert!(style.color.is_some());
    assert!(style.width.is_none());
}

#[test]
fn unparseable_value_does_not_break_following() {
    // Even if `width: nonsense` cannot map to a known length variant,
    // subsequent valid declarations must still parse.
    let style = parse_inline_style("width: nonsense; color: red;");
    assert!(style.color.is_some());
}

#[test]
fn duplicate_property_last_wins() {
    let style = parse_inline_style("color: blue; color: red;");
    let CssColor::Named(n) = style.color.unwrap() else {
        panic!("expected named")
    };
    assert_eq!(n, "red");
}
