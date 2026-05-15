use std::sync::{Arc, Mutex};

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn pointer_event_has_mouse_type_and_primary() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <div id="target" style="width: 100px; height: 100px; background: red"></div>
    </body></html>"#,
  );

  let info = Arc::new(Mutex::new(None::<(String, i32, bool)>));
  {
    let i = info.clone();
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "target").unwrap();
    node.add_event_listener(
      "pointerdown",
      Arc::new(move |_, event| {
        if let lui_core::events::DocumentEvent::PointerEvent(pe) = &*event {
          *i.lock().unwrap() = Some((pe.pointer_type.clone(), pe.pointer_id, pe.is_primary));
        }
      }),
    );
  }

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(50.0, 50.0);
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  let (ptr_type, ptr_id, primary) = info.lock().unwrap().clone().expect("pointerdown should fire");
  assert_eq!(ptr_type, "mouse", "pointer_type should be 'mouse'");
  assert_eq!(ptr_id, 1, "pointer_id should be 1 for mouse");
  assert!(primary, "is_primary should be true for mouse");
}
