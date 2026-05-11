use lui_css::tokenizer::{tokenize, Token};

#[test]
fn tokenizes_empty_quoted_string() {
    assert_eq!(tokenize("\"\""), vec![Token::String(String::new())]);
}
