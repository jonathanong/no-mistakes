use super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Function;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    /// Copied evaluation frames share inherited binding ownership. Declarations
    /// and ordinary arguments still create invocation-local bindings.
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn register_captured_bindings(
        &mut self,
        scope: Environment,
        captured: Environment,
        function: &Function,
    ) {
        let shadowed = super::super::calls::scopes::shadow_names(function);
        let bindings = self.scopes[captured]
            .keys()
            .filter(|name| !shadowed.contains(name.as_str()))
            .map(|name| {
                let origin = self
                    .captured_bindings
                    .get(&captured)
                    .and_then(|bindings| bindings.get(name))
                    .copied()
                    .unwrap_or(captured);
                (name.clone(), origin)
            })
            .collect::<FxHashMap<_, _>>();
        if !bindings.is_empty() {
            for (name, origin) in &bindings {
                self.captured_binding_readers
                    .entry(*origin)
                    .or_default()
                    .entry(name.clone())
                    .or_default()
                    .insert(scope);
            }
            self.captured_bindings.insert(scope, bindings.into());
        }
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn write_captured_binding(
        &mut self,
        env: Environment,
        name: &str,
        value: &Value,
    ) {
        let frames = self.captured_write_targets(env, name);
        for frame in frames {
            self.update_mapped_parameter(frame, name, value);
            self.scopes[frame].insert(name.to_string(), value.clone());
        }
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn write_captured_binding_evaluated(
        &mut self,
        env: Environment,
        name: &str,
        value: &Value,
        depth: u8,
    ) {
        let mut mapped = Vec::new();
        let mut escaped = false;
        for frame in self.captured_write_targets(env, name) {
            let (connected, was_escaped) = self.sync_mapped_parameter(frame, name, value);
            if connected {
                mapped.push(frame);
            }
            escaped |= was_escaped;
            self.scopes[frame].insert(name.to_string(), value.clone());
        }
        for frame in mapped {
            self.mark_mapped_fresh(frame, name);
        }
        // Callback effects happen after assignment and may revoke its proof.
        if escaped {
            self.opaque_callbacks(std::slice::from_ref(value), depth);
        }
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn captured_write_targets(
        &self,
        env: Environment,
        name: &str,
    ) -> Vec<Environment> {
        let origin = self
            .captured_bindings
            .get(&env)
            .and_then(|bindings| bindings.get(name))
            .copied()
            .unwrap_or(env);
        let mut frames = vec![origin];
        if let Some(readers) = self
            .captured_binding_readers
            .get(&origin)
            .and_then(|bindings| bindings.get(name))
        {
            frames.extend(readers.iter().copied());
        }
        frames
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn rebuild_captured_readers(
        &mut self,
    ) {
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("postgres.annotation.captured_reader_rebuilds", 1);
        }
        self.captured_binding_readers.clear();
        for (frame, bindings) in &self.captured_bindings {
            for (name, origin) in bindings {
                self.captured_binding_readers
                    .entry(*origin)
                    .or_default()
                    .entry(name.clone())
                    .or_default()
                    .insert(*frame);
            }
        }
    }

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
        for (frame, bindings) in &self.captured_bindings {
            for (name, origin) in bindings {
                let value = self.scopes[*origin].get(name).unwrap_or(&Value::Unknown);
                // A missing binding must still acquire Unknown; equal writes
                // would detach shared scopes and invalidate their indexes.
                if self.scopes[*frame].get(name) != Some(value) {
                    let value = value.clone();
                    self.scopes[*frame].insert(name.clone(), value);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
