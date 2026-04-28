pub fn shorthand_members(prop: &str) -> Option<&'static [&'static str]> {
    match prop {
        "animation" => Some(&ANIMATION),
        "animation-range" => Some(&ANIMATION_RANGE),
        "background" => Some(&BACKGROUND),
        "background-position" => Some(&BACKGROUND_POSITION),
        "border" => Some(&BORDER),
        "border-block" => Some(&BORDER_BLOCK),
        "border-block-color" => Some(&BORDER_BLOCK_COLOR),
        "border-block-end" => Some(&BORDER_BLOCK_END),
        "border-block-start" => Some(&BORDER_BLOCK_START),
        "border-block-style" => Some(&BORDER_BLOCK_STYLE),
        "border-block-width" => Some(&BORDER_BLOCK_WIDTH),
        "border-bottom" => Some(&BORDER_BOTTOM),
        "border-color" => Some(&BORDER_COLOR),
        "border-image" => Some(&BORDER_IMAGE),
        "border-inline" => Some(&BORDER_INLINE),
        "border-inline-color" => Some(&BORDER_INLINE_COLOR),
        "border-inline-end" => Some(&BORDER_INLINE_END),
        "border-inline-start" => Some(&BORDER_INLINE_START),
        "border-inline-style" => Some(&BORDER_INLINE_STYLE),
        "border-inline-width" => Some(&BORDER_INLINE_WIDTH),
        "border-left" => Some(&BORDER_LEFT),
        "border-radius" => Some(&BORDER_RADIUS),
        "border-right" => Some(&BORDER_RIGHT),
        "border-style" => Some(&BORDER_STYLE),
        "border-top" => Some(&BORDER_TOP),
        "border-width" => Some(&BORDER_WIDTH),
        "column-rule" => Some(&COLUMN_RULE),
        "columns" => Some(&COLUMNS),
        "contain-intrinsic-size" => Some(&CONTAIN_INTRINSIC_SIZE),
        "container" => Some(&CONTAINER),
        "cue" => Some(&CUE),
        "flex" => Some(&FLEX),
        "flex-flow" => Some(&FLEX_FLOW),
        "font" => Some(&FONT),
        "font-synthesis" => Some(&FONT_SYNTHESIS),
        "font-variant" => Some(&FONT_VARIANT),
        "font-variant-ligatures" => Some(&FONT_VARIANT_LIGATURES),
        "gap" => Some(&GAP),
        "grid" => Some(&GRID),
        "grid-area" => Some(&GRID_AREA),
        "grid-column" => Some(&GRID_COLUMN),
        "grid-row" => Some(&GRID_ROW),
        "grid-template" => Some(&GRID_TEMPLATE),
        "inset" => Some(&INSET),
        "inset-block" => Some(&INSET_BLOCK),
        "inset-inline" => Some(&INSET_INLINE),
        "line-clamp" => Some(&LINE_CLAMP),
        "list-style" => Some(&LIST_STYLE),
        "margin" => Some(&MARGIN),
        "margin-block" => Some(&MARGIN_BLOCK),
        "margin-inline" => Some(&MARGIN_INLINE),
        "marker" => Some(&MARKER),
        "mask" => Some(&MASK),
        "mask-border" => Some(&MASK_BORDER),
        "offset" => Some(&OFFSET),
        "outline" => Some(&OUTLINE),
        "overflow" => Some(&OVERFLOW),
        "overscroll-behavior" => Some(&OVERSCROLL_BEHAVIOR),
        "overscroll-behavior-block" => Some(&OVERSCROLL_BEHAVIOR_BLOCK),
        "overscroll-behavior-inline" => Some(&OVERSCROLL_BEHAVIOR_INLINE),
        "padding" => Some(&PADDING),
        "padding-block" => Some(&PADDING_BLOCK),
        "padding-inline" => Some(&PADDING_INLINE),
        "pause" => Some(&PAUSE),
        "place-content" => Some(&PLACE_CONTENT),
        "place-items" => Some(&PLACE_ITEMS),
        "place-self" => Some(&PLACE_SELF),
        "rest" => Some(&REST),
        "scroll-margin" => Some(&SCROLL_MARGIN),
        "scroll-margin-block" => Some(&SCROLL_MARGIN_BLOCK),
        "scroll-margin-inline" => Some(&SCROLL_MARGIN_INLINE),
        "scroll-padding" => Some(&SCROLL_PADDING),
        "scroll-padding-block" => Some(&SCROLL_PADDING_BLOCK),
        "scroll-padding-inline" => Some(&SCROLL_PADDING_INLINE),
        "scroll-timeline" => Some(&SCROLL_TIMELINE),
        "text-box" => Some(&TEXT_BOX),
        "text-decoration" => Some(&TEXT_DECORATION),
        "text-emphasis" => Some(&TEXT_EMPHASIS),
        "transition" => Some(&TRANSITION),
        "view-timeline" => Some(&VIEW_TIMELINE),
        "white-space" => Some(&WHITE_SPACE_SHORTHAND),
        _ => None,
    }
}

