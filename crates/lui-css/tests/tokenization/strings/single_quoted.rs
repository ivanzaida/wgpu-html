use lui_css::tokenizer::{tokenize, Token};

#[test]
fn tokenizes_single_quoted_string() {
    assert_eq!(tokenize("'foo bar'"), vec![Token::String("foo bar".into())]);
}
