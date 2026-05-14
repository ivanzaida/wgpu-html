use lui_cascade::{cascade::InteractionState, media::MediaContext};
use lui_layout::{BoxKind, engine::layout_tree};

use crate::helpers::*;

macro_rules! cross_test {
  ($name:ident, $html:expr, $vw:expr, |$body:ident| $test:block) => {
    #[test]
    fn $name() {
      let (doc, ctx) = flex_lt($html, $vw);
      let media = MediaContext::default();
      let interaction = InteractionState::default();
      let styled = ctx.cascade(&doc.root, &media, &interaction);
      let tree = layout_tree(&styled, $vw, 600.0);
      let $body = find_by_tag(&tree.root, "body").unwrap();
      $test
    }
  };
  ($name:ident, $html:expr, $vw:expr, |$body:ident, $tree:ident| $test:block) => {
    #[test]
    fn $name() {
      let (doc, ctx) = flex_lt($html, $vw);
      let media = MediaContext::default();
      let interaction = InteractionState::default();
      let styled = ctx.cascade(&doc.root, &media, &interaction);
      let $tree = layout_tree(&styled, $vw, 600.0);
      let $body = find_by_tag(&$tree.root, "body").unwrap();
      $test
    }
  };
}

// ============================================================================
// Flex inside Block
// ============================================================================

cross_test!(
  flex_inside_block_inherits_width,
  r#"<div style="width:400px">
    <div style="display:flex"><div style="width:100px; height:50px">A</div></div>
</div>"#,
  800.0,
  |body| {
    let block = &body.children[0];
    assert!(
      (block.content.width - 400.0).abs() < 1.0,
      "block w=400, got {}",
      block.content.width
    );
    let flex = &block.children[0];
    assert_eq!(flex.kind, BoxKind::FlexContainer);
    assert!(
      (flex.content.width - 400.0).abs() < 1.0,
      "block-level flex fills parent 400, got {}",
      flex.content.width
    );
    assert!(
      (flex.children[0].content.width - 100.0).abs() < 1.0,
      "child A w=100, got {}",
      flex.children[0].content.width
    );
  }
);

cross_test!(
  flex_inside_block_margin_auto_centers,
  r#"<div style="width:400px">
    <div style="display:flex; width:200px; margin:0 auto">
        <div style="width:100px; height:50px">A</div>
    </div>
</div>"#,
  800.0,
  |body| {
    let block = &body.children[0];
    let flex = &block.children[0];
    assert_eq!(flex.kind, BoxKind::FlexContainer);
    assert!(
      (flex.content.width - 200.0).abs() < 1.0,
      "flex w=200, got {}",
      flex.content.width
    );
    let flex_center = flex.content.x + flex.content.width / 2.0;
    let block_center = block.content.x + block.content.width / 2.0;
    assert!(
      (flex_center - block_center).abs() < 2.0,
      "margin:auto should center flex. flex_center={}, block_center={}",
      flex_center,
      block_center
    );
  }
);

cross_test!(
  multiple_flex_in_block_stack,
  r#"<div style="width:400px">
    <div style="display:flex; height:50px"><div style="width:100px">A</div></div>
    <div style="display:flex; height:60px"><div style="width:100px">B</div></div>
</div>"#,
  800.0,
  |body| {
    let block = &body.children[0];
    assert!(
      block.children[1].content.y >= block.children[0].content.y + 49.0,
      "second below first"
    );
  }
);

// ============================================================================
// Grid inside Block
// ============================================================================

cross_test!(
  grid_inside_block_inherits_width,
  r#"<div style="width:400px">
    <div style="display:grid; grid-template-columns:1fr 1fr">
        <div style="height:50px">A</div><div style="height:50px">B</div>
    </div>
</div>"#,
  800.0,
  |body| {
    let grid = &body.children[0].children[0];
    assert_eq!(grid.kind, BoxKind::GridContainer);
    assert!(
      (grid.content.width - 400.0).abs() < 1.0,
      "grid w=400, got {}",
      grid.content.width
    );
    assert!(
      (grid.children[0].content.width - 200.0).abs() < 1.0,
      "1fr=200, got {}",
      grid.children[0].content.width
    );
  }
);

// ============================================================================
// Table inside Block
// ============================================================================

cross_test!(
  table_inside_block,
  r#"<div style="width:300px">
    <table style="width:200px"><tr><td style="height:30px">A</td><td style="height:30px">B</td></tr></table>
</div>"#,
  800.0,
  |body, tree| {
    let table = find_by_tag(&tree.root, "table").unwrap();
    assert!(
      (table.content.width - 200.0).abs() < 2.0,
      "table w=200, got {}",
      table.content.width
    );
  }
);

