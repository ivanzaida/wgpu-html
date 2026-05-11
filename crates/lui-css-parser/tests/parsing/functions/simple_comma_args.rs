use lui_css_parser::{parse_value, CssFunction, CssValue};
use lui_css_parser::ArcStr;

#[test]
fn parses_rgb_as_function() {
    assert_eq!(
        parse_value("rgb(255, 0, 128)").unwrap(),
        CssValue::Function {
            function: CssFunction::Rgb,
            args: vec![CssValue::Number(255.0), CssValue::Number(0.0), CssValue::Number(128.0)],
        }
    );
}

#[test]
fn parses_hsl_as_function() {
    assert_eq!(
        parse_value("hsl(180, 100%, 50%)").unwrap(),
        CssValue::Function {
            function: CssFunction::Hsl,
            args: vec![CssValue::Number(180.0), CssValue::Percentage(100.0), CssValue::Percentage(50.0)],
        }
    );
}

#[test]
fn parses_var_with_fallback() {
    assert_eq!(
        parse_value("var(--color, red)").unwrap(),
        CssValue::Var {
            name: ArcStr::from("--color"),
            fallback: Some(Box::new(CssValue::Color(
                lui_css_parser::CssColor::Named(ArcStr::from("red"))
            ))),
        }
    );
}

#[test]
fn parses_var_without_fallback() {
    assert_eq!(
        parse_value("var(--color)").unwrap(),
        CssValue::Var {
            name: ArcStr::from("--color"),
            fallback: None,
        }
    );
}

#[test]
fn var_with_nested_var_fallback() {
    assert_eq!(
        parse_value("var(--a, var(--b))").unwrap(),
        CssValue::Var {
            name: ArcStr::from("--a"),
            fallback: Some(Box::new(CssValue::Var {
                name: ArcStr::from("--b"),
                fallback: None,
            })),
        }
    );
}

#[test]
fn modern_rgb_space_syntax() {
    assert_eq!(
        parse_value("rgb(255 0 128)").unwrap(),
        CssValue::Function {
            function: CssFunction::Rgb,
            args: vec![CssValue::Number(255.0), CssValue::Number(0.0), CssValue::Number(128.0)],
        }
    );
}

#[test]
fn modern_rgb_slash_alpha() {
    assert_eq!(
        parse_value("rgb(255 0 128 / 0.5)").unwrap(),
        CssValue::Function {
            function: CssFunction::Rgb,
            args: vec![CssValue::Number(255.0), CssValue::Number(0.0), CssValue::Number(128.0), CssValue::Number(0.5)],
        }
    );
}