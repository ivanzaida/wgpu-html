use std::sync::{
  Arc, Mutex,
  atomic::{AtomicUsize, Ordering::SeqCst},
};

use wgpu_html_tree::Tree;

use crate::{Component, Ctx, El, MsgSender, ShouldRender, core::runtime::Runtime, el};

// ── Shared counter bundle ───────────────────────────────────────────────────

/// Tracks calls to each lifecycle method for a single component instance.
#[derive(Clone, Default)]
struct Spy {
  views: Arc<AtomicUsize>,
  mounts: Arc<AtomicUsize>,
  destroys: Arc<AtomicUsize>,
  updated: Arc<AtomicUsize>,
}

impl Spy {
  fn views(&self) -> usize {
    self.views.load(SeqCst)
  }
  fn mounts(&self) -> usize {
    self.mounts.load(SeqCst)
  }
  fn destroys(&self) -> usize {
    self.destroys.load(SeqCst)
  }
  fn updated(&self) -> usize {
    self.updated.load(SeqCst)
  }
}

/// Slot that a component writes its `MsgSender` into from `mounted()`.
type SenderSlot<M> = Arc<Mutex<Option<MsgSender<M>>>>;

fn slot<M: 'static>() -> SenderSlot<M> {
  Arc::new(Mutex::new(None))
}

fn poke<M: Clone + Send + Sync + 'static>(s: &SenderSlot<M>, msg: M) {
  s.lock().unwrap().as_ref().expect("sender not yet stored").send(msg);
}

// ── Leaf component ──────────────────────────────────────────────────────────
//
// Leaf is a standalone component (no children). Its `mounted()` writes
// the sender into a caller-owned `SenderSlot` so tests can inject messages.

struct LeafComp {
  spy: Spy,
  on_mount: Arc<dyn Fn(MsgSender<LeafMsg>) + Send + Sync>,
}

#[derive(Clone)]
struct LeafProps {
  spy: Spy,
  sender_slot: SenderSlot<LeafMsg>,
}

#[derive(Clone)]
enum LeafMsg {
  Poke,
}

impl Component for LeafComp {
  type Props = LeafProps;
  type Msg = LeafMsg;
  type Env = ();

  fn create(p: &LeafProps) -> Self {
    let slot = p.sender_slot.clone();
    LeafComp {
      spy: p.spy.clone(),
      on_mount: Arc::new(move |s| {
        *slot.lock().unwrap() = Some(s);
      }),
    }
  }

  fn update(&mut self, _: LeafMsg, _: &LeafProps) -> ShouldRender {
    ShouldRender::Yes
  }

  fn view(&self, _: &LeafProps, _: &Ctx<LeafMsg>, _: &()) -> El {
    self.spy.views.fetch_add(1, SeqCst);
    el::span()
  }

  fn mounted(&mut self, sender: MsgSender<LeafMsg>) {
    self.spy.mounts.fetch_add(1, SeqCst);
    (self.on_mount)(sender);
  }

  fn destroyed(&mut self) {
    self.spy.destroys.fetch_add(1, SeqCst);
  }

  fn updated(&mut self, _: &LeafProps) {
    self.spy.updated.fetch_add(1, SeqCst);
  }
}

// ── ParentComp — one LeafComp child ────────────────────────────────────────

struct ParentComp {
  spy: Spy,
}

#[derive(Clone)]
struct ParentProps {
  spy: Spy,
  leaf_spy: Spy,
  leaf_slot: SenderSlot<LeafMsg>,
}

#[derive(Clone)]
enum ParentMsg {
  Poke,
}

impl Component for ParentComp {
  type Props = ParentProps;
  type Msg = ParentMsg;
  type Env = ();

  fn create(p: &ParentProps) -> Self {
    ParentComp { spy: p.spy.clone() }
  }

  fn update(&mut self, _: ParentMsg, _: &ParentProps) -> ShouldRender {
    ShouldRender::Yes
  }

  fn view(&self, props: &ParentProps, ctx: &Ctx<ParentMsg>, _: &()) -> El {
    self.spy.views.fetch_add(1, SeqCst);
    el::div().child(ctx.child::<LeafComp>(LeafProps {
      spy: props.leaf_spy.clone(),
      sender_slot: props.leaf_slot.clone(),
    }))
  }

