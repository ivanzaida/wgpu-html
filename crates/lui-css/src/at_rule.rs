use crate::stylesheet::CssRule;

/// Trait for parsing a specific at-rule (e.g., @media, @keyframes, @font-face).
///
/// Implementors handle one `@name` and convert raw prelude + block text into
/// a typed [`CssRule`]. The parser dispatches to registered implementors by
/// matching the at-rule name.
///
/// # Adding a new at-rule
///
/// 1. Add a variant to [`CssRule`] in `stylesheet.rs`.
/// 2. Create a struct implementing this trait.
/// 3. Register it in [`AtRuleRegistry::with_builtins`] (or at runtime via [`AtRuleRegistry::register`]).
pub trait AtRuleParser: Send + Sync {
  /// The at-rule name this parser handles, lowercase, without `@`.
  fn name(&self) -> &'static str;

  /// Parse a statement at-rule (one that ends with `;`, no block).
  /// e.g., `@import url("foo.css");`
  ///
  /// `prelude` is the text between the at-keyword and the `;`.
  /// Return `None` to signal that this at-rule requires a block.
  fn parse_statement(&self, prelude: &str) -> Option<CssRule> {
    let _ = prelude;
    None
  }

  /// Parse a block at-rule (one that has `{ ... }`).
  /// e.g., `@media screen { ... }`
  ///
  /// `prelude` is the text between the at-keyword and `{`.
  /// `block` is the text inside the braces.
  /// `parse_nested` is a callback to recursively parse nested rule lists
  /// (used by @media, @supports, @layer, etc.).
  fn parse_block(&self, prelude: &str, block: &str, parse_nested: &dyn Fn(&str) -> Vec<CssRule>) -> Option<CssRule>;
}

pub struct AtRuleRegistry {
  parsers: Vec<Box<dyn AtRuleParser>>,
}

impl AtRuleRegistry {
  pub fn new() -> Self {
    Self { parsers: Vec::new() }
  }

  pub fn with_builtins() -> Self {
    use crate::at_rules::*;

    let mut registry = Self::new();
    registry.register(MediaAtRuleParser);
    registry.register(ImportAtRuleParser);
    registry.register(KeyframesAtRuleParser);
    registry.register(FontFaceAtRuleParser);
    registry.register(SupportsAtRuleParser);
    registry
  }

  pub fn register<P: AtRuleParser + 'static>(&mut self, parser: P) {
    self.parsers.push(Box::new(parser));
  }

  pub fn try_parse_statement(&self, name: &str, prelude: &str) -> Option<CssRule> {
    let lower = name.to_ascii_lowercase();
    for parser in &self.parsers {
      if parser.name() == lower {
        return parser.parse_statement(prelude);
      }
    }
    None
  }

  pub fn try_parse_block(
    &self,
    name: &str,
    prelude: &str,
    block: &str,
    parse_nested: &dyn Fn(&str) -> Vec<CssRule>,
  ) -> Option<CssRule> {
    let lower = name.to_ascii_lowercase();
    for parser in &self.parsers {
      if parser.name() == lower {
        return parser.parse_block(prelude, block, parse_nested);
      }
    }
    None
  }

  pub fn knows(&self, name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    self.parsers.iter().any(|p| p.name() == lower)
  }
}

impl Default for AtRuleRegistry {
  fn default() -> Self {
    Self::with_builtins()
  }
}
