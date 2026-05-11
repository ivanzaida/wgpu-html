use lui_css::tokenizer::{tokenize, Token};

#[test]
fn tokenizes_positive_integer() {
    assert_eq!(tokenize("42"), vec![Token::Number(42.0)]);
}

#[test]
fn tokenizes_positive_float() {
    assert_eq!(tokenize("3.14"), vec![Token::Number(3.14)]);
}

#[test]
fn tokenizes_explicit_positive_sign() {
    assert_eq!(tokenize("+42"), vec![Token::Number(42.0)]);
}
