use crate::CssAtRule;
use crate::CssProperty;
use crate::media::MediaQueryList;
use crate::selector::SelectorList;
use crate::supports::SupportsCondition;
use crate::value::CssValue;

/// One parsed CSS rule: a selector list with its declarations and computed specificity.
#[derive(Debug, Clone)]
pub struct StyleRule {
    pub selector: SelectorList,
    pub declarations: Vec<Declaration>,
    pub specificity: (u32, u32, u32),
}

impl StyleRule {
    pub fn new(selector: SelectorList, declarations: Vec<Declaration>, specificity: (u32, u32, u32)) -> Self {
        Self { selector, declarations, specificity }
    }
}

/// A single `property: value` pair (potentially `!important`).
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub property: CssProperty,
    pub value: CssValue,
    pub important: bool,
}

/// A parsed at-rule (e.g. `@media`, `@keyframes`, `@font-face`).
#[derive(Debug, Clone)]
pub struct AtRule {
    pub at_rule: CssAtRule,
    pub prelude: String,
    pub media: Option<MediaQueryList>,
    pub supports: Option<SupportsCondition>,
    pub rules: Vec<StyleRule>,
    pub at_rules: Vec<AtRule>,
    pub comments: Vec<String>,
}

/// A full parsed stylesheet.
#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    pub rules: Vec<StyleRule>,
    pub at_rules: Vec<AtRule>,
    pub comments: Vec<String>,
}
