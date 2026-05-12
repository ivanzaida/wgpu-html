/// Decode HTML entities (&amp;, &lt;, etc.) in a string.
///
/// Handles named entities, decimal `&#...;` and hex `&#x...;` numeric references.
/// Unknown entities are left as-is.
pub fn decode_entities(input: &str) -> String {
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
                i += 1;
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
                        result.push('&');
                        result.push_str(&entity);
                        result.push(';');
                    }
                }
            } else {
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
    fn no_entities() {
        assert_eq!(decode_entities("hello"), "hello");
    }

    #[test]
    fn amp() {
        assert_eq!(decode_entities("a &amp; b"), "a & b");
    }

    #[test]
    fn lt_gt() {
        assert_eq!(decode_entities("&lt;div&gt;"), "<div>");
    }

    #[test]
    fn quot_apos() {
        assert_eq!(decode_entities("&quot;hi&apos;"), "\"hi'");
    }

    #[test]
    fn nbsp() {
        assert_eq!(decode_entities("a&nbsp;b"), "a\u{00A0}b");
    }

    #[test]
    fn decimal_numeric() {
        assert_eq!(decode_entities("&#65;"), "A");
    }

    #[test]
    fn hex_numeric() {
        assert_eq!(decode_entities("&#x41;"), "A");
        assert_eq!(decode_entities("&#X41;"), "A");
    }

    #[test]
    fn unknown_entity_preserved() {
        assert_eq!(decode_entities("&unknown;"), "&unknown;");
    }

    #[test]
    fn unterminated_entity() {
        assert_eq!(decode_entities("a &amp b"), "a &amp b");
    }
}
