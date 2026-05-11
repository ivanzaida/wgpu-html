use crate::CssProperty;
use crate::CssValue;

/// Returns the longhand properties for a shorthand. Empty if not a shorthand.
pub fn longhands_of(property: CssProperty) -> &'static [CssProperty] {
    match property {
        CssProperty::Animation => &[CssProperty::AnimationName, CssProperty::AnimationDuration, CssProperty::AnimationTimingFunction, CssProperty::AnimationDelay, CssProperty::AnimationIterationCount, CssProperty::AnimationDirection, CssProperty::AnimationFillMode, CssProperty::AnimationPlayState, CssProperty::AnimationTimeline, CssProperty::AnimationRange, CssProperty::AnimationTrigger],
        CssProperty::Background => &[CssProperty::BackgroundColor, CssProperty::BackgroundImage, CssProperty::BackgroundPosition, CssProperty::BackgroundSize, CssProperty::BackgroundRepeat, CssProperty::BackgroundAttachment, CssProperty::BackgroundClip, CssProperty::BackgroundOrigin],
        CssProperty::Border => &[CssProperty::BorderWidth, CssProperty::BorderStyle, CssProperty::BorderColor],
        CssProperty::BorderBlock => &[CssProperty::BorderBlockWidth, CssProperty::BorderBlockStyle, CssProperty::BorderBlockColor],
        CssProperty::BorderBlockEnd => &[CssProperty::BorderBlockEndWidth, CssProperty::BorderBlockEndStyle, CssProperty::BorderBlockEndColor],
        CssProperty::BorderBlockStart => &[CssProperty::BorderBlockStartWidth, CssProperty::BorderBlockStartStyle, CssProperty::BorderBlockStartColor],
        CssProperty::BorderBottom => &[CssProperty::BorderBottomWidth, CssProperty::BorderBottomStyle, CssProperty::BorderBottomColor],
        CssProperty::BorderColor => &[CssProperty::BorderTopColor, CssProperty::BorderRightColor, CssProperty::BorderBottomColor, CssProperty::BorderLeftColor],
        CssProperty::BorderImage => &[CssProperty::BorderImageSource, CssProperty::BorderImageSlice, CssProperty::BorderImageWidth, CssProperty::BorderImageOutset, CssProperty::BorderImageRepeat],
        CssProperty::BorderInline => &[CssProperty::BorderInlineWidth, CssProperty::BorderInlineStyle, CssProperty::BorderInlineColor],
        CssProperty::BorderInlineEnd => &[CssProperty::BorderInlineEndWidth, CssProperty::BorderInlineEndStyle, CssProperty::BorderInlineEndColor],
        CssProperty::BorderInlineStart => &[CssProperty::BorderInlineStartWidth, CssProperty::BorderInlineStartStyle, CssProperty::BorderInlineStartColor],
        CssProperty::BorderLeft => &[CssProperty::BorderLeftWidth, CssProperty::BorderLeftStyle, CssProperty::BorderLeftColor],
        CssProperty::BorderRadius => &[CssProperty::BorderTopLeftRadius, CssProperty::BorderTopRightRadius, CssProperty::BorderBottomRightRadius, CssProperty::BorderBottomLeftRadius],
        CssProperty::BorderRight => &[CssProperty::BorderRightWidth, CssProperty::BorderRightStyle, CssProperty::BorderRightColor],
        CssProperty::BorderStyle => &[CssProperty::BorderTopStyle, CssProperty::BorderRightStyle, CssProperty::BorderBottomStyle, CssProperty::BorderLeftStyle],
        CssProperty::BorderTop => &[CssProperty::BorderTopWidth, CssProperty::BorderTopStyle, CssProperty::BorderTopColor],
        CssProperty::BorderWidth => &[CssProperty::BorderTopWidth, CssProperty::BorderRightWidth, CssProperty::BorderBottomWidth, CssProperty::BorderLeftWidth],
        CssProperty::ColumnRule => &[CssProperty::ColumnRuleWidth, CssProperty::ColumnRuleStyle, CssProperty::ColumnRuleColor],
        CssProperty::Columns => &[CssProperty::ColumnWidth, CssProperty::ColumnCount],
        CssProperty::ContainIntrinsicSize => &[CssProperty::ContainIntrinsicWidth, CssProperty::ContainIntrinsicHeight],
        CssProperty::Flex => &[CssProperty::FlexGrow, CssProperty::FlexShrink, CssProperty::FlexBasis],
        CssProperty::FlexFlow => &[CssProperty::FlexDirection, CssProperty::FlexWrap],
        CssProperty::Font => &[CssProperty::FontStyle, CssProperty::FontWeight, CssProperty::FontStretch, CssProperty::FontSize, CssProperty::LineHeight, CssProperty::FontFamily],
        CssProperty::Gap => &[CssProperty::RowGap, CssProperty::ColumnGap],
        CssProperty::Grid => &[CssProperty::GridTemplateRows, CssProperty::GridTemplateColumns, CssProperty::GridTemplateAreas, CssProperty::GridAutoRows, CssProperty::GridAutoColumns, CssProperty::GridAutoFlow],
        CssProperty::GridArea => &[CssProperty::GridRowStart, CssProperty::GridColumnStart, CssProperty::GridRowEnd, CssProperty::GridColumnEnd],
        CssProperty::GridColumn => &[CssProperty::GridColumnStart, CssProperty::GridColumnEnd],
        CssProperty::GridRow => &[CssProperty::GridRowStart, CssProperty::GridRowEnd],
        CssProperty::GridTemplate => &[CssProperty::GridTemplateRows, CssProperty::GridTemplateColumns, CssProperty::GridTemplateAreas],
        CssProperty::Inset => &[CssProperty::Top, CssProperty::Right, CssProperty::Bottom, CssProperty::Left],
        CssProperty::InsetBlock => &[CssProperty::InsetBlockStart, CssProperty::InsetBlockEnd],
        CssProperty::InsetInline => &[CssProperty::InsetInlineStart, CssProperty::InsetInlineEnd],
        CssProperty::ListStyle => &[CssProperty::ListStylePosition, CssProperty::ListStyleImage, CssProperty::ListStyleType],
        CssProperty::Margin => &[CssProperty::MarginTop, CssProperty::MarginRight, CssProperty::MarginBottom, CssProperty::MarginLeft],
        CssProperty::MarginBlock => &[CssProperty::MarginBlockStart, CssProperty::MarginBlockEnd],
        CssProperty::MarginInline => &[CssProperty::MarginInlineStart, CssProperty::MarginInlineEnd],
        CssProperty::Mask => &[CssProperty::MaskImage, CssProperty::MaskPosition, CssProperty::MaskSize, CssProperty::MaskRepeat, CssProperty::MaskClip, CssProperty::MaskOrigin, CssProperty::MaskComposite, CssProperty::MaskMode],
        CssProperty::Offset => &[CssProperty::OffsetPosition, CssProperty::OffsetPath, CssProperty::OffsetDistance, CssProperty::OffsetRotate, CssProperty::OffsetAnchor],
        CssProperty::Outline => &[CssProperty::OutlineWidth, CssProperty::OutlineStyle, CssProperty::OutlineColor],
        CssProperty::Overflow => &[CssProperty::OverflowX, CssProperty::OverflowY],
        CssProperty::OverscrollBehavior => &[CssProperty::OverscrollBehaviorX, CssProperty::OverscrollBehaviorY],
        CssProperty::Padding => &[CssProperty::PaddingTop, CssProperty::PaddingRight, CssProperty::PaddingBottom, CssProperty::PaddingLeft],
        CssProperty::PaddingBlock => &[CssProperty::PaddingBlockStart, CssProperty::PaddingBlockEnd],
        CssProperty::PaddingInline => &[CssProperty::PaddingInlineStart, CssProperty::PaddingInlineEnd],
        CssProperty::PlaceContent => &[CssProperty::AlignContent, CssProperty::JustifyContent],
        CssProperty::PlaceItems => &[CssProperty::AlignItems, CssProperty::JustifyItems],
        CssProperty::PlaceSelf => &[CssProperty::AlignSelf, CssProperty::JustifySelf],
        CssProperty::ScrollMargin => &[CssProperty::ScrollMarginTop, CssProperty::ScrollMarginRight, CssProperty::ScrollMarginBottom, CssProperty::ScrollMarginLeft],
        CssProperty::ScrollMarginBlock => &[CssProperty::ScrollMarginBlockStart, CssProperty::ScrollMarginBlockEnd],
        CssProperty::ScrollMarginInline => &[CssProperty::ScrollMarginInlineStart, CssProperty::ScrollMarginInlineEnd],
        CssProperty::ScrollPadding => &[CssProperty::ScrollPaddingTop, CssProperty::ScrollPaddingRight, CssProperty::ScrollPaddingBottom, CssProperty::ScrollPaddingLeft],
        CssProperty::ScrollPaddingBlock => &[CssProperty::ScrollPaddingBlockStart, CssProperty::ScrollPaddingBlockEnd],
        CssProperty::ScrollPaddingInline => &[CssProperty::ScrollPaddingInlineStart, CssProperty::ScrollPaddingInlineEnd],
        CssProperty::TextDecoration => &[CssProperty::TextDecorationLine, CssProperty::TextDecorationStyle, CssProperty::TextDecorationColor, CssProperty::TextDecorationThickness],
        CssProperty::TextEmphasis => &[CssProperty::TextEmphasisStyle, CssProperty::TextEmphasisColor],
        CssProperty::Transition => &[CssProperty::TransitionProperty, CssProperty::TransitionDuration, CssProperty::TransitionTimingFunction, CssProperty::TransitionDelay],
        _ => &[],
    }
}

/// Distribute 1–4 values following the standard CSS pattern
/// (top→right→bottom→left, replicated symmetrically).
pub fn distribute_values(values: &[CssValue], longhands: &[CssProperty]) -> Vec<(CssProperty, CssValue)> {
    let n = longhands.len();
    if n == 0 || values.is_empty() { return vec![]; }

    let out: Vec<CssValue> = match values.len() {
        1 => vec![values[0].clone(); n],
        2 => {
            vec![values[0].clone(), values[1].clone(), values[0].clone(), values[1].clone()]
        }
        3 => {
            vec![values[0].clone(), values[1].clone(), values[2].clone(), values[1].clone()]
        }
        _ => values.iter().take(n).cloned().collect(),
    };

    longhands.iter().zip(out.into_iter())
        .map(|(p, v)| (p.clone(), v))
        .collect()
}

/// Expand a shorthand property value into its longhand declarations.
pub fn expand(property: CssProperty, values: &[CssValue]) -> Vec<(CssProperty, CssValue)> {
    let longhands = longhands_of(property);
    if longhands.is_empty() {
        return vec![];
    }
    distribute_values(values, longhands)
}
