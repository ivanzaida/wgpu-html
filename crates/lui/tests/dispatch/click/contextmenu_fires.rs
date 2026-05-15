use std::sync::{
  Arc,
  atomic::{AtomicU32, Ordering},
};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn contextmenu_fires_on_right_click() {
  let (mut lui, _spy) = test_lui(
    r#"<html><body>
      <div id="target" style="width: 100px; height: 100px; background: red"></div>
    </body></html>"#,
  );

  let ctx_count = Arc::new(AtomicU32::new(0));
  {
    let c = ctx_count.clone();
    let target = find_node_by_id_mut(&mut lui.doc_mut().root, "target").unwrap();
    target.add_event_listener(
      "contextmenu",
      Arc::new(move |_, _| {
        c.fetch_add(1, Ordering::Relaxed);
      }),
    );
  }

  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(50.0, 50.0);

  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 2);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 2);

  assert_eq!(
    ctx_count.load(Ordering::Relaxed),
    1,
    "contextmenu should fire on right-click"
  );
}

#[test]
fn right_click_does_not_fire_click_event() {
  let (mut lui, _spy) = test_lui(
    r#"<html><body>
      <div id="target" style="width: 100px; height: 100px; background: red"></div>
    </body></html>"#,
  );

  let clicks = Arc::new(AtomicU32::new(0));
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

  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(50.0, 50.0);

  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 2);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 2);

  assert_eq!(
    clicks.load(Ordering::Relaxed),
    0,
    "right-click should not fire click event"
  );
}
