use super::*;

#[test]
fn insert_at_end() {
    let (v, c) = insert_text("abc", &EditCursor::collapsed(3), "d");
    assert_eq!(v, "abcd");
    assert_eq!(c.cursor, 4);
    assert!(!c.has_selection());
}

#[test]
fn insert_at_start() {
    let (v, c) = insert_text("abc", &EditCursor::collapsed(0), "x");
    assert_eq!(v, "xabc");
    assert_eq!(c.cursor, 1);
}

#[test]
fn insert_at_middle() {
    let (v, c) = insert_text("ac", &EditCursor::collapsed(1), "b");
    assert_eq!(v, "abc");
    assert_eq!(c.cursor, 2);
}

#[test]
fn insert_replaces_selection() {
    let cursor = EditCursor {
        cursor: 3,
        selection_anchor: Some(1),
    };
    let (v, c) = insert_text("abcd", &cursor, "XY");
    assert_eq!(v, "aXYd");
    assert_eq!(c.cursor, 3);
    assert!(!c.has_selection());
}

#[test]
fn backspace_at_start_is_noop() {
    let (v, c) = delete_backward("abc", &EditCursor::collapsed(0));
    assert_eq!(v, "abc");
    assert_eq!(c.cursor, 0);
}

#[test]
fn backspace_deletes_one_char() {
    let (v, c) = delete_backward("abc", &EditCursor::collapsed(2));
    assert_eq!(v, "ac");
    assert_eq!(c.cursor, 1);
}

#[test]
fn backspace_deletes_selection() {
    let cursor = EditCursor {
        cursor: 3,
        selection_anchor: Some(1),
    };
    let (v, c) = delete_backward("abcd", &cursor);
    assert_eq!(v, "ad");
    assert_eq!(c.cursor, 1);
}

#[test]
fn delete_forward_at_end_is_noop() {
    let (v, c) = delete_forward("abc", &EditCursor::collapsed(3));
    assert_eq!(v, "abc");
    assert_eq!(c.cursor, 3);
}

#[test]
fn delete_forward_deletes_one_char() {
    let (v, c) = delete_forward("abc", &EditCursor::collapsed(1));
    assert_eq!(v, "ac");
    assert_eq!(c.cursor, 1);
}

#[test]
fn move_left_collapses_selection() {
    let cursor = EditCursor {
        cursor: 3,
        selection_anchor: Some(1),
    };
    let c = move_left("abcd", &cursor, false);
    assert_eq!(c.cursor, 1);
    assert!(!c.has_selection());
}

#[test]
fn move_left_with_shift_extends_selection() {
    let c = move_left("abc", &EditCursor::collapsed(2), true);
    assert_eq!(c.cursor, 1);
    assert_eq!(c.selection_anchor, Some(2));
}

#[test]
fn move_right_collapses_selection() {
    let cursor = EditCursor {
        cursor: 1,
        selection_anchor: Some(3),
    };
    let c = move_right("abcd", &cursor, false);
    assert_eq!(c.cursor, 3);
    assert!(!c.has_selection());
}

#[test]
fn home_goes_to_line_start() {
    let c = move_home("abc\ndef", &EditCursor::collapsed(5), false);
    assert_eq!(c.cursor, 4); // start of "def"
}

#[test]
fn end_goes_to_line_end() {
    let c = move_end("abc\ndef", &EditCursor::collapsed(1), false);
    assert_eq!(c.cursor, 3); // end of "abc"
}

#[test]
fn select_all_selects_everything() {
    let c = select_all("hello");
    assert_eq!(c.cursor, 5);
    assert_eq!(c.selection_anchor, Some(0));
}

#[test]
fn move_up_on_first_line_goes_to_start() {
    let c = move_up("abc\ndef", &EditCursor::collapsed(2), false);
    assert_eq!(c.cursor, 0);
}

#[test]
fn move_up_preserves_column() {
    let c = move_up("abc\ndef", &EditCursor::collapsed(5), false);
    assert_eq!(c.cursor, 1); // col 1 of "abc"
}

#[test]
fn move_up_clamps_column() {
    let c = move_up("ab\ncdef", &EditCursor::collapsed(7), false);
    assert_eq!(c.cursor, 2); // col 2 of "ab" (clamped from col 4)
}

#[test]
fn move_down_on_last_line_goes_to_end() {
    let c = move_down("abc\ndef", &EditCursor::collapsed(5), false);
    assert_eq!(c.cursor, 7);
}

#[test]
fn move_down_preserves_column() {
    let c = move_down("abc\ndef", &EditCursor::collapsed(1), false);
    assert_eq!(c.cursor, 5); // col 1 of "def"
}

#[test]
fn insert_line_break_adds_newline() {
    let (v, c) = insert_line_break("abc", &EditCursor::collapsed(1));
    assert_eq!(v, "a\nbc");
    assert_eq!(c.cursor, 2);
}

#[test]
fn multibyte_backspace() {
    // "aéb" — é is 2 bytes (U+00E9)
    let (v, c) = delete_backward("aéb", &EditCursor::collapsed(3));
    assert_eq!(v, "ab");
    assert_eq!(c.cursor, 1);
}

#[test]
fn multibyte_move_right() {
    let c = move_right("aéb", &EditCursor::collapsed(1), false);
    assert_eq!(c.cursor, 3); // skip the 2-byte é
}
