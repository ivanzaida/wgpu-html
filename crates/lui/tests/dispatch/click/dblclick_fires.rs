use std::sync::{
  Arc,
  atomic::{AtomicU32, Ordering},
};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn dblclick_fires_on_rapid_double_click() {
  let (mut lui, _spy) = test_lui(
    r#"<html><body>
      <div id="target" style="width: 100px; height: 100px; background: red"></div>
    </body></html>"#,
  );

  let clicks = Arc::new(AtomicU32::new(0));
  let dblclicks = Arc::new(AtomicU32::new(0));

  {
    let c = clicks.clone();
    let target = find_node_by_id_mut(&mut lui.doc_mut().root, "target").unwrap();
    target.add_event_listener(
      "click",
      Arc::new(move |_, _| {
        c.fetch_add(1, Ordering::Relaxed);
      }),
    );
  }
  {
    let d = dblclicks.clone();
    let target = find_node_by_id_mut(&mut lui.doc_mut().root, "target").unwrap();
    target.add_event_listener(
      "dblclick",
      Arc::new(move |_, _| {
        d.fetch_add(1, Ordering::Relaxed);
      }),
    );
  }

  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(50.0, 50.0);

  // First click
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  // Second click (rapid — same thread, well within 500ms)
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  assert_eq!(clicks.load(Ordering::Relaxed), 2, "should fire two click events");
  assert_eq!(dblclicks.load(Ordering::Relaxed), 1, "should fire one dblclick event");
}

#[test]
fn single_click_does_not_fire_dblclick() {
  let (mut lui, _spy) = test_lui(
    r#"<html><body>
      <div id="target" style="width: 100px; height: 100px; background: red"></div>
    </body></html>"#,
  );

  let dblclicks = Arc::new(AtomicU32::new(0));
  {
    let d = dblclicks.clone();
    let target = find_node_by_id_mut(&mut lui.doc_mut().root, "target").unwrap();
    target.add_event_listener(
      "dblclick",
      Arc::new(move |_, _| {
        d.fetch_add(1, Ordering::Relaxed);
      }),
    );
  }

  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(50.0, 50.0);

  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  assert_eq!(
    dblclicks.load(Ordering::Relaxed),
    0,
    "single click should not fire dblclick"
  );
}

#[test]
fn triple_click_fires_only_one_dblclick() {
  let (mut lui, _spy) = test_lui(
    r#"<html><body>
      <div id="target" style="width: 100px; height: 100px; background: red"></div>
    </body></html>"#,
  );

  let dblclicks = Arc::new(AtomicU32::new(0));
  {
    let d = dblclicks.clone();
    let target = find_node_by_id_mut(&mut lui.doc_mut().root, "target").unwrap();
    target.add_event_listener(
      "dblclick",
      Arc::new(move |_, _| {
        d.fetch_add(1, Ordering::Relaxed);
      }),
    );
  }

  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(50.0, 50.0);

  for _ in 0..3 {
    lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
    lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  }

  assert_eq!(
    dblclicks.load(Ordering::Relaxed),
    1,
    "triple click should fire exactly one dblclick"
  );
}
