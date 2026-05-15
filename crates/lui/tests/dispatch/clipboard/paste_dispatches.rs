use std::sync::{Arc, Mutex};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, test_lui};

#[test]
fn paste_dispatches_clipboard_event_with_data() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();

  let (mut lui, mut spy) = test_lui(
    r#"<html><body><div>text</div></body></html>"#,
  );
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);

  lui.doc.add_event_listener(
    "paste",
    Arc::new(move |_node, event| {
      if let lui_core::events::DocumentEvent::ClipboardEvent(cb) = &*event {
        if let Some(data) = &cb.clipboard_data {
          r.lock().unwrap().push(format!("paste:{data}"));
        }
      }
    }),
  );

  lui.doc.focus_path = Some(vec![0, 0, 0]);
  lui.handle_paste("pasted text");

  let evs = received.lock().unwrap();
  assert!(
    evs.iter().any(|e| e == "paste:pasted text"),
    "expected paste event with data, got {evs:?}"
  );
}

#[test]
fn paste_prevented_does_not_insert() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body><input type="text" value="original"></body></html>"#,
  );
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);

  lui.doc.add_event_listener(
    "paste",
    Arc::new(|_node, event| {
      event.prevent_default();
    }),
  );

  // Focus the input (body > input is at path [0, 0, 0])
  lui.doc.focus_path = Some(vec![0, 0, 0]);
  lui.handle_paste("should not appear");

  let path = lui.doc.focus_path.clone().unwrap();
  let val = lui.form_value(&path).unwrap_or("");
  assert!(!val.contains("should not appear"), "paste should not insert when prevented, got: {val}");
}
