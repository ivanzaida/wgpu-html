use std::sync::{
  Arc,
  atomic::{AtomicU32, Ordering},
};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn mousemove_fires_when_cursor_moves_over_element() {
  let (mut lui, _spy) = test_lui(
    r#"<html><body>
      <div id="target" style="width: 100px; height: 100px; background: red"></div>
    </body></html>"#,
  );

  let count = Arc::new(AtomicU32::new(0));
  let c = count.clone();
  let target = find_node_by_id_mut(&mut lui.doc_mut().root, "target").unwrap();
  target.add_event_listener(
    "mousemove",
    Arc::new(move |_, _| {
      c.fetch_add(1, Ordering::Relaxed);
    }),
  );

  lui.set_cursor_position(50.0, 50.0);
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  assert_eq!(
    count.load(Ordering::Relaxed),
    1,
    "mousemove should fire on first cursor entry"
  );

  // Same position — no event
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  assert_eq!(
    count.load(Ordering::Relaxed),
    1,
    "no mousemove when cursor hasn't moved"
  );

  // Move within same element
  lui.set_cursor_position(60.0, 60.0);
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  assert_eq!(
    count.load(Ordering::Relaxed),
    2,
    "mousemove on cursor move within element"
  );
}