  fn mounted(&mut self, _: MsgSender<ParentMsg>) {
    self.spy.mounts.fetch_add(1, SeqCst);
  }

  fn destroyed(&mut self) {
    self.spy.destroys.fetch_add(1, SeqCst);
  }

  fn updated(&mut self, _: &ParentProps) {
    self.spy.updated.fetch_add(1, SeqCst);
  }
}

// ── RootComp — one ParentComp child ────────────────────────────────────────
// Used for the 3-level depth tests.

struct RootComp {
  spy: Spy,
}

#[derive(Clone)]
struct RootProps {
  spy: Spy,
  parent_spy: Spy,
  leaf_spy: Spy,
  leaf_slot: SenderSlot<LeafMsg>,
}

#[derive(Clone)]
enum RootMsg {}

impl Component for RootComp {
  type Props = RootProps;
  type Msg = RootMsg;
  type Env = ();

  fn create(p: &RootProps) -> Self {
    RootComp { spy: p.spy.clone() }
  }

  fn update(&mut self, _: RootMsg, _: &RootProps) -> ShouldRender {
    ShouldRender::No
  }

  fn view(&self, props: &RootProps, ctx: &Ctx<RootMsg>, _: &()) -> El {
    self.spy.views.fetch_add(1, SeqCst);
    el::div().child(ctx.child::<ParentComp>(ParentProps {
      spy: props.parent_spy.clone(),
      leaf_spy: props.leaf_spy.clone(),
      leaf_slot: props.leaf_slot.clone(),
    }))
  }

  fn mounted(&mut self, _: MsgSender<RootMsg>) {
    self.spy.mounts.fetch_add(1, SeqCst);
  }
}

// ── TwoChildrenComp — two keyed LeafComp children ──────────────────────────

struct TwoChildrenComp {
  spy: Spy,
}

#[derive(Clone)]
struct TwoChildrenProps {
  spy: Spy,
  spy_a: Spy,
  spy_b: Spy,
  slot_a: SenderSlot<LeafMsg>,
  slot_b: SenderSlot<LeafMsg>,
}

#[derive(Clone)]
enum TwoChildrenMsg {}

impl Component for TwoChildrenComp {
  type Props = TwoChildrenProps;
  type Msg = TwoChildrenMsg;
  type Env = ();

  fn create(p: &TwoChildrenProps) -> Self {
    TwoChildrenComp { spy: p.spy.clone() }
  }

  fn update(&mut self, _: TwoChildrenMsg, _: &TwoChildrenProps) -> ShouldRender {
    ShouldRender::No
  }

  fn view(&self, props: &TwoChildrenProps, ctx: &Ctx<TwoChildrenMsg>, _: &()) -> El {
    self.spy.views.fetch_add(1, SeqCst);
    el::div().children([
      ctx.keyed_child::<LeafComp>(
        "a",
        LeafProps {
          spy: props.spy_a.clone(),
          sender_slot: props.slot_a.clone(),
        },
      ),
      ctx.keyed_child::<LeafComp>(
        "b",
        LeafProps {
          spy: props.spy_b.clone(),
          sender_slot: props.slot_b.clone(),
        },
      ),
    ])
  }

  fn mounted(&mut self, _: MsgSender<TwoChildrenMsg>) {
    self.spy.mounts.fetch_add(1, SeqCst);
  }
}

// ── StableLeaf — like LeafComp but props_changed returns No ────────────────

struct StableLeaf {
  spy: Spy,
  on_mount: Arc<dyn Fn(MsgSender<LeafMsg>) + Send + Sync>,
}

impl Component for StableLeaf {
  type Props = LeafProps;
  type Msg = LeafMsg;
  type Env = ();

  fn create(p: &LeafProps) -> Self {
    let slot = p.sender_slot.clone();
    StableLeaf {
      spy: p.spy.clone(),
      on_mount: Arc::new(move |s| {
        *slot.lock().unwrap() = Some(s);
      }),
    }
  }

  fn update(&mut self, _: LeafMsg, _: &LeafProps) -> ShouldRender {
    ShouldRender::Yes
  }

  fn view(&self, _: &LeafProps, _: &Ctx<LeafMsg>, _: &()) -> El {
    self.spy.views.fetch_add(1, SeqCst);
    el::span()
  }

