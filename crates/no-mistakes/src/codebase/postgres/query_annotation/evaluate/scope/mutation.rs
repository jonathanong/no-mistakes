use super::*;

// Below this size, a borrowed scan costs less than journal allocation.
const MIN_TRACKED_BINDINGS: usize = 32;

impl Scope {
    #[inline]
    fn keyed_values(&mut self, name: &str) -> &mut FxHashMap<String, Value> {
        if self.len() < MIN_TRACKED_BINDINGS {
            return self.deref_mut();
        }
        let shared = Arc::strong_count(&self.0) > 1;
        if !shared && self.1.is_empty() {
            return self.deref_mut();
        }
        let changes = shared.then(|| self.1.fork(Arc::downgrade(&self.0)));
        if let Some(changes) = changes {
            self.1 = changes;
        }
        self.1.record(name);
        let bindings = Arc::make_mut(&mut self.0);
        bindings.invalidate();
        &mut bindings.values
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn insert(
        &mut self,
        name: String,
        value: Value,
    ) -> Option<Value> {
        self.keyed_values(&name).insert(name, value)
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn get_mut(
        &mut self,
        name: &str,
    ) -> Option<&mut Value> {
        self.keyed_values(name).get_mut(name)
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn remove(
        &mut self,
        name: &str,
    ) -> Option<Value> {
        self.keyed_values(name).remove(name)
    }
    #[inline]
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn has_binding_changes(
        &self,
    ) -> bool {
        !self.1.is_empty()
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn changed_binding_names(
        &self,
        original: &Self,
    ) -> Option<FxHashSet<String>> {
        if self.len() < MIN_TRACKED_BINDINGS {
            return None;
        }
        if self.same_bindings(original) {
            return Some(FxHashSet::default());
        }
        self.1.since(original)
    }
}
