use std::sync::{Arc, Mutex};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn mouseout_fires_when_leaving_element() {
  let (mut lui, _spy) = test_lui(
    r#"<html><body>
      <div id="a" style="width: 100px; height: 50px; background: red"></div>
      <div id="b" style="width: 100px; height: 50px; background: blue"></div>
    </body></html>"#,
  );

  let log = Arc::new(Mutex::new(Vec::<String>::new()));

  {
    let l = log.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "a").unwrap();
    node.add_event_listener(
      "mouseout",
      Arc::new(move |_, _| {
        l.lock().unwrap().push("out-a".into());
      }),
    );
  }
  {
    let l = log.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "b").unwrap();
    node.add_event_listener(
      "mouseover",
      Arc::new(move |_, _| {
        l.lock().unwrap().push("over-b".into());
      }),
    );
  }

  // Enter A
  lui.set_cursor_position(50.0, 25.0);
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);

  // Move to B
  lui.set_cursor_position(50.0, 75.0);
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);

  let events = log.lock().unwrap().clone();
  assert!(
    events.contains(&"out-a".to_string()),
    "mouseout should fire on element A: {events:?}"
  );
  assert!(
    events.contains(&"over-b".to_string()),
    "mouseover should fire on element B: {events:?}"
  );

  // Per spec: mouseout fires before mouseover
  let out_idx = events.iter().position(|e| e == "out-a").unwrap();
  let over_idx = events.iter().position(|e| e == "over-b").unwrap();
  assert!(out_idx < over_idx, "mouseout should fire before mouseover: {events:?}");
}