  fn props_changed(&mut self, _: &LeafProps, _: &LeafProps) -> ShouldRender {
    ShouldRender::No
  }

  fn mounted(&mut self, sender: MsgSender<LeafMsg>) {
    self.spy.mounts.fetch_add(1, SeqCst);
    (self.on_mount)(sender);
  }

  fn destroyed(&mut self) {
    self.spy.destroys.fetch_add(1, SeqCst);
  }

  fn updated(&mut self, _: &LeafProps) {
    self.spy.updated.fetch_add(1, SeqCst);
  }
}

// ── ConditionalComp — toggleable child ─────────────────────────────────────

struct ConditionalComp {
  show_child: bool,
  spy: Spy,
  on_mount: Arc<dyn Fn(MsgSender<ConditionalMsg>) + Send + Sync>,
}

#[derive(Clone)]
struct ConditionalProps {
  spy: Spy,
  sender_slot: SenderSlot<ConditionalMsg>,
  child_spy: Spy,
  child_slot: SenderSlot<LeafMsg>,
}

#[derive(Clone)]
enum ConditionalMsg {
  Toggle,
}

impl Component for ConditionalComp {
  type Props = ConditionalProps;
  type Msg = ConditionalMsg;
  type Env = ();

  fn create(p: &ConditionalProps) -> Self {
    let s = p.sender_slot.clone();
    ConditionalComp {
      show_child: true,
      spy: p.spy.clone(),
      on_mount: Arc::new(move |sender| {
        *s.lock().unwrap() = Some(sender);
      }),
    }
  }

  fn update(&mut self, msg: ConditionalMsg, _: &ConditionalProps) -> ShouldRender {
    match msg {
      ConditionalMsg::Toggle => {
        self.show_child = !self.show_child;
        ShouldRender::Yes
      }
    }
  }

  fn view(&self, props: &ConditionalProps, ctx: &Ctx<ConditionalMsg>, _: &()) -> El {
    self.spy.views.fetch_add(1, SeqCst);
    if self.show_child {
      el::div().child(ctx.child::<LeafComp>(LeafProps {
        spy: props.child_spy.clone(),
        sender_slot: props.child_slot.clone(),
      }))
    } else {
      el::div()
    }
  }

  fn mounted(&mut self, sender: MsgSender<ConditionalMsg>) {
    self.spy.mounts.fetch_add(1, SeqCst);
    (self.on_mount)(sender);
  }

  fn destroyed(&mut self) {
    self.spy.destroys.fetch_add(1, SeqCst);
  }
}

// ── SelfPokingComp — update() sends a follow-up message ───────────────────

struct SelfPokingComp {
  spy: Spy,
  sender: Option<MsgSender<SelfPokingMsg>>,
  on_mount: Arc<dyn Fn(MsgSender<SelfPokingMsg>) + Send + Sync>,
}

#[derive(Clone)]
struct SelfPokingProps {
  spy: Spy,
  sender_slot: SenderSlot<SelfPokingMsg>,
}

#[derive(Clone)]
enum SelfPokingMsg {
  Start,
  FollowUp,
}

impl Component for SelfPokingComp {
  type Props = SelfPokingProps;
  type Msg = SelfPokingMsg;
  type Env = ();

  fn create(p: &SelfPokingProps) -> Self {
    let s = p.sender_slot.clone();
    SelfPokingComp {
      spy: p.spy.clone(),
      sender: None,
      on_mount: Arc::new(move |sender| {
        *s.lock().unwrap() = Some(sender);
      }),
    }
  }

  fn update(&mut self, msg: SelfPokingMsg, _: &SelfPokingProps) -> ShouldRender {
    match msg {
      SelfPokingMsg::Start => {
        if let Some(s) = &self.sender {
          s.send(SelfPokingMsg::FollowUp);
        }
        ShouldRender::Yes
      }
      SelfPokingMsg::FollowUp => ShouldRender::Yes,
    }
  }

  fn view(&self, _: &SelfPokingProps, _: &Ctx<SelfPokingMsg>, _: &()) -> El {
    self.spy.views.fetch_add(1, SeqCst);
    el::span()
  }

  fn mounted(&mut self, sender: MsgSender<SelfPokingMsg>) {
    self.spy.mounts.fetch_add(1, SeqCst);
    self.sender = Some(sender.clone());
    (self.on_mount)(sender);
  }
}

