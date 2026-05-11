use lui_css_parser::tokenizer::{tokenize, Token};
use lui_css_parser::ArcStr;

#[test]
fn tokenizes_double_quoted_string() {
    assert_eq!(tokenize("\"hello world\""), vec![Token::String("hello world".into())]);
}