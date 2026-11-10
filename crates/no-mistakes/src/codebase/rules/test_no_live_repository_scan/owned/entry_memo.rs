//! Caller-entry walks, once per symbol for the run.
use super::Owners;
use crate::codebase::dependencies::graph::{NodeId, ResolvedCallSite};
use crate::codebase::rules::forbidden_calls::site_source_node;
use crate::fx::{FxHashMap, FxHashSet};

#[derive(Default)]
pub(super) struct EntryMemo<'g> {
    cache: FxHashMap<NodeId, Option<Vec<&'g ResolvedCallSite>>>,
    pub(super) walks: u64,
    pub(super) hits: u64,
}

impl<'g> Owners<'g> {
    /// Call sites that start a chain reaching `target`: sites whose own
    /// callable nothing calls. `None` when `target` itself has no caller.
    pub(super) fn entries(&self, target: NodeId) -> Option<Vec<&'g ResolvedCallSite>> {
        if let Some(cached) = self.cached_entries(&target) {
            return cached;
        }
        let computed = self.walk_entries(target.clone());
        self.entry_memo
            .borrow_mut()
            .cache
            .insert(target, computed.clone());
        computed
    }

    fn cached_entries(&self, target: &NodeId) -> Option<Option<Vec<&'g ResolvedCallSite>>> {
        let mut memo = self.entry_memo.borrow_mut();
        let cached = memo.cache.get(target)?.clone();
        memo.hits += 1;
        Some(cached)
    }

    fn walk_entries(&self, target: NodeId) -> Option<Vec<&'g ResolvedCallSite>> {
        self.entry_memo.borrow_mut().walks += 1;
        let mut seen = FxHashSet::default();
        let mut stack = vec![target];
        let mut entries = Vec::new();
        while let Some(node) = stack.pop() {
            if !seen.insert(node.clone()) {
                continue;
            }
            let callers = self.callers.get(&node)?;
            for caller in callers {
                let source = site_source_node(caller);
                if self.callers.contains_key(&source) {
                    stack.push(source);
                } else {
                    entries.push(*caller);
                }
            }
        }
        entries.sort_by(|left, right| (&left.file, left.offset).cmp(&(&right.file, right.offset)));
        entries.dedup_by(|left, right| left.file == right.file && left.offset == right.offset);
        Some(entries)
    }
}
