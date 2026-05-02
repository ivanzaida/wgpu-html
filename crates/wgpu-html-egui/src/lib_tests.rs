use wgpu_html_tree::{Node, Tree};

use super::*;

#[test]
fn pointer_buttons_map_to_tree_buttons() {
  assert!(matches!(pointer_button(PointerButton::Primary), MouseButton::Primary));
  assert!(matches!(
    pointer_button(PointerButton::Secondary),
    MouseButton::Secondary
  ));
  assert!(matches!(pointer_button(PointerButton::Middle), MouseButton::Middle));
  assert!(matches!(pointer_button(PointerButton::Extra1), MouseButton::Other(3)));
  assert!(matches!(pointer_button(PointerButton::Extra2), MouseButton::Other(4)));
}

#[test]
fn key_names_match_dom_strings_for_common_keys() {
  assert_eq!(key_to_dom_key(egui::Key::A, false), "a");
  assert_eq!(key_to_dom_key(egui::Key::A, true), "A");
  assert_eq!(key_to_dom_key(egui::Key::Enter, false), "Enter");
  assert_eq!(key_to_dom_code(egui::Key::A), "KeyA");
  assert_eq!(key_to_dom_code(egui::Key::Num1), "Digit1");
}

#[test]
fn forward_key_updates_tree_keyboard_path() {
  let mut tree = Tree::new(Node::new("root"));
  assert!(forward_key(&mut tree, egui::Key::A, true, false));
  assert!(forward_key(&mut tree, egui::Key::A, false, false));
  assert!(!forward_key(&mut tree, egui::Key::F1, true, false));
}