pub fn all_shorthands() -> &'static [&'static str] {
    &ALL_SHORTHANDS
}

pub fn shorthand_contains_member(shorthand: &str, member: &str) -> bool {
    let Some(members) = shorthand_members(shorthand) else {
        return false;
    };
    members.iter().any(|candidate| {
        *candidate == member
            || (*candidate != shorthand && shorthand_contains_member(candidate, member))
    })
}

pub fn is_deferred_longhand(prop: &str) -> bool {
    DEFERRED_LONGHANDS.contains(&prop)
}

pub fn is_inherited_deferred_longhand(prop: &str) -> bool {
    INHERITED_DEFERRED_LONGHANDS.contains(&prop)
}

pub fn all_deferred_longhands() -> &'static [&'static str] {
    &DEFERRED_LONGHANDS
}

const ANIMATION: [&str; 13] = [
    "animation-name",
    "animation-duration",
    "animation-timing-function",
    "animation-delay",
    "animation-iteration-count",
    "animation-direction",
    "animation-fill-mode",
    "animation-play-state",
    "animation-composition",
    "animation-timeline",
    "animation-range-start",
    "animation-range-end",
    "animation",
];
const ANIMATION_RANGE: [&str; 2] = ["animation-range-start", "animation-range-end"];
const BACKGROUND: [&str; 9] = [
    "background",
    "background-image",
    "background-position",
    "background-size",
    "background-repeat",
    "background-origin",
    "background-clip",
    "background-attachment",
    "background-color",
];
const BACKGROUND_POSITION: [&str; 2] = ["background-position-x", "background-position-y"];
const BORDER: [&str; 4] = ["border", "border-width", "border-style", "border-color"];
const BORDER_BLOCK: [&str; 3] = [
    "border-block-width",
    "border-block-style",
    "border-block-color",
];
const BORDER_BLOCK_COLOR: [&str; 2] = ["border-block-start-color", "border-block-end-color"];
const BORDER_BLOCK_END: [&str; 3] = [
    "border-block-end-width",
    "border-block-end-style",
    "border-block-end-color",
];
const BORDER_BLOCK_START: [&str; 3] = [
    "border-block-start-width",
    "border-block-start-style",
    "border-block-start-color",
];
const BORDER_BLOCK_STYLE: [&str; 2] = ["border-block-start-style", "border-block-end-style"];
const BORDER_BLOCK_WIDTH: [&str; 2] = ["border-block-start-width", "border-block-end-width"];
const BORDER_BOTTOM: [&str; 3] = [
    "border-bottom-width",
    "border-bottom-style",
    "border-bottom-color",
];
const BORDER_COLOR: [&str; 4] = [
    "border-top-color",
    "border-right-color",
    "border-bottom-color",
    "border-left-color",
];
const BORDER_IMAGE: [&str; 5] = [
    "border-image-source",
    "border-image-slice",
    "border-image-width",
    "border-image-outset",
    "border-image-repeat",
];
const BORDER_INLINE: [&str; 3] = [
    "border-inline-width",
    "border-inline-style",
    "border-inline-color",
];
const BORDER_INLINE_COLOR: [&str; 2] = ["border-inline-start-color", "border-inline-end-color"];
const BORDER_INLINE_END: [&str; 3] = [
    "border-inline-end-width",
    "border-inline-end-style",
    "border-inline-end-color",
];
const BORDER_INLINE_START: [&str; 3] = [
    "border-inline-start-width",
    "border-inline-start-style",
    "border-inline-start-color",
];
const BORDER_INLINE_STYLE: [&str; 2] = ["border-inline-start-style", "border-inline-end-style"];
const BORDER_INLINE_WIDTH: [&str; 2] = ["border-inline-start-width", "border-inline-end-width"];
const BORDER_LEFT: [&str; 3] = [
    "border-left-width",
    "border-left-style",
    "border-left-color",
];
const BORDER_RADIUS: [&str; 4] = [
    "border-top-left-radius",
    "border-top-right-radius",
    "border-bottom-right-radius",
    "border-bottom-left-radius",
];
const BORDER_RIGHT: [&str; 3] = [
    "border-right-width",
    "border-right-style",
    "border-right-color",
];
const BORDER_STYLE: [&str; 4] = [
    "border-top-style",
    "border-right-style",
    "border-bottom-style",
    "border-left-style",
];
const BORDER_TOP: [&str; 3] = ["border-top-width", "border-top-style", "border-top-color"];
const BORDER_WIDTH: [&str; 4] = [
    "border-top-width",
    "border-right-width",
    "border-bottom-width",
    "border-left-width",
];
const COLUMN_RULE: [&str; 3] = [
    "column-rule-width",
    "column-rule-style",
    "column-rule-color",
];
const COLUMNS: [&str; 2] = ["column-width", "column-count"];
const CONTAIN_INTRINSIC_SIZE: [&str; 2] = ["contain-intrinsic-width", "contain-intrinsic-height"];
const CONTAINER: [&str; 2] = ["container-name", "container-type"];
const CUE: [&str; 2] = ["cue-before", "cue-after"];
const FLEX: [&str; 4] = ["flex", "flex-grow", "flex-shrink", "flex-basis"];
const FLEX_FLOW: [&str; 2] = ["flex-direction", "flex-wrap"];
const FONT: [&str; 8] = [
    "font",
    "font-style",
    "font-variant",
    "font-weight",
    "font-stretch",
    "font-size",
    "line-height",
    "font-family",
];
const FONT_SYNTHESIS: [&str; 4] = [
    "font-synthesis-weight",
    "font-synthesis-style",
    "font-synthesis-small-caps",
    "font-synthesis-position",
];
const FONT_VARIANT: [&str; 7] = [
    "font-variant-ligatures",
    "font-variant-caps",
    "font-variant-numeric",
    "font-variant-east-asian",
    "font-variant-alternates",
    "font-variant-position",
    "font-variant-emoji",
];
const FONT_VARIANT_LIGATURES: [&str; 4] = [
    "font-variant-ligatures-common",
    "font-variant-ligatures-discretionary",
    "font-variant-ligatures-historical",
    "font-variant-ligatures-contextual",
];
const GAP: [&str; 2] = ["row-gap", "column-gap"];
const GRID: [&str; 6] = [
    "grid-template-rows",
    "grid-template-columns",
    "grid-template-areas",
    "grid-auto-rows",
    "grid-auto-columns",
    "grid-auto-flow",
];
const GRID_AREA: [&str; 4] = [
    "grid-row-start",
    "grid-column-start",
    "grid-row-end",
    "grid-column-end",
];
const GRID_COLUMN: [&str; 3] = ["grid-column", "grid-column-start", "grid-column-end"];
const GRID_ROW: [&str; 3] = ["grid-row", "grid-row-start", "grid-row-end"];
const GRID_TEMPLATE: [&str; 3] = [
    "grid-template-rows",
    "grid-template-columns",
    "grid-template-areas",
];
const INSET: [&str; 4] = ["top", "right", "bottom", "left"];
const INSET_BLOCK: [&str; 2] = ["inset-block-start", "inset-block-end"];
const INSET_INLINE: [&str; 2] = ["inset-inline-start", "inset-inline-end"];
const LINE_CLAMP: [&str; 3] = ["max-lines", "block-ellipsis", "continue"];
const LIST_STYLE: [&str; 3] = ["list-style-type", "list-style-position", "list-style-image"];
const MARGIN: [&str; 4] = ["margin-top", "margin-right", "margin-bottom", "margin-left"];
const MARGIN_BLOCK: [&str; 2] = ["margin-block-start", "margin-block-end"];
const MARGIN_INLINE: [&str; 2] = ["margin-inline-start", "margin-inline-end"];
const MARKER: [&str; 3] = ["marker-start", "marker-mid", "marker-end"];
const MASK: [&str; 8] = [
    "mask-image",
    "mask-mode",
    "mask-position",
    "mask-size",
    "mask-repeat",
    "mask-origin",
    "mask-clip",
    "mask-composite",
];
const MASK_BORDER: [&str; 6] = [
    "mask-border-source",
    "mask-border-slice",
    "mask-border-width",
    "mask-border-outset",
    "mask-border-repeat",
    "mask-border-mode",
];
const OFFSET: [&str; 5] = [
    "offset-position",
    "offset-path",
    "offset-distance",
    "offset-rotate",
    "offset-anchor",
];
const OUTLINE: [&str; 3] = ["outline-width", "outline-style", "outline-color"];
const OVERFLOW: [&str; 2] = ["overflow-x", "overflow-y"];
const OVERSCROLL_BEHAVIOR: [&str; 2] = ["overscroll-behavior-x", "overscroll-behavior-y"];
const OVERSCROLL_BEHAVIOR_BLOCK: [&str; 2] = [
    "overscroll-behavior-block-start",
    "overscroll-behavior-block-end",
];
const OVERSCROLL_BEHAVIOR_INLINE: [&str; 2] = [
    "overscroll-behavior-inline-start",
    "overscroll-behavior-inline-end",
];
const PADDING: [&str; 4] = [
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
];
const PADDING_BLOCK: [&str; 2] = ["padding-block-start", "padding-block-end"];
const PADDING_INLINE: [&str; 2] = ["padding-inline-start", "padding-inline-end"];
const PAUSE: [&str; 2] = ["pause-before", "pause-after"];
const PLACE_CONTENT: [&str; 2] = ["align-content", "justify-content"];
const PLACE_ITEMS: [&str; 2] = ["align-items", "justify-items"];
const PLACE_SELF: [&str; 2] = ["align-self", "justify-self"];
const REST: [&str; 2] = ["rest-before", "rest-after"];
const SCROLL_MARGIN: [&str; 4] = [
    "scroll-margin-top",
    "scroll-margin-right",
    "scroll-margin-bottom",
    "scroll-margin-left",
];
const SCROLL_MARGIN_BLOCK: [&str; 2] = ["scroll-margin-block-start", "scroll-margin-block-end"];
const SCROLL_MARGIN_INLINE: [&str; 2] = ["scroll-margin-inline-start", "scroll-margin-inline-end"];
const SCROLL_PADDING: [&str; 4] = [
    "scroll-padding-top",
    "scroll-padding-right",
    "scroll-padding-bottom",
    "scroll-padding-left",
];
const SCROLL_PADDING_BLOCK: [&str; 2] = ["scroll-padding-block-start", "scroll-padding-block-end"];
const SCROLL_PADDING_INLINE: [&str; 2] =
    ["scroll-padding-inline-start", "scroll-padding-inline-end"];
