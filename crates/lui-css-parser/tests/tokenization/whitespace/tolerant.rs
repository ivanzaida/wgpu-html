use lui_css_parser::tokenizer::{tokenize, Token};

#[test]
fn tokenizer_ignores_leading_and_interleaved_whitespace() {
    assert_eq!(tokenize("  acos (  1\t) "), vec![
        Token::Function("acos".into()), Token::Delim('('),
        Token::Number(1.0), Token::Delim(')'),
    ]);
}

#[test]
fn tokenizer_ignores_newlines_inside_function() {
    assert_eq!(tokenize("acos(\n1\n)"), vec![
        Token::Function("acos".into()), Token::Delim('('),
        Token::Number(1.0), Token::Delim(')'),
    ]);
}
