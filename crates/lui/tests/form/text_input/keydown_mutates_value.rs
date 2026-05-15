use crate::support::{TEST_HEIGHT, TEST_WIDTH, find_node_by_id_mut, test_lui};

#[test]
fn typing_character_updates_form_value() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <input id="field" type="text" value="" style="width: 200px; height: 30px" />
    </body></html>"#,
  );

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);

  // Click to focus the input
  lui.set_cursor_position(100.0, 15.0);
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  // Type "abc"
  lui.handle_text_input("a");
  lui.handle_text_input("b");
  lui.handle_text_input("c");

  // Find the path to the input to query form value
  let path = {
    let node = find_node_by_id_mut(&mut lui.doc_mut().root, "field").unwrap();
    // We need the path — find it via dispatch helper
    let ptr = node as *const lui_core::HtmlNode;
    lui::dispatch::find_node_path(&lui.doc().root, ptr).unwrap()
  };

  assert_eq!(lui.form_value(&path), Some("abc"), "form value should be 'abc' after typing");
}

#[test]
fn backspace_deletes_last_character() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <input id="field" type="text" value="hello" style="width: 200px; height: 30px" />
    </body></html>"#,
  );

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

  assert_eq!(lui.form_value(&path), Some("hell"), "backspace should delete last char");
}
