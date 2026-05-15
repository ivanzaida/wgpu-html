use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn pointer_events_none_makes_element_transparent_to_click() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <div id="behind" style="width: 100px; height: 100px; background: red"></div>
      <div id="overlay" style="width: 100px; height: 100px; pointer-events: none;
           position: absolute; top: 0; left: 0; background: transparent"></div>
    </body></html>"#,
  );

  let behind_clicked = Arc::new(AtomicBool::new(false));
  let overlay_clicked = Arc::new(AtomicBool::new(false));

  {
    let f = behind_clicked.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "behind").unwrap();
    node.add_event_listener(
      "click",
      Arc::new(move |_, _| {
        f.store(true, Ordering::Relaxed);
      }),
    );
  }
  {
    let f = overlay_clicked.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "overlay").unwrap();
    node.add_event_listener(
      "click",
      Arc::new(move |_, _| {
        f.store(true, Ordering::Relaxed);
      }),
    );
  }

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(50.0, 50.0);
  lui.handle_click(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  assert!(
    !overlay_clicked.load(Ordering::Relaxed),
    "overlay with pointer-events:none should not receive click"
  );
  assert!(
    behind_clicked.load(Ordering::Relaxed),
    "element behind should receive click through overlay"
  );
}
