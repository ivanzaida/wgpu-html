use lui_css_parser::{parse_value, CssValue};
use lui_css_parser::ArcStr;

#[test]
fn url_quoted_is_typed() {
    assert_eq!(
        parse_value("url(\"bg.png\")").unwrap(),
        CssValue::Url(ArcStr::from("bg.png"))
    );
}

#[test]
fn url_single_quoted() {
    assert_eq!(
        parse_value("url('font.woff2')").unwrap(),
        CssValue::Url(ArcStr::from("font.woff2"))
    );
}

#[test]
fn url_unquoted() {
    assert_eq!(
        parse_value("url(bg.png)").unwrap(),
        CssValue::Url(ArcStr::from("bg.png"))
    );
}

#[test]
fn url_relative_path() {
    assert_eq!(
        parse_value("url(../images/bg.png)").unwrap(),
        CssValue::Url(ArcStr::from("../images/bg.png"))
    );
}

#[test]
fn url_data_uri() {
    assert_eq!(
        parse_value("url(data:image/png;base64,abc)").unwrap(),
        CssValue::Url(ArcStr::from("data:image/png;base64,abc"))
    );
}