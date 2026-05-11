use lui_css_parser::tokenizer::{tokenize, Token};
use lui_css_parser::ArcStr;

#[test]
fn tokenizes_nested_function_calls_with_quoted_strings() {
    assert_eq!(tokenize("-webkit-image-set(url(\"a\"))"), vec![
        Token::Function("-webkit-image-set".into()), Token::Delim('('),
        Token::Function("url".into()), Token::Delim('('),
        Token::String("a".into()), Token::Delim(')'), Token::Delim(')'),
    ]);
}