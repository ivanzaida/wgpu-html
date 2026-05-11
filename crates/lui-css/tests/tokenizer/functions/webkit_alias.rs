use lui_css::tokenizer::{Token, tokenize};

#[test]
fn tokenizes_webkit_alias_function_with_nested_call() {
    assert_eq!(
        tokenize("-webkit-image-set(url(\"a\"))"),
        vec![
            Token::Function("-webkit-image-set".into()),
            Token::Delim('('),
            Token::Function("url".into()),
            Token::Delim('('),
            Token::String("a".into()),
            Token::Delim(')'),
            Token::Delim(')'),
        ]
    );
}
