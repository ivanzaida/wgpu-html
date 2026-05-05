use std::sync::{Arc, Mutex};

use wgpu_html_models as m;
use wgpu_html_models::common::html_enums::InputType;
use wgpu_html_tree::{Element, MouseButton, Node, Tree};

// ── Scroll / Selectionchange tests ───────────────────────────────────────────

#[test]
fn scroll_event_dispatches_non_bubbling() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut tree = Tree::new(Node::new(m::Div::default()));
  if let Some(n) = tree.root.as_mut() {
    n.on_scroll.push(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
  }

  tree.scroll_event(&[]);

  let evs = received.lock().unwrap().clone();
  assert!(evs.contains(&"scroll".to_string()), "expected scroll, got {evs:?}");
}

#[test]
fn selectionchange_dispatches_on_root() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut tree = Tree::new(Node::new(m::Div::default()));
  if let Some(n) = tree.root.as_mut() {
    n.on_event.push(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
  }

  tree.selectionchange_event();

  assert!(received.lock().unwrap().contains(&"selectionchange".to_string()));
}

// ── Capture phase tests ──────────────────────────────────────────────────────

fn capture_test_tree() -> Tree {
  let span = Node::new(m::Span::default());
  let mut outer = Node::new(m::Div::default());
  outer.children.push(span);
  let mut body = Node::new(m::Body::default());
  body.children.push(outer);
  Tree::new(body)
}

fn install_mousedown_path_loggers(tree: &mut Tree, log: Arc<Mutex<Vec<usize>>>) {
  let paths = vec![vec![], vec![0], vec![0, 0]];
  for path in &paths {
    if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(path)) {
      let l = log.clone();
      let p = path.clone();
      node.on_mouse_down.push(Arc::new(move |_ev| {
        l.lock().unwrap().push(p.len());
      }));
    }
  }
}

fn install_capture_loggers(tree: &mut Tree, log: Arc<Mutex<Vec<(String, String, usize)>>>) {
  let paths = vec![vec![], vec![0], vec![0, 0]];
  for path in &paths {
    if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(path)) {
      let l = log.clone();
      let p = path.clone();
      node.on_event.push(Arc::new(move |ev| {
        let phase = format!("{:?}", ev.base().event_phase);
        l.lock().unwrap().push((ev.event_type().to_string(), phase, p.len()));
      }));
    }
  }
}

#[test]
fn mousedown_capture_phase_fires_root_first_then_target_then_bubble() {
  let mut tree = capture_test_tree();
  let log = Arc::new(Mutex::new(Vec::new()));
  install_capture_loggers(&mut tree, log.clone());

  tree.dispatch_mouse_down(Some(&[0, 0]), (10.0, 10.0), MouseButton::Primary, None);

  let events = log.lock().unwrap().clone();
  assert_eq!(
    events,
    vec![
      ("mousedown".to_string(), "CapturingPhase".to_string(), 0),
      ("mousedown".to_string(), "CapturingPhase".to_string(), 1),
      ("mousedown".to_string(), "AtTarget".to_string(), 2),
      ("mousedown".to_string(), "BubblingPhase".to_string(), 1),
      ("mousedown".to_string(), "BubblingPhase".to_string(), 0),
    ],
    "expected capture→target→bubble order for mousedown"
  );
}

#[test]
fn keydown_capture_phase_root_first() {
  let mut tree = capture_test_tree();
  let log = Arc::new(Mutex::new(Vec::new()));
  if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0, 0])) {
    if let Element::Span(s) = &mut node.element {
      s.tabindex = Some(0);
    }
  }
  install_capture_loggers(&mut tree, log.clone());
  tree.focus(Some(&[0, 0]));
  log.lock().unwrap().clear();

  tree.key_down("a", "KeyA", false);

  let events = log.lock().unwrap().clone();
  assert_eq!(events[0], ("keydown".to_string(), "CapturingPhase".to_string(), 0));
  assert_eq!(events[1], ("keydown".to_string(), "CapturingPhase".to_string(), 1));
  assert_eq!(events[2], ("keydown".to_string(), "AtTarget".to_string(), 2));
  assert_eq!(events[3], ("keydown".to_string(), "BubblingPhase".to_string(), 1));
  assert_eq!(events[4], ("keydown".to_string(), "BubblingPhase".to_string(), 0));
}

#[test]
fn stoppropagation_in_capture_prevents_target_and_bubble() {
  let mut tree = capture_test_tree();
  let log = Arc::new(Mutex::new(Vec::new()));
  if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0])) {
    node.on_event.push(Arc::new(move |ev| {
      ev.stop_propagation();
    }));
  }
  let l = log.clone();
  for path in [vec![], vec![0], vec![0, 0]] {
    if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&path)) {
      let p = path.clone();
      let ll = l.clone();
      node.on_event.push(Arc::new(move |ev| {
        let phase = format!("{:?}", ev.base().event_phase);
        ll.lock().unwrap().push((ev.event_type().to_string(), phase, p.len()));
      }));
    }
  }

  tree.dispatch_mouse_down(Some(&[0, 0]), (10.0, 10.0), MouseButton::Primary, None);

  let events = log.lock().unwrap().clone();
  assert_eq!(
    events,
    vec![
      ("mousedown".to_string(), "CapturingPhase".to_string(), 0),
      ("mousedown".to_string(), "CapturingPhase".to_string(), 1),
    ],
    "stopPropagation during capture should prevent target and bubble phases"
  );
}

