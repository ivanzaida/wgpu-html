/// A token produced by the HTML tokenizer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  Doctype(String),
  OpenTag {
    name: String,
    attrs: Vec<(String, String)>,
    self_closing: bool,
  },
  CloseTag(String),
  Text(String),
  Comment(String),
}

/// Tokenizes an HTML string into a sequence of tokens.
pub fn tokenize(input: &str) -> Vec<Token> {
  let mut tokenizer = Tokenizer::new(input);
  tokenizer.run();
  tokenizer.tokens
}

struct Tokenizer {
  chars: Vec<char>,
  pos: usize,
  tokens: Vec<Token>,
}

impl Tokenizer {
  fn new(input: &str) -> Self {
    Self {
      chars: input.chars().collect(),
      pos: 0,
      tokens: Vec::new(),
    }
  }

  fn run(&mut self) {
    while self.pos < self.chars.len() {
      if self.current() == '<' {
        self.consume_tag();
      } else {
        self.consume_text();
      }
    }
  }

  fn current(&self) -> char {
    self.chars[self.pos]
  }

  fn peek(&self, offset: usize) -> Option<char> {
    self.chars.get(self.pos + offset).copied()
  }

  fn remaining(&self) -> usize {
    self.chars.len() - self.pos
  }

  fn starts_with(&self, s: &str) -> bool {
    let s_chars: Vec<char> = s.chars().collect();
    if self.remaining() < s_chars.len() {
      return false;
    }
    for (i, &c) in s_chars.iter().enumerate() {
      if self.chars[self.pos + i] != c {
        return false;
      }
    }
    true
  }

  fn starts_with_ci(&self, s: &str) -> bool {
    let s_chars: Vec<char> = s.chars().collect();
    if self.remaining() < s_chars.len() {
      return false;
    }
    for (i, &c) in s_chars.iter().enumerate() {
      if self.chars[self.pos + i].to_ascii_lowercase() != c.to_ascii_lowercase() {
        return false;
      }
    }
    true
  }

  fn consume_text(&mut self) {
    let start = self.pos;
    while self.pos < self.chars.len() && self.chars[self.pos] != '<' {
      self.pos += 1;
    }
    let text: String = self.chars[start..self.pos].iter().collect();
    if !text.is_empty() {
      self.tokens.push(Token::Text(decode_entities(&text)));
    }
  }

  fn consume_tag(&mut self) {
    // We are at '<'
    if self.starts_with("<!--") {
      self.consume_comment();
    } else if self.starts_with_ci("<!doctype") {
      self.consume_doctype();
    } else if self.starts_with("</") {
      self.consume_close_tag();
    } else if self.starts_with("<") && self.peek(1).map_or(false, |c| c.is_ascii_alphabetic()) {
      self.consume_open_tag();
    } else {
      // Not a valid tag start, treat '<' as text
      self.pos += 1;
      self.tokens.push(Token::Text("<".into()));
    }
  }

  fn consume_comment(&mut self) {
    // Skip '<!--'
    self.pos += 4;
    let start = self.pos;
    loop {
      if self.pos >= self.chars.len() {
        break;
      }
      if self.starts_with("-->") {
        let comment: String = self.chars[start..self.pos].iter().collect();
        self.pos += 3;
        self.tokens.push(Token::Comment(comment));
        return;
      }
      self.pos += 1;
    }
    // Unterminated comment - emit what we have
    let comment: String = self.chars[start..self.pos].iter().collect();
    self.tokens.push(Token::Comment(comment));
  }

  fn consume_doctype(&mut self) {
    // Skip '<!'
    self.pos += 2;
    let start = self.pos;
    while self.pos < self.chars.len() && self.chars[self.pos] != '>' {
      self.pos += 1;
    }
    let content: String = self.chars[start..self.pos].iter().collect();
    if self.pos < self.chars.len() {
      self.pos += 1; // skip '>'
    }
    self.tokens.push(Token::Doctype(content));
  }

  fn consume_close_tag(&mut self) {
    // Skip '</'
    self.pos += 2;
    self.skip_whitespace();
    let name = self.consume_tag_name();
    self.skip_whitespace();
    if self.pos < self.chars.len() && self.chars[self.pos] == '>' {
      self.pos += 1;
    }
    if !name.is_empty() {
      self.tokens.push(Token::CloseTag(name));
    }
  }

