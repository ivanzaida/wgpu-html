use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn click_outside_target_does_not_fire_handler() {
  let (mut lui, mut spy) = test_lui(
    r#"
    <html><body>
      <div id="target" style="width: 50px; height: 50px; background: red"></div>
    </body></html>
    "#,
  );

  let clicked = Arc::new(AtomicBool::new(false));
  let flag = clicked.clone();
  let target = find_node_by_id_mut(&mut lui.doc_mut().root, "target").unwrap();
  target.add_event_listener(
    "click",
    Arc::new(move |_node, _event| {
      flag.store(true, Ordering::Relaxed);
    }),
  );

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(150.0, 150.0);
  lui.handle_click(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  assert!(
    !clicked.load(Ordering::Relaxed),
    "click outside should not fire handler"
  );
}
