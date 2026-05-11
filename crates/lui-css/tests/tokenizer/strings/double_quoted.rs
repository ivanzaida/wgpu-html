use lui_css::tokenizer::{Token, tokenize};

#[test]
fn tokenizes_double_quoted_string() {
    assert_eq!(
        tokenize("\"hello world\""),
        vec![Token::String("hello world".into())]
    );
}
