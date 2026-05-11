use lui_css::tokenizer::{Token, tokenize};

#[test]
fn tokenizes_custom_property_name_starting_with_double_dash() {
    assert_eq!(tokenize("--my-prop"), vec![Token::Ident("--my-prop".into())]);
}
