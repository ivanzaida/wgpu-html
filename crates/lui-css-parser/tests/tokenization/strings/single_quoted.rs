use lui_css_parser::tokenizer::{tokenize, Token};
use lui_css_parser::ArcStr;

#[test]
fn tokenizes_single_quoted_string() {
    assert_eq!(tokenize("'foo bar'"), vec![Token::String("foo bar".into())]);
}