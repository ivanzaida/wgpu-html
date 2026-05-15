use std::sync::{Arc, atomic::{AtomicU32, Ordering}};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn change_event_fires_on_blur_after_edit() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <input id="field" type="text" value="" style="width: 200px; height: 30px" />
      <div id="other" style="width: 100px; height: 100px; background: blue" tabindex="0"></div>
    </body></html>"#,
  );

  let changes = Arc::new(AtomicU32::new(0));
  {
    let c = changes.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "field").unwrap();
    node.add_event_listener("change", Arc::new(move |_, _| {
      c.fetch_add(1, Ordering::Relaxed);
    }));
  }

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);

  // Focus and type
  lui.set_cursor_position(100.0, 15.0);
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_text_input("hello");

  assert_eq!(changes.load(Ordering::Relaxed), 0, "no change event while still focused");

  // Click elsewhere to blur
  lui.set_cursor_position(50.0, 80.0);
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  assert_eq!(changes.load(Ordering::Relaxed), 1, "change should fire on blur after edit");
}

#[test]
fn no_change_event_when_value_unchanged() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <input id="field" type="text" value="" style="width: 200px; height: 30px" />
      <div id="other" style="width: 100px; height: 100px; background: blue" tabindex="0"></div>
    </body></html>"#,
  );

  let changes = Arc::new(AtomicU32::new(0));
  {
    let c = changes.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "field").unwrap();
    node.add_event_listener("change", Arc::new(move |_, _| {
      c.fetch_add(1, Ordering::Relaxed);
    }));
  }

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);

  // Focus without typing
  lui.set_cursor_position(100.0, 15.0);
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  // Blur without editing
  lui.set_cursor_position(50.0, 80.0);
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  assert_eq!(changes.load(Ordering::Relaxed), 0, "no change event when value was not modified");
}