// ============================================================================
// Flex inside Flex (nested)
// ============================================================================

cross_test!(
  row_flex_inside_column_flex,
  r#"<div style="display:flex; flex-direction:column; width:500px">
    <div style="display:flex; gap:10px">
        <div style="width:100px; height:50px">A</div>
        <div style="width:100px; height:50px">B</div>
    </div>
</div>"#,
  800.0,
  |body| {
    let inner = &body.children[0].children[0];
    assert_eq!(inner.kind, BoxKind::FlexContainer);
    assert!(
      (inner.children[0].content.width - 100.0).abs() < 1.0,
      "A w=100, got {}",
      inner.children[0].content.width
    );
    assert!(
      (inner.children[1].content.width - 100.0).abs() < 1.0,
      "B w=100, got {}",
      inner.children[1].content.width
    );
    assert!(
      inner.children[1].content.x > inner.children[0].content.x + 100.0,
      "B right of A"
    );
  }
);

cross_test!(
  column_flex_inside_row_flex,
  r#"<div style="display:flex; width:400px">
    <div style="display:flex; flex-direction:column; flex:1">
        <div style="height:30px">top</div><div style="height:30px">bottom</div>
    </div>
    <div style="width:100px; height:100px">side</div>
</div>"#,
  800.0,
  |body| {
    let outer = &body.children[0];
    let col = &outer.children[0];
    assert!(
      (col.content.width - 300.0).abs() < 1.0,
      "col flex fill 300, got {}",
      col.content.width
    );
    assert!(
      col.children[1].content.y > col.children[0].content.y,
      "bottom below top"
    );
    assert!((outer.children[1].content.width - 100.0).abs() < 1.0, "side w=100");
  }
);

cross_test!(
  deeply_nested_flex_three_levels,
  r#"<div style="display:flex; flex-direction:column; width:600px; align-items:center">
    <div style="display:flex; gap:10px">
        <div style="display:flex; flex-direction:column; width:100px">
            <div style="height:20px">a1</div><div style="height:20px">a2</div>
        </div>
        <div style="display:flex; flex-direction:column; width:100px">
            <div style="height:20px">b1</div><div style="height:20px">b2</div>
        </div>
    </div>
</div>"#,
  800.0,
  |body| {
    let row = &body.children[0].children[0];
    assert!((row.children[0].content.width - 100.0).abs() < 1.0, "col_a w=100");
    assert!((row.children[1].content.width - 100.0).abs() < 1.0, "col_b w=100");
    assert!(
      row.children[1].content.x > row.children[0].content.x + 99.0,
      "col_b right of col_a"
    );
    assert!(
      row.children[0].children[1].content.y > row.children[0].children[0].content.y,
      "a2 below a1"
    );
  }
);

// ============================================================================
// Grid inside Flex
// ============================================================================

cross_test!(
  grid_inside_flex_item,
  r#"<div style="display:flex; width:400px">
    <div style="display:grid; grid-template-columns:1fr 1fr; flex:1">
        <div style="height:40px">A</div><div style="height:40px">B</div>
    </div>
    <div style="width:100px; height:100px">side</div>
</div>"#,
  800.0,
  |body| {
    let grid = &body.children[0].children[0];
    assert_eq!(grid.kind, BoxKind::GridContainer);
    assert!(
      (grid.content.width - 300.0).abs() < 2.0,
      "grid fills 300, got {}",
      grid.content.width
    );
    assert!(
      (grid.children[0].content.width - 150.0).abs() < 2.0,
      "col=150, got {}",
      grid.children[0].content.width
    );
  }
);

cross_test!(
  grid_inside_column_flex,
  r#"<div style="display:flex; flex-direction:column; width:400px">
    <div style="display:grid; grid-template-columns:100px 100px; gap:10px; height:50px">
        <div>A</div><div>B</div>
    </div>
    <div style="height:50px">below</div>
</div>"#,
  800.0,
  |body| {
    let outer = &body.children[0];
    assert_eq!(outer.children[0].kind, BoxKind::GridContainer);
    assert!(
      outer.children[1].content.y >= outer.children[0].content.y + 49.0,
      "below after grid"
    );
  }
);

// ============================================================================
// Flex inside Grid
// ============================================================================

cross_test!(
  flex_inside_grid_cell,
  r#"<div style="display:grid; grid-template-columns:200px 200px; width:400px">
    <div style="display:flex; gap:5px">
        <div style="width:50px; height:40px">A</div><div style="width:50px; height:40px">B</div>
    </div>
    <div style="height:40px">plain</div>
</div>"#,
  800.0,
  |body| {
    let flex = &body.children[0].children[0];
    assert_eq!(flex.kind, BoxKind::FlexContainer);
    assert!(
      (flex.content.width - 200.0).abs() < 1.0,
      "flex in grid cell=200, got {}",
      flex.content.width
    );
    assert!((flex.children[0].content.width - 50.0).abs() < 1.0, "A w=50");
    assert!(
      flex.children[1].content.x > flex.children[0].content.x + 49.0,
      "B right of A, B.x={} A.x={}",
      flex.children[1].content.x,
      flex.children[0].content.x
    );
  }
);

