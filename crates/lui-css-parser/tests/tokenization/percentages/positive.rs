use lui_css_parser::tokenizer::{tokenize, Token};

#[test]
fn tokenizes_positive_percentage() {
    assert_eq!(tokenize("50%"), vec![Token::Percentage(50.0)]);
}
