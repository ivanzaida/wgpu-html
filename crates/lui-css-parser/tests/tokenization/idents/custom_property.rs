use lui_css_parser::tokenizer::{tokenize, Token};
use lui_css_parser::ArcStr;

#[test]
fn tokenizes_custom_property_starting_with_double_dash() {
    assert_eq!(tokenize("--my-prop"), vec![Token::Ident("--my-prop".into())]);
}

#[test]
fn tokenizes_nested_custom_property() {
    assert_eq!(tokenize("--color-primary-dark"), vec![Token::Ident("--color-primary-dark".into())]);
}