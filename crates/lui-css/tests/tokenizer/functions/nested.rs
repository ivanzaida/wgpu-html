use lui_css::tokenizer::{Token, tokenize};

#[test]
fn tokenizes_function_with_nested_parens_and_operators() {
    assert_eq!(
        tokenize("calc(100% - 20px)"),
        vec![
            Token::Function("calc".into()),
            Token::Delim('('),
            Token::Percentage(100.0),
            Token::Delim('-'),
            Token::Dimension { value: 20.0, unit: "px".into() },
            Token::Delim(')'),
        ]
    );
}
