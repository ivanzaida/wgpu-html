use std::sync::{Arc, Mutex};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn pointermove_fires_before_mousemove() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <div id="target" style="width: 100px; height: 100px; background: red"></div>
    </body></html>"#,
  );

  let log = Arc::new(Mutex::new(Vec::<String>::new()));

  {
    let l = log.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "target").unwrap();
    node.add_event_listener(
      "pointermove",
      Arc::new(move |_, _| {
        l.lock().unwrap().push("pointermove".into());
      }),
    );
  }
  {
    let l = log.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "target").unwrap();
    node.add_event_listener(
      "mousemove",
      Arc::new(move |_, _| {
        l.lock().unwrap().push("mousemove".into());
      }),
    );
  }

  lui.set_cursor_position(50.0, 50.0);
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);

  let events = log.lock().unwrap().clone();
  assert_eq!(events, vec!["pointermove", "mousemove"], "pointermove must fire before mousemove");
}
