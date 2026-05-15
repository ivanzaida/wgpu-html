use std::sync::{
  Arc,
  atomic::{AtomicU32, Ordering},
};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn mouseover_bubbles_to_parent() {
  let (mut lui, _spy) = test_lui(
    r#"<html><body>
      <div id="parent" style="width: 100px; height: 100px">
        <div id="child" style="width: 100px; height: 100px; background: red"></div>
      </div>
    </body></html>"#,
  );

  let parent_over = Arc::new(AtomicU32::new(0));
  {
    let c = parent_over.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "parent").unwrap();
    node.add_event_listener(
      "mouseover",
      Arc::new(move |_, _| {
        c.fetch_add(1, Ordering::Relaxed);
      }),
    );
  }

  lui.set_cursor_position(50.0, 50.0);
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  assert!(
    parent_over.load(Ordering::Relaxed) >= 1,
    "mouseover should bubble from child to parent"
  );
}

#[test]
fn mouseenter_does_not_bubble() {
  let (mut lui, _spy) = test_lui(
    r#"<html><body>
      <div id="parent" style="width: 200px; height: 200px">
        <div id="a" style="width: 200px; height: 100px; background: red"></div>
        <div id="b" style="width: 200px; height: 100px; background: blue"></div>
      </div>
    </body></html>"#,
  );

  // Enter child A first
  lui.set_cursor_position(50.0, 50.0);
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);

  // Now attach enter listener on parent AFTER initial entry
  let parent_enters = Arc::new(AtomicU32::new(0));
  {
    let c = parent_enters.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "parent").unwrap();
    node.add_event_listener(
      "mouseenter",
      Arc::new(move |_, _| {
        c.fetch_add(1, Ordering::Relaxed);
      }),
    );
  }

  // Move from child A to child B — parent stays hovered, shouldn't get mouseenter
  lui.set_cursor_position(50.0, 150.0);
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  assert_eq!(
    parent_enters.load(Ordering::Relaxed),
    0,
    "parent should not get mouseenter when moving between its children"
  );
}
