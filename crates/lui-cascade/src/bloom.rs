use std::hash::{Hash, Hasher};
use rustc_hash::FxHasher;

const BLOOM_SIZE: usize = 256;

/// Counting bloom filter for ancestor tag/id/class membership.
/// Supports push (enter child) and pop (leave child) as the
/// cascade walks the tree.
#[derive(Clone, Copy)]
pub struct AncestorBloom {
    counts: [u8; BLOOM_SIZE],
}

impl AncestorBloom {
    pub fn new() -> Self {
        Self { counts: [0; BLOOM_SIZE] }
    }

    pub fn insert_tag(&mut self, tag: &str) {
        let idx = bloom_hash(tag, 0);
        self.counts[idx] = self.counts[idx].saturating_add(1);
    }

    pub fn insert_id(&mut self, id: &str) {
        let idx = bloom_hash(id, 1);
        self.counts[idx] = self.counts[idx].saturating_add(1);
    }

    pub fn insert_class(&mut self, class: &str) {
        let idx = bloom_hash(class, 2);
        self.counts[idx] = self.counts[idx].saturating_add(1);
    }

    pub fn remove_tag(&mut self, tag: &str) {
        let idx = bloom_hash(tag, 0);
        self.counts[idx] = self.counts[idx].saturating_sub(1);
    }

    pub fn remove_id(&mut self, id: &str) {
        let idx = bloom_hash(id, 1);
        self.counts[idx] = self.counts[idx].saturating_sub(1);
    }

    pub fn remove_class(&mut self, class: &str) {
        let idx = bloom_hash(class, 2);
        self.counts[idx] = self.counts[idx].saturating_sub(1);
    }

    pub fn might_contain_tag(&self, tag: &str) -> bool {
        self.counts[bloom_hash(tag, 0)] > 0
    }

    pub fn might_contain_id(&self, id: &str) -> bool {
        self.counts[bloom_hash(id, 1)] > 0
    }

    pub fn might_contain_class(&self, class: &str) -> bool {
        self.counts[bloom_hash(class, 2)] > 0
    }

    /// Push an element's tag/id/classes into the filter.
    pub fn push(&mut self, tag: &str, id: Option<&str>, class_list: &[impl AsRef<str>]) {
        self.insert_tag(tag);
        if let Some(id) = id {
            self.insert_id(id);
        }
        for c in class_list {
            self.insert_class(c.as_ref());
        }
    }

    /// Pop an element's tag/id/classes from the filter.
    pub fn pop(&mut self, tag: &str, id: Option<&str>, class_list: &[impl AsRef<str>]) {
        self.remove_tag(tag);
        if let Some(id) = id {
            self.remove_id(id);
        }
        for c in class_list {
            self.remove_class(c.as_ref());
        }
    }
}

fn bloom_hash(s: &str, salt: u8) -> usize {
    let mut h = FxHasher::default();
    salt.hash(&mut h);
    s.hash(&mut h);
    (h.finish() as usize) % BLOOM_SIZE
}

/// Quick-reject: can a complex selector's ancestor compounds possibly
/// match given the current bloom filter? Returns false if any ancestor
/// compound requires a tag/id/class that's definitely not in the filter.
pub fn bloom_might_match(
    selector: &lui_css_parser::selector::ComplexSelector,
    bloom: &AncestorBloom,
) -> bool {
    let compounds = &selector.compounds;
    if compounds.len() <= 1 {
        return true;
    }
    // Check all non-subject compounds (ancestor requirements)
    for compound in &compounds[..compounds.len() - 1] {
        if let Some(ref tag) = compound.tag {
            if tag != "*" && !bloom.might_contain_tag(tag) {
                return false;
            }
        }
        if let Some(ref id) = compound.id {
            if !bloom.might_contain_id(id) {
                return false;
            }
        }
        for class in &compound.classes {
            if !bloom.might_contain_class(class) {
                return false;
            }
        }
    }
    true
}
