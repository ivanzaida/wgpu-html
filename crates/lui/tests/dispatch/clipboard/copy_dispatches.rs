use std::sync::{Arc, Mutex};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, test_lui};

#[test]
fn copy_dispatches_clipboard_event() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();

  let (mut lui, mut spy) = test_lui(
    r#"<html><body><div id="target">text</div></body></html>"#,
  );
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);

  lui.doc.add_event_listener(
    "copy",
    Arc::new(move |_node, _event| {
      r.lock().unwrap().push("copy".into());
    }),
  );

  // Focus any element so clipboard dispatches somewhere
  lui.doc.focus_path = Some(vec![0, 0, 0]);

  lui.handle_copy(TEST_WIDTH, TEST_HEIGHT, 1.0);

  let evs = received.lock().unwrap();
  assert!(evs.contains(&"copy".into()), "expected copy event, got {evs:?}");
}

#[test]
fn copy_prevented_returns_none() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body><div>text</div></body></html>"#,
  );
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);

  lui.doc.add_event_listener(
    "copy",
    Arc::new(|_node, event| {
      event.prevent_default();
    }),
  );

  lui.doc.focus_path = Some(vec![0, 0, 0]);
  let result = lui.handle_copy(TEST_WIDTH, TEST_HEIGHT, 1.0);
  assert!(result.is_none(), "copy should return None when prevented");
}
