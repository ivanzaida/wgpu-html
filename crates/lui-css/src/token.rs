use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
  Ident(&'a str),
  Function(&'a str),
  AtKeyword(&'a str),
  Hash(&'a str),
  String(&'a str),
  Number(f64),
  Percentage(f64),
  Dimension(f64, &'a str),
  Delim(char),
  Whitespace,
  Colon,
  Semicolon,
  Comma,
  OpenParen,
  CloseParen,
  OpenBracket,
  CloseBracket,
  OpenBrace,
  CloseBrace,
}

impl fmt::Display for Token<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Token::Ident(s) => f.write_str(s),
      Token::Function(s) => write!(f, "{s}("),
      Token::AtKeyword(s) => write!(f, "@{s}"),
      Token::Hash(s) => write!(f, "#{s}"),
      Token::String(s) => write!(f, "\"{s}\""),
      Token::Number(n) => write!(f, "{n}"),
      Token::Percentage(n) => write!(f, "{n}%"),
      Token::Dimension(n, u) => write!(f, "{n}{u}"),
      Token::Delim(c) => write!(f, "{c}"),
      Token::Whitespace => f.write_str(" "),
      Token::Colon => f.write_str(":"),
      Token::Semicolon => f.write_str(";"),
      Token::Comma => f.write_str(","),
      Token::OpenParen => f.write_str("("),
      Token::CloseParen => f.write_str(")"),
      Token::OpenBracket => f.write_str("["),
      Token::CloseBracket => f.write_str("]"),
      Token::OpenBrace => f.write_str("{"),
      Token::CloseBrace => f.write_str("}"),
    }
  }
}

pub struct Tokenizer<'a> {
  input: &'a str,
  pos: usize,
}

impl<'a> Tokenizer<'a> {
  pub fn new(input: &'a str) -> Self {
    Self { input, pos: 0 }
  }

  pub fn position(&self) -> usize {
    self.pos
  }

  fn remaining(&self) -> &'a str {
    &self.input[self.pos..]
  }

  fn peek_byte(&self) -> Option<u8> {
    self.input.as_bytes().get(self.pos).copied()
  }

  fn advance(&mut self, n: usize) {
    self.pos = (self.pos + n).min(self.input.len());
  }

  fn consume_while(&mut self, pred: impl Fn(u8) -> bool) -> &'a str {
    let start = self.pos;
    let bytes = self.input.as_bytes();
    while self.pos < bytes.len() && pred(bytes[self.pos]) {
      self.pos += 1;
    }
    &self.input[start..self.pos]
  }

  fn consume_whitespace(&mut self) -> Token<'a> {
    self.consume_while(|b| b.is_ascii_whitespace());
    Token::Whitespace
  }

  fn consume_string(&mut self, quote: u8) -> Token<'a> {
    self.advance(1);
    let start = self.pos;
    let bytes = self.input.as_bytes();
    while self.pos < bytes.len() {
      if bytes[self.pos] == b'\\' && self.pos + 1 < bytes.len() {
        self.pos += 2;
      } else if bytes[self.pos] == quote {
        let s = &self.input[start..self.pos];
        self.advance(1);
        return Token::String(s);
      } else {
        self.pos += 1;
      }
    }
    Token::String(&self.input[start..self.pos])
  }

  fn consume_numeric(&mut self) -> Token<'a> {
    let start = self.pos;
    let bytes = self.input.as_bytes();
    if matches!(self.peek_byte(), Some(b'+') | Some(b'-')) {
      self.pos += 1;
    }
    self.consume_while(|b| b.is_ascii_digit());
    if self.pos < bytes.len() && bytes[self.pos] == b'.' {
      let next = bytes.get(self.pos + 1);
      if next.is_some_and(|b| b.is_ascii_digit()) {
        self.pos += 1;
        self.consume_while(|b| b.is_ascii_digit());
      }
    }
    if matches!(self.peek_byte(), Some(b'e') | Some(b'E')) {
      let saved = self.pos;
      self.pos += 1;
      if matches!(self.peek_byte(), Some(b'+') | Some(b'-')) {
        self.pos += 1;
      }
      if self.peek_byte().is_some_and(|b| b.is_ascii_digit()) {
        self.consume_while(|b| b.is_ascii_digit());
      } else {
        self.pos = saved;
      }
    }
    let num_str = &self.input[start..self.pos];
    let value: f64 = num_str.parse().unwrap_or(0.0);

    if self.peek_byte() == Some(b'%') {
      self.advance(1);
      return Token::Percentage(value);
    }
    let unit_start = self.pos;
    if self.peek_byte().is_some_and(|b| is_name_start(b)) {
      self.consume_while(|b| is_name_char(b));
      let unit = &self.input[unit_start..self.pos];
      return Token::Dimension(value, unit);
    }
    Token::Number(value)
  }

  fn consume_ident_like(&mut self) -> Token<'a> {
    let start = self.pos;
    self.consume_while(|b| is_name_char(b));
    let name = &self.input[start..self.pos];
    if self.peek_byte() == Some(b'(') {
      self.advance(1);
      Token::Function(name)
    } else {
      Token::Ident(name)
    }
  }

  fn consume_hash(&mut self) -> Token<'a> {
    self.advance(1);
    let start = self.pos;
    self.consume_while(|b| is_name_char(b));
    Token::Hash(&self.input[start..self.pos])
  }

  fn skip_comment(&mut self) -> bool {
    if self.remaining().starts_with("/*") {
      self.advance(2);
      if let Some(end) = self.remaining().find("*/") {
        self.advance(end + 2);
      } else {
        self.pos = self.input.len();
      }
      true
    } else {
      false
    }
  }
}

