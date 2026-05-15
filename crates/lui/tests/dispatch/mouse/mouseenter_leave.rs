use std::sync::{
  Arc,
  atomic::{AtomicU32, Ordering},
};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn mouseenter_fires_on_entry_and_mouseleave_on_exit() {
  let (mut lui, _spy) = test_lui(
    r#"<html><body>
      <div id="box" style="width: 50px; height: 50px; background: red"></div>
    </body></html>"#,
  );

  let enters = Arc::new(AtomicU32::new(0));
  let leaves = Arc::new(AtomicU32::new(0));

  {
    let e = enters.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "box").unwrap();
    node.add_event_listener(
      "mouseenter",
      Arc::new(move |_, _| {
        e.fetch_add(1, Ordering::Relaxed);
      }),
    );
  }
  {
    let l = leaves.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "box").unwrap();
    node.add_event_listener(
      "mouseleave",
      Arc::new(move |_, _| {
        l.fetch_add(1, Ordering::Relaxed);
      }),
    );
  }

  // Enter element
  lui.set_cursor_position(25.0, 25.0);
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  assert_eq!(enters.load(Ordering::Relaxed), 1, "mouseenter on entry");
  assert_eq!(leaves.load(Ordering::Relaxed), 0, "no mouseleave yet");

  // Leave element
  lui.set_cursor_position(150.0, 150.0);
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  assert_eq!(leaves.load(Ordering::Relaxed), 1, "mouseleave on exit");
  assert_eq!(enters.load(Ordering::Relaxed), 1, "no extra mouseenter");
}
