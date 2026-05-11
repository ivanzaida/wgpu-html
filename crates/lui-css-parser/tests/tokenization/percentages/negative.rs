use lui_css_parser::tokenizer::{tokenize, Token};

#[test]
fn tokenizes_negative_percentage() {
    assert_eq!(tokenize("-20%"), vec![Token::Percentage(-20.0)]);
}
