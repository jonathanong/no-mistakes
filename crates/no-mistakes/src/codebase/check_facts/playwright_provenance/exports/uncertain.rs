use super::{CheckFileFacts, EdgeIndex, EdgeKind, ExportKey, ExportLookup, FxHashSet, ImportRef};

impl ExportLookup<'_> {
    pub(super) fn record_uncertain(
        &mut self,
        next: &mut Vec<ExportKey>,
        key: &ExportKey,
        facts: Option<&CheckFileFacts>,
    ) {
        let Some(file) = facts else {
            return;
        };
        let local_name = file
            .ts
            .exported_bindings
            .iter()
            .find(|binding| binding.exported == key.name.as_ref())
            .map_or(key.name.as_ref(), |binding| binding.local.as_str());
        let Some(refs) = file
            .fixture_initializers
            .as_ref()
            .and_then(|fixtures| fixtures.uncertain.get(local_name))
        else {
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
        self.reaches_uncertain_inner(relationships, root, &mut seen)
    }

    fn reaches_uncertain_inner(
        &self,
        relationships: &EdgeIndex<ExportKey, EdgeKind>,
        root: &ExportKey,
        seen: &mut FxHashSet<ExportKey>,
    ) -> bool {
        for key in self.reachable_exports(relationships, root) {
            if !seen.insert(key.clone()) {
                continue;
            }
            if self.uncertain.contains(&key) {
                return true;
            }
            if let Some(deps) = self.uncertain_deps.get(&key) {
                if deps.iter().any(|dep| {
                    self.proves_playwright(relationships, dep)
                        || self.reaches_uncertain_inner(relationships, dep, seen)
                }) {
                    return true;
                }
            }
        }
        false
    }
}
