use wgpu_html_tree::Node;

use super::*;

#[test]
fn mouse_button_maps_known_buttons() {
  assert!(matches!(mouse_button(WinitMouseButton::Left), MouseButton::Primary));
  assert!(matches!(mouse_button(WinitMouseButton::Right), MouseButton::Secondary));
  assert!(matches!(mouse_button(WinitMouseButton::Middle), MouseButton::Middle));
  assert!(matches!(mouse_button(WinitMouseButton::Back), MouseButton::Other(3)));
  assert!(matches!(mouse_button(WinitMouseButton::Forward), MouseButton::Other(4)));
  assert!(matches!(
    mouse_button(WinitMouseButton::Other(7)),
    MouseButton::Other(7)
  ));
}

#[test]
fn keycode_to_modifier_recognises_modifier_keys() {
  assert_eq!(keycode_to_modifier(KeyCode::ControlLeft), Some(Modifier::Ctrl));
  assert_eq!(keycode_to_modifier(KeyCode::ShiftRight), Some(Modifier::Shift));
  assert_eq!(keycode_to_modifier(KeyCode::AltLeft), Some(Modifier::Alt));
  assert_eq!(keycode_to_modifier(KeyCode::SuperRight), Some(Modifier::Meta));
  assert_eq!(keycode_to_modifier(KeyCode::KeyA), None);
  assert_eq!(keycode_to_modifier(KeyCode::Tab), None);
}

#[test]
fn key_to_dom_key_handles_shift() {
  assert_eq!(key_to_dom_key(KeyCode::KeyA, false), "a");
  assert_eq!(key_to_dom_key(KeyCode::KeyA, true), "A");
  assert_eq!(key_to_dom_key(KeyCode::Digit1, false), "1");
  assert_eq!(key_to_dom_key(KeyCode::Digit1, true), "!");
  assert_eq!(key_to_dom_key(KeyCode::Tab, false), "Tab");
  assert_eq!(key_to_dom_key(KeyCode::Tab, true), "Tab");
}

#[test]
fn keycode_to_dom_code_is_layout_independent() {
  // Same code regardless of whether shift is held.
  assert_eq!(keycode_to_dom_code(KeyCode::KeyA), "KeyA");
  assert_eq!(keycode_to_dom_code(KeyCode::Digit1), "Digit1");
  assert_eq!(keycode_to_dom_code(KeyCode::ShiftLeft), "ShiftLeft");
  assert_eq!(keycode_to_dom_code(KeyCode::SuperLeft), "MetaLeft");
}

#[test]
fn update_modifiers_flips_only_modifier_keys() {
  let mut tree = Tree::new(Node::new("text"));
  assert!(!tree.modifiers().shift);
  assert!(update_modifiers(&mut tree, KeyCode::ShiftLeft, ElementState::Pressed));
  assert!(tree.modifiers().shift);
  assert!(update_modifiers(&mut tree, KeyCode::ShiftLeft, ElementState::Released));
  assert!(!tree.modifiers().shift);
  // Non-modifier keys leave the bitmask alone.
  assert!(!update_modifiers(&mut tree, KeyCode::KeyA, ElementState::Pressed));
  assert_eq!(tree.modifiers(), Default::default());
}
