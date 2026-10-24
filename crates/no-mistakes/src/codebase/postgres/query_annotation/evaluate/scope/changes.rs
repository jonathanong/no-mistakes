use super::{Bindings, Scope};
use crate::fx::FxHashSet;
use std::sync::{Arc, Weak};

const MAX_DEPTH: usize = 32;
const MAX_NAMES: usize = 256;

// Only keys and weak snapshot identities survive, never old binding maps.
// Cold scopes allocate no journal; the first keyed copy-on-write starts one.
#[derive(Clone, Default)]
pub(super) struct Changes(Option<Arc<Node>>);
#[derive(Clone)]
struct Node {
    origin: Weak<Bindings>,
    parent: Option<Arc<Node>>,
    names: FxHashSet<String>,
    depth: usize,
    retained_names: usize,
}
impl Changes {
    pub(super) fn is_empty(&self) -> bool {
        self.0.is_none()
    }
    pub(super) fn clear(&mut self) {
        self.0 = None;
    }
    pub(super) fn fork(&self, origin: Weak<Bindings>) -> Self {
        let parent = self
            .0
            .as_ref()
            .filter(|node| node.depth < MAX_DEPTH)
            .cloned();
        let depth = parent.as_ref().map_or(0, |parent| parent.depth + 1);
        let retained_names = parent.as_ref().map_or(0, |parent| parent.retained_names);
        Self(Some(Arc::new(Node {
            origin,
            parent,
            names: FxHashSet::default(),
            depth,
            retained_names,
        })))
    }
    pub(super) fn record(&mut self, name: &str) {
        let Some(current) = &mut self.0 else {
            return;
        };
        if current.names.len() >= MAX_NAMES {
            // Losing ancestry is conservative: that comparison uses a full scan.
            self.clear();
            return;
        }
        let node = Arc::make_mut(current);
        if node.retained_names >= MAX_NAMES {
            // Bound the complete ancestor chain, not just the current write set.
            node.parent = None;
            node.depth = 0;
            node.retained_names = node.names.len();
        }
        if node.names.insert(name.to_owned()) {
            node.retained_names += 1;
        }
    }
    pub(super) fn since(&self, original: &Scope) -> Option<FxHashSet<String>> {
        let original = Arc::as_ptr(&original.0);
        let mut match_at = self.0.as_ref()?;
        // Probe ancestry before allocating keys: unrelated snapshots keep the
        // old full-scan path without paying for a speculative dirty-name set.
        while match_at.origin.as_ptr() != original {
            match_at = match_at.parent.as_ref()?;
        }
        let mut current = self.0.as_ref()?;
        let mut names = FxHashSet::default();
        loop {
            names.extend(current.names.iter().cloned());
            if Arc::ptr_eq(current, match_at) {
                return Some(names);
            }
            current = current.parent.as_ref()?;
        }
    }
}

#[cfg(test)]
mod tests;
