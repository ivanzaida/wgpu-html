use std::sync::{Arc, Mutex};

use lui_models as m;
use lui_models::common::html_enums::InputType;
use lui_tree::{Element, MouseButton, Node, SelectionColors, Tree};

fn checkbox_tree() -> Tree {
  let mut inp = m::Input::default();
  inp.r#type = Some(InputType::Checkbox);
  let mut tree = Tree::new(Node::new(inp));
  tree.interaction.selection_colors = SelectionColors::default();
  tree
}

#[test]
fn click_toggles_checkbox() {
  let mut tree = checkbox_tree();
  let root = tree.root.as_ref().unwrap();
  let was_checked = if let Element::Input(inp) = &root.element {
    inp.checked.unwrap_or(false)
  } else {
    false
  };
  assert!(!was_checked);

  tree.dispatch_mouse_down(Some(&[]), (0.0, 0.0), MouseButton::Primary, None);
  tree.dispatch_mouse_up(Some(&[]), (0.0, 0.0), MouseButton::Primary, None);

  let now_checked = if let Element::Input(inp) = &tree.root.as_ref().unwrap().element {
    inp.checked.unwrap_or(false)
  } else {
    false
  };
  assert!(now_checked);
}

#[test]
fn checkbox_click_fires_change_and_input() {
  let received = Arc::new(Mutex::new(Vec::<(String, Option<bool>)>::new()));
  let r = received.clone();
  let mut tree = checkbox_tree();
  if let Some(n) = tree.root.as_mut() {
    n.on_event.push(Arc::new(move |ev| {
      let checked = match ev {
        lui_events::HtmlEvent::Input(input) => input.checked,
        _ => None,
      };
      r.lock().unwrap().push((ev.event_type().to_string(), checked));
    }));
  }

  tree.dispatch_mouse_down(Some(&[]), (0.0, 0.0), MouseButton::Primary, None);
  tree.dispatch_mouse_up(Some(&[]), (0.0, 0.0), MouseButton::Primary, None);

  let evs = received.lock().unwrap().clone();
  assert!(
    evs
      .iter()
      .any(|(event, checked)| event == "input" && *checked == Some(true)),
    "expected input with checked=true, got {evs:?}"
  );
  assert!(
    evs.iter().any(|(event, _)| event == "change"),
    "expected change, got {evs:?}"
  );
}

fn radio_tree() -> Tree {
  let mut r1 = m::Input::default();
  r1.r#type = Some(InputType::Radio);
  r1.name = Some("group1".into());
  r1.checked = Some(true);
  let mut r2 = m::Input::default();
  r2.r#type = Some(InputType::Radio);
  r2.name = Some("group1".into());
  let mut root = Node::new(m::Div::default());
  root.children.push(Node::new(r1));
  root.children.push(Node::new(r2));
  let mut tree = Tree::new(root);
  tree.interaction.selection_colors = SelectionColors::default();
  tree
}

#[test]
fn click_on_radio_unchecks_sibling_and_checks_self() {
  let mut tree = radio_tree();

  tree.dispatch_mouse_down(Some(&[1]), (0.0, 0.0), MouseButton::Primary, None);
  tree.dispatch_mouse_up(Some(&[1]), (0.0, 0.0), MouseButton::Primary, None);

  let body = tree.root.as_ref().unwrap();

  let r1 = if let Element::Input(inp) = &body.children[0].element {
    inp.checked.unwrap_or(false)
  } else {
    true
  };
  let r2 = if let Element::Input(inp) = &body.children[1].element {
    inp.checked.unwrap_or(false)
  } else {
    false
  };
  assert!(!r1, "first radio should be unchecked");
  assert!(r2, "second radio should be checked");
}

#[test]
fn enter_on_button_synthesises_click() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut button = m::Button::default();
  button.id = Some("btn".into());
  let mut tree = Tree::new(Node::new(button));
  if let Some(n) = tree.root.as_mut() {
    n.on_click.push(Arc::new(move |ev| {
      r.lock().unwrap().push(format!("click:{:?}", ev.pos));
    }));
  }
  tree.interaction.selection_colors = SelectionColors::default();

  tree.focus(Some(&[]));
  tree.key_down("Enter", "Enter", false);

  let evs = received.lock().unwrap().clone();
  assert!(!evs.is_empty(), "expected click from Enter, got nothing");
  assert!(evs[0].starts_with("click:"));
}

#[test]
fn space_on_checkbox_toggles() {
  let mut tree = checkbox_tree();
  tree.focus(Some(&[]));

  tree.key_down(" ", "Space", false);

  let now_checked = if let Element::Input(inp) = &tree.root.as_ref().unwrap().element {
    inp.checked.unwrap_or(false)
  } else {
    false
  };
  assert!(now_checked, "Space should toggle checkbox via synthetic click");
}
