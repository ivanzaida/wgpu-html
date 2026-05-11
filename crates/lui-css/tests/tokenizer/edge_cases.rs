use lui_css::token::{Token, Tokenizer};

#[test]
fn empty_input_yields_no_tokens() {
  let tokens: Vec<_> = Tokenizer::new("").collect();
  assert!(tokens.is_empty());
}

#[test]
fn whitespace_only_input() {
  let tokens: Vec<_> = Tokenizer::new("   \n\t  ").collect();
  assert_eq!(tokens, vec![Token::Whitespace]);
}

#[test]
fn escaped_quote_in_string() {
  let tokens: Vec<_> = Tokenizer::new(r#""hello \"world\"""#).collect();
  assert_eq!(tokens.len(), 1);
  match &tokens[0] {
    Token::String(s) => assert!(s.contains("hello")),
    _ => panic!("expected string token"),
  }
}

#[test]
fn scientific_notation() {
  let tokens: Vec<_> = Tokenizer::new("1e3").collect();
  assert_eq!(tokens, vec![Token::Number(1000.0)]);
}

#[test]
fn scientific_notation_with_sign() {
  let tokens: Vec<_> = Tokenizer::new("2.5e+2").collect();
  assert_eq!(tokens, vec![Token::Number(250.0)]);
}

#[test]
fn leading_dot_number() {
  let tokens: Vec<_> = Tokenizer::new(".5px").collect();
  assert_eq!(tokens, vec![Token::Dimension(0.5, "px")]);
}

#[test]
fn hash_with_short_hex() {
  let tokens: Vec<_> = Tokenizer::new("#fff").collect();
  assert_eq!(tokens, vec![Token::Hash("fff")]);
}

#[test]
fn consecutive_semicolons() {
  let tokens: Vec<_> = Tokenizer::new(";;").collect();
  assert_eq!(tokens, vec![Token::Semicolon, Token::Semicolon]);
}

#[test]
fn at_keyword_with_hyphen() {
  let tokens: Vec<_> = Tokenizer::new("@font-face").collect();
  assert_eq!(tokens, vec![Token::AtKeyword("font-face")]);
}

#[test]
fn percentage_zero() {
  let tokens: Vec<_> = Tokenizer::new("0%").collect();
  assert_eq!(tokens, vec![Token::Percentage(0.0)]);
}

#[test]
fn multiple_units_in_sequence() {
  let tokens: Vec<_> = Tokenizer::new("10px 20em 30% 40vh").collect();
  let non_ws: Vec<_> = tokens.into_iter().filter(|t| !matches!(t, Token::Whitespace)).collect();
  assert_eq!(
    non_ws,
    vec![
      Token::Dimension(10.0, "px"),
      Token::Dimension(20.0, "em"),
      Token::Percentage(30.0),
      Token::Dimension(40.0, "vh"),
    ]
  );
}

#[test]
fn bang_is_delim() {
  let tokens: Vec<_> = Tokenizer::new("!important").collect();
  assert_eq!(tokens[0], Token::Delim('!'));
  assert_eq!(tokens[1], Token::Ident("important"));
}

#[test]
fn full_declaration_round_trip() {
  let input = "background: linear-gradient(to right, #ff0000, rgba(0, 0, 255, 0.5)) no-repeat center;";
  let tokens: Vec<_> = Tokenizer::new(input).collect();
  assert!(tokens.len() > 10);
  assert_eq!(tokens[0], Token::Ident("background"));
  assert_eq!(tokens[1], Token::Colon);
  assert_eq!(*tokens.last().unwrap(), Token::Semicolon);
}

#[test]
fn negative_percentage() {
  let tokens: Vec<_> = Tokenizer::new("-50%").collect();
  assert_eq!(tokens, vec![Token::Percentage(-50.0)]);
}
