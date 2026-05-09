use wgpu_html::{layout_at_path, select_all_text, select_line_at_cursor, select_word_at_cursor, selected_text};
use wgpu_html_layout::{LayoutBox, Rect as LR, UserSelect};
use wgpu_html_models::common::PointerEvents;
use wgpu_html_tree::{Node, TextCursor, TextSelection, Tree};

fn text_box(text: &str, x: f32) -> LayoutBox {
  let r = LR::new(x, 0.0, 100.0, 20.0);
  let glyphs = text
    .chars()
    .enumerate()
    .map(|(i, _)| wgpu_html_text::PositionedGlyph {
      x: i as f32 * 8.0,
      y: 0.0,
      w: 8.0,
      h: 16.0,
      uv_min: [0.0, 0.0],
      uv_max: [1.0, 1.0],
      color: [0.0, 0.0, 0.0, 1.0],
    })
    .collect();
  LayoutBox {
    margin_rect: r,
    border_rect: r,
    content_rect: r,
    background: None,
    background_rect: r,
    background_radii: wgpu_html_layout::CornerRadii::zero(),
    border: wgpu_html_layout::Insets::zero(),
    border_colors: wgpu_html_layout::BorderColors::default(),
    border_styles: wgpu_html_layout::BorderStyles::default(),
    border_radius: wgpu_html_layout::CornerRadii::zero(),
    kind: wgpu_html_layout::BoxKind::Text,
    text_run: Some(wgpu_html_text::ShapedRun {
      glyphs,
      glyph_chars: vec![],
      lines: vec![wgpu_html_text::ShapedLine {
        top: 0.0,
        height: 20.0,
        glyph_range: (0, text.chars().count()),
      }],
      text: text.to_owned(),
      byte_boundaries: wgpu_html_text::utf8_boundaries(text),
      width: text.chars().count() as f32 * 8.0,
      height: 20.0,
      ascent: 14.0,
    }),
    text_color: Some([0.0, 0.0, 0.0, 1.0]),
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: wgpu_html_layout::OverflowAxes::visible(),
    resize: wgpu_html_layout::Resize::None,
    opacity: 1.0,
    pointer_events: PointerEvents::Auto,
    user_select: UserSelect::Auto,
    cursor: wgpu_html_layout::Cursor::Default,
    z_index: None,
    image: None,
    background_image: None,
    first_line_color: None,
    first_letter_color: None,
    selection_bg: None,
    selection_fg: None,
    accent_color: None,
    lui: wgpu_html_layout::LuiProperties::default(),
    lui_popup: None,
    lui_color_picker: None,
    lui_calendar: None,
    children: Vec::new(),
    is_fixed: false,
    form_control: None,
  }
}

#[test]
fn layout_at_path_walks_children() {
  let leaf_a = LayoutBox {
    margin_rect: LR::new(0.0, 0.0, 50.0, 20.0),
    border_rect: LR::new(0.0, 0.0, 50.0, 20.0),
    content_rect: LR::new(0.0, 0.0, 50.0, 20.0),
    background: None,
    background_rect: LR::new(0.0, 0.0, 50.0, 20.0),
    background_radii: wgpu_html_layout::CornerRadii::zero(),
    border: wgpu_html_layout::Insets::zero(),
    border_colors: wgpu_html_layout::BorderColors::default(),
    border_styles: wgpu_html_layout::BorderStyles::default(),
    border_radius: wgpu_html_layout::CornerRadii::zero(),
    kind: wgpu_html_layout::BoxKind::Block,
    text_run: None,
    text_color: None,
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: wgpu_html_layout::OverflowAxes::visible(),
    resize: wgpu_html_layout::Resize::None,
    opacity: 1.0,
    pointer_events: PointerEvents::Auto,
    user_select: UserSelect::Auto,
    cursor: wgpu_html_layout::Cursor::Default,
    z_index: None,
    image: None,
    background_image: None,
    first_line_color: None,
    first_letter_color: None,
    selection_bg: None,
    selection_fg: None,
    accent_color: None,
    lui: wgpu_html_layout::LuiProperties::default(),
    lui_popup: None,
    lui_color_picker: None,
    lui_calendar: None,
    children: Vec::new(),
    is_fixed: false,
    form_control: None,
  };
  let mut leaf_b = leaf_a.clone();
  leaf_b.border_rect = LR::new(50.0, 0.0, 50.0, 20.0);
  let mut root = leaf_a.clone();
  root.border_rect = LR::new(0.0, 0.0, 100.0, 20.0);
  root.children = vec![leaf_a.clone(), leaf_b.clone()];

  assert_eq!(layout_at_path(&root, &[]).unwrap().border_rect, root.border_rect);
  assert_eq!(layout_at_path(&root, &[0]).unwrap().border_rect, leaf_a.border_rect);
  assert_eq!(layout_at_path(&root, &[1]).unwrap().border_rect, leaf_b.border_rect);
  assert!(layout_at_path(&root, &[2]).is_none());
  assert!(layout_at_path(&root, &[0, 0]).is_none());
}

