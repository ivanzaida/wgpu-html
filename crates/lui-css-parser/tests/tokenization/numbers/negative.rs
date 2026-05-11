use lui_css_parser::tokenizer::{tokenize, Token};
use lui_css_parser::ArcStr;

#[test]
fn tokenizes_negative_integer() {
    assert_eq!(tokenize("-5"), vec![Token::Number(-5.0)]);
}

#[test]
fn tokenizes_negative_float() {
    assert_eq!(tokenize("-3.14"), vec![Token::Number(-3.14)]);
}