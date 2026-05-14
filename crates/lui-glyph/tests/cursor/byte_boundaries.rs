use lui_glyph::utf8_boundaries;

#[test]
fn empty_string_produces_single_zero() {
  let bb = utf8_boundaries("");
  assert_eq!(bb, vec![0]);
}

#[test]
fn ascii_string_has_sequential_boundaries() {
  let bb = utf8_boundaries("abc");
  assert_eq!(bb, vec![0, 1, 2, 3]);
}

#[test]
fn single_char_has_two_boundaries() {
  let bb = utf8_boundaries("x");
  assert_eq!(bb, vec![0, 1]);
}

#[test]
fn multibyte_chars_produce_correct_offsets() {
  // é is 2 bytes (U+00E9), 中 is 3 bytes (U+4E2D), 🎉 is 4 bytes (U+1F389)
  let bb = utf8_boundaries("é中🎉");
  assert_eq!(bb, vec![0, 2, 5, 9]);
}

#[test]
fn mixed_ascii_and_multibyte() {
  let bb = utf8_boundaries("aé");
  assert_eq!(bb, vec![0, 1, 3]);
}

#[test]
fn length_equals_char_count_plus_one() {
  let text = "Hello, 世界!";
  let bb = utf8_boundaries(text);
  assert_eq!(bb.len(), text.chars().count() + 1);
}

#[test]
fn last_boundary_equals_byte_length() {
  let text = "café";
  let bb = utf8_boundaries(text);
  assert_eq!(*bb.last().unwrap(), text.len());
}

#[test]
fn boundaries_are_monotonically_increasing() {
  let bb = utf8_boundaries("Héllo 🌍 wörld");
  for window in bb.windows(2) {
    assert!(window[1] > window[0]);
  }
}
