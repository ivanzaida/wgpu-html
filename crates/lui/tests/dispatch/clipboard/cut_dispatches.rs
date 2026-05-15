use std::sync::{Arc, Mutex};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, test_lui};

#[test]
fn cut_dispatches_clipboard_event() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();

  let (mut lui, mut spy) = test_lui(
    r#"<html><body><div>text</div></body></html>"#,
  );
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);

  lui.doc.add_event_listener(
    "cut",
    Arc::new(move |_node, _event| {
      r.lock().unwrap().push("cut".into());
    }),
  );

  lui.doc.focus_path = Some(vec![0, 0, 0]);
  lui.handle_cut(TEST_WIDTH, TEST_HEIGHT, 1.0);

  let evs = received.lock().unwrap();
  assert!(evs.contains(&"cut".into()), "expected cut event, got {evs:?}");
}

#[test]
fn cut_prevented_returns_none() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body><div>text</div></body></html>"#,
  );
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);

  lui.doc.add_event_listener(
    "cut",
    Arc::new(|_node, event| {
      event.prevent_default();
    }),
  );

  lui.doc.focus_path = Some(vec![0, 0, 0]);
  let result = lui.handle_cut(TEST_WIDTH, TEST_HEIGHT, 1.0);
  assert!(result.is_none(), "cut should return None when prevented");
}