#[test]
fn select_word_at_cursor_selects_token() {
  let layout = text_box("hello, world", 0.0);
  let mut tree = Tree::new(Node::new("hello, world"));
  let cursor = TextCursor {
    path: vec![],
    glyph_index: 8,
  };

  assert!(select_word_at_cursor(&mut tree, &layout, &cursor));
  assert_eq!(selected_text(&tree, &layout).as_deref(), Some("world"));
  assert!(!tree.interaction.selecting_text);
}

fn text_box_with_spaces(text: &str, x: f32) -> LayoutBox {
  let r = LR::new(x, 0.0, 500.0, 20.0);
  let mut glyph_x = 0.0;
  let mut glyphs = Vec::new();
  let mut glyph_chars = Vec::new();
  for (char_idx, ch) in text.chars().enumerate() {
    if ch == ' ' {
      glyph_x += 4.0;
      continue;
    }
    glyphs.push(wgpu_html_text::PositionedGlyph {
      x: glyph_x,
      y: 0.0,
      w: 8.0,
      h: 16.0,
      uv_min: [0.0, 0.0],
      uv_max: [1.0, 1.0],
      color: [0.0, 0.0, 0.0, 1.0],
    });
    glyph_chars.push(char_idx);
    glyph_x += 8.0;
  }
  LayoutBox {
    margin_rect: r,
    border_rect: r,
    content_rect: r,
    background: None,
    background_rect: r,
    background_radii: wgpu_html_layout::CornerRadii::zero(),
    border: wgpu_html_layout::Insets::zero(),
    border_colors: wgpu_html_layout::BorderColors::default(),
    border_styles: wgpu_html_layout::BorderStyles::default(),
    border_radius: wgpu_html_layout::CornerRadii::zero(),
    kind: wgpu_html_layout::BoxKind::Text,
    text_run: Some(wgpu_html_text::ShapedRun {
      glyphs,
      glyph_chars,
      lines: vec![wgpu_html_text::ShapedLine {
        top: 0.0,
        height: 20.0,
        glyph_range: (0, text.chars().filter(|&c| c != ' ').count()),
      }],
      text: text.to_owned(),
      byte_boundaries: wgpu_html_text::utf8_boundaries(text),
      width: glyph_x,
      height: 20.0,
      ascent: 14.0,
    }),
    text_color: Some([0.0, 0.0, 0.0, 1.0]),
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: wgpu_html_layout::OverflowAxes::visible(),
    resize: wgpu_html_layout::Resize::None,
    opacity: 1.0,
    pointer_events: PointerEvents::Auto,
    user_select: UserSelect::Auto,
    cursor: wgpu_html_layout::Cursor::Default,
    z_index: None,
    image: None,
    background_image: None,
    first_line_color: None,
    first_letter_color: None,
    selection_bg: None,
    selection_fg: None,
    accent_color: None,
    lui: wgpu_html_layout::LuiProperties::default(),
    lui_popup: None,
    lui_color_picker: None,
    lui_calendar: None,
    children: Vec::new(),
    is_fixed: false,
    form_control: None,
  }
}

#[test]
fn select_word_skips_invisible_chars_correctly() {
  let text = "amet, consectetur";
  let layout = text_box_with_spaces(text, 0.0);
  let mut tree = Tree::new(Node::new(text));

  let _cursor = TextCursor {
    path: vec![],
    glyph_index: 6,
  };
  let cursor_c = TextCursor {
    path: vec![],
    glyph_index: 7,
  };
  assert!(select_word_at_cursor(&mut tree, &layout, &cursor_c));
  assert_eq!(
    selected_text(&tree, &layout).as_deref(),
    Some("consectetur"),
    "double-click on 'c' of 'consectetur' must select the whole word"
  );
}

#[test]
fn select_line_at_cursor_selects_line() {
  let layout = text_box("hello, world", 0.0);
  let mut tree = Tree::new(Node::new("hello, world"));
  let cursor = TextCursor {
    path: vec![],
    glyph_index: 4,
  };

  assert!(select_line_at_cursor(&mut tree, &layout, &cursor));
  assert_eq!(selected_text(&tree, &layout).as_deref(), Some("hello, world"));
  assert!(!tree.interaction.selecting_text);
}

