use lui_cascade::matching::any_selector_matches;
use lui_parse::{parse_selector_list, parse};
use crate::helpers::child_ctx;

#[test]
fn first_of_type_matches_first_li() {
    let doc = parse("<ul><li>a</li><li>b</li><li>c</li></ul>");
    let ul = &doc.root.children[0];
    let sel = parse_selector_list("li:first-of-type").unwrap();

    assert!(any_selector_matches(&sel, &ul.children[0], &child_ctx(0, 3), &[], Some(ul)).is_some());
    assert!(any_selector_matches(&sel, &ul.children[1], &child_ctx(1, 3), &[], Some(ul)).is_none());
}

#[test]
fn first_of_type_with_mixed_tags() {
    let doc = parse("<div><span>a</span><p>b</p><span>c</span><p>d</p></div>");
    let div = &doc.root.children[0];
    let sel = parse_selector_list("p:first-of-type").unwrap();

    assert!(any_selector_matches(&sel, &div.children[0], &child_ctx(0, 4), &[], Some(div)).is_none());
    assert!(any_selector_matches(&sel, &div.children[1], &child_ctx(1, 4), &[], Some(div)).is_some());
    assert!(any_selector_matches(&sel, &div.children[3], &child_ctx(3, 4), &[], Some(div)).is_none());
}

#[test]
fn last_of_type_matches_last_span() {
    let doc = parse("<div><span>a</span><p>b</p><span>c</span></div>");
    let div = &doc.root.children[0];
    let sel = parse_selector_list("span:last-of-type").unwrap();

    assert!(any_selector_matches(&sel, &div.children[0], &child_ctx(0, 3), &[], Some(div)).is_none());
    assert!(any_selector_matches(&sel, &div.children[2], &child_ctx(2, 3), &[], Some(div)).is_some());
}

#[test]
fn only_of_type_matches_sole_tag() {
    let doc = parse("<div><span>a</span><p>b</p><span>c</span></div>");
    let div = &doc.root.children[0];
    let sel_p = parse_selector_list("p:only-of-type").unwrap();
    let sel_span = parse_selector_list("span:only-of-type").unwrap();

    assert!(any_selector_matches(&sel_p, &div.children[1], &child_ctx(1, 3), &[], Some(div)).is_some());
    assert!(any_selector_matches(&sel_span, &div.children[0], &child_ctx(0, 3), &[], Some(div)).is_none());
}

#[test]
fn nth_of_type_2_matches_second_of_tag() {
    let doc = parse("<div><span>1</span><p>2</p><span>3</span><p>4</p><span>5</span></div>");
    let div = &doc.root.children[0];
    let sel = parse_selector_list("span:nth-of-type(2)").unwrap();

    assert!(any_selector_matches(&sel, &div.children[0], &child_ctx(0, 5), &[], Some(div)).is_none());
    assert!(any_selector_matches(&sel, &div.children[2], &child_ctx(2, 5), &[], Some(div)).is_some());
    assert!(any_selector_matches(&sel, &div.children[4], &child_ctx(4, 5), &[], Some(div)).is_none());
}

#[test]
fn nth_of_type_odd() {
    let doc = parse("<ul><li>1</li><li>2</li><li>3</li><li>4</li></ul>");
    let ul = &doc.root.children[0];
    let sel = parse_selector_list("li:nth-of-type(odd)").unwrap();

    assert!(any_selector_matches(&sel, &ul.children[0], &child_ctx(0, 4), &[], Some(ul)).is_some());
    assert!(any_selector_matches(&sel, &ul.children[1], &child_ctx(1, 4), &[], Some(ul)).is_none());
    assert!(any_selector_matches(&sel, &ul.children[2], &child_ctx(2, 4), &[], Some(ul)).is_some());
    assert!(any_selector_matches(&sel, &ul.children[3], &child_ctx(3, 4), &[], Some(ul)).is_none());
}

#[test]
fn nth_last_of_type_1_matches_last() {
    let doc = parse("<div><span>a</span><p>b</p><span>c</span></div>");
    let div = &doc.root.children[0];
    let sel = parse_selector_list("span:nth-last-of-type(1)").unwrap();

    assert!(any_selector_matches(&sel, &div.children[0], &child_ctx(0, 3), &[], Some(div)).is_none());
    assert!(any_selector_matches(&sel, &div.children[2], &child_ctx(2, 3), &[], Some(div)).is_some());
}
