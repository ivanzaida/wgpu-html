use std::sync::{Arc, Mutex};

use lui_core::EventListenerOptions;

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn capture_listener_fires_before_bubble_listener() {
  let (mut lui, mut spy) = test_lui(
    r#"
    <html><body>
      <div id="outer" style="width: 100px; height: 100px">
        <div id="inner" style="width: 100px; height: 100px; background: red"></div>
      </div>
    </body></html>
    "#,
  );

  let order = Arc::new(Mutex::new(Vec::<&'static str>::new()));

  {
    let log = order.clone();
    let outer = find_node_by_id_mut(&mut lui.doc_mut().root, "outer").unwrap();
    outer.add_event_listener_with_options(
      "click",
      Arc::new(move |_, _| {
        log.lock().unwrap().push("capture-outer");
      }),
      EventListenerOptions {
        capture: true,
        ..Default::default()
      },
    );
  }
  {
    let log = order.clone();
    let outer = find_node_by_id_mut(&mut lui.doc_mut().root, "outer").unwrap();
    outer.add_event_listener(
      "click",
      Arc::new(move |_, _| {
        log.lock().unwrap().push("bubble-outer");
      }),
    );
  }
  {
    let log = order.clone();
    let inner = find_node_by_id_mut(&mut lui.doc_mut().root, "inner").unwrap();
    inner.add_event_listener(
      "click",
      Arc::new(move |_, _| {
        log.lock().unwrap().push("target-inner");
      }),
    );
  }

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(50.0, 50.0);
  lui.handle_click(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  let phases = order.lock().unwrap().clone();
  assert_eq!(
    phases,
    vec!["capture-outer", "target-inner", "bubble-outer"],
    "capture should fire before target, target before bubble"
  );
}