const SCROLL_TIMELINE: [&str; 2] = ["scroll-timeline-name", "scroll-timeline-axis"];
const TEXT_BOX: [&str; 2] = ["text-box-trim", "text-box-edge"];
const TEXT_DECORATION: [&str; 4] = [
    "text-decoration-line",
    "text-decoration-style",
    "text-decoration-color",
    "text-decoration-thickness",
];
const TEXT_EMPHASIS: [&str; 2] = ["text-emphasis-style", "text-emphasis-color"];
const TRANSITION: [&str; 6] = [
    "transition",
    "transition-property",
    "transition-duration",
    "transition-timing-function",
    "transition-delay",
    "transition-behavior",
];
const VIEW_TIMELINE: [&str; 3] = [
    "view-timeline-name",
    "view-timeline-axis",
    "view-timeline-inset",
];
const WHITE_SPACE_SHORTHAND: [&str; 4] = [
    "white-space",
    "white-space-collapse",
    "text-wrap-mode",
    "white-space-trim",
];

static ALL_SHORTHANDS: &[&str] = &[
    "animation",
    "animation-range",
    "background",
    "background-position",
    "border",
    "border-block",
    "border-block-color",
    "border-block-end",
    "border-block-start",
    "border-block-style",
    "border-block-width",
    "border-bottom",
    "border-color",
    "border-image",
    "border-inline",
    "border-inline-color",
    "border-inline-end",
    "border-inline-start",
    "border-inline-style",
    "border-inline-width",
    "border-left",
    "border-radius",
    "border-right",
    "border-style",
    "border-top",
    "border-width",
    "column-rule",
    "columns",
    "contain-intrinsic-size",
    "container",
    "cue",
    "flex",
    "flex-flow",
    "font",
    "font-synthesis",
    "font-variant",
    "font-variant-ligatures",
    "gap",
    "grid",
    "grid-area",
    "grid-column",
    "grid-row",
    "grid-template",
    "inset",
    "inset-block",
    "inset-inline",
    "line-clamp",
    "list-style",
    "margin",
    "margin-block",
    "margin-inline",
    "marker",
    "mask",
    "mask-border",
    "offset",
    "outline",
    "overflow",
    "overscroll-behavior",
    "overscroll-behavior-block",
    "overscroll-behavior-inline",
    "padding",
    "padding-block",
    "padding-inline",
    "pause",
    "place-content",
    "place-items",
    "place-self",
    "rest",
    "scroll-margin",
    "scroll-margin-block",
    "scroll-margin-inline",
    "scroll-padding",
    "scroll-padding-block",
    "scroll-padding-inline",
    "scroll-timeline",
    "text-box",
    "text-decoration",
    "text-emphasis",
    "transition",
    "view-timeline",
    "white-space",
];

