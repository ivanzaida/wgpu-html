use std::sync::{Arc, atomic::{AtomicU32, Ordering}};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn wheel_event_dispatches_to_target() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <div id="box" style="width: 100px; height: 100px; overflow: scroll">
        <div style="height: 400px; background: red"></div>
      </div>
    </body></html>"#,
  );

  let count = Arc::new(AtomicU32::new(0));
  {
    let c = count.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "box").unwrap();
    node.add_event_listener("wheel", Arc::new(move |_, _| {
      c.fetch_add(1, Ordering::Relaxed);
    }));
  }

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(50.0, 50.0);
  lui.handle_wheel(TEST_WIDTH, TEST_HEIGHT, 1.0, 0.0, 60.0);

  assert_eq!(count.load(Ordering::Relaxed), 1, "wheel event should dispatch to target");
}

#[test]
fn wheel_prevent_default_cancels_scroll() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <div id="box" style="width: 100px; height: 100px; overflow: scroll">
        <div style="height: 400px; background: red"></div>
      </div>
    </body></html>"#,
  );

  {
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "box").unwrap();
    node.add_event_listener("wheel", Arc::new(|_, event| {
      event.prevent_default();
    }));
  }

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(50.0, 50.0);

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let before = crate::support::red_quad_y(&spy.take_last_list());

  lui.handle_wheel(TEST_WIDTH, TEST_HEIGHT, 1.0, 0.0, 60.0);
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let after = crate::support::red_quad_y(&spy.take_last_list());

  assert!(
    (after - before).abs() < 1.0,
    "preventDefault on wheel should cancel scrolling: before={before} after={after}"
  );
}