impl<'a> Iterator for Tokenizer<'a> {
  type Item = Token<'a>;

  fn next(&mut self) -> Option<Token<'a>> {
    while self.skip_comment() {}

    let b = self.peek_byte()?;
    Some(match b {
      b if b.is_ascii_whitespace() => self.consume_whitespace(),
      b'"' => self.consume_string(b'"'),
      b'\'' => self.consume_string(b'\''),
      b'#' => self.consume_hash(),
      b'(' => {
        self.advance(1);
        Token::OpenParen
      }
      b')' => {
        self.advance(1);
        Token::CloseParen
      }
      b'[' => {
        self.advance(1);
        Token::OpenBracket
      }
      b']' => {
        self.advance(1);
        Token::CloseBracket
      }
      b'{' => {
        self.advance(1);
        Token::OpenBrace
      }
      b'}' => {
        self.advance(1);
        Token::CloseBrace
      }
      b':' => {
        self.advance(1);
        Token::Colon
      }
      b';' => {
        self.advance(1);
        Token::Semicolon
      }
      b',' => {
        self.advance(1);
        Token::Comma
      }
      b'@' => {
        self.advance(1);
        let start = self.pos;
        self.consume_while(|b| is_name_char(b));
        Token::AtKeyword(&self.input[start..self.pos])
      }
      b'+' | b'-' => {
        let next = self.input.as_bytes().get(self.pos + 1);
        if next.is_some_and(|n| n.is_ascii_digit())
          || (next == Some(&b'.')
            && self
              .input
              .as_bytes()
              .get(self.pos + 2)
              .is_some_and(|n| n.is_ascii_digit()))
        {
          self.consume_numeric()
        } else if b == b'-' && next.is_some_and(|n| is_name_start(*n) || *n == b'-') {
          self.consume_ident_like()
        } else {
          self.advance(1);
          Token::Delim(b as char)
        }
      }
      b'.' => {
        let next = self.input.as_bytes().get(self.pos + 1);
        if next.is_some_and(|n| n.is_ascii_digit()) {
          self.consume_numeric()
        } else {
          self.advance(1);
          Token::Delim('.')
        }
      }
      b'0'..=b'9' => self.consume_numeric(),
      b if is_name_start(b) => self.consume_ident_like(),
      _ => {
        self.advance(1);
        Token::Delim(b as char)
      }
    })
  }
}

fn is_name_start(b: u8) -> bool {
  b.is_ascii_alphabetic() || b == b'_' || b > 0x7F
}

fn is_name_char(b: u8) -> bool {
  is_name_start(b) || b.is_ascii_digit() || b == b'-'
}
