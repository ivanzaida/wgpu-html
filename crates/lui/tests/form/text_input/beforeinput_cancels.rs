use std::sync::Arc;

use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn beforeinput_prevent_default_cancels_insertion() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <input id="field" type="text" value="" style="width: 200px; height: 30px" />
    </body></html>"#,
  );

  {
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "field").unwrap();
    node.add_event_listener(
      "beforeinput",
      Arc::new(|_, event| {
        event.prevent_default();
      }),
    );
  }

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(100.0, 15.0);
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  lui.handle_text_input("x");

  let path = {
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "field").unwrap();
    let ptr = node as *const lui_core::HtmlNode;
    lui::dispatch::find_node_path(&lui.doc().root, ptr).unwrap()
  };

  assert_eq!(
    lui.form_value(&path),
    Some(""),
    "value should remain empty when beforeinput is prevented"
  );
}

#[test]
fn beforeinput_prevent_default_cancels_deletion() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <input id="field" type="text" value="abc" style="width: 200px; height: 30px" />
    </body></html>"#,
  );

  {
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "field").unwrap();
    node.add_event_listener(
      "beforeinput",
      Arc::new(|_, event| {
        event.prevent_default();
      }),
    );
  }

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(100.0, 15.0);
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  lui.handle_key_down("Backspace", "Backspace", false, lui::KeyModifiers::default());

  let path = {
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "field").unwrap();
    let ptr = node as *const lui_core::HtmlNode;
    lui::dispatch::find_node_path(&lui.doc().root, ptr).unwrap()
  };

  assert_eq!(
    lui.form_value(&path),
    Some("abc"),
    "value should remain unchanged when beforeinput is prevented on backspace"
  );
}
