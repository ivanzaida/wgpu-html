use wgpu_html::{paint::*, renderer::DisplayList};
use wgpu_html_layout::LayoutBox;
use wgpu_html_tree::{SelectionColors, TextCursor, TextSelection};

fn synthetic_text_layout() -> LayoutBox {
  let r = wgpu_html_layout::Rect::new(10.0, 20.0, 100.0, 24.0);
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
      glyphs: vec![
        wgpu_html_text::PositionedGlyph {
          x: 0.0,
          y: 4.0,
          w: 8.0,
          h: 14.0,
          uv_min: [0.0, 0.0],
          uv_max: [1.0, 1.0],
          color: [0.1, 0.2, 0.3, 1.0],
        },
        wgpu_html_text::PositionedGlyph {
          x: 8.0,
          y: 4.0,
          w: 8.0,
          h: 14.0,
          uv_min: [0.0, 0.0],
          uv_max: [1.0, 1.0],
          color: [0.2, 0.3, 0.4, 1.0],
        },
        wgpu_html_text::PositionedGlyph {
          x: 16.0,
          y: 4.0,
          w: 8.0,
          h: 14.0,
          uv_min: [0.0, 0.0],
          uv_max: [1.0, 1.0],
          color: [0.3, 0.4, 0.5, 1.0],
        },
      ],
      lines: vec![wgpu_html_text::ShapedLine {
        top: 0.0,
        height: 22.0,
        glyph_range: (0, 3),
      }],
      glyph_chars: vec![],
      text: "abc".to_string(),
      byte_boundaries: wgpu_html_text::utf8_boundaries("abc"),
      width: 24.0,
      height: 22.0,
      ascent: 10.0,
    }),
    text_color: Some([0.0, 0.0, 0.0, 1.0]),
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: wgpu_html_layout::OverflowAxes::visible(),
    resize: wgpu_html_layout::Resize::None,
    opacity: 1.0,
    pointer_events: wgpu_html_models::common::PointerEvents::Auto,
    user_select: wgpu_html_models::common::UserSelect::Auto,
    cursor: wgpu_html_models::common::Cursor::Auto,
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
fn selection_paints_background_and_overrides_glyph_color() {
  let root = synthetic_text_layout();
  let mut list = DisplayList::new();
  let selection = TextSelection {
    anchor: TextCursor {
      path: vec![],
      glyph_index: 1,
    },
    focus: TextCursor {
      path: vec![],
      glyph_index: 3,
    },
  };
  let colors = SelectionColors {
    background: [0.9, 0.8, 0.1, 0.4],
    foreground: [1.0, 1.0, 1.0, 1.0],
  };
  paint_layout_with_selection(&root, &mut list, Some(&selection), colors, 0.0);
  list.finalize();

  assert_eq!(list.quads.len(), 1, "single line emits one merged highlight span");
  assert_eq!(list.quads[0].color, colors.background);
  assert_eq!(list.quads[0].rect.y, 20.0, "selection starts at line top");
  assert_eq!(
    list.quads[0].rect.h, 22.0,
    "selection uses line height, not glyph height"
  );
  assert_eq!(list.glyphs.len(), 3);
  assert_eq!(list.glyphs[0].color, [0.1, 0.2, 0.3, 1.0]);
  assert_eq!(list.glyphs[1].color, colors.foreground);
  assert_eq!(list.glyphs[2].color, colors.foreground);
}
