use crate::html::entities::decode_entities;

/// Token produced by the HTML tokenizer.
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

/// Tokenize an HTML string into a flat token sequence.
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
        s_chars.iter().enumerate().all(|(i, &c)| self.chars[self.pos + i] == c)
    }

    fn starts_with_ci(&self, s: &str) -> bool {
        let s_chars: Vec<char> = s.chars().collect();
        if self.remaining() < s_chars.len() {
            return false;
        }
        s_chars
            .iter()
            .enumerate()
            .all(|(i, &c)| self.chars[self.pos + i].to_ascii_lowercase() == c.to_ascii_lowercase())
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
        if self.starts_with("<!--") {
            self.consume_comment();
        } else if self.starts_with_ci("<!doctype") {
            self.consume_doctype();
        } else if self.starts_with("</") {
            self.consume_close_tag();
        } else if self.starts_with("<") && self.peek(1).map_or(false, |c| c.is_ascii_alphabetic()) {
            self.consume_open_tag();
        } else {
            self.pos += 1;
            self.tokens.push(Token::Text("<".into()));
        }
    }

    fn consume_comment(&mut self) {
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
        let comment: String = self.chars[start..self.pos].iter().collect();
        self.tokens.push(Token::Comment(comment));
    }

    fn consume_doctype(&mut self) {
        self.pos += 2;
        let start = self.pos;
        while self.pos < self.chars.len() && self.chars[self.pos] != '>' {
            self.pos += 1;
        }
        let content: String = self.chars[start..self.pos].iter().collect();
        if self.pos < self.chars.len() {
            self.pos += 1;
        }
        self.tokens.push(Token::Doctype(content));
    }

    fn consume_close_tag(&mut self) {
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
            if let Some(attr) = self.consume_attribute() {
                attrs.push(attr);
            } else {
                self.pos += 1;
            }
        }

        let lower_name = name.to_ascii_lowercase();
        if !self_closing && is_raw_text_element(&lower_name) {
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
            self.pos += 1;
            self.skip_whitespace();
            let value = self.consume_attr_value();
            Some((name, value))
        } else {
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
            self.pos += 1;
            let start = self.pos;
            while self.pos < self.chars.len() && self.chars[self.pos] != quote {
                self.pos += 1;
            }
            let value: String = self.chars[start..self.pos].iter().collect();
            if self.pos < self.chars.len() {
                self.pos += 1;
            }
            decode_entities(&value)
        } else {
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

fn is_raw_text_element(tag: &str) -> bool {
    matches!(tag, "script" | "style" | "textarea" | "title")
}

