use std::sync::{Arc, atomic::AtomicBool};

use crate::Lui;
use super::super::builder::el::El;
use super::ctx::{ComponentId, Ctx, StyleRegistry};

type ComponentFn = Box<dyn Fn(&Ctx) -> El>;

pub struct Runtime {
    ctx: Ctx,
    component: ComponentFn,
    selector: String,
    mounted: bool,
}

impl Runtime {
    pub fn new<F: Fn(&Ctx) -> El + 'static>(selector: impl Into<String>, component: F) -> Self {
        let dirty = Arc::new(AtomicBool::new(true));
        let commands = Arc::new(parking_lot::Mutex::new(Vec::new()));
        let styles = Arc::new(parking_lot::Mutex::new(StyleRegistry::new()));
        let id = ComponentId::of::<F>();
        Self {
            ctx: Ctx::new(id, dirty, commands, styles, Default::default()),
            component: Box::new(component),
            selector: selector.into(),
            mounted: false,
        }
    }

    pub fn render(&mut self, lui: &mut Lui) {
        self.ctx.begin_render();
        let el = (self.component)(&self.ctx);
        self.ctx.end_render();
        let node = el.into_node();

        if let Some(mount) = lui.doc.root.query_selector_mut(&self.selector) {
            mount.set_children(vec![node]);
        }

        self.ctx.flush_styles(lui);
        self.ctx.clear_dirty();

        if !self.mounted {
            self.mounted = true;
            for cb in self.ctx.drain_mounted_callbacks() {
                cb();
            }
        }
    }

    pub fn process(&mut self, lui: &mut Lui) -> bool {
        if !self.ctx.is_dirty() {
            return false;
        }

        for cmd in self.ctx.drain_commands() {
            cmd(lui);
        }

        self.render(lui);
        true
    }
}