cross_test!(
  column_flex_inside_grid_cell,
  r#"<div style="display:grid; grid-template-columns:200px; width:200px">
    <div style="display:flex; flex-direction:column">
        <div style="height:30px">top</div><div style="height:30px">bottom</div>
    </div>
</div>"#,
  800.0,
  |body| {
    let flex = &body.children[0].children[0];
    assert!(
      flex.children[1].content.y > flex.children[0].content.y + 29.0,
      "bottom below top"
    );
  }
);

// ============================================================================
// Table inside Flex
// ============================================================================

cross_test!(
  table_inside_row_flex,
  r#"<div style="display:flex; width:500px">
    <table style="width:300px"><tr><td style="height:30px">A</td><td style="height:30px">B</td></tr></table>
    <div style="width:100px; height:50px">side</div>
</div>"#,
  800.0,
  |body| {
    let flex = &body.children[0];
    assert_eq!(flex.children[0].kind, BoxKind::Table);
    assert!(
      flex.children[1].content.x > flex.children[0].content.x + 100.0,
      "side right of table"
    );
  }
);

cross_test!(
  table_inside_column_flex,
  r#"<div style="display:flex; flex-direction:column; width:400px">
    <div style="height:40px">header</div>
    <table style="width:300px"><tr><td style="height:30px">A</td><td style="height:30px">B</td></tr></table>
</div>"#,
  800.0,
  |body, tree| {
    let header = &body.children[0].children[0];
    let table = find_by_tag(&tree.root, "table").unwrap();
    assert!(table.content.y >= header.content.y + 39.0, "table below header");
  }
);

// ============================================================================
// Flex inside Table cell
// ============================================================================

cross_test!(
  flex_inside_table_cell,
  r#"<table style="width:400px"><tr>
    <td style="height:60px">
        <div style="display:flex; gap:5px">
            <div style="width:50px; height:30px">A</div><div style="width:50px; height:30px">B</div>
        </div>
    </td>
    <td style="height:60px">plain</td>
</tr></table>"#,
  800.0,
  |body, tree| {
    let table = find_by_tag(&tree.root, "table").unwrap();
    let cell = &table.children[0].children[0];
    let flex = &cell.children[0];
    assert_eq!(flex.kind, BoxKind::FlexContainer);
    assert!((flex.children[0].content.width - 50.0).abs() < 1.0, "A w=50");
    assert!(
      flex.children[1].content.x > flex.children[0].content.x + 49.0,
      "B right of A"
    );
  }
);

// ============================================================================
// Grid inside Table cell
// ============================================================================

cross_test!(
  grid_inside_table_cell,
  r#"<table style="width:400px"><tr><td>
    <div style="display:grid; grid-template-columns:1fr 1fr">
        <div style="height:30px">A</div><div style="height:30px">B</div>
    </div>
</td></tr></table>"#,
  800.0,
  |body, tree| {
    let table = find_by_tag(&tree.root, "table").unwrap();
    let grid = &table.children[0].children[0].children[0];
    assert_eq!(grid.kind, BoxKind::GridContainer);
    assert!(
      grid.children[1].content.x > grid.children[0].content.x + 10.0,
      "columns side by side"
    );
  }
);

// ============================================================================
// Table inside Grid
// ============================================================================

cross_test!(
  table_inside_grid_cell,
  r#"<div style="display:grid; grid-template-columns:300px 100px; width:400px">
    <table><tr><td style="height:30px">A</td><td style="height:30px">B</td></tr></table>
    <div style="height:30px">side</div>
</div>"#,
  800.0,
  |body| {
    let table = &body.children[0].children[0];
    assert_eq!(table.kind, BoxKind::Table);
    assert!(
      (table.content.width - 300.0).abs() < 2.0,
      "table fills 300px col, got {}",
      table.content.width
    );
  }
);

// ============================================================================
// Grid inside Grid (nested)
// ============================================================================