// ── Test helpers ────────────────────────────────────────────────────────────

fn bootstrap<C: Component<Env = ()>>(props: C::Props) -> (Runtime, Tree)
where
  C::Msg: Clone + Send + Sync + 'static,
  C::Props: 'static,
{
  let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(|| {});
  let mut rt = Runtime::new::<C>(&props, wake);
  let tree = Tree::default();
  rt.initial_render(&());
  (rt, tree)
}

// ── Render-path tests ─────────────────────────────────────────────────────

/// PATH 1: A clean component (no messages, no dirty descendants) returns its
/// cached `last_node` without calling `view()`.
#[test]
fn path1_clean_component_skips_view() {
  let leaf_spy = Spy::default();
  let leaf_slot = slot::<LeafMsg>();

  let (mut rt, mut tree) = bootstrap::<LeafComp>(LeafProps {
    spy: leaf_spy.clone(),
    sender_slot: leaf_slot.clone(),
  });

  assert_eq!(leaf_spy.views(), 1, "initial render calls view once");

  // No messages — process() should do nothing.
  let changed = rt.process(&mut tree, &());

  assert!(!changed, "process() reports no change");
  assert_eq!(leaf_spy.views(), 1, "view() not called again (path 1)");
}

/// PATH 3: A component with a pending message has its `view()` called.
#[test]
fn path3_dirty_component_calls_view() {
  let leaf_spy = Spy::default();
  let leaf_slot = slot::<LeafMsg>();

  let (mut rt, mut tree) = bootstrap::<LeafComp>(LeafProps {
    spy: leaf_spy.clone(),
    sender_slot: leaf_slot.clone(),
  });

  assert_eq!(leaf_spy.views(), 1);

  poke(&leaf_slot, LeafMsg::Poke);
  let changed = rt.process(&mut tree, &());

  assert!(changed, "process() reports a change");
  assert_eq!(leaf_spy.views(), 2, "view() called after dirty message (path 3)");
}

/// PATH 2: When a child is dirty but its parent is clean, the parent's
/// `view()` must NOT be called — only the child's.
#[test]
fn path2_dirty_child_skips_parent_view() {
  let parent_spy = Spy::default();
  let leaf_spy = Spy::default();
  let leaf_slot = slot::<LeafMsg>();

  let (mut rt, mut tree) = bootstrap::<ParentComp>(ParentProps {
    spy: parent_spy.clone(),
    leaf_spy: leaf_spy.clone(),
    leaf_slot: leaf_slot.clone(),
  });

  assert_eq!(parent_spy.views(), 1, "initial render");
  assert_eq!(leaf_spy.views(), 1, "initial render");

  // Poke only the child.
  poke(&leaf_slot, LeafMsg::Poke);
  rt.process(&mut tree, &());

  assert_eq!(leaf_spy.views(), 2, "leaf re-rendered (path 3)");
  assert_eq!(parent_spy.views(), 1, "parent view() NOT called (path 2)");
}

/// PATH 2 chains: with a Root->Parent->Leaf tree and only the Leaf dirty,
/// neither Root nor Parent should have their `view()` called.
#[test]
fn path2_chains_through_two_ancestors() {
  let root_spy = Spy::default();
  let parent_spy = Spy::default();
  let leaf_spy = Spy::default();
  let leaf_slot = slot::<LeafMsg>();

  let (mut rt, mut tree) = bootstrap::<RootComp>(RootProps {
    spy: root_spy.clone(),
    parent_spy: parent_spy.clone(),
    leaf_spy: leaf_spy.clone(),
    leaf_slot: leaf_slot.clone(),
  });

  assert_eq!(root_spy.views(), 1);
  assert_eq!(parent_spy.views(), 1);
  assert_eq!(leaf_spy.views(), 1);

  poke(&leaf_slot, LeafMsg::Poke);
  rt.process(&mut tree, &());

  assert_eq!(leaf_spy.views(), 2, "leaf re-rendered (path 3)");
  assert_eq!(parent_spy.views(), 1, "parent view() skipped (path 2)");
  assert_eq!(root_spy.views(), 1, "root view() skipped (path 2)");
}

