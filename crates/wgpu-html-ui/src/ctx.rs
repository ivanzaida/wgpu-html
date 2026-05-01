//! Callback factory and message channel.

use std::any::{Any, TypeId};
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};

use wgpu_html_models as m;
use wgpu_html_tree::{EventCallback, HtmlEvent, MouseCallback, MouseEvent, Node};

use crate::el::El;

// ── MsgSender ───────────────────────────────────────────────────────────────

/// Thread-safe message queue.
///
/// Cloned into callback closures so that event handlers (which must be
/// `Send + Sync`) can enqueue messages for the component's update loop.
#[derive(Clone)]
pub struct MsgSender<M: 'static> {
    queue: Arc<Mutex<Vec<M>>>,
    wake: Arc<dyn Fn() + Send + Sync>,
}

impl<M: 'static> MsgSender<M> {
    pub(crate) fn new(wake: Arc<dyn Fn() + Send + Sync>) -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            wake,
        }
    }

    /// Enqueue a message and request a redraw.
    pub fn send(&self, msg: M) {
        self.queue.lock().unwrap().push(msg);
        (self.wake)();
    }

    /// Drain all pending messages.
    pub(crate) fn drain(&self) -> Vec<M> {
        std::mem::take(&mut *self.queue.lock().unwrap())
    }

}

// ── ChildSlot ───────────────────────────────────────────────────────────────

/// Descriptor for a child component declared during [`Ctx::child`].
pub(crate) struct ChildSlot {
    pub index: usize,
    pub marker_id: String,
    pub component_type_id: TypeId,
    pub props: Box<dyn Any>,
    /// Factory: `(props: &dyn Any, wake) -> Box<dyn AnyComponent>`
    pub create:
        Box<dyn FnOnce(&dyn Any, Arc<dyn Fn() + Send + Sync>) -> Box<dyn crate::runtime::AnyComponent>>,
}

// ── Ctx ─────────────────────────────────────────────────────────────────────

/// Callback factory for a component.
///
/// Created by the runtime and passed to [`Component::view`].  Provides
/// methods to create event handlers that send messages, and to embed
/// child components.
pub struct Ctx<Msg: 'static> {
    pub(crate) sender: MsgSender<Msg>,
    pub(crate) children: RefCell<Vec<ChildSlot>>,
    pub(crate) child_counter: Cell<usize>,
}

impl<Msg: Clone + Send + Sync + 'static> Ctx<Msg> {
    pub(crate) fn new(sender: MsgSender<Msg>) -> Self {
        Self {
            sender,
            children: RefCell::new(Vec::new()),
            child_counter: Cell::new(0),
        }
    }

    // ── Callback factories ──────────────────────────────────────────────

    /// Create a [`MouseCallback`] that sends a pre-built message.
    pub fn msg(&self, msg: Msg) -> MouseCallback {
        let sender = self.sender.clone();
        Arc::new(move |_: &MouseEvent| {
            sender.send(msg.clone());
        })
    }

    /// Create a [`MouseCallback`] that maps the event to a message.
    pub fn callback(
        &self,
        f: impl Fn(&MouseEvent) -> Msg + Send + Sync + 'static,
    ) -> MouseCallback {
        let sender = self.sender.clone();
        Arc::new(move |ev: &MouseEvent| {
            sender.send(f(ev));
        })
    }

    /// Create an [`EventCallback`] that optionally maps an event to a
    /// message.
    pub fn event_callback(
        &self,
        f: impl Fn(&HtmlEvent) -> Option<Msg> + Send + Sync + 'static,
    ) -> EventCallback {
        let sender = self.sender.clone();
        Arc::new(move |ev: &HtmlEvent| {
            if let Some(m) = f(ev) {
                sender.send(m);
            }
        })
    }

    /// Get a clone of the message sender for building custom callbacks
    /// (e.g. parent-provided closures in props).
    pub fn sender(&self) -> MsgSender<Msg> {
        self.sender.clone()
    }

    // ── Child components ────────────────────────────────────────────────

    /// Embed a child component.  Returns a placeholder [`El`] that the
    /// runtime replaces with the child's rendered subtree.
    pub fn child<C: crate::Component>(&self, props: C::Props) -> El
    where
        C::Props: 'static,
        C::Msg: Clone + Send + Sync + 'static,
        C::Env: 'static,
    {
        let idx = self.child_counter.get();
        self.child_counter.set(idx + 1);

        let marker_id = format!("__ui_child_{idx}");

        let placeholder = El {
            node: Node::new(m::Div {
                id: Some(marker_id.clone()),
                ..Default::default()
            }),
        };

        self.children.borrow_mut().push(ChildSlot {
            index: idx,
            marker_id,
            component_type_id: TypeId::of::<C>(),
            props: Box::new(props),
            create: Box::new(|props_any, wake| {
                let props = props_any.downcast_ref::<C::Props>().unwrap();
                Box::new(crate::runtime::ComponentState::<C>::new(props, wake))
            }),
        });

        placeholder
    }
}