cross_test!(
  nested_grids,
  r#"<div style="display:grid; grid-template-columns:300px; width:300px">
    <div style="display:grid; grid-template-columns:100px 100px; gap:10px">
        <div style="height:30px">A</div><div style="height:30px">B</div>
    </div>
</div>"#,
  800.0,
  |body| {
    let inner = &body.children[0].children[0];
    assert_eq!(inner.kind, BoxKind::GridContainer);
    assert_eq!(inner.children.len(), 2, "inner grid has 2 children");
    assert!(
      (inner.children[0].content.width - 100.0).abs() < 1.0,
      "col A=100, got {}",
      inner.children[0].content.width
    );
    assert!(
      (inner.children[1].content.width - 100.0).abs() < 1.0,
      "col B=100, got {}",
      inner.children[1].content.width
    );
    assert!(
      inner.children[1].content.x > inner.children[0].content.x + 99.0,
      "B right of A"
    );
  }
);

// ============================================================================
// Positioned inside Flex
// ============================================================================

cross_test!(
  absolute_in_flex_removed_from_flow,
  r#"<div style="display:flex; width:300px; position:relative">
    <div style="width:100px; height:50px">A</div>
    <div style="position:absolute; top:10px; left:10px; width:50px; height:50px">abs</div>
    <div style="width:100px; height:50px">B</div>
</div>"#,
  800.0,
  |body| {
    let flex = &body.children[0];
    let a = &flex.children[0];
    let b = &flex.children[1];
    let gap = (b.content.x - b.margin.left - b.border.left)
      - (a.content.x + a.content.width + a.padding.right + a.border.right);
    assert!(gap < 2.0, "A and B adjacent (abs out of flow), gap={}", gap);
  }
);

cross_test!(
  relative_in_flex_offsets,
  r#"<div style="display:flex; width:300px">
    <div style="width:100px; height:50px">A</div>
    <div style="width:100px; height:50px; position:relative; top:20px; left:10px">B</div>
</div>"#,
  800.0,
  |body| {
    let flex = &body.children[0];
    let a = &flex.children[0];
    let b = &flex.children[1];
    let natural_x = a.content.x + a.content.width;
    assert!(
      (b.content.x - natural_x - 10.0).abs() < 2.0,
      "relative left:10px should offset B, expected ~{}, got {}",
      natural_x + 10.0,
      b.content.x
    );
    let natural_y = a.content.y;
    assert!(
      (b.content.y - natural_y - 20.0).abs() < 2.0,
      "relative top:20px should offset B, expected ~{}, got {}",
      natural_y + 20.0,
      b.content.y
    );
  }
);

cross_test!(
  absolute_in_column_flex,
  r#"<div style="display:flex; flex-direction:column; width:300px; height:200px; position:relative">
    <div style="height:50px">A</div>
    <div style="position:absolute; bottom:0; right:0; width:40px; height:40px">abs</div>
    <div style="height:50px">B</div>
</div>"#,
  800.0,
  |body| {
    let flex = &body.children[0];
    assert!(
      flex.children[1].content.y >= flex.children[0].content.y + 49.0,
      "B follows A (abs out of flow)"
    );
  }
);

// ============================================================================
// Positioned inside Grid / Table
// ============================================================================

cross_test!(
  absolute_in_grid_cell,
  r#"<div style="display:grid; grid-template-columns:200px; width:200px">
    <div style="position:relative; height:100px">
        <div style="position:absolute; top:10px; left:10px; width:50px; height:50px">abs</div>
        <div style="height:30px">normal</div>
    </div>
</div>"#,
  800.0,
  |body| {
    let cell = &body.children[0].children[0];
    assert!(
      (cell.content.height - 100.0).abs() < 1.0,
      "cell h=100, got {}",
      cell.content.height
    );
  }
);

cross_test!(
  absolute_in_table_cell,
  r#"<table style="width:400px"><tr>
    <td style="position:relative; height:80px">
        <div style="position:absolute; top:5px; left:5px; width:30px; height:30px">abs</div>
        <div style="height:20px">normal</div>
    </td>
</tr></table>"#,
  800.0,
  |body, tree| {
    let cell = &find_by_tag(&tree.root, "table").unwrap().children[0].children[0];
    assert!(
      (cell.content.height - 80.0).abs() < 2.0,
      "cell h=80, got {}",
      cell.content.height
    );
  }
);

// ============================================================================
// Column flex + align-items combinations
// ============================================================================

cross_test!(
  column_flex_center_mixed_children,
  r#"<div style="display:flex; flex-direction:column; align-items:center; width:600px">
    <h1 style="font-size:32px">Title</h1>
    <p style="font-size:16px">Subtitle text here</p>
    <div style="display:flex; gap:10px">
        <div style="width:80px; height:80px">A</div><div style="width:80px; height:80px">B</div>
    </div>
</div>"#,
  800.0,
  |body| {
    let outer = &body.children[0];
    assert!(
      outer.children[0].content.width < 590.0,
      "h1 shrink-wraps, got w={}",
      outer.children[0].content.width
    );
    assert!(
      outer.children[1].content.width < 590.0,
      "p shrink-wraps, got w={}",
      outer.children[1].content.width
    );
    let row = &outer.children[2];
    assert!((row.children[0].content.width - 80.0).abs() < 1.0, "card A w=80");
    assert!((row.children[1].content.width - 80.0).abs() < 1.0, "card B w=80");
    assert!(outer.children[1].content.y > outer.children[0].content.y, "p below h1");
    assert!(outer.children[2].content.y > outer.children[1].content.y, "row below p");
  }
);