/// PATH 1 + 2 siblings: only the dirty sibling is re-rendered; the clean
/// sibling and the parent stay untouched.
#[test]
fn path1_clean_sibling_not_re_rendered() {
  let parent_spy = Spy::default();
  let spy_a = Spy::default();
  let spy_b = Spy::default();
  let slot_a = slot::<LeafMsg>();
  let slot_b = slot::<LeafMsg>();

  let (mut rt, mut tree) = bootstrap::<TwoChildrenComp>(TwoChildrenProps {
    spy: parent_spy.clone(),
    spy_a: spy_a.clone(),
    spy_b: spy_b.clone(),
    slot_a: slot_a.clone(),
    slot_b: slot_b.clone(),
  });

  assert_eq!(parent_spy.views(), 1);
  assert_eq!(spy_a.views(), 1);
  assert_eq!(spy_b.views(), 1);

  // Only poke child A.
  poke(&slot_a, LeafMsg::Poke);
  rt.process(&mut tree, &());

  assert_eq!(spy_a.views(), 2, "child A re-rendered");
  assert_eq!(spy_b.views(), 1, "child B view() not called (path 1)");
  assert_eq!(parent_spy.views(), 1, "parent view() not called (path 2)");
}

/// Verify `skeleton_node` is populated after initial render and that
/// multiple pokes accumulate correctly (each takes path 3 on the leaf
/// and path 2 on its ancestors).
#[test]
fn skeleton_stored_and_patch_path_repeatable() {
  let parent_spy = Spy::default();
  let leaf_spy = Spy::default();
  let leaf_slot = slot::<LeafMsg>();

  let (mut rt, mut tree) = bootstrap::<ParentComp>(ParentProps {
    spy: parent_spy.clone(),
    leaf_spy: leaf_spy.clone(),
    leaf_slot: leaf_slot.clone(),
  });

  // Poke the child three times.
  for _ in 0..3 {
    poke(&leaf_slot, LeafMsg::Poke);
    rt.process(&mut tree, &());
  }

  assert_eq!(leaf_spy.views(), 4, "initial + 3 re-renders");
  assert_eq!(parent_spy.views(), 1, "parent view() never called again");
}

/// `force_render` marks every component dirty so all `view()` calls happen.
#[test]
fn force_render_calls_view_on_all() {
  let parent_spy = Spy::default();
  let leaf_spy = Spy::default();
  let leaf_slot = slot::<LeafMsg>();

  let (mut rt, mut tree) = bootstrap::<ParentComp>(ParentProps {
    spy: parent_spy.clone(),
    leaf_spy: leaf_spy.clone(),
    leaf_slot: leaf_slot.clone(),
  });

  assert_eq!(parent_spy.views(), 1);
  assert_eq!(leaf_spy.views(), 1);

  rt.force_render(&mut tree, &());

  assert_eq!(parent_spy.views(), 2, "parent re-rendered by force_render");
  assert_eq!(leaf_spy.views(), 2, "leaf re-rendered by force_render");
}

/// When the parent is dirty (path 3) and the child has `props_changed`
/// returning `No`, the child's `view()` must be skipped (path 1 inside
/// path 3).
#[test]
fn path3_parent_dirty_does_not_re_render_clean_child() {
  let parent_spy = Spy::default();
  let leaf_spy = Spy::default();
  let leaf_slot = slot::<LeafMsg>();
  let parent_slot = slot::<ParentMsg>();

  struct ObservableParent {
    spy: Spy,
    on_mount: Arc<dyn Fn(MsgSender<ParentMsg>) + Send + Sync>,
  }

  #[derive(Clone)]
  struct ObservableParentProps {
    spy: Spy,
    parent_slot: SenderSlot<ParentMsg>,
    leaf_spy: Spy,
    leaf_slot: SenderSlot<LeafMsg>,
  }

  impl Component for ObservableParent {
    type Props = ObservableParentProps;
    type Msg = ParentMsg;
    type Env = ();

    fn create(p: &ObservableParentProps) -> Self {
      let s = p.parent_slot.clone();
      ObservableParent {
        spy: p.spy.clone(),
        on_mount: Arc::new(move |sender| {
          *s.lock().unwrap() = Some(sender);
        }),
      }
    }

    fn update(&mut self, _: ParentMsg, _: &ObservableParentProps) -> ShouldRender {
      ShouldRender::Yes
    }

    fn view(&self, props: &ObservableParentProps, ctx: &Ctx<ParentMsg>, _: &()) -> El {
      self.spy.views.fetch_add(1, SeqCst);
      el::div().child(ctx.child::<StableLeaf>(LeafProps {
        spy: props.leaf_spy.clone(),
        sender_slot: props.leaf_slot.clone(),
      }))
    }

    fn mounted(&mut self, sender: MsgSender<ParentMsg>) {
      self.spy.mounts.fetch_add(1, SeqCst);
      (self.on_mount)(sender);
    }
  }

  let (mut rt, mut tree) = bootstrap::<ObservableParent>(ObservableParentProps {
    spy: parent_spy.clone(),
    parent_slot: parent_slot.clone(),
    leaf_spy: leaf_spy.clone(),
    leaf_slot: leaf_slot.clone(),
  });

  assert_eq!(parent_spy.views(), 1);
  assert_eq!(leaf_spy.views(), 1);

  // Poke the parent (makes parent dirty, not the leaf).
  poke(&parent_slot, ParentMsg::Poke);
  rt.process(&mut tree, &());

  assert_eq!(parent_spy.views(), 2, "parent view() called (path 3)");
  assert_eq!(leaf_spy.views(), 1, "leaf view() skipped (path 1 inside path 3)");
}