static DEFERRED_LONGHANDS: &[&str] = &[
    "animation-composition",
    "animation-delay",
    "animation-direction",
    "animation-duration",
    "animation-fill-mode",
    "animation-iteration-count",
    "animation-name",
    "animation-play-state",
    "animation-range-end",
    "animation-range-start",
    "animation-timeline",
    "animation-timing-function",
    "background-attachment",
    "background-origin",
    "background-position-x",
    "background-position-y",
    "block-ellipsis",
    "border-collapse",
    "border-spacing",
    "border-block-end-color",
    "border-block-end-style",
    "border-block-end-width",
    "border-block-start-color",
    "border-block-start-style",
    "border-block-start-width",
    "border-image-outset",
    "border-image-repeat",
    "border-image-slice",
    "border-image-source",
    "border-image-width",
    "border-inline-end-color",
    "border-inline-end-style",
    "border-inline-end-width",
    "border-inline-start-color",
    "border-inline-start-style",
    "border-inline-start-width",
    "column-count",
    "column-rule-color",
    "column-rule-style",
    "column-rule-width",
    "column-width",
    "container-name",
    "container-type",
    "contain-intrinsic-height",
    "contain-intrinsic-width",
    "continue",
    "cue-after",
    "cue-before",
    "direction",
    "font-stretch",
    "font-synthesis-position",
    "font-synthesis-small-caps",
    "font-synthesis-style",
    "font-synthesis-weight",
    "font-variant",
    "font-variant-alternates",
    "font-variant-caps",
    "font-variant-east-asian",
    "font-variant-emoji",
    "font-variant-ligatures",
    "font-variant-ligatures-common",
    "font-variant-ligatures-contextual",
    "font-variant-ligatures-discretionary",
    "font-variant-ligatures-historical",
    "font-variant-numeric",
    "font-variant-position",
    "grid-template-areas",
    "inset-block-end",
    "inset-block-start",
    "inset-inline-end",
    "inset-inline-start",
    "list-style-image",
    "list-style-position",
    "list-style-type",
    "margin-block-end",
    "margin-block-start",
    "margin-inline-end",
    "margin-inline-start",
    "marker-end",
    "marker-mid",
    "marker-start",
    "mask-border-mode",
    "mask-border-outset",
    "mask-border-repeat",
    "mask-border-slice",
    "mask-border-source",
    "mask-border-width",
    "mask-clip",
    "mask-composite",
    "mask-image",
    "mask-mode",
    "mask-origin",
    "mask-position",
    "mask-repeat",
    "mask-size",
    "max-lines",
    "offset-anchor",
    "offset-distance",
    "offset-path",
    "offset-position",
    "offset-rotate",
    "overflow-wrap",
    "outline-color",
    "outline-style",
    "outline-width",
    "overscroll-behavior-block-end",
    "overscroll-behavior-block-start",
    "overscroll-behavior-inline-end",
    "overscroll-behavior-inline-start",
    "overscroll-behavior-x",
    "overscroll-behavior-y",
    "padding-block-end",
    "padding-block-start",
    "padding-inline-end",
    "padding-inline-start",
    "pause-after",
    "pause-before",
    "rest-after",
    "rest-before",
    "resize",
    "scroll-margin-block-end",
    "scroll-margin-block-start",
    "scroll-margin-bottom",
    "scroll-margin-inline-end",
    "scroll-margin-inline-start",
    "scroll-margin-left",
    "scroll-margin-right",
    "scroll-margin-top",
    "scroll-padding-block-end",
    "scroll-padding-block-start",
    "scroll-padding-bottom",
    "scroll-padding-inline-end",
    "scroll-padding-inline-start",
    "scroll-padding-left",
    "scroll-padding-right",
    "scroll-padding-top",
    "scroll-timeline-axis",
    "scroll-timeline-name",
    "text-box-edge",
    "text-box-trim",
    "text-indent",
    "text-shadow",
    "text-decoration-color",
    "text-decoration-line",
    "text-decoration-style",
    "text-decoration-thickness",
    "text-emphasis-color",
    "text-emphasis-style",
    "text-wrap-mode",
    "transition-behavior",
    "transition-delay",
    "transition-duration",
    "transition-property",
    "transition-timing-function",
    "unicode-bidi",
    "view-timeline-axis",
    "view-timeline-inset",
    "view-timeline-name",
    "white-space-collapse",
    "white-space-trim",
    "word-spacing",
];

static INHERITED_DEFERRED_LONGHANDS: &[&str] = &[
    "font-stretch",
    "font-synthesis-position",
    "font-synthesis-small-caps",
    "font-synthesis-style",
    "font-synthesis-weight",
    "font-variant",
    "font-variant-alternates",
    "font-variant-caps",
    "font-variant-east-asian",
    "font-variant-emoji",
    "font-variant-ligatures",
    "font-variant-ligatures-common",
    "font-variant-ligatures-contextual",
    "font-variant-ligatures-discretionary",
    "font-variant-ligatures-historical",
    "font-variant-numeric",
    "font-variant-position",
    "border-collapse",
    "border-spacing",
    "direction",
    "list-style-image",
    "list-style-position",
    "list-style-type",
    "overflow-wrap",
    "text-indent",
    "text-shadow",
    "text-wrap-mode",
    "word-spacing",
    "white-space-collapse",
    "white-space-trim",
];