cross_test!(
  column_flex_end_shrinks_and_aligns,
  r#"<div style="display:flex; flex-direction:column; align-items:flex-end; width:600px">
    <div style="width:200px; height:40px">explicit</div>
    <div style="font-size:16px">text</div>
</div>"#,
  800.0,
  |body| {
    let outer = &body.children[0];
    assert!((outer.children[0].content.width - 200.0).abs() < 1.0, "explicit w=200");
    assert!(outer.children[1].content.width < 590.0, "text shrink-wraps");
    let r = outer.content.x + outer.content.width;
    let r0 = outer.children[0].content.x + outer.children[0].content.width;
    let r1 = outer.children[1].content.x + outer.children[1].content.width;
    assert!((r - r0).abs() < 2.0, "explicit right-aligned");
    assert!((r - r1).abs() < 2.0, "text right-aligned");
  }
);

cross_test!(
  column_flex_start_shrinks,
  r#"<div style="display:flex; flex-direction:column; align-items:flex-start; width:600px">
    <div style="font-size:16px">short</div>
</div>"#,
  800.0,
  |body| {
    let child = &body.children[0].children[0];
    assert!(
      child.content.width < 590.0,
      "shrink-wraps, got w={}",
      child.content.width
    );
  }
);

cross_test!(
  column_flex_stretch_fills,
  r#"<div style="display:flex; flex-direction:column; align-items:stretch; width:600px">
    <div style="height:40px">stretched</div>
</div>"#,
  800.0,
  |body| {
    assert!(
      (body.children[0].children[0].content.width - 600.0).abs() < 1.0,
      "stretch fills 600, got {}",
      body.children[0].children[0].content.width
    );
  }
);

cross_test!(
  column_flex_align_self_overrides,
  r#"<div style="display:flex; flex-direction:column; align-items:stretch; width:600px">
    <div style="height:40px">stretched</div>
    <div style="height:40px; align-self:center; font-size:16px">centered</div>
    <div style="height:40px; width:200px; align-self:flex-end">end</div>
</div>"#,
  800.0,
  |body| {
    let outer = &body.children[0];
    assert!((outer.children[0].content.width - 600.0).abs() < 1.0, "stretched fills");
    assert!(outer.children[1].content.width < 590.0, "centered shrink-wraps");
    let end_right = outer.children[2].content.x
      + outer.children[2].content.width
      + outer.children[2].padding.right
      + outer.children[2].border.right
      + outer.children[2].margin.right;
    assert!(
      (outer.content.x + outer.content.width - end_right).abs() < 2.0,
      "end right-aligned"
    );
  }
);

// ============================================================================
// Demo page integration test
// ============================================================================

cross_test!(
  demo_page_layout,
  r#"<div style="display:flex; flex-direction:column; align-items:center; padding:40px">
    <h1 style="font-size:32px; margin-bottom:16px">lui v2</h1>
    <p style="font-size:16px; margin-bottom:32px">HTML parse cascade layout paint wgpu</p>
    <div style="display:flex; gap:16px; margin-bottom:32px">
        <div style="width:120px; height:120px; display:flex; align-items:center; justify-content:center">
            <span style="font-size:14px">Block</span>
        </div>
        <div style="width:120px; height:120px; display:flex; align-items:center; justify-content:center">
            <span style="font-size:14px">Flex</span>
        </div>
        <div style="width:120px; height:120px; display:flex; align-items:center; justify-content:center">
            <span style="font-size:14px">Grid</span>
        </div>
    </div>
    <table style="width:400px; border-spacing:2px">
        <tr><td style="padding:8px; height:20px">Feature</td><td style="padding:8px; height:20px">Status</td></tr>
        <tr><td style="padding:8px; height:18px">Layout</td><td style="padding:8px; height:18px">Done</td></tr>
    </table>
</div>"#,
  800.0,
  |body, tree| {
    let outer = &body.children[0];
    assert_eq!(outer.kind, BoxKind::FlexContainer);
    let mut prev_bottom = 0.0_f32;
    for (i, child) in outer.children.iter().enumerate() {
      if i > 0 {
        let top = child.content.y - child.padding.top - child.border.top - child.margin.top;
        assert!(
          top >= prev_bottom - 1.0,
          "child[{}] overlaps: top={}, prev_bottom={}",
          i,
          top,
          prev_bottom
        );
      }
      prev_bottom =
        child.content.y + child.content.height + child.padding.bottom + child.border.bottom + child.margin.bottom;
    }
    let card_row = outer
      .children
      .iter()
      .find(|c| {
        c.kind == BoxKind::FlexContainer
          && c.children.len() == 3
          && c.children.iter().all(|card| (card.content.width - 120.0).abs() < 2.0)
      })
      .expect("card row with 3x 120px children");
    for (i, card) in card_row.children.iter().enumerate() {
      assert!(
        (card.content.width - 120.0).abs() < 1.0,
        "card[{}] w=120, got {}",
        i,
        card.content.width
      );
      assert!(
        (card.content.height - 120.0).abs() < 1.0,
        "card[{}] h=120, got {}",
        i,
        card.content.height
      );
    }
    assert!(
      (card_row.children[0].content.y - card_row.children[2].content.y).abs() < 1.0,
      "cards share y"
    );
    let table = find_by_tag(&tree.root, "table").unwrap();
    assert!(
      (table.content.width - 400.0).abs() < 2.0,
      "table w=400, got {}",
      table.content.width
    );
  }
);

