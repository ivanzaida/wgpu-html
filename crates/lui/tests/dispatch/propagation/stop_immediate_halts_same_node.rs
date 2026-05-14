use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn stop_immediate_propagation_halts_remaining_listeners_on_same_node() {
  let (mut lui, _spy) = test_lui(
    r#"
    <html><body>
      <div id="target" style="width: 100px; height: 100px; background: red"></div>
    </body></html>
    "#,
  );

  let second_fired = Arc::new(AtomicBool::new(false));

  {
    let target = find_node_by_id_mut(&mut lui.doc_mut().root, "target").unwrap();
    target.add_event_listener("click", Arc::new(|_node, event| {
      event.stop_immediate_propagation();
    }));
  }
  {
    let flag = second_fired.clone();
    let target = find_node_by_id_mut(&mut lui.doc_mut().root, "target").unwrap();
    target.add_event_listener("click", Arc::new(move |_node, _event| {
      flag.store(true, Ordering::Relaxed);
    }));
  }

  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(50.0, 50.0);
  lui.handle_click(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  assert!(
    !second_fired.load(Ordering::Relaxed),
    "stopImmediatePropagation should prevent second listener on same node"
  );
}
