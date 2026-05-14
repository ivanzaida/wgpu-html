use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn stop_propagation_prevents_bubble_to_parent() {
  let (mut lui, _spy) = test_lui(
    r#"
    <html><body>
      <div id="parent" style="width: 100px; height: 100px">
        <div id="child" style="width: 100px; height: 100px; background: red"></div>
      </div>
    </body></html>
    "#,
  );

  let parent_fired = Arc::new(AtomicBool::new(false));

  {
    let child = find_node_by_id_mut(&mut lui.doc_mut().root, "child").unwrap();
    child.add_event_listener("click", Arc::new(|_node, event| {
      event.stop_propagation();
    }));
  }
  {
    let flag = parent_fired.clone();
    let parent = find_node_by_id_mut(&mut lui.doc_mut().root, "parent").unwrap();
    parent.add_event_listener("click", Arc::new(move |_node, _event| {
      flag.store(true, Ordering::Relaxed);
    }));
  }

  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(50.0, 50.0);
  lui.handle_click(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  assert!(!parent_fired.load(Ordering::Relaxed), "stopPropagation should prevent bubble to parent");
}