  fn consume_open_tag(&mut self) {
    // Skip '<'
    self.pos += 1;
    let name = self.consume_tag_name();
    let mut attrs = Vec::new();
    let mut self_closing = false;

    loop {
      self.skip_whitespace();
      if self.pos >= self.chars.len() {
        break;
      }
      if self.chars[self.pos] == '>' {
        self.pos += 1;
        break;
      }
      if self.chars[self.pos] == '/' && self.peek(1) == Some('>') {
        self_closing = true;
        self.pos += 2;
        break;
      }
      // Try to consume an attribute
      if let Some(attr) = self.consume_attribute() {
        attrs.push(attr);
      } else {
        // Skip unknown char to avoid infinite loop
        self.pos += 1;
      }
    }

    // Check for raw-text elements: script, style, textarea, title
    let lower_name = name.to_ascii_lowercase();
    if !self_closing && matches!(lower_name.as_str(), "script" | "style" | "textarea" | "title") {
      self.tokens.push(Token::OpenTag {
        name: name.clone(),
        attrs,
        self_closing: false,
      });
      self.consume_raw_text(&lower_name);
      return;
    }

    self.tokens.push(Token::OpenTag {
      name,
      attrs,
      self_closing,
    });
  }

  fn consume_raw_text(&mut self, tag_name: &str) {
    let start = self.pos;
    let close_tag = format!("</{}", tag_name);
    loop {
      if self.pos >= self.chars.len() {
        break;
      }
      if self.starts_with_ci(&close_tag) {
        let text: String = self.chars[start..self.pos].iter().collect();
        if !text.is_empty() {
          self.tokens.push(Token::Text(text));
        }
        self.consume_close_tag();
        return;
      }
      self.pos += 1;
    }
    // Unterminated raw text element
    let text: String = self.chars[start..self.pos].iter().collect();
    if !text.is_empty() {
      self.tokens.push(Token::Text(text));
    }
  }

  fn consume_tag_name(&mut self) -> String {
    let start = self.pos;
    while self.pos < self.chars.len() {
      let c = self.chars[self.pos];
      if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':' {
        self.pos += 1;
      } else {
        break;
      }
    }
    let name: String = self.chars[start..self.pos].iter().collect();
    name.to_ascii_lowercase()
  }

  fn consume_attribute(&mut self) -> Option<(String, String)> {
    let name = self.consume_attr_name();
    if name.is_empty() {
      return None;
    }
    self.skip_whitespace();
    if self.pos < self.chars.len() && self.chars[self.pos] == '=' {
      self.pos += 1; // skip '='
      self.skip_whitespace();
      let value = self.consume_attr_value();
      Some((name, value))
    } else {
      // Boolean attribute (no value)
      Some((name, String::new()))
    }
  }

  fn consume_attr_name(&mut self) -> String {
    let start = self.pos;
    while self.pos < self.chars.len() {
      let c = self.chars[self.pos];
      if c == '=' || c == '>' || c == '/' || c.is_ascii_whitespace() {
        break;
      }
      self.pos += 1;
    }
    let name: String = self.chars[start..self.pos].iter().collect();
    name.to_ascii_lowercase()
  }

  fn consume_attr_value(&mut self) -> String {
    if self.pos >= self.chars.len() {
      return String::new();
    }
    let quote = self.chars[self.pos];
    if quote == '"' || quote == '\'' {
      self.pos += 1; // skip opening quote
      let start = self.pos;
      while self.pos < self.chars.len() && self.chars[self.pos] != quote {
        self.pos += 1;
      }
      let value: String = self.chars[start..self.pos].iter().collect();
      if self.pos < self.chars.len() {
        self.pos += 1; // skip closing quote
      }
      decode_entities(&value)
    } else {
      // Unquoted value
      let start = self.pos;
      while self.pos < self.chars.len() {
        let c = self.chars[self.pos];
        if c.is_ascii_whitespace() || c == '>' || c == '/' {
          break;
        }
        self.pos += 1;
      }
      let value: String = self.chars[start..self.pos].iter().collect();
      decode_entities(&value)
    }
  }

  fn skip_whitespace(&mut self) {
    while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_whitespace() {
      self.pos += 1;
    }
  }
}

