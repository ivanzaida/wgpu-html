use lui_css_parser::tokenizer::{tokenize, Token};

#[test]
fn tokenizes_plain_identifier() {
    assert_eq!(tokenize("auto"), vec![Token::Ident("auto".into())]);
}

#[test]
fn tokenizes_color_name_as_identifier() {
    assert_eq!(tokenize("red"), vec![Token::Ident("red".into())]);
}
