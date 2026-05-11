mod animation;
mod background;
mod border;
mod box_model;
mod flex;
mod grid;
mod position;
mod property_group;
mod svg;
mod typography;
mod visual;

pub use animation::AnimationGroup;
pub use background::BackgroundGroup;
pub use border::BorderGroup;
pub use box_model::BoxModelGroup;
pub use flex::FlexGroup;
pub use grid::GridGroup;
pub use position::PositionGroup;
pub use property_group::PropertyGroup;
pub use svg::SvgGroup;
pub use typography::TypographyGroup;
pub use visual::VisualGroup;

use crate::{
  css_parser::apply_generic_shorthand,
  declaration::DeclarationBlock,
  shorthands::{is_deferred_longhand, shorthand_members},
  style::Style,
  values::ArcStr,
};

static GROUPS: &[&dyn PropertyGroup] = &[
  &BorderGroup,
  &BackgroundGroup,
  &PositionGroup,
  &BoxModelGroup,
  &TypographyGroup,
  &FlexGroup,
  &GridGroup,
  &VisualGroup,
  &SvgGroup,
  &AnimationGroup,
];

pub fn apply_declarations(declarations: &DeclarationBlock, style: &mut Style) {
  for group in GROUPS {
    group.apply(declarations, style);
  }

  for decl in &declarations.declarations {
    let prop = &*decl.property;
    if is_handled_by_groups(prop) {
      continue;
    }
    if shorthand_members(prop).is_some() {
      apply_generic_shorthand(style, prop, &decl.value);
    } else if is_deferred_longhand(prop) {
      style
        .deferred_longhands
        .insert(decl.property.clone(), ArcStr::from(decl.value.as_ref()));
    }
  }
}

fn is_handled_by_groups(property: &str) -> bool {
  GROUPS.iter().any(|g| g.handled_properties().contains(&property))
}
