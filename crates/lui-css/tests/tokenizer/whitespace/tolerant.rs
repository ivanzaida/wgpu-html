use lui_css::tokenizer::{Token, tokenize};

#[test]
fn ignores_leading_trailing_and_interstitial_whitespace() {
    assert_eq!(
        tokenize("  acos (  1\t) "),
        vec![
            Token::Function("acos".into()),
            Token::Delim('('),
            Token::Number(1.0),
            Token::Delim(')'),
        ]
    );
}
