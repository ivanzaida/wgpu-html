use lui_css::tokenizer::{Token, tokenize};

#[test]
fn tokenizes_dot_prefix_as_float_number() {
    assert_eq!(tokenize(".5"), vec![Token::Number(0.5)]);
}
