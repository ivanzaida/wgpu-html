use lui_css_old::token::{Token, Tokenizer};

#[test]
fn skips_block_comments() {
  let tokens: Vec<_> = Tokenizer::new("a /* comment */ b").collect();
  assert_eq!(
    tokens,
    vec![
      Token::Ident("a"),
      Token::Whitespace,
      Token::Whitespace,
      Token::Ident("b"),
    ]
  );
}

#[test]
fn skips_comment_at_start() {
  let tokens: Vec<_> = Tokenizer::new("/* header */ div").collect();
  assert_eq!(tokens, vec![Token::Whitespace, Token::Ident("div")]);
}

#[test]
fn skips_comment_at_end() {
  let tokens: Vec<_> = Tokenizer::new("div /* end */").collect();
  assert_eq!(tokens, vec![Token::Ident("div"), Token::Whitespace]);
}

#[test]
fn skips_multiple_comments() {
  let tokens: Vec<_> = Tokenizer::new("/* a */ b /* c */ d /* e */").collect();
  let idents: Vec<_> = tokens.iter().filter(|t| matches!(t, Token::Ident(_))).collect();
  assert_eq!(idents.len(), 2);
}

#[test]
fn handles_unclosed_comment() {
  let tokens: Vec<_> = Tokenizer::new("a /* never closed").collect();
  assert_eq!(tokens, vec![Token::Ident("a"), Token::Whitespace]);
}

#[test]
fn comment_inside_string_is_not_stripped() {
  let tokens: Vec<_> = Tokenizer::new(r#""/* not a comment */""#).collect();
  assert_eq!(tokens, vec![Token::String("/* not a comment */")]);
}

#[test]
fn adjacent_comments() {
  let tokens: Vec<_> = Tokenizer::new("/* a *//* b */x").collect();
  let idents: Vec<_> = tokens.iter().filter(|t| matches!(t, Token::Ident(_))).collect();
  assert_eq!(idents.len(), 1);
}
