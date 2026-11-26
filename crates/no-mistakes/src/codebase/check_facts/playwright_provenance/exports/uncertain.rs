use super::{
    CheckFileFacts, EdgeIndex, EdgeKind, ExportKey, ExportLookup, FxHashSet, UNRESOLVED_EXPORT,
};
use std::path::Path;

pub(super) fn is_declaration(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
}

impl ExportLookup<'_> {
    pub(super) fn record_uncertain(
        &mut self,
        next: &mut Vec<ExportKey>,
        key: &ExportKey,
        facts: Option<&CheckFileFacts>,
    ) {
        let Some((reason, refs)) = facts.and_then(|file| {
            let binding = file
                .ts
                .exported_bindings
                .iter()
                .find(|binding| binding.exported == key.name.as_ref());
            if binding.is_some_and(|binding| binding.specifier.is_some()) {
                return None;
            }
            let local_name = binding.map_or(key.name.as_ref(), |binding| binding.local.as_str());
            file.fixture_initializers
                .as_ref()?
                .uncertain
                .get(local_name)
        }) else {
            return;
        };
        if refs
            .iter()
            .any(|import| import.is_playwright_test() || self.local(&key.file, import).is_none())
        {
            self.uncertain.insert(key.clone(), reason);
            return;
        }
        let deps: Vec<_> = refs
            .iter()
            .filter_map(|import| self.local(&key.file, import))
            .collect();
        if !deps.is_empty() {
            next.extend(deps.iter().cloned());
            self.uncertain_deps.insert(key.clone(), (reason, deps));
        }
    }

    pub(in crate::codebase::check_facts::playwright_provenance) fn uncertainty_reason(
        &self,
        relationships: &EdgeIndex<ExportKey, EdgeKind>,
        root: &ExportKey,
    ) -> Option<&'static str> {
        let mut reason = None;
        let mut seen = FxHashSet::default();
        let mut stack = vec![root.clone()];
        while let Some(key) = stack.pop() {
            if !seen.insert(key.clone()) {
                continue;
            }
            if let Some(&direct) = self.uncertain.get(&key) {
                reason = Some(prefer_reason(reason, direct));
            }
            if let Some((candidate, deps)) = self.uncertain_deps.get(&key) {
                if deps
                    .iter()
                    .any(|dep| self.proves_playwright(relationships, dep))
                {
                    reason = Some(prefer_reason(reason, candidate));
                }
                stack.extend(deps.iter().cloned());
            }
            if let Some(neighbors) = relationships.forward().get(&key) {
                stack.extend(neighbors.iter().map(|(target, _)| target.clone()));
            }
        }
        reason
    }
}

fn prefer_reason(current: Option<&'static str>, next: &'static str) -> &'static str {
    match current {
        None | Some(UNRESOLVED_EXPORT) => next,
        Some(current) if next == UNRESOLVED_EXPORT => current,
        Some(current) => current.min(next),
    }
}
