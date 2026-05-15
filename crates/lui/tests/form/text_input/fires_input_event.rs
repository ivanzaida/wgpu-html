use std::sync::{Arc, atomic::{AtomicU32, Ordering}};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn input_event_fires_on_text_mutation() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <input id="field" type="text" value="" style="width: 200px; height: 30px" />
    </body></html>"#,
  );

  let count = Arc::new(AtomicU32::new(0));
  {
    let c = count.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "field").unwrap();
    node.add_event_listener("input", Arc::new(move |_, _| {
      c.fetch_add(1, Ordering::Relaxed);
    }));
  }

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(100.0, 15.0);
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  lui.handle_text_input("x");

  assert_eq!(count.load(Ordering::Relaxed), 1, "input event should fire on text insertion");
}
