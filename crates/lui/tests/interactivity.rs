use std::sync::{
  Arc, Mutex,
  atomic::{AtomicUsize, Ordering},
};

use lui::interactivity::*;
use lui_layout::LayoutBox;
use lui_models::common::{Cursor, PointerEvents, UserSelect};
use lui_tree::{MouseButton, Node, SelectionColors, Tree};

fn synthetic_text_layout() -> LayoutBox {
  let r = lui_layout::Rect::new(0.0, 0.0, 80.0, 20.0);
  LayoutBox {
    margin_rect: r,
    border_rect: r,
    content_rect: r,
    background: None,
    background_rect: r,
    background_radii: lui_layout::CornerRadii::zero(),
    border: lui_layout::Insets::zero(),
    border_colors: lui_layout::BorderColors::default(),
    border_styles: lui_layout::BorderStyles::default(),
    border_radius: lui_layout::CornerRadii::zero(),
    kind: lui_layout::BoxKind::Text,
    text_run: Some(lui_text::ShapedRun {
      glyphs: vec![
        lui_text::PositionedGlyph {
          x: 0.0,
          y: 0.0,
          w: 10.0,
          h: 16.0,
          uv_min: [0.0, 0.0],
          uv_max: [1.0, 1.0],
          color: [0.0, 0.0, 0.0, 1.0],
        },
        lui_text::PositionedGlyph {
          x: 10.0,
          y: 0.0,
          w: 10.0,
          h: 16.0,
          uv_min: [0.0, 0.0],
          uv_max: [1.0, 1.0],
          color: [0.0, 0.0, 0.0, 1.0],
        },
        lui_text::PositionedGlyph {
          x: 20.0,
          y: 0.0,
          w: 10.0,
          h: 16.0,
          uv_min: [0.0, 0.0],
          uv_max: [1.0, 1.0],
          color: [0.0, 0.0, 0.0, 1.0],
        },
      ],
      lines: vec![lui_text::ShapedLine {
        top: 0.0,
        height: 16.0,
        glyph_range: (0, 3),
      }],
      glyph_chars: vec![],
      text: "abc".to_string(),
      byte_boundaries: lui_text::utf8_boundaries("abc"),
      width: 30.0,
      height: 16.0,
      ascent: 12.0,
    }),
    text_color: Some([0.0, 0.0, 0.0, 1.0]),
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: lui_layout::OverflowAxes::visible(),
    resize: lui_layout::Resize::None,
    opacity: 1.0,
    pointer_events: PointerEvents::Auto,
    user_select: UserSelect::Auto,
    cursor: Cursor::Default,
    z_index: None,
    image: None,
    background_image: None,
    first_line_color: None,
    first_letter_color: None,
    selection_bg: None,
    selection_fg: None,
    accent_color: None,
    lui: lui_layout::LuiProperties::default(),
    lui_popup: None,
    lui_color_picker: None,
    lui_calendar: None,
    file_button: None,
    children: Vec::new(),
    is_fixed: false,
    form_control: None,
  }
}

fn make_tree(counter: Arc<AtomicUsize>) -> Tree {
  let mut root = Node::new("text");
  root.on_click.push(Arc::new(move |_| {
    counter.fetch_add(1, Ordering::Relaxed);
  }));
  let mut tree = Tree::new(root);
  tree.interaction.selection_colors = SelectionColors::default();
  tree
}

#[test]
fn drag_selection_suppresses_click() {
  let counter = Arc::new(AtomicUsize::new(0));
  let mut tree = make_tree(counter.clone());
  let lay = synthetic_text_layout();

  mouse_down(&mut tree, &lay, (1.0, 4.0), MouseButton::Primary);
  pointer_move(&mut tree, &lay, (26.0, 4.0));
  mouse_up(&mut tree, &lay, (26.0, 4.0), MouseButton::Primary);

  assert_eq!(counter.load(Ordering::Relaxed), 0);
  let sel = tree.interaction.selection.expect("selection");
  assert!(!sel.is_collapsed());
}

#[test]
fn collapsed_selection_keeps_click() {
  let counter = Arc::new(AtomicUsize::new(0));
  let mut tree = make_tree(counter.clone());
  let lay = synthetic_text_layout();

  mouse_down(&mut tree, &lay, (1.0, 4.0), MouseButton::Primary);
  mouse_up(&mut tree, &lay, (1.0, 4.0), MouseButton::Primary);

  assert_eq!(counter.load(Ordering::Relaxed), 1);
}

#[test]
fn double_click_selects_word() {
  let counter = Arc::new(AtomicUsize::new(0));
  let mut tree = make_tree(counter);
  let lay = synthetic_text_layout();

  mouse_down_with_click_count(&mut tree, &lay, (11.0, 4.0), MouseButton::Primary, 2);

  let sel = tree.interaction.selection.expect("selection");
  assert_eq!(sel.anchor.glyph_index, 0);
  assert_eq!(sel.focus.glyph_index, 3);
  assert!(!tree.interaction.selecting_text);
}

#[test]
fn edit_double_click_selects_word_and_triple_click_selects_line() {
  let cursor = edit_cursor_for_click_count("one two\nthree", 5, 2);
  assert_eq!(cursor.selection_range(), (4, 7));

  let cursor = edit_cursor_for_click_count("one two\nthree", 5, 3);
  assert_eq!(cursor.selection_range(), (0, 7));
}

#[test]
fn pointer_move_then_leave_via_layout() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut root = Node::new("text");
  root.on_event.push(Arc::new(move |ev| {
    r.lock().unwrap().push(ev.event_type().to_string());
  }));
  let lay = synthetic_text_layout();
  let mut tree = Tree::new(root);

  pointer_move(&mut tree, &lay, (5.0, 5.0));
  pointer_leave(&mut tree);

  let evs = received.lock().unwrap().clone();
  assert!(evs.contains(&"mouseenter".into()), "got {evs:?}");
  assert!(evs.contains(&"mouseleave".into()), "got {evs:?}");
}
