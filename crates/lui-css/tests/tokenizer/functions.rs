use lui_css::token::{Token, Tokenizer};

#[test]
fn tokenizes_rgb_function() {
  let tokens: Vec<_> = Tokenizer::new("rgb(1, 2, 3)").collect();
  assert_eq!(
    tokens,
    vec![
      Token::Function("rgb"),
      Token::Number(1.0),
      Token::Comma,
      Token::Whitespace,
      Token::Number(2.0),
      Token::Comma,
      Token::Whitespace,
      Token::Number(3.0),
      Token::CloseParen,
    ]
  );
}

#[test]
fn tokenizes_calc_with_nested_parens() {
  let tokens: Vec<_> = Tokenizer::new("calc(100% - 20px)").collect();
  assert_eq!(
    tokens,
    vec![
      Token::Function("calc"),
      Token::Percentage(100.0),
      Token::Whitespace,
      Token::Delim('-'),
      Token::Whitespace,
      Token::Dimension(20.0, "px"),
      Token::CloseParen,
    ]
  );
}

#[test]
fn tokenizes_url_function_with_string() {
  let tokens: Vec<_> = Tokenizer::new("url(\"image.png\")").collect();
  assert_eq!(
    tokens,
    vec![Token::Function("url"), Token::String("image.png"), Token::CloseParen,]
  );
}

#[test]
fn tokenizes_linear_gradient() {
  let tokens: Vec<_> = Tokenizer::new("linear-gradient(to right, red, blue)").collect();
  assert_eq!(tokens[0], Token::Function("linear-gradient"));
  assert_eq!(tokens[1], Token::Ident("to"));
  assert!(tokens.last() == Some(&Token::CloseParen));
}

#[test]
fn tokenizes_var_function() {
  let tokens: Vec<_> = Tokenizer::new("var(--main-color, red)").collect();
  assert_eq!(tokens[0], Token::Function("var"));
  assert_eq!(tokens[1], Token::Ident("--main-color"));
}

#[test]
fn tokenizes_nested_functions() {
  let tokens: Vec<_> = Tokenizer::new("calc(min(50%, 200px) + 10px)").collect();
  assert_eq!(tokens[0], Token::Function("calc"));
  assert_eq!(tokens[1], Token::Function("min"));
}

#[test]
fn tokenizes_rgba_with_modern_syntax() {
  let tokens: Vec<_> = Tokenizer::new("rgb(255 128 0 / 0.5)").collect();
  assert_eq!(tokens[0], Token::Function("rgb"));
  assert!(tokens.contains(&Token::Delim('/')));
}

#[test]
fn tokenizes_cubic_bezier() {
  let tokens: Vec<_> = Tokenizer::new("cubic-bezier(0.25, 0.1, 0.25, 1)").collect();
  assert_eq!(tokens[0], Token::Function("cubic-bezier"));
}
