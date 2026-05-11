use lui_css::tokenizer::{Token, tokenize};

#[test]
fn tokenizes_empty_double_quoted_string() {
    assert_eq!(tokenize("\"\""), vec![Token::String(String::new())]);
}
