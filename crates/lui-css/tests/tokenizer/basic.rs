use lui_css::token::{Token, Tokenizer};

#[test]
fn tokenizes_ident_colon_ident_semicolon() {
  let tokens: Vec<_> = Tokenizer::new("color: red;").collect();
  assert_eq!(
    tokens,
    vec![
      Token::Ident("color"),
      Token::Colon,
      Token::Whitespace,
      Token::Ident("red"),
      Token::Semicolon,
    ]
  );
}

#[test]
fn tokenizes_at_keyword() {
  let tokens: Vec<_> = Tokenizer::new("@media screen").collect();
  assert_eq!(
    tokens,
    vec![Token::AtKeyword("media"), Token::Whitespace, Token::Ident("screen"),]
  );
}

#[test]
fn tokenizes_dimension_and_percentage() {
  let tokens: Vec<_> = Tokenizer::new("10px 50%").collect();
  assert_eq!(
    tokens,
    vec![Token::Dimension(10.0, "px"), Token::Whitespace, Token::Percentage(50.0),]
  );
}

#[test]
fn tokenizes_string_double_quoted() {
  let tokens: Vec<_> = Tokenizer::new(r#""hello world""#).collect();
  assert_eq!(tokens, vec![Token::String("hello world")]);
}

#[test]
fn tokenizes_string_single_quoted() {
  let tokens: Vec<_> = Tokenizer::new("'hello world'").collect();
  assert_eq!(tokens, vec![Token::String("hello world")]);
}

#[test]
fn tokenizes_hash() {
  let tokens: Vec<_> = Tokenizer::new("#ff0000").collect();
  assert_eq!(tokens, vec![Token::Hash("ff0000")]);
}

#[test]
fn tokenizes_negative_dimension() {
  let tokens: Vec<_> = Tokenizer::new("-10px").collect();
  assert_eq!(tokens, vec![Token::Dimension(-10.0, "px")]);
}

#[test]
fn tokenizes_positive_dimension_with_sign() {
  let tokens: Vec<_> = Tokenizer::new("+5em").collect();
  assert_eq!(tokens, vec![Token::Dimension(5.0, "em")]);
}

#[test]
fn tokenizes_decimal_number() {
  let tokens: Vec<_> = Tokenizer::new("0.75").collect();
  assert_eq!(tokens, vec![Token::Number(0.75)]);
}

#[test]
fn tokenizes_all_bracket_types() {
  let tokens: Vec<_> = Tokenizer::new("()[]{}").collect();
  assert_eq!(
    tokens,
    vec![
      Token::OpenParen,
      Token::CloseParen,
      Token::OpenBracket,
      Token::CloseBracket,
      Token::OpenBrace,
      Token::CloseBrace,
    ]
  );
}

#[test]
fn tokenizes_comma() {
  let tokens: Vec<_> = Tokenizer::new("a, b").collect();
  assert_eq!(
    tokens,
    vec![Token::Ident("a"), Token::Comma, Token::Whitespace, Token::Ident("b"),]
  );
}

#[test]
fn tokenizes_delimiters() {
  let tokens: Vec<_> = Tokenizer::new("*>~").collect();
  assert_eq!(tokens, vec![Token::Delim('*'), Token::Delim('>'), Token::Delim('~'),]);
}

#[test]
fn tokenizes_vendor_prefixed_ident() {
  let tokens: Vec<_> = Tokenizer::new("-webkit-transform").collect();
  assert_eq!(tokens, vec![Token::Ident("-webkit-transform")]);
}

#[test]
fn tokenizes_custom_property_name() {
  let tokens: Vec<_> = Tokenizer::new("--my-color").collect();
  assert_eq!(tokens, vec![Token::Ident("--my-color")]);
}

#[test]
fn tokenizes_zero_as_number() {
  let tokens: Vec<_> = Tokenizer::new("0").collect();
  assert_eq!(tokens, vec![Token::Number(0.0)]);
}
