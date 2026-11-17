use super::*;

impl Scope {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn remove(
        &mut self,
        name: &str,
    ) -> Option<Value> {
        if !self.0.values.contains_key(name) {
            return None;
        }
        self.deref_mut().remove(name)
    }
}
