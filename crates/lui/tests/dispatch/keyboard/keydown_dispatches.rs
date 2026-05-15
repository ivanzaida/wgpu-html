use std::sync::{Arc, Mutex};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn keydown_dispatches_to_focused_element() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <div id="target" tabindex="0" style="width: 100px; height: 100px; background: red"></div>
    </body></html>"#,
  );

  let keys = Arc::new(Mutex::new(Vec::<String>::new()));
  {
    let k = keys.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "target").unwrap();
    node.add_event_listener("keydown", Arc::new(move |_, event| {
      if let lui_core::events::DocumentEvent::KeyboardEvent(kb) = &*event {
        k.lock().unwrap().push(kb.key.clone());
      }
    }));
  }

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);

  // Focus the element by clicking it
  lui.set_cursor_position(50.0, 50.0);
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  // Fire key event
  lui.handle_key_down("a", "KeyA", false, lui::KeyModifiers::default());

  let pressed = keys.lock().unwrap().clone();
  assert_eq!(pressed, vec!["a"], "keydown should dispatch to focused element");
}

#[test]
fn keydown_bubbles_from_focused_element() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <div id="parent" style="width: 100px; height: 100px">
        <div id="child" tabindex="0" style="width: 100px; height: 100px; background: red"></div>
      </div>
    </body></html>"#,
  );

  let parent_keys = Arc::new(Mutex::new(Vec::<String>::new()));
  {
    let k = parent_keys.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "parent").unwrap();
    node.add_event_listener("keydown", Arc::new(move |_, event| {
      if let lui_core::events::DocumentEvent::KeyboardEvent(kb) = &*event {
        k.lock().unwrap().push(kb.key.clone());
      }
    }));
  }

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);

  lui.set_cursor_position(50.0, 50.0);
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  lui.handle_key_down("Enter", "Enter", false, lui::KeyModifiers::default());

  let pressed = parent_keys.lock().unwrap().clone();
  assert_eq!(pressed, vec!["Enter"], "keydown should bubble from child to parent");
}
