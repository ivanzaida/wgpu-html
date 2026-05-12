use std::collections::HashMap;

use lui_css_parser::{CssAtRule, Stylesheet, StyleRule, AtRule};

/// A reference to a specific selector within a specific rule.
#[derive(Debug, Clone, Copy)]
pub struct RuleRef {
    pub rule_idx: usize,
    pub selector_idx: usize,
}

/// Pre-indexed rules for fast lookup by element id/class/tag.
#[derive(Debug, Default)]
pub struct RuleIndex {
    pub by_id: HashMap<String, Vec<RuleRef>>,
    pub by_class: HashMap<String, Vec<RuleRef>>,
    pub by_tag: HashMap<String, Vec<RuleRef>>,
    pub universal: Vec<RuleRef>,
}

/// A flattened rule that came from inside an @media or @supports block.
/// Carries the condition index so the cascade can check it at match time.
#[derive(Debug)]
pub struct ConditionalRule {
    pub rule: StyleRule,
    pub condition_idx: usize,
}

/// Condition attached to conditional rules.
#[derive(Debug)]
pub enum RuleCondition {
    Media(lui_css_parser::MediaQueryList),
    Supports(lui_css_parser::SupportsCondition),
}

/// A stylesheet prepared for fast per-element rule lookup.
#[derive(Debug)]
pub struct PreparedStylesheet {
    pub rules: Vec<StyleRule>,
    pub conditional_rules: Vec<ConditionalRule>,
    pub conditions: Vec<RuleCondition>,
    pub index: RuleIndex,
    pub conditional_index: RuleIndex,
}

impl PreparedStylesheet {
    pub fn new(sheet: Stylesheet) -> Self {
        let mut conditions: Vec<RuleCondition> = Vec::new();
        let mut conditional_rules: Vec<ConditionalRule> = Vec::new();

        flatten_at_rules(&sheet.at_rules, &mut conditions, &mut conditional_rules);

        let index = build_index(&sheet.rules);
        let conditional_index = build_conditional_index(&conditional_rules);

        Self {
            rules: sheet.rules,
            conditional_rules,
            conditions,
            index,
            conditional_index,
        }
    }
}

fn flatten_at_rules(
    at_rules: &[AtRule],
    conditions: &mut Vec<RuleCondition>,
    out: &mut Vec<ConditionalRule>,
) {
    for at_rule in at_rules {
        match at_rule.at_rule {
            CssAtRule::Media => {
                if let Some(ref mql) = at_rule.media {
                    let cond_idx = conditions.len();
                    conditions.push(RuleCondition::Media(mql.clone()));
                    for rule in &at_rule.rules {
                        out.push(ConditionalRule {
                            rule: rule.clone(),
                            condition_idx: cond_idx,
                        });
                    }
                    flatten_at_rules(&at_rule.at_rules, conditions, out);
                }
            }
            CssAtRule::Supports => {
                if let Some(ref sc) = at_rule.supports {
                    let cond_idx = conditions.len();
                    conditions.push(RuleCondition::Supports(sc.clone()));
                    for rule in &at_rule.rules {
                        out.push(ConditionalRule {
                            rule: rule.clone(),
                            condition_idx: cond_idx,
                        });
                    }
                    flatten_at_rules(&at_rule.at_rules, conditions, out);
                }
            }
            _ => {}
        }
    }
}

fn build_index(rules: &[StyleRule]) -> RuleIndex {
    let mut index = RuleIndex::default();
    for (rule_idx, rule) in rules.iter().enumerate() {
        for (selector_idx, complex) in rule.selector.0.iter().enumerate() {
            let entry = RuleRef { rule_idx, selector_idx };
            index_selector(&mut index, &entry, complex);
        }
    }
    index
}

fn build_conditional_index(rules: &[ConditionalRule]) -> RuleIndex {
    let mut index = RuleIndex::default();
    for (rule_idx, cond_rule) in rules.iter().enumerate() {
        for (selector_idx, complex) in cond_rule.rule.selector.0.iter().enumerate() {
            let entry = RuleRef { rule_idx, selector_idx };
            index_selector(&mut index, &entry, complex);
        }
    }
    index
}

fn index_selector(
    index: &mut RuleIndex,
    entry: &RuleRef,
    complex: &lui_css_parser::selector::ComplexSelector,
) {
    let subject = match complex.compounds.last() {
        Some(s) => s,
        None => return,
    };

    if let Some(ref id) = subject.id {
        index.by_id.entry(id.clone()).or_default().push(*entry);
    } else if let Some(class) = subject.classes.first() {
        index.by_class.entry(class.clone()).or_default().push(*entry);
    } else if let Some(ref tag) = subject.tag {
        if tag != "*" {
            index.by_tag.entry(tag.clone()).or_default().push(*entry);
        } else {
            index.universal.push(*entry);
        }
    } else {
        index.universal.push(*entry);
    }
}

/// Collect candidate `RuleRef`s that might match an element with the given
/// id, classes, and tag. The caller must still do full selector matching
/// on each candidate.
pub fn candidate_rules<'a>(
    index: &'a RuleIndex,
    tag: &str,
    id: Option<&str>,
    classes: &[&str],
) -> Vec<&'a RuleRef> {
    let mut seen = std::collections::HashSet::new();
    let mut candidates = Vec::new();

    let mut add = |refs: &'a [RuleRef]| {
        for r in refs {
            let key = (r.rule_idx, r.selector_idx);
            if seen.insert(key) {
                candidates.push(r);
            }
        }
    };

    if let Some(id) = id {
        if let Some(refs) = index.by_id.get(id) {
            add(refs);
        }
    }

    for class in classes {
        if let Some(refs) = index.by_class.get(*class) {
            add(refs);
        }
    }

    if let Some(refs) = index.by_tag.get(tag) {
        add(refs);
    }

    add(&index.universal);

    candidates
}