#[test]
fn stoppropagation_at_target_prevents_bubble_phase() {
  let mut tree = capture_test_tree();
  let log = Arc::new(Mutex::new(Vec::new()));
  if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0, 0])) {
    let l = log.clone();
    node.on_event.push(Arc::new(move |ev| {
      let phase = format!("{:?}", ev.base().event_phase);
      l.lock().unwrap().push((ev.event_type().to_string(), phase, 2));
      ev.stop_propagation();
    }));
  }
  for path in [vec![], vec![0]] {
    if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&path)) {
      let l = log.clone();
      let p = path.clone();
      node.on_event.push(Arc::new(move |ev| {
        let phase = format!("{:?}", ev.base().event_phase);
        l.lock().unwrap().push((ev.event_type().to_string(), phase, p.len()));
      }));
    }
  }

  tree.dispatch_mouse_down(Some(&[0, 0]), (10.0, 10.0), MouseButton::Primary, None);

  let events = log.lock().unwrap().clone();
  assert_eq!(events.len(), 3);
  assert_eq!(events[0], ("mousedown".to_string(), "CapturingPhase".to_string(), 0));
  assert_eq!(events[1], ("mousedown".to_string(), "CapturingPhase".to_string(), 1));
  assert_eq!(events[2], ("mousedown".to_string(), "AtTarget".to_string(), 2));
}

#[test]
fn scroll_event_fires_capture_on_ancestors() {
  let mut tree = capture_test_tree();
  let log = Arc::new(Mutex::new(Vec::new()));
  for path in [vec![], vec![0], vec![0, 0]] {
    if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&path)) {
      let l = log.clone();
      let p = path.clone();
      node.on_event.push(Arc::new(move |ev| {
        let phase = format!("{:?}", ev.base().event_phase);
        l.lock().unwrap().push((ev.event_type().to_string(), phase, p.len()));
      }));
    }
  }

  tree.scroll_event(&[0, 0]);

  let events = log.lock().unwrap().clone();
  assert_eq!(
    events,
    vec![
      ("scroll".to_string(), "CapturingPhase".to_string(), 0),
      ("scroll".to_string(), "CapturingPhase".to_string(), 1),
      ("scroll".to_string(), "AtTarget".to_string(), 2),
    ],
    "scroll should fire capture on ancestors then target"
  );
}

#[test]
fn focus_event_capture_fires_on_non_bubbling_event() {
  let mut tree = capture_test_tree();
  if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0, 0])) {
    if let Element::Span(s) = &mut node.element {
      s.tabindex = Some(0);
    }
  }
  let log = Arc::new(Mutex::new(Vec::new()));
  for path in [vec![], vec![0], vec![0, 0]] {
    if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&path)) {
      let l = log.clone();
      let p = path.clone();
      node.on_event.push(Arc::new(move |ev| {
        let phase = format!("{:?}", ev.base().event_phase);
        l.lock().unwrap().push((ev.event_type().to_string(), phase, p.len()));
      }));
    }
  }

  tree.focus(Some(&[0, 0]));

  let events: Vec<_> = log
    .lock()
    .unwrap()
    .iter()
    .filter(|(t, ..)| t == "focusin")
    .cloned()
    .collect();
  assert_eq!(events.len(), 5);
  assert_eq!(events[0], ("focusin".to_string(), "CapturingPhase".to_string(), 0));
  assert_eq!(events[1], ("focusin".to_string(), "CapturingPhase".to_string(), 1));
  assert_eq!(events[2], ("focusin".to_string(), "AtTarget".to_string(), 2));
  assert_eq!(events[3], ("focusin".to_string(), "BubblingPhase".to_string(), 1));
  assert_eq!(events[4], ("focusin".to_string(), "BubblingPhase".to_string(), 0));
}

#[test]
fn submit_event_capture_phase_root_first() {
  let log = Arc::new(Mutex::new(Vec::new()));
  let mut input = m::Input::default();
  input.r#type = Some(InputType::Text);
  let mut form_node = Node::new(m::Form::default());
  form_node.children.push(Node::new(input));
  {
    let l = log.clone();
    form_node.on_event.push(Arc::new(move |ev| {
      let phase = format!("{:?}", ev.base().event_phase);
      l.lock().unwrap().push((ev.event_type().to_string(), phase, 0));
    }));
  }
  let mut tree = Tree::new(form_node);

  tree.focus(Some(&[0]));
  log.lock().unwrap().clear();

  tree.key_down("Enter", "Enter", false);

  let events: Vec<_> = log
    .lock()
    .unwrap()
    .iter()
    .filter(|(t, ..)| t == "submit")
    .cloned()
    .collect();
  assert_eq!(events.len(), 1);
  assert_eq!(events[0], ("submit".to_string(), "AtTarget".to_string(), 0));
}

#[test]
fn dedicated_slots_receive_mousedown_in_capture_to_bubble_order() {
  let mut tree = capture_test_tree();
  let log = Arc::new(Mutex::new(Vec::new()));
  install_mousedown_path_loggers(&mut tree, log.clone());

  tree.dispatch_mouse_down(Some(&[0, 0]), (10.0, 10.0), MouseButton::Primary, None);

  let events = log.lock().unwrap().clone();
  assert_eq!(
    events,
    vec![0, 1, 2, 1, 0],
    "dedicated mouse callbacks should fire capture→target→bubble"
  );
}