// ── Lifecycle tests ─────────────────────────────────────────────────────────

/// `Component::mounted()` fires exactly once on initial render.
#[test]
fn mounted_lifecycle_fires_once() {
  let leaf_spy = Spy::default();
  let leaf_slot = slot::<LeafMsg>();

  let (_rt, _tree) = bootstrap::<LeafComp>(LeafProps {
    spy: leaf_spy.clone(),
    sender_slot: leaf_slot.clone(),
  });

  assert_eq!(leaf_spy.mounts(), 1, "mounted() called exactly once");
}

/// `Component::updated()` is called after a dirty re-render, but NOT after
/// a clean (path 1) non-render or the initial mount render.
#[test]
fn updated_hook_fires_on_dirty_render_only() {
  let leaf_spy = Spy::default();
  let leaf_slot = slot::<LeafMsg>();

  let (mut rt, mut tree) = bootstrap::<LeafComp>(LeafProps {
    spy: leaf_spy.clone(),
    sender_slot: leaf_slot.clone(),
  });

  // `updated` should NOT fire on the initial mount render.
  assert_eq!(leaf_spy.updated(), 0, "updated() not called after initial render");

  // After a dirty message:
  poke(&leaf_slot, LeafMsg::Poke);
  rt.process(&mut tree, &());
  assert_eq!(leaf_spy.updated(), 1, "updated() called once after dirty render");

  // With no new message (clean pass):
  rt.process(&mut tree, &());
  assert_eq!(leaf_spy.updated(), 1, "updated() not called on clean pass");

  // Another dirty message:
  poke(&leaf_slot, LeafMsg::Poke);
  rt.process(&mut tree, &());
  assert_eq!(
    leaf_spy.updated(),
    2,
    "updated() called again after second dirty render"
  );
}

/// `destroyed()` fires when a parent stops rendering a child.
#[test]
fn destroyed_fires_on_child_removal() {
  let cond_spy = Spy::default();
  let child_spy = Spy::default();
  let cond_slot = slot::<ConditionalMsg>();
  let child_slot = slot::<LeafMsg>();

  let (mut rt, mut tree) = bootstrap::<ConditionalComp>(ConditionalProps {
    spy: cond_spy.clone(),
    sender_slot: cond_slot.clone(),
    child_spy: child_spy.clone(),
    child_slot: child_slot.clone(),
  });

  assert_eq!(child_spy.mounts(), 1, "child mounted after initial render");
  assert_eq!(child_spy.destroys(), 0, "child not yet destroyed");

  // Toggle child off.
  poke(&cond_slot, ConditionalMsg::Toggle);
  rt.process(&mut tree, &());

  assert_eq!(child_spy.destroys(), 1, "child destroyed after toggle off");
}

