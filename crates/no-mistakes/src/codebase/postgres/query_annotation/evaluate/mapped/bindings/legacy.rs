use super::*;
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    /// An alternative only creates capture ownership on frames at or above its
    /// entry depth. Remove those readers before restoring an arm's bindings;
    /// readers on earlier frames still describe the entry snapshot exactly.
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn remove_alternative_captured_readers(
        &mut self,
        base: Environment,
    ) {
        if self.captured_bindings.keys().all(|frame| *frame >= base) {
            self.captured_binding_readers.clear();
            return;
        }
        for (&frame, bindings) in self
            .captured_bindings
            .iter()
            .filter(|(frame, _)| **frame >= base)
        {
            for (name, origin) in bindings {
                // Registration and ownership-changing compaction keep this
                // reverse index exact before each alternative restoration.
                let names = self
                    .captured_binding_readers
                    .get_mut(origin)
                    .expect("registered capture origin has reverse readers");
                let readers = names
                    .get_mut(name)
                    .expect("registered capture has a reverse reader");
                readers.remove(&frame);
                if readers.is_empty() {
                    names.remove(name);
                }
                if names.is_empty() {
                    self.captured_binding_readers.remove(origin);
                }
            }
        }
    }

    /// Reinstall readers for arm-created frames, including module-initial
    /// frames restored from an earlier arm. Compaction rebuilds this index if
    /// it moves or drops any capture ownership.
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn add_alternative_captured_readers(
        &mut self,
        base: Environment,
    ) {
        for (&frame, bindings) in self
            .captured_bindings
            .iter()
            .filter(|(frame, _)| **frame >= base)
        {
            for (name, origin) in bindings {
                self.captured_binding_readers
                    .entry(*origin)
                    .or_default()
                    .entry(name.clone())
                    .or_default()
                    .insert(frame);
            }
        }
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn refresh_captured_bindings(
        &mut self,
    ) {
        let mut writes = Vec::new();
        for (frame, bindings) in &self.captured_bindings {
            for (name, origin) in bindings {
                let value = self.scopes[*origin].get(name).unwrap_or(&Value::Unknown);
                // A missing binding must still acquire Unknown; equal writes
                // would detach shared scopes and invalidate their indexes.
                if self.scopes[*frame].get(name) != Some(value) {
                    let value = value.clone();
                    writes.push((*frame, name.clone(), value));
                }
            }
        }
        for (frame, name, value) in writes {
            self.write_binding(frame, name, value);
        }
    }
}