#[test]
fn select_all_spans_first_to_last_text_box() {
  let root = LayoutBox {
    margin_rect: LR::new(0.0, 0.0, 200.0, 40.0),
    border_rect: LR::new(0.0, 0.0, 200.0, 40.0),
    content_rect: LR::new(0.0, 0.0, 200.0, 40.0),
    background: None,
    background_rect: LR::new(0.0, 0.0, 200.0, 40.0),
    background_radii: wgpu_html_layout::CornerRadii::zero(),
    border: wgpu_html_layout::Insets::zero(),
    border_colors: wgpu_html_layout::BorderColors::default(),
    border_styles: wgpu_html_layout::BorderStyles::default(),
    border_radius: wgpu_html_layout::CornerRadii::zero(),
    kind: wgpu_html_layout::BoxKind::Block,
    text_run: None,
    text_color: None,
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: wgpu_html_layout::OverflowAxes::visible(),
    resize: wgpu_html_layout::Resize::None,
    opacity: 1.0,
    pointer_events: PointerEvents::Auto,
    user_select: UserSelect::Auto,
    cursor: wgpu_html_layout::Cursor::Default,
    z_index: None,
    image: None,
    background_image: None,
    first_line_color: None,
    first_letter_color: None,
    selection_bg: None,
    selection_fg: None,
    accent_color: None,
    lui: wgpu_html_layout::LuiProperties::default(),
    lui_popup: None,
    lui_color_picker: None,
    lui_calendar: None,
    children: vec![text_box("Hello", 0.0), text_box("World", 0.0)],
    is_fixed: false,
    form_control: None,
  };
  let mut tree = Tree::new(Node::new("root"));
  assert!(select_all_text(&mut tree, &root));
  let sel = tree.interaction.selection.expect("selection");
  assert_eq!(sel.anchor.path, vec![0]);
  assert_eq!(sel.anchor.glyph_index, 0);
  assert_eq!(sel.focus.path, vec![1]);
  assert_eq!(sel.focus.glyph_index, 5);
}

#[test]
fn selected_text_uses_newlines_between_different_parents_and_not_within_same_parent() {
  let inline_parent = LayoutBox {
    margin_rect: LR::new(0.0, 0.0, 200.0, 20.0),
    border_rect: LR::new(0.0, 0.0, 200.0, 20.0),
    content_rect: LR::new(0.0, 0.0, 200.0, 20.0),
    background: None,
    background_rect: LR::new(0.0, 0.0, 200.0, 20.0),
    background_radii: wgpu_html_layout::CornerRadii::zero(),
    border: wgpu_html_layout::Insets::zero(),
    border_colors: wgpu_html_layout::BorderColors::default(),
    border_styles: wgpu_html_layout::BorderStyles::default(),
    border_radius: wgpu_html_layout::CornerRadii::zero(),
    kind: wgpu_html_layout::BoxKind::Block,
    text_run: None,
    text_color: None,
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: wgpu_html_layout::OverflowAxes::visible(),
    resize: wgpu_html_layout::Resize::None,
    opacity: 1.0,
    pointer_events: PointerEvents::Auto,
    user_select: UserSelect::Auto,
    cursor: wgpu_html_layout::Cursor::Default,
    z_index: None,
    image: None,
    background_image: None,
    first_line_color: None,
    first_letter_color: None,
    selection_bg: None,
    selection_fg: None,
    accent_color: None,
    lui: wgpu_html_layout::LuiProperties::default(),
    lui_popup: None,
    lui_color_picker: None,
    lui_calendar: None,
    children: vec![text_box("Hello ", 0.0), text_box("world", 48.0)],
    is_fixed: false,
    form_control: None,
  };
  let root = LayoutBox {
    margin_rect: LR::new(0.0, 0.0, 300.0, 60.0),
    border_rect: LR::new(0.0, 0.0, 300.0, 60.0),
    content_rect: LR::new(0.0, 0.0, 300.0, 60.0),
    background: None,
    background_rect: LR::new(0.0, 0.0, 300.0, 60.0),
    background_radii: wgpu_html_layout::CornerRadii::zero(),
    border: wgpu_html_layout::Insets::zero(),
    border_colors: wgpu_html_layout::BorderColors::default(),
    border_styles: wgpu_html_layout::BorderStyles::default(),
    border_radius: wgpu_html_layout::CornerRadii::zero(),
    kind: wgpu_html_layout::BoxKind::Block,
    text_run: None,
    text_color: None,
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: wgpu_html_layout::OverflowAxes::visible(),
    resize: wgpu_html_layout::Resize::None,
    opacity: 1.0,
    pointer_events: PointerEvents::Auto,
    user_select: UserSelect::Auto,
    cursor: wgpu_html_layout::Cursor::Default,
    z_index: None,
    image: None,
    background_image: None,
    first_line_color: None,
    first_letter_color: None,
    selection_bg: None,
    selection_fg: None,
    accent_color: None,
    lui: wgpu_html_layout::LuiProperties::default(),
    lui_popup: None,
    lui_color_picker: None,
    lui_calendar: None,
    children: vec![inline_parent, text_box("Second", 0.0)],
    is_fixed: false,
    form_control: None,
  };
  let mut tree = Tree::new(Node::new("root"));
  tree.interaction.selection = Some(TextSelection {
    anchor: TextCursor {
      path: vec![0, 0],
      glyph_index: 0,
    },
    focus: TextCursor {
      path: vec![1],
      glyph_index: 6,
    },
  });

  assert_eq!(selected_text(&tree, &root).as_deref(), Some("Hello world\nSecond"));
}
