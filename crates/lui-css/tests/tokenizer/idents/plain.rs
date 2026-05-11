use lui_css::tokenizer::{Token, tokenize};

#[test]
fn tokenizes_plain_identifier() {
    assert_eq!(tokenize("auto"), vec![Token::Ident("auto".into())]);
}