/// After removing and re-adding a child, a fresh instance is mounted.
#[test]
fn child_remounted_after_removal() {
  let cond_spy = Spy::default();
  let child_spy = Spy::default();
  let cond_slot = slot::<ConditionalMsg>();
  let child_slot = slot::<LeafMsg>();

  let (mut rt, mut tree) = bootstrap::<ConditionalComp>(ConditionalProps {
    spy: cond_spy.clone(),
    sender_slot: cond_slot.clone(),
    child_spy: child_spy.clone(),
    child_slot: child_slot.clone(),
  });

  assert_eq!(child_spy.mounts(), 1);

  // Toggle off then on.
  poke(&cond_slot, ConditionalMsg::Toggle);
  rt.process(&mut tree, &());
  poke(&cond_slot, ConditionalMsg::Toggle);
  rt.process(&mut tree, &());

  assert_eq!(child_spy.destroys(), 1, "first instance destroyed");
  assert_eq!(child_spy.mounts(), 2, "second instance mounted");
}

/// Both parent and child components receive `mounted()` on initial render.
#[test]
fn nested_components_both_mounted() {
  let parent_spy = Spy::default();
  let leaf_spy = Spy::default();
  let leaf_slot = slot::<LeafMsg>();

  let (_rt, _tree) = bootstrap::<ParentComp>(ParentProps {
    spy: parent_spy.clone(),
    leaf_spy: leaf_spy.clone(),
    leaf_slot: leaf_slot.clone(),
  });

  assert_eq!(parent_spy.mounts(), 1, "parent mounted");
  assert_eq!(leaf_spy.mounts(), 1, "leaf mounted");
}

// ── Sibling and message batching tests ───────────────────────────────────────

/// When both siblings are dirty, both re-render; the parent skips (path 2).
#[test]
fn both_siblings_dirty_both_rerender() {
  let parent_spy = Spy::default();
  let spy_a = Spy::default();
  let spy_b = Spy::default();
  let slot_a = slot::<LeafMsg>();
  let slot_b = slot::<LeafMsg>();

  let (mut rt, mut tree) = bootstrap::<TwoChildrenComp>(TwoChildrenProps {
    spy: parent_spy.clone(),
    spy_a: spy_a.clone(),
    spy_b: spy_b.clone(),
    slot_a: slot_a.clone(),
    slot_b: slot_b.clone(),
  });

  // Poke both children.
  poke(&slot_a, LeafMsg::Poke);
  poke(&slot_b, LeafMsg::Poke);
  rt.process(&mut tree, &());

  assert_eq!(spy_a.views(), 2, "child A re-rendered");
  assert_eq!(spy_b.views(), 2, "child B re-rendered");
  assert_eq!(parent_spy.views(), 1, "parent view() not called (path 2)");
}

/// Multiple messages queued before `process()` result in a single re-render.
#[test]
fn multiple_messages_single_render() {
  let spy = Spy::default();
  let leaf_slot = slot::<LeafMsg>();

  let (mut rt, mut tree) = bootstrap::<LeafComp>(LeafProps {
    spy: spy.clone(),
    sender_slot: leaf_slot.clone(),
  });

  assert_eq!(spy.views(), 1);

  // Queue three messages before processing.
  poke(&leaf_slot, LeafMsg::Poke);
  poke(&leaf_slot, LeafMsg::Poke);
  poke(&leaf_slot, LeafMsg::Poke);
  rt.process(&mut tree, &());

  assert_eq!(spy.views(), 2, "only one re-render despite three messages");
}

/// A follow-up message sent from `update()` is handled in the same
/// `process()` call, producing only one render.
#[test]
fn followup_message_handled_in_same_process() {
  let spy = Spy::default();
  let poke_slot = slot::<SelfPokingMsg>();

  let (mut rt, mut tree) = bootstrap::<SelfPokingComp>(SelfPokingProps {
    spy: spy.clone(),
    sender_slot: poke_slot.clone(),
  });

  assert_eq!(spy.views(), 1);

  // Start sends FollowUp internally; both handled in one process().
  poke(&poke_slot, SelfPokingMsg::Start);
  let changed = rt.process(&mut tree, &());

  assert!(changed);
  assert_eq!(spy.views(), 2, "single re-render covers Start + FollowUp");
}

// ── Lifecycle hook and props_changed tests ───────────────────────────────────

