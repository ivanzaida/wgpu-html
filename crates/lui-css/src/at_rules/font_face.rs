use crate::{
  at_rule::AtRuleParser,
  stylesheet::{CssRule, FontFaceDescriptor, FontFaceRule},
  values::ArcStr,
};

pub struct FontFaceAtRuleParser;

impl AtRuleParser for FontFaceAtRuleParser {
  fn name(&self) -> &'static str {
    "font-face"
  }

  fn parse_block(&self, prelude: &str, block: &str, _parse_nested: &dyn Fn(&str) -> Vec<CssRule>) -> Option<CssRule> {
    if !prelude.trim().is_empty() {
      return None;
    }
    let descriptors = parse_font_face_descriptors(block);
    Some(CssRule::FontFace(FontFaceRule { descriptors }))
  }
}

fn parse_font_face_descriptors(input: &str) -> Vec<FontFaceDescriptor> {
  let mut descriptors = Vec::new();
  for decl in input.split(';') {
    let decl = decl.trim();
    if decl.is_empty() {
      continue;
    }
    if let Some((name, value)) = decl.split_once(':') {
      descriptors.push(FontFaceDescriptor {
        name: ArcStr::from(name.trim().to_ascii_lowercase().as_str()),
        value: ArcStr::from(value.trim()),
      });
    }
  }
  descriptors
}
