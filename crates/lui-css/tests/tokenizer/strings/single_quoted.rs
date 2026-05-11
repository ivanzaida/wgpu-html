use lui_css::tokenizer::{Token, tokenize};

#[test]
fn tokenizes_single_quoted_string() {
    assert_eq!(
        tokenize("'foo bar'"),
        vec![Token::String("foo bar".into())]
    );
}