// ============================================================================
// Width propagation through nesting
// ============================================================================

cross_test!(
  block_in_flex_in_grid,
  r#"<div style="display:grid; grid-template-columns:300px; width:300px">
    <div style="display:flex"><div style="flex:1"><div style="height:30px">content</div></div></div>
</div>"#,
  800.0,
  |body| {
    let item = &body.children[0].children[0].children[0];
    assert!(
      (item.content.width - 300.0).abs() < 2.0,
      "flex item fills 300px grid col, got {}",
      item.content.width
    );
  }
);

cross_test!(
  flex_in_table_cell_in_grid,
  r#"<div style="display:grid; grid-template-columns:400px; width:400px">
    <table style="width:400px"><tr><td>
        <div style="display:flex; gap:5px">
            <div style="width:50px; height:30px">A</div><div style="width:50px; height:30px">B</div>
        </div>
    </td></tr></table>
</div>"#,
  800.0,
  |body, tree| {
    let flex = &find_by_tag(&tree.root, "table").unwrap().children[0].children[0].children[0];
    assert_eq!(flex.kind, BoxKind::FlexContainer);
    assert!((flex.children[0].content.width - 50.0).abs() < 1.0, "A w=50");
    assert!(
      flex.children[1].content.x > flex.children[0].content.x + 49.0,
      "B right of A"
    );
  }
);

// ============================================================================
// Overflow interactions
// ============================================================================

cross_test!(
  overflow_hidden_flex,
  r#"<div style="display:flex; width:200px; overflow:hidden">
    <div style="width:300px; height:50px">overflows</div>
</div>"#,
  800.0,
  |body| {
    assert!(
      (body.children[0].content.width - 200.0).abs() < 1.0,
      "flex w=200, got {}",
      body.children[0].content.width
    );
  }
);

cross_test!(
  overflow_hidden_grid,
  r#"<div style="display:grid; grid-template-columns:300px; width:200px; overflow:hidden">
    <div style="height:50px">overflows</div>
</div>"#,
  800.0,
  |body| {
    assert!(
      (body.children[0].content.width - 200.0).abs() < 1.0,
      "grid w=200, got {}",
      body.children[0].content.width
    );
  }
);

// ============================================================================
// Height propagation across layout modes
// ============================================================================

cross_test!(
  flex_item_height_from_grid,
  r#"<div style="display:flex; width:400px">
    <div style="display:grid; grid-template-columns:100px; flex:1"><div style="height:80px">tall</div></div>
    <div style="width:100px">side</div>
</div>"#,
  800.0,
  |body| {
    assert!(
      body.children[0].children[0].content.height >= 79.0,
      "grid h propagates, got {}",
      body.children[0].children[0].content.height
    );
  }
);

cross_test!(
  grid_row_height_from_flex,
  r#"<div style="display:grid; grid-template-columns:200px 200px; width:400px">
    <div style="display:flex; flex-direction:column">
        <div style="height:40px">a</div><div style="height:40px">b</div>
    </div>
    <div>short</div>
</div>"#,
  800.0,
  |body| {
    assert!(
      body.children[0].children[0].content.height >= 79.0,
      "col flex h from children, got {}",
      body.children[0].children[0].content.height
    );
  }
);

// ============================================================================
// Float interactions
// ============================================================================

