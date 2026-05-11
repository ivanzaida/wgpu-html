use lui_css_parser::tokenizer::{tokenize, Token};

#[test]
fn tokenizes_single_quoted_string() {
    assert_eq!(tokenize("'foo bar'"), vec![Token::String("foo bar".into())]);
}
