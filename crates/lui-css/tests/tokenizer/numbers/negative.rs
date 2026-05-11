use lui_css::tokenizer::{Token, tokenize};

#[test]
fn tokenizes_negative_number() {
    assert_eq!(tokenize("-3.14"), vec![Token::Number(-3.14)]);
}
