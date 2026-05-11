use lui_css_parser::tokenizer::{tokenize, Token};
use lui_css_parser::ArcStr;

#[test]
fn tokenizes_bare_function_with_number_arg() {
    assert_eq!(tokenize("acos(1)"), vec![
        Token::Function("acos".into()), Token::Delim('('),
        Token::Number(1.0), Token::Delim(')'),
    ]);
}