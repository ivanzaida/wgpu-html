use lui_css_parser::{parse_value, CssValue};

#[test]
fn url_quoted_is_typed() {
    assert_eq!(
        parse_value("url(\"bg.png\")").unwrap(),
        CssValue::Url("bg.png".into())
    );
}

#[test]
fn url_single_quoted() {
    assert_eq!(
        parse_value("url('font.woff2')").unwrap(),
        CssValue::Url("font.woff2".into())
    );
}

#[test]
fn url_unquoted() {
    assert_eq!(
        parse_value("url(bg.png)").unwrap(),
        CssValue::Url("bg.png".into())
    );
}

#[test]
fn url_relative_path() {
    assert_eq!(
        parse_value("url(../images/bg.png)").unwrap(),
        CssValue::Url("../images/bg.png".into())
    );
}

#[test]
fn url_data_uri() {
    assert_eq!(
        parse_value("url(data:image/png;base64,abc)").unwrap(),
        CssValue::Url("data:image/png;base64,abc".into())
    );
}
