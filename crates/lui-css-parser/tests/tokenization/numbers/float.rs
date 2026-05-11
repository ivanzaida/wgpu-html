use lui_css_parser::tokenizer::{tokenize, Token};
use lui_css_parser::ArcStr;

#[test]
fn tokenizes_leading_dot_float() {
    assert_eq!(tokenize(".5"), vec![Token::Number(0.5)]);
}