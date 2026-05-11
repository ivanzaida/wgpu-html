use lui_css::tokenizer::{Token, tokenize};

#[test]
fn tokenizes_positive_percentage() {
    assert_eq!(tokenize("50%"), vec![Token::Percentage(50.0)]);
}
