use lui_html_parser::entities::decode_entities;

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
