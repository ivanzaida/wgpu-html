use lui_css_parser::tokenizer::{tokenize, Token};
use lui_css_parser::ArcStr;

#[test]
fn tokenizes_empty_quoted_string() {
    assert_eq!(tokenize("\"\""), vec![Token::String(String::new())]);
}