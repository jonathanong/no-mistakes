use super::{CheckFileFacts, EdgeIndex, EdgeKind, ExportKey, ExportLookup, FxHashSet, ImportRef};

impl ExportLookup<'_> {
    pub(super) fn record_uncertain(
        &mut self,
        next: &mut Vec<ExportKey>,
        key: &ExportKey,
        facts: Option<&CheckFileFacts>,
    ) {
        let Some(refs) = facts.and_then(|file| {
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
        if refs.iter().any(ImportRef::is_playwright_test) {
            self.uncertain.insert(key.clone());
            return;
        }
        let deps: Vec<_> = refs
            .iter()
            .filter_map(|import| self.local(&key.file, import))
            .collect();
        if !deps.is_empty() {
            next.extend(deps.iter().cloned());
            self.uncertain_deps.insert(key.clone(), deps);
        }
    }

    pub(in crate::codebase::check_facts::playwright_provenance) fn reaches_uncertain(
        &self,
        relationships: &EdgeIndex<ExportKey, EdgeKind>,
        root: &ExportKey,
    ) -> bool {
        let mut seen = FxHashSet::default();
        let mut stack = vec![root.clone()];
        while let Some(key) = stack.pop() {
            if !seen.insert(key.clone()) {
                continue;
            }
            if self.uncertain.contains(&key) {
                return true;
            }
            if let Some(deps) = self.uncertain_deps.get(&key) {
                if deps
                    .iter()
                    .any(|dep| self.proves_playwright(relationships, dep))
                {
                    return true;
                }
                stack.extend(deps.iter().cloned());
            }
            if let Some(neighbors) = relationships.forward().get(&key) {
                stack.extend(neighbors.iter().map(|(target, _)| target.clone()));
            }
        }
        false
    }
}