/// `force_render` triggers `updated()` on components that already have a
/// skeleton (i.e. not on their first render).
#[test]
fn force_render_triggers_updated() {
  let parent_spy = Spy::default();
  let leaf_spy = Spy::default();
  let leaf_slot = slot::<LeafMsg>();

  let (mut rt, mut tree) = bootstrap::<ParentComp>(ParentProps {
    spy: parent_spy.clone(),
    leaf_spy: leaf_spy.clone(),
    leaf_slot: leaf_slot.clone(),
  });

  assert_eq!(parent_spy.updated(), 0, "no updated() after initial render");
  assert_eq!(leaf_spy.updated(), 0, "no updated() after initial render");

  rt.force_render(&mut tree, &());

  assert_eq!(parent_spy.updated(), 1, "parent updated() fired by force_render");
  assert_eq!(leaf_spy.updated(), 1, "leaf updated() fired by force_render");
}

/// With the default `props_changed` (returns Yes), a parent re-render
/// (path 3) also re-renders the child, but the child instance is reused
/// (not destroyed and re-created).
#[test]
fn props_changed_default_rerenders_child() {
  let parent_spy = Spy::default();
  let leaf_spy = Spy::default();
  let leaf_slot = slot::<LeafMsg>();
  let parent_slot = slot::<ParentMsg>();

  struct PokableParent {
    spy: Spy,
    on_mount: Arc<dyn Fn(MsgSender<ParentMsg>) + Send + Sync>,
  }

  #[derive(Clone)]
  struct PokableParentProps {
    spy: Spy,
    parent_slot: SenderSlot<ParentMsg>,
    leaf_spy: Spy,
    leaf_slot: SenderSlot<LeafMsg>,
  }

  impl Component for PokableParent {
    type Props = PokableParentProps;
    type Msg = ParentMsg;
    type Env = ();

    fn create(p: &PokableParentProps) -> Self {
      let s = p.parent_slot.clone();
      PokableParent {
        spy: p.spy.clone(),
        on_mount: Arc::new(move |sender| {
          *s.lock().unwrap() = Some(sender);
        }),
      }
    }

    fn update(&mut self, _: ParentMsg, _: &PokableParentProps) -> ShouldRender {
      ShouldRender::Yes
    }

    fn view(&self, props: &PokableParentProps, ctx: &Ctx<ParentMsg>, _: &()) -> El {
      self.spy.views.fetch_add(1, SeqCst);
      // Uses LeafComp which has default props_changed -> Yes.
      el::div().child(ctx.child::<LeafComp>(LeafProps {
        spy: props.leaf_spy.clone(),
        sender_slot: props.leaf_slot.clone(),
      }))
    }

    fn mounted(&mut self, sender: MsgSender<ParentMsg>) {
      self.spy.mounts.fetch_add(1, SeqCst);
      (self.on_mount)(sender);
    }
  }

  let (mut rt, mut tree) = bootstrap::<PokableParent>(PokableParentProps {
    spy: parent_spy.clone(),
    parent_slot: parent_slot.clone(),
    leaf_spy: leaf_spy.clone(),
    leaf_slot: leaf_slot.clone(),
  });

  assert_eq!(parent_spy.views(), 1);
  assert_eq!(leaf_spy.views(), 1);

  // Poke parent -> parent re-renders (path 3). Default props_changed = Yes
  // so child is also re-rendered.
  poke(&parent_slot, ParentMsg::Poke);
  rt.process(&mut tree, &());

  assert_eq!(parent_spy.views(), 2, "parent re-rendered (path 3)");
  assert_eq!(leaf_spy.views(), 2, "child re-rendered (default props_changed = Yes)");
  assert_eq!(leaf_spy.mounts(), 1, "child not re-mounted (reused)");
  assert_eq!(leaf_spy.destroys(), 0, "child not destroyed (reused)");
}

/// Calling `process()` on a fully clean tree (parent + children) is a no-op.
#[test]
fn process_noop_on_clean_tree() {
  let parent_spy = Spy::default();
  let leaf_spy = Spy::default();
  let leaf_slot = slot::<LeafMsg>();

  let (mut rt, mut tree) = bootstrap::<ParentComp>(ParentProps {
    spy: parent_spy.clone(),
    leaf_spy: leaf_spy.clone(),
    leaf_slot: leaf_slot.clone(),
  });

  // Three clean process() calls.
  for _ in 0..3 {
    let changed = rt.process(&mut tree, &());
    assert!(!changed, "process() on clean tree returns false");
  }

  assert_eq!(parent_spy.views(), 1, "parent view() not called again");
  assert_eq!(leaf_spy.views(), 1, "leaf view() not called again");
}
