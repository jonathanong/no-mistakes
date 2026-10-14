use super::*;
impl State {
    pub(super) fn record_dependencies(&mut self, owner: Owner, dependencies: FxHashSet<Owner>) {
        for dependency in dependencies {
            if dependency != owner {
                self.reverse_dependencies
                    .entry(dependency)
                    .or_default()
                    .insert(owner.clone());
            }
        }
    }
}
impl ParameterCache {
    // Keep direct cache dependencies across generations; a changed owner reaches
    // transitive readers once through this request-owned reverse index.
    pub(in super::super) fn affected(&self, changed: FxHashSet<Owner>) -> FxHashSet<Owner> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let mut pending = changed.into_iter().collect::<Vec<_>>();
        let mut affected = fx_set();
        while let Some(owner) = pending.pop() {
            if !affected.insert(owner.clone()) {
                continue;
            }
            if let Some(readers) = state.reverse_dependencies.get(&owner) {
                pending.extend(readers.iter().cloned());
            }
        }
        affected
    }
    #[cfg(feature = "test-instrumentation")]
    pub(in super::super::super) fn dependency_links(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .reverse_dependencies
            .values()
            .map(FxHashSet::len)
            .sum()
    }
}
