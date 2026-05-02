use super::*;

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
  // Quad / glyph / image rects all shift uniformly.
  assert_eq!(shifted.quads[0].rect, Rect::new(5.0, 5.0, 30.0, 30.0));
  assert_eq!(shifted.glyphs[0].rect, Rect::new(6.0, 6.0, 8.0, 12.0));
  assert_eq!(shifted.images[0].rect, Rect::new(10.0, 10.0, 40.0, 40.0));
  // The clip's rect shifts with them; clips with `None` stay
  // `None`.
  let scissored = shifted
    .clips
    .iter()
    .find_map(|c| c.rect)
    .expect("the pushed clip should still be present");
  assert_eq!(scissored, Rect::new(0.0, 0.0, 100.0, 100.0));
  // Original list is untouched.
  assert_eq!(list.quads[0].rect, Rect::new(15.0, 25.0, 30.0, 30.0));
}

#[test]
fn finalize_remaps_command_clip_index_when_empty_ranges_dropped() {
  // Mirrors the textarea-overflow:auto bug: after the parent's
  // text run is pushed (clip_index 0), an `overflow:auto` push
  // opens a new clip range that nothing pushes commands into,
  // and a pop opens a third "post-textarea" range that
  // accumulates the rest of the document. Retain drops the
  // empty middle range; the post-textarea commands are pinned
  // to clip_index 2, but after retain only two clip slots
  // exist. They must be remapped to 1 so the renderer's
  // per-slot bookkeeping can find them.
  let mut list = DisplayList::new();
  // Pre-textarea content (clip_index 0).
  list.push_glyph(
    Rect::new(0.0, 0.0, 8.0, 12.0),
    [0.0, 0.0, 0.0, 1.0],
    [0.0, 0.0],
    [1.0, 1.0],
  );
  // Open the empty `overflow: auto` clip and immediately close
  // it without pushing anything inside.
  list.push_clip(Some(Rect::new(0.0, 0.0, 320.0, 64.0)), [0.0; 4], [0.0; 4]);
  list.pop_clip(None, [0.0; 4], [0.0; 4]);
  // Post-textarea content lands on clip_index 2 in the raw,
  // pre-finalize numbering.
  list.push_glyph(
    Rect::new(100.0, 100.0, 8.0, 12.0),
    [1.0, 1.0, 1.0, 1.0],
    [0.0, 0.0],
    [1.0, 1.0],
  );
  list.push_quad(Rect::new(100.0, 100.0, 50.0, 20.0), [0.5, 0.5, 0.5, 1.0]);
  // Right before finalize the post-textarea command should
  // sit in a clip slot beyond index 0.
  let posttext_index_pre = list.commands.last().unwrap().clip_index;
  assert!(posttext_index_pre >= 1);

  list.finalize();

  // Empty middle range is gone.
  assert_eq!(list.clips.len(), 2, "empty middle clip should have been retained out");
  // Every command must reference an in-bounds slot.
  let max = list.clips.len() as u32;
  for cmd in &list.commands {
    assert!(
      cmd.clip_index < max,
      "command {:?} still points past the trimmed clip table (len={max})",
      cmd
    );
  }
  // Post-textarea commands are remapped to the surviving
  // post-textarea slot (now index 1), and that slot's glyph
  // range covers the post-textarea glyph.
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
