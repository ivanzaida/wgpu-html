use std::sync::{Arc, Mutex};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn pointerdown_fires_before_mousedown() {
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
      "pointerdown",
      Arc::new(move |_, _| {
        l.lock().unwrap().push("pointerdown".into());
      }),
    );
  }
  {
    let l = log.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "target").unwrap();
    node.add_event_listener(
      "mousedown",
      Arc::new(move |_, _| {
        l.lock().unwrap().push("mousedown".into());
      }),
    );
  }

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(50.0, 50.0);
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  let events = log.lock().unwrap().clone();
  assert_eq!(events, vec!["pointerdown", "mousedown"], "pointerdown must fire before mousedown");
}

#[test]
fn pointerup_fires_before_mouseup_and_click() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <div id="target" style="width: 100px; height: 100px; background: red"></div>
    </body></html>"#,
  );

  let log = Arc::new(Mutex::new(Vec::<String>::new()));

  for event_type in &["pointerup", "mouseup", "click"] {
    let l = log.clone();
    let et = event_type.to_string();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "target").unwrap();
    node.add_event_listener(
      event_type,
      Arc::new(move |_, _| {
        l.lock().unwrap().push(et.clone());
      }),
    );
  }

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(50.0, 50.0);
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  let events = log.lock().unwrap().clone();
  let up_order: Vec<&String> = events.iter().filter(|e| *e == "pointerup" || *e == "mouseup" || *e == "click").collect();
  assert_eq!(
    up_order,
    vec!["pointerup", "mouseup", "click"],
    "order should be pointerup → mouseup → click, got: {up_order:?}"
  );
}
