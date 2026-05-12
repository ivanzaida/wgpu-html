use lui_css_old::{
  properties::{is_shorthand, shorthand_longhands},
  PropertyId,
};

#[test]
fn detects_common_shorthands() {
  assert!(is_shorthand("margin"));
  assert!(is_shorthand("padding"));
  assert!(is_shorthand("border"));
  assert!(is_shorthand("background"));
  assert!(is_shorthand("flex"));
  assert!(is_shorthand("gap"));
  assert!(is_shorthand("overflow"));
  assert!(is_shorthand("transition"));
  assert!(is_shorthand("animation"));
}

#[test]
fn longhands_are_not_shorthands() {
  assert!(!is_shorthand("color"));
  assert!(!is_shorthand("display"));
  assert!(!is_shorthand("margin-top"));
  assert!(!is_shorthand("flex-grow"));
  assert!(!is_shorthand("grid-column-start"));
}

#[test]
fn margin_expands_to_four_sides() {
  let longhands = shorthand_longhands("margin").unwrap();
  assert_eq!(longhands.len(), 4);
  assert!(longhands.contains(&"margin-top"));
  assert!(longhands.contains(&"margin-right"));
  assert!(longhands.contains(&"margin-bottom"));
  assert!(longhands.contains(&"margin-left"));
}

#[test]
fn padding_expands_to_four_sides() {
  let longhands = shorthand_longhands("padding").unwrap();
  assert_eq!(longhands.len(), 4);
  assert!(longhands.contains(&"padding-top"));
  assert!(longhands.contains(&"padding-left"));
}

#[test]
fn border_expands_to_twelve_longhands() {
  let longhands = shorthand_longhands("border").unwrap();
  assert_eq!(longhands.len(), 12);
  assert!(longhands.contains(&"border-top-width"));
  assert!(longhands.contains(&"border-left-color"));
  assert!(longhands.contains(&"border-bottom-style"));
}

#[test]
fn flex_expands_to_grow_shrink_basis() {
  let longhands = shorthand_longhands("flex").unwrap();
  assert_eq!(longhands.len(), 3);
  assert!(longhands.contains(&"flex-grow"));
  assert!(longhands.contains(&"flex-shrink"));
  assert!(longhands.contains(&"flex-basis"));
}

#[test]
fn grid_column_expands_to_start_end() {
  let longhands = shorthand_longhands("grid-column").unwrap();
  assert_eq!(longhands.len(), 2);
  assert!(longhands.contains(&"grid-column-start"));
  assert!(longhands.contains(&"grid-column-end"));
}

#[test]
fn grid_row_expands_to_start_end() {
  let longhands = shorthand_longhands("grid-row").unwrap();
  assert_eq!(longhands.len(), 2);
  assert!(longhands.contains(&"grid-row-start"));
  assert!(longhands.contains(&"grid-row-end"));
}

#[test]
fn overflow_expands_to_x_y() {
  let longhands = shorthand_longhands("overflow").unwrap();
  assert_eq!(longhands.len(), 2);
  assert!(longhands.contains(&"overflow-x"));
  assert!(longhands.contains(&"overflow-y"));
}

#[test]
fn border_side_shorthands() {
  for side in ["top", "right", "bottom", "left"] {
    let name = format!("border-{side}");
    let longhands = shorthand_longhands(&name).unwrap();
    assert_eq!(longhands.len(), 3);
    let width_prop = format!("border-{side}-width");
    assert!(longhands.iter().any(|l| *l == width_prop));
  }
}

#[test]
fn place_shorthands() {
  let longhands = shorthand_longhands("place-content").unwrap();
  assert!(longhands.contains(&"align-content"));
  assert!(longhands.contains(&"justify-content"));

  let longhands = shorthand_longhands("place-items").unwrap();
  assert!(longhands.contains(&"align-items"));
  assert!(longhands.contains(&"justify-items"));

  let longhands = shorthand_longhands("place-self").unwrap();
  assert!(longhands.contains(&"align-self"));
  assert!(longhands.contains(&"justify-self"));
}

#[test]
fn property_id_is_shorthand_matches_function() {
  let id = PropertyId::from_name("margin").unwrap();
  assert!(id.is_shorthand());

  let id = PropertyId::from_name("color").unwrap();
  assert!(!id.is_shorthand());
}

#[test]
fn unknown_property_is_not_shorthand() {
  assert!(!is_shorthand("nonexistent"));
  assert!(shorthand_longhands("nonexistent").is_none());
}