/// Decode common HTML entities.
fn decode_entities(input: &str) -> String {
  if !input.contains('&') {
    return input.to_string();
  }
  let mut result = String::with_capacity(input.len());
  let chars: Vec<char> = input.chars().collect();
  let mut i = 0;
  while i < chars.len() {
    if chars[i] == '&' {
      let start = i;
      i += 1;
      let mut entity = String::new();
      while i < chars.len() && chars[i] != ';' && entity.len() < 10 {
        entity.push(chars[i]);
        i += 1;
      }
      if i < chars.len() && chars[i] == ';' {
        i += 1; // skip ';'
        match entity.as_str() {
          "amp" => result.push('&'),
          "lt" => result.push('<'),
          "gt" => result.push('>'),
          "quot" => result.push('"'),
          "apos" => result.push('\''),
          "nbsp" => result.push('\u{00A0}'),
          _ if entity.starts_with('#') => {
            let num_str = &entity[1..];
            let code = if num_str.starts_with('x') || num_str.starts_with('X') {
              u32::from_str_radix(&num_str[1..], 16).ok()
            } else {
              num_str.parse::<u32>().ok()
            };
            if let Some(c) = code.and_then(char::from_u32) {
              result.push(c);
            } else {
              result.push_str(&input[start..start + entity.len() + 2]);
            }
          }
          _ => {
            // Unknown entity, keep as-is
            result.push('&');
            result.push_str(&entity);
            result.push(';');
          }
        }
      } else {
        // No closing semicolon, output raw
        result.push('&');
        result.push_str(&entity);
      }
    } else {
      result.push(chars[i]);
      i += 1;
    }
  }
  result
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_simple_tag() {
    let tokens = tokenize("<div></div>");
    assert_eq!(
      tokens,
      vec![
        Token::OpenTag {
          name: "div".into(),
          attrs: vec![],
          self_closing: false
        },
        Token::CloseTag("div".into()),
      ]
    );
  }

  #[test]
  fn test_self_closing() {
    let tokens = tokenize("<br/>");
    assert_eq!(
      tokens,
      vec![Token::OpenTag {
        name: "br".into(),
        attrs: vec![],
        self_closing: true
      },]
    );
  }

  #[test]
  fn test_attributes() {
    let tokens = tokenize(r#"<a href="http://example.com" target="_blank">link</a>"#);
    assert_eq!(
      tokens,
      vec![
        Token::OpenTag {
          name: "a".into(),
          attrs: vec![
            ("href".into(), "http://example.com".into()),
            ("target".into(), "_blank".into()),
          ],
          self_closing: false,
        },
        Token::Text("link".into()),
        Token::CloseTag("a".into()),
      ]
    );
  }

  #[test]
  fn test_boolean_attribute() {
    let tokens = tokenize("<input disabled>");
    assert_eq!(
      tokens,
      vec![Token::OpenTag {
        name: "input".into(),
        attrs: vec![("disabled".into(), String::new())],
        self_closing: false,
      },]
    );
  }

  #[test]
  fn test_comment() {
    let tokens = tokenize("<!-- hello -->");
    assert_eq!(tokens, vec![Token::Comment(" hello ".into())]);
  }

  #[test]
  fn test_doctype() {
    let tokens = tokenize("<!DOCTYPE html>");
    assert_eq!(tokens, vec![Token::Doctype("DOCTYPE html".into())]);
  }

  #[test]
  fn test_entities() {
    assert_eq!(decode_entities("&amp;&lt;&gt;"), "&<>");
    assert_eq!(decode_entities("&#65;"), "A");
    assert_eq!(decode_entities("&#x41;"), "A");
  }

  #[test]
  fn test_text_and_elements() {
    let tokens = tokenize("<p>Hello <b>world</b></p>");
    assert_eq!(
      tokens,
      vec![
        Token::OpenTag {
          name: "p".into(),
          attrs: vec![],
          self_closing: false
        },
        Token::Text("Hello ".into()),
        Token::OpenTag {
          name: "b".into(),
          attrs: vec![],
          self_closing: false
        },
        Token::Text("world".into()),
        Token::CloseTag("b".into()),
        Token::CloseTag("p".into()),
      ]
    );
  }

  #[test]
  fn test_script_raw_text() {
    let tokens = tokenize("<script>var x = '<div>';</script>");
    assert_eq!(
      tokens,
      vec![
        Token::OpenTag {
          name: "script".into(),
          attrs: vec![],
          self_closing: false
        },
        Token::Text("var x = '<div>';".into()),
        Token::CloseTag("script".into()),
      ]
    );
  }
}
