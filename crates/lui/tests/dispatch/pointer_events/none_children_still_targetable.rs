use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn pointer_events_none_children_with_auto_still_receive_clicks() {
  let (mut lui, _spy) = test_lui(
    r#"<html><body>
      <div id="parent" style="width: 100px; height: 100px; pointer-events: none">
        <div id="child" style="width: 100px; height: 100px; pointer-events: auto; background: red"></div>
      </div>
    </body></html>"#,
  );

  let child_clicked = Arc::new(AtomicBool::new(false));
  {
    let f = child_clicked.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "child").unwrap();
    node.add_event_listener(
      "click",
      Arc::new(move |_, _| {
        f.store(true, Ordering::Relaxed);
      }),
    );
  }

  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(50.0, 50.0);
  lui.handle_click(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  assert!(
    child_clicked.load(Ordering::Relaxed),
    "child with pointer-events:auto inside none parent should be clickable"
  );
}
