use lui_css_old::PropertyId;

#[test]
fn resolves_common_properties() {
  assert_eq!(PropertyId::from_name("display"), Some(PropertyId::Display));
  assert_eq!(PropertyId::from_name("color"), Some(PropertyId::Color));
  assert_eq!(PropertyId::from_name("margin"), Some(PropertyId::Margin));
  assert_eq!(PropertyId::from_name("padding-top"), Some(PropertyId::PaddingTop));
  assert_eq!(PropertyId::from_name("flex-direction"), Some(PropertyId::FlexDirection));
}

#[test]
fn resolves_custom_property() {
  assert_eq!(PropertyId::from_name("--my-var"), Some(PropertyId::Custom));
  assert_eq!(PropertyId::from_name("--X"), Some(PropertyId::Custom));
}

#[test]
fn returns_none_for_unknown_property() {
  assert_eq!(PropertyId::from_name("nonexistent"), None);
  assert_eq!(PropertyId::from_name(""), None);
}

#[test]
fn round_trips_all_known_properties() {
  let props = [
    "display",
    "position",
    "top",
    "right",
    "bottom",
    "left",
    "width",
    "height",
    "min-width",
    "min-height",
    "max-width",
    "max-height",
    "margin",
    "margin-top",
    "margin-right",
    "margin-bottom",
    "margin-left",
    "padding",
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
    "color",
    "accent-color",
    "background",
    "background-color",
    "background-image",
    "border",
    "border-top-width",
    "border-radius",
    "font-family",
    "font-size",
    "font-weight",
    "font-style",
    "line-height",
    "letter-spacing",
    "text-align",
    "text-transform",
    "opacity",
    "visibility",
    "z-index",
    "flex-direction",
    "flex-wrap",
    "flex-grow",
    "flex-shrink",
    "flex-basis",
    "grid-template-columns",
    "grid-template-rows",
    "grid-auto-flow",
    "justify-content",
    "align-items",
    "align-self",
    "transform",
    "transition",
    "animation",
    "cursor",
    "pointer-events",
    "user-select",
    "box-sizing",
    "box-shadow",
  ];
  for name in props {
    let id = PropertyId::from_name(name).unwrap_or_else(|| panic!("PropertyId::from_name({name:?}) returned None"));
    assert_eq!(id.as_css_name(), name, "round-trip failed for {name}");
  }
}

#[test]
fn resolves_svg_properties() {
  assert_eq!(PropertyId::from_name("fill"), Some(PropertyId::Fill));
  assert_eq!(PropertyId::from_name("stroke"), Some(PropertyId::Stroke));
  assert_eq!(PropertyId::from_name("stroke-width"), Some(PropertyId::StrokeWidth));
  assert_eq!(PropertyId::from_name("fill-opacity"), Some(PropertyId::FillOpacity));
}

#[test]
fn resolves_grid_properties() {
  assert_eq!(PropertyId::from_name("grid-column"), Some(PropertyId::GridColumn));
  assert_eq!(PropertyId::from_name("grid-row-start"), Some(PropertyId::GridRowStart));
  assert_eq!(
    PropertyId::from_name("grid-auto-columns"),
    Some(PropertyId::GridAutoColumns)
  );
  assert_eq!(PropertyId::from_name("justify-self"), Some(PropertyId::JustifySelf));
}

#[test]
fn display_impl_shows_css_name() {
  assert_eq!(PropertyId::Display.to_string(), "display");
  assert_eq!(PropertyId::FlexDirection.to_string(), "flex-direction");
  assert_eq!(PropertyId::GridTemplateColumns.to_string(), "grid-template-columns");
}