cross_test!(
  float_ignored_in_flex,
  r#"<div style="display:flex; width:300px">
    <div style="float:left; width:100px; height:50px">floated</div>
    <div style="width:100px; height:50px">normal</div>
</div>"#,
  800.0,
  |body| {
    assert!(body.children[0].children.len() >= 2, "both children present");
  }
);

// ============================================================================
// min/max constraints across boundaries
// ============================================================================

cross_test!(
  min_width_flex_in_grid,
  r#"<div style="display:grid; grid-template-columns:200px; width:200px">
    <div style="display:flex">
        <div style="min-width:150px; height:30px">wide</div>
        <div style="width:30px; height:30px">small</div>
    </div>
</div>"#,
  800.0,
  |body| {
    let wide = &body.children[0].children[0].children[0];
    assert!(
      wide.content.width >= 149.0,
      "min-width:150 respected, got {}",
      wide.content.width
    );
  }
);

cross_test!(
  max_width_grid_in_flex,
  r#"<div style="display:flex; width:400px">
    <div style="display:grid; grid-template-columns:1fr; max-width:200px; flex:1">
        <div style="height:30px">constrained</div>
    </div>
    <div style="width:100px; height:30px">side</div>
</div>"#,
  800.0,
  |body| {
    assert!(
      body.children[0].children[0].content.width <= 201.0,
      "max-width:200 constrains, got {}",
      body.children[0].children[0].content.width
    );
  }
);

// ============================================================================
// Padding/margin propagation
// ============================================================================

cross_test!(
  padding_flex_in_grid,
  r#"<div style="display:grid; grid-template-columns:200px; width:200px">
    <div style="display:flex; padding:10px">
        <div style="width:50px; height:30px">A</div>
    </div>
</div>"#,
  800.0,
  |body| {
    let flex = &body.children[0].children[0];
    assert!(
      (flex.padding.left - 10.0).abs() < 1.0,
      "pad.left=10, got {}",
      flex.padding.left
    );
    assert!((flex.children[0].content.width - 50.0).abs() < 1.0, "A w=50");
  }
);

cross_test!(
  margin_grid_in_flex,
  r#"<div style="display:flex; width:400px">
    <div style="display:grid; grid-template-columns:1fr; margin:20px; flex:1">
        <div style="height:30px">content</div>
    </div>
</div>"#,
  800.0,
  |body| {
    let grid = &body.children[0].children[0];
    assert!(
      (grid.margin.left - 20.0).abs() < 1.0,
      "margin.left=20, got {}",
      grid.margin.left
    );
    assert!(
      (grid.margin.right - 20.0).abs() < 1.0,
      "margin.right=20, got {}",
      grid.margin.right
    );
  }
);

// ============================================================================
// Percentage sizing across boundaries
// ============================================================================

cross_test!(
  percentage_width_in_flex,
  r#"<div style="display:flex; width:400px">
    <div style="flex:1"><div style="width:50%; height:30px">half</div></div>
</div>"#,
  800.0,
  |body| {
    let child = &body.children[0].children[0].children[0];
    assert!(
      (child.content.width - 200.0).abs() < 2.0,
      "50% of 400=200, got {}",
      child.content.width
    );
  }
);

cross_test!(
  percentage_width_in_grid,
  r#"<div style="display:grid; grid-template-columns:400px; width:400px">
    <div><div style="width:50%; height:30px">half</div></div>
</div>"#,
  800.0,
  |body| {
    let child = &body.children[0].children[0].children[0];
    assert!(
      (child.content.width - 200.0).abs() < 2.0,
      "50% of 400=200, got {}",
      child.content.width
    );
  }
);

// ============================================================================
// display:none
// ============================================================================

cross_test!(
  display_none_in_flex_in_grid,
  r#"<div style="display:grid; grid-template-columns:300px; width:300px">
    <div style="display:flex; gap:10px">
        <div style="width:100px; height:30px">visible</div>
        <div style="display:none; width:100px; height:30px">hidden</div>
        <div style="width:100px; height:30px">also visible</div>
    </div>
</div>"#,
  800.0,
  |body| {
    let flex = &body.children[0].children[0];
    let visible = flex.children.iter().filter(|c| c.content.width > 1.0).count();
    assert!(visible >= 2, "2 visible children, got {}", visible);
  }
);

// ============================================================================
// Complex real-world scenarios
// ============================================================================

