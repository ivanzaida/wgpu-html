pub fn is_inherited(property: &str) -> bool {
  matches!(
    property,
    "color"
      | "font-family"
      | "font-size"
      | "font-weight"
      | "font-style"
      | "line-height"
      | "letter-spacing"
      | "text-align"
      | "text-decoration"
      | "text-transform"
      | "text-overflow"
      | "white-space"
      | "word-break"
      | "visibility"
      | "cursor"
      | "list-style-type"
      | "list-style-position"
      | "fill"
      | "fill-opacity"
      | "fill-rule"
      | "stroke"
      | "stroke-width"
      | "stroke-opacity"
      | "stroke-linecap"
      | "stroke-linejoin"
      | "stroke-dasharray"
      | "stroke-dashoffset"
      | "pointer-events"
      | "user-select"
  )
}

pub fn is_shorthand(property: &str) -> bool {
  shorthand_longhands(property).is_some()
}

pub fn shorthand_longhands(property: &str) -> Option<&'static [&'static str]> {
  match property {
    "margin" => Some(&["margin-top", "margin-right", "margin-bottom", "margin-left"]),
    "padding" => Some(&["padding-top", "padding-right", "padding-bottom", "padding-left"]),
    "border" => Some(&[
      "border-top-width",
      "border-right-width",
      "border-bottom-width",
      "border-left-width",
      "border-top-style",
      "border-right-style",
      "border-bottom-style",
      "border-left-style",
      "border-top-color",
      "border-right-color",
      "border-bottom-color",
      "border-left-color",
    ]),
    "border-width" => Some(&[
      "border-top-width",
      "border-right-width",
      "border-bottom-width",
      "border-left-width",
    ]),
    "border-style" => Some(&[
      "border-top-style",
      "border-right-style",
      "border-bottom-style",
      "border-left-style",
    ]),
    "border-color" => Some(&[
      "border-top-color",
      "border-right-color",
      "border-bottom-color",
      "border-left-color",
    ]),
    "border-radius" => Some(&[
      "border-top-left-radius",
      "border-top-right-radius",
      "border-bottom-right-radius",
      "border-bottom-left-radius",
    ]),
    "border-top" => Some(&["border-top-width", "border-top-style", "border-top-color"]),
    "border-right" => Some(&["border-right-width", "border-right-style", "border-right-color"]),
    "border-bottom" => Some(&["border-bottom-width", "border-bottom-style", "border-bottom-color"]),
    "border-left" => Some(&["border-left-width", "border-left-style", "border-left-color"]),
    "background" => Some(&[
      "background-color",
      "background-image",
      "background-size",
      "background-position",
      "background-repeat",
      "background-clip",
    ]),
    "flex" => Some(&["flex-grow", "flex-shrink", "flex-basis"]),
    "flex-flow" => Some(&["flex-direction", "flex-wrap"]),
    "gap" => Some(&["row-gap", "column-gap"]),
    "grid-column" => Some(&["grid-column-start", "grid-column-end"]),
    "grid-row" => Some(&["grid-row-start", "grid-row-end"]),
    "grid-template" => Some(&["grid-template-columns", "grid-template-rows"]),
    "overflow" => Some(&["overflow-x", "overflow-y"]),
    "inset" => Some(&["top", "right", "bottom", "left"]),
    "place-content" => Some(&["align-content", "justify-content"]),
    "place-items" => Some(&["align-items", "justify-items"]),
    "place-self" => Some(&["align-self", "justify-self"]),
    "list-style" => Some(&["list-style-type", "list-style-position"]),
    "transition" => Some(&[
      "transition-property",
      "transition-duration",
      "transition-timing-function",
      "transition-delay",
    ]),
    "animation" => Some(&[
      "animation-name",
      "animation-duration",
      "animation-timing-function",
      "animation-delay",
      "animation-iteration-count",
      "animation-direction",
      "animation-fill-mode",
      "animation-play-state",
    ]),
    _ => None,
  }
}
