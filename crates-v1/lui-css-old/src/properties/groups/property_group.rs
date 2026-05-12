use crate::{declaration::DeclarationBlock, style::Style};

pub trait PropertyGroup: Send + Sync {
  fn apply(&self, declarations: &DeclarationBlock, style: &mut Style);
  fn handled_properties(&self) -> &'static [&'static str];
}
