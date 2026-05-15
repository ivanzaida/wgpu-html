use lui_cascade::matching::{MatchContext, any_selector_matches};
use lui_parse::{parse, parse_selector_list};

pub fn root_ctx() -> MatchContext<'static> {
  MatchContext {
    is_root: true,
    is_first_child: true,
    is_last_child: true,
    is_only_child: true,
    sibling_index: 0,
    sibling_count: 1,
    ..Default::default()
  }
}

pub fn child_ctx(index: usize, count: usize) -> MatchContext<'static> {
  MatchContext {
    is_first_child: index == 0,
    is_last_child: index == count - 1,
    is_only_child: count == 1,
    sibling_index: index,
    sibling_count: count,
    ..Default::default()
  }
}

#[track_caller]
pub fn assert_matches(selector: &str, html: &str) {
  let doc = parse(html);
  let node = &doc.root.children()[0];
  let sel = parse_selector_list(selector).unwrap();
  assert!(
    any_selector_matches(&sel, node, &root_ctx(), &[], Some(&doc.root)).is_some(),
    "expected `{}` to match `{}`",
    selector,
    html,
  );
}

#[track_caller]
pub fn assert_no_match(selector: &str, html: &str) {
  let doc = parse(html);
  let node = &doc.root.children()[0];
  let sel = parse_selector_list(selector).unwrap();
  assert!(
    any_selector_matches(&sel, node, &root_ctx(), &[], Some(&doc.root)).is_none(),
    "expected `{}` NOT to match `{}`",
    selector,
    html,
  );
}
