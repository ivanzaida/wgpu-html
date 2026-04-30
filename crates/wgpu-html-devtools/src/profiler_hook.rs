use wgpu_html_tree::{Tree, TreeHook, TreeHookResponse, TreeRenderEvent};

pub struct ProfilerHook;

impl TreeHook for ProfilerHook {
  fn on_render(&mut self, tree: &mut Tree, event: &TreeRenderEvent<'_>) -> TreeHookResponse {
    if let Some(p) = &tree.profiler {
      for entry in p.entries() {
        println!("{}: {:.2}ms", entry.label, entry.duration.as_secs_f64() * 1000.0);
      }
    }
    TreeHookResponse::Continue
  }
}
