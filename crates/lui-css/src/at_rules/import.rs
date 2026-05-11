use crate::{
  at_rule::AtRuleParser,
  stylesheet::{CssRule, ImportRule},
  values::ArcStr,
};

pub struct ImportAtRuleParser;

impl AtRuleParser for ImportAtRuleParser {
  fn name(&self) -> &'static str {
    "import"
  }

  fn parse_statement(&self, prelude: &str) -> Option<CssRule> {
    let (url, media) = parse_import_prelude(prelude)?;
    Some(CssRule::Import(ImportRule {
      url: ArcStr::from(url),
      media: media.map(|m| ArcStr::from(m.trim())),
    }))
  }

  fn parse_block(&self, _prelude: &str, _block: &str, _parse_nested: &dyn Fn(&str) -> Vec<CssRule>) -> Option<CssRule> {
    None
  }
}

fn parse_import_prelude(s: &str) -> Option<(&str, Option<&str>)> {
  let s = s.trim();
  let (url, rest) = if let Some(inner) = s.strip_prefix("url(") {
    let inner = inner.trim_start();
    if let Some(inner) = inner.strip_prefix('"') {
      let end = inner.find('"')?;
      let rest = inner[end + 1..].trim_start().strip_prefix(')')?.trim();
      (&inner[..end], rest)
    } else if let Some(inner) = inner.strip_prefix('\'') {
      let end = inner.find('\'')?;
      let rest = inner[end + 1..].trim_start().strip_prefix(')')?.trim();
      (&inner[..end], rest)
    } else {
      let end = inner.find(')')?;
      (inner[..end].trim(), inner[end + 1..].trim())
    }
  } else if let Some(inner) = s.strip_prefix('"') {
    let end = inner.find('"')?;
    (&inner[..end], inner[end + 1..].trim())
  } else if let Some(inner) = s.strip_prefix('\'') {
    let end = inner.find('\'')?;
    (&inner[..end], inner[end + 1..].trim())
  } else {
    return None;
  };

  let media = if rest.is_empty() { None } else { Some(rest) };
  Some((url, media))
}
