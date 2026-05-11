use lui_css::tokenizer::{Token, tokenize};

#[test]
fn tokenizes_negative_percentage() {
    assert_eq!(tokenize("-20%"), vec![Token::Percentage(-20.0)]);
}