cross_test!(
  dashboard_grid_flex_cards,
  r#"<div style="display:grid; grid-template-columns:1fr 1fr; gap:16px; width:600px">
    <div style="display:flex; flex-direction:column; height:150px">
        <div style="font-size:14px">Title</div>
        <div style="flex:1; display:flex; align-items:center; justify-content:center">
            <span style="font-size:24px">42</span>
        </div>
    </div>
    <div style="display:flex; flex-direction:column; height:150px">
        <div style="font-size:14px">Title</div>
        <div style="flex:1; display:flex; align-items:center; justify-content:center">
            <span style="font-size:24px">99</span>
        </div>
    </div>
</div>"#,
  800.0,
  |body| {
    let grid = &body.children[0];
    assert_eq!(grid.kind, BoxKind::GridContainer);
    assert!(
      grid.children[1].content.x > grid.children[0].content.x + 100.0,
      "cards side by side"
    );
    let col_w = (600.0 - 16.0) / 2.0;
    assert!(
      (grid.children[0].content.width - col_w).abs() < 2.0,
      "card1 w~{}, got {}",
      col_w,
      grid.children[0].content.width
    );
    assert!(
      (grid.children[1].content.width - col_w).abs() < 2.0,
      "card2 w~{}, got {}",
      col_w,
      grid.children[1].content.width
    );
  }
);

cross_test!(
  sidebar_flex_grid,
  r#"<div style="display:flex; width:800px">
    <div style="width:200px; display:flex; flex-direction:column">
        <div style="height:40px">Nav 1</div><div style="height:40px">Nav 2</div>
    </div>
    <div style="flex:1; display:grid; grid-template-columns:1fr 1fr; gap:10px">
        <div style="height:100px">A</div><div style="height:100px">B</div>
    </div>
</div>"#,
  800.0,
  |body| {
    let flex = &body.children[0];
    assert!((flex.children[0].content.width - 200.0).abs() < 1.0, "sidebar w=200");
    assert!((flex.children[1].content.width - 600.0).abs() < 2.0, "content w~600");
    assert_eq!(flex.children[1].kind, BoxKind::GridContainer);
  }
);

cross_test!(
  form_table_flex_buttons,
  r#"<div style="width:500px">
    <table style="width:500px">
        <tr><td style="padding:8px; height:20px">Name</td><td style="padding:8px; height:20px">Value</td></tr>
    </table>
    <div style="display:flex; justify-content:flex-end; gap:10px; margin-top:16px">
        <div style="width:80px; height:36px">Cancel</div>
        <div style="width:80px; height:36px">Save</div>
    </div>
</div>"#,
  800.0,
  |body, tree| {
    let table = find_by_tag(&tree.root, "table").unwrap();
    assert!((table.content.width - 500.0).abs() < 2.0, "table w=500");
    let block = &body.children[0];
    let btn = block
      .children
      .iter()
      .find(|c| c.kind == BoxKind::FlexContainer && c.children.len() == 2)
      .expect("button row");
    assert!((btn.children[1].content.width - 80.0).abs() < 1.0, "save w=80");
    // justify-content:flex-end pushes items to the right. The save button's
    // right edge should align with the parent block's right edge.
    let save_r = btn.children[1].content.x
      + btn.children[1].content.width
      + btn.children[1].padding.right
      + btn.children[1].border.right;
    let block_r = block.content.x + block.content.width;
    assert!(
      (block_r - save_r).abs() < 2.0,
      "buttons right-aligned to parent, save_r={} block_r={}",
      save_r,
      block_r
    );
  }
);

cross_test!(
  holy_grail_layout,
  r#"<div style="display:flex; flex-direction:column; width:800px; height:600px">
    <div style="height:60px; display:flex; align-items:center; padding:0 20px">
        <span style="font-size:20px">Logo</span>
    </div>
    <div style="flex:1; display:flex">
        <div style="width:150px; display:flex; flex-direction:column">
            <div style="height:30px">Nav 1</div><div style="height:30px">Nav 2</div>
        </div>
        <div style="flex:1; display:grid; grid-template-columns:1fr 1fr; gap:10px; padding:10px">
            <div style="height:100px">A</div><div style="height:100px">B</div>
        </div>
        <div style="width:150px"><div style="height:50px">Aside</div></div>
    </div>
    <div style="height:40px">Footer</div>
</div>"#,
  800.0,
  |body| {
    let outer = &body.children[0];
    assert_eq!(outer.kind, BoxKind::FlexContainer);
    assert!((outer.children[0].content.height - 60.0).abs() < 1.0, "header h=60");
    let middle = &outer.children[1];
    assert_eq!(middle.kind, BoxKind::FlexContainer);
    assert!((middle.children[0].content.width - 150.0).abs() < 1.0, "left nav w=150");
    assert_eq!(middle.children[1].kind, BoxKind::GridContainer);
    assert!(
      (middle.children[2].content.width - 150.0).abs() < 1.0,
      "right aside w=150"
    );
    assert!((outer.children[2].content.height - 40.0).abs() < 1.0, "footer h=40");
    assert!(outer.children[2].content.y > middle.content.y, "footer below middle");
  }
);
