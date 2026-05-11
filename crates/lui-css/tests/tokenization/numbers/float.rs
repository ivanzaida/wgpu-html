use lui_css::tokenizer::{tokenize, Token};

#[test]
fn tokenizes_leading_dot_float() {
    assert_eq!(tokenize(".5"), vec![Token::Number(0.5)]);
}
