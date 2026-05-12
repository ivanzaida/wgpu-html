use crate::helpers::*;

#[test]
fn matches_div() {
    assert_matches("div", "<div></div>");
}

#[test]
fn matches_p() {
    assert_matches("p", "<p></p>");
}

#[test]
fn matches_span() {
    assert_matches("span", "<span></span>");
}

#[test]
fn rejects_wrong_tag() {
    assert_no_match("span", "<div></div>");
}

#[test]
fn matches_case_insensitive_tag() {
    assert_matches("div", "<DIV></DIV>");
}

#[test]
fn universal_matches_any_tag() {
    assert_matches("*", "<p></p>");
    assert_matches("*", "<div></div>");
    assert_matches("*", "<section></section>");
}
