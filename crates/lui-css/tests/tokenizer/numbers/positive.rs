use lui_css::tokenizer::{Token, tokenize};

#[test]
fn tokenizes_positive_number_with_explicit_plus_sign() {
    assert_eq!(tokenize("+42"), vec![Token::Number(42.0)]);
}

#[test]
fn tokenizes_plain_integer_as_number_not_ident() {
    assert_eq!(tokenize("42"), vec![Token::Number(42.0)]);
}
