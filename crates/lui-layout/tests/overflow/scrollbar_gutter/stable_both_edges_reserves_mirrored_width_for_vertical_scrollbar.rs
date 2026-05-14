use lui_cascade::{cascade::InteractionState, media::MediaContext};
use lui_layout::engine::layout_tree;

use crate::helpers::{find_by_tag, flex_lt};

#[test]
fn scrollbar_gutter_stable_both_edges_reserves_mirrored_width_for_vertical_scrollbar() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-y:auto; scrollbar-gutter:stable both-edges">
            <div style="height:50px">fits</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();

  assert!(
    (container.content.width - 170.0).abs() < 1.0,
    "stable both-edges should reserve two 15px vertical gutters, got {}",
    container.content.width
  );
  assert!(
    (container.content.x - 15.0).abs() < 1.0,
    "stable both-edges should shift content by one gutter width, got {}",
    container.content.x
  );
}
