/// Layout sub-profiler. Lives in `Ctx` — zero overhead when `None`.
/// Accumulated layout counters. Only populated when profiling is
/// enabled (passed as `Some(&mut LayoutProfiler)` in `Ctx`).
#[derive(Default)]
pub(crate) struct LayoutProfiler {
  pub block_calls: u32,
  pub flex_calls: u32,
  pub grid_calls: u32,
  pub table_calls: u32,
  pub inline_para_calls: u32,
  pub text_shape_calls: u32,
  pub para_shape_calls: u32,
  pub total_nodes: u32,
}

impl LayoutProfiler {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn dump(&self) {
    eprintln!(
      "[layout-profile] nodes={} block_calls={} | flex_calls={} | grid_calls={} | table_calls={} | inline_para_calls={} | text_shape_calls={} | para_shape_calls={}",
      self.total_nodes,
      self.block_calls,
      self.flex_calls,
      self.grid_calls,
      self.table_calls,
      self.inline_para_calls,
      self.text_shape_calls,
      self.para_shape_calls,
    );
  }
}

// Inline helpers — compile to nothing when profiler is None.
#[inline(always)]
pub(crate) fn count_block(p: &mut Option<LayoutProfiler>) {
  if let Some(p) = p {
    p.block_calls += 1;
    p.total_nodes += 1;
  }
}
#[inline(always)]
pub(crate) fn count_flex(p: &mut Option<LayoutProfiler>) {
  if let Some(p) = p {
    p.flex_calls += 1;
  }
}
#[inline(always)]
pub(crate) fn count_grid(p: &mut Option<LayoutProfiler>) {
  if let Some(p) = p {
    p.grid_calls += 1;
  }
}
#[inline(always)]
pub(crate) fn count_table(p: &mut Option<LayoutProfiler>) {
  if let Some(p) = p {
    p.table_calls += 1;
  }
}
#[inline(always)]
pub(crate) fn count_inline_para(p: &mut Option<LayoutProfiler>) {
  if let Some(p) = p {
    p.inline_para_calls += 1;
  }
}
#[inline(always)]
pub(crate) fn count_text_shape(p: &mut Option<LayoutProfiler>) {
  if let Some(p) = p {
    p.text_shape_calls += 1;
  }
}
#[inline(always)]
pub(crate) fn count_para_shape(p: &mut Option<LayoutProfiler>) {
  if let Some(p) = p {
    p.para_shape_calls += 1;
  }
}
