use lui_renderer::{DisplayCommand, DisplayCommandKind, DisplayList, Rect};

#[test]
fn translated_shifts_quads_glyphs_images_and_clips() {
  let mut list = DisplayList::new();
  list.push_clip(Some(Rect::new(10.0, 20.0, 100.0, 100.0)), [0.0; 4], [0.0; 4]);
  list.push_quad(Rect::new(15.0, 25.0, 30.0, 30.0), [1.0, 0.0, 0.0, 1.0]);
  list.push_glyph(
    Rect::new(16.0, 26.0, 8.0, 12.0),
    [0.0, 0.0, 0.0, 1.0],
    [0.0, 0.0],
    [1.0, 1.0],
  );
  list.push_image(
    Rect::new(20.0, 30.0, 40.0, 40.0),
    42,
    std::sync::Arc::new(vec![0u8; 4]),
    1,
    1,
  );
  list.finalize();

  let shifted = list.translated(-10.0, -20.0);
  assert_eq!(shifted.quads[0].rect, Rect::new(5.0, 5.0, 30.0, 30.0));
  assert_eq!(shifted.glyphs[0].rect, Rect::new(6.0, 6.0, 8.0, 12.0));
  assert_eq!(shifted.images[0].rect, Rect::new(10.0, 10.0, 40.0, 40.0));
  let scissored = shifted
    .clips
    .iter()
    .find_map(|c| c.rect)
    .expect("the pushed clip should still be present");
  assert_eq!(scissored, Rect::new(0.0, 0.0, 100.0, 100.0));
  assert_eq!(list.quads[0].rect, Rect::new(15.0, 25.0, 30.0, 30.0));
}

#[test]
fn finalize_remaps_command_clip_index_when_empty_ranges_dropped() {
  let mut list = DisplayList::new();
  list.push_glyph(
    Rect::new(0.0, 0.0, 8.0, 12.0),
    [0.0, 0.0, 0.0, 1.0],
    [0.0, 0.0],
    [1.0, 1.0],
  );
  list.push_clip(Some(Rect::new(0.0, 0.0, 320.0, 64.0)), [0.0; 4], [0.0; 4]);
  list.pop_clip(None, [0.0; 4], [0.0; 4]);
  list.push_glyph(
    Rect::new(100.0, 100.0, 8.0, 12.0),
    [1.0, 1.0, 1.0, 1.0],
    [0.0, 0.0],
    [1.0, 1.0],
  );
  list.push_quad(Rect::new(100.0, 100.0, 50.0, 20.0), [0.5, 0.5, 0.5, 1.0]);
  let posttext_index_pre = list.commands.last().unwrap().clip_index;
  assert!(posttext_index_pre >= 1);

  list.finalize();

  assert_eq!(list.clips.len(), 2, "empty middle clip should have been retained out");
  let max = list.clips.len() as u32;
  for cmd in &list.commands {
    assert!(
      cmd.clip_index < max,
      "command {:?} still points past the trimmed clip table (len={max})",
      cmd
    );
  }
  let last_glyph_cmd = list
    .commands
    .iter()
    .rev()
    .find(|c| c.kind == DisplayCommandKind::Glyph)
    .unwrap();
  assert_eq!(last_glyph_cmd.clip_index, 1);
  let slot = list.clips[last_glyph_cmd.clip_index as usize];
  assert!(
    last_glyph_cmd.index >= slot.glyph_range.0 && last_glyph_cmd.index < slot.glyph_range.1,
    "remapped slot {:?} should contain glyph index {}",
    slot.glyph_range,
    last_glyph_cmd.index
  );
}

#[test]
fn display_commands_preserve_cross_type_push_order() {
  let mut list = DisplayList::new();
  list.push_glyph(
    Rect::new(0.0, 0.0, 10.0, 10.0),
    [0.0, 0.0, 0.0, 1.0],
    [0.0, 0.0],
    [1.0, 1.0],
  );
  list.push_quad(Rect::new(0.0, 0.0, 20.0, 20.0), [1.0, 0.0, 0.0, 1.0]);
  list.push_glyph(
    Rect::new(0.0, 0.0, 10.0, 10.0),
    [1.0, 1.0, 1.0, 1.0],
    [0.0, 0.0],
    [1.0, 1.0],
  );

  assert_eq!(
    list.commands,
    vec![
      DisplayCommand {
        kind: DisplayCommandKind::Glyph,
        index: 0,
        clip_index: 0,
      },
      DisplayCommand {
        kind: DisplayCommandKind::Quad,
        index: 0,
        clip_index: 0,
      },
      DisplayCommand {
        kind: DisplayCommandKind::Glyph,
        index: 1,
        clip_index: 0,
      },
    ]
  );
}
