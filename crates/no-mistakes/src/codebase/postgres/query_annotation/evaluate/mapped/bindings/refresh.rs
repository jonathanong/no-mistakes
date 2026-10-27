use super::*;
use crate::fx::FxHashSet;

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn refresh_captured_bindings(
        &mut self,
    ) {
        self.refresh_captured_bindings_with_visit(std::mem::drop);
    }

    fn refresh_captured_bindings_with_visit(&mut self, mut visit: impl FnMut(())) {
        for (frame, bindings) in &self.captured_bindings {
            // Empty ownership never indexed a frame or origin in the original loop.
            if bindings.is_empty() {
                continue;
            }
            if self
                .scopes
                .get(*frame)
                .is_some_and(|scope| bindings.refresh_is_current(scope, &self.scopes))
            {
                continue;
            }
            let mut origins = FxHashSet::default();
            for (name, origin) in bindings {
                visit(());
                origins.insert(*origin);
                let value = self.scopes[*origin].get(name).unwrap_or(&Value::Unknown);
                // A missing binding must still acquire Unknown; equal writes
                // would detach shared scopes and invalidate their indexes.
                if self.scopes[*frame].get(name) != Some(value) {
                    let value = value.clone();
                    self.scopes[*frame].insert(name.clone(), value);
                }
            }
            // Record post-write states, including frame-as-origin ownership.
            // A later outer iteration can change these origins and force a miss.
            bindings.remember_refresh(&self.scopes[*frame], &origins, &self.scopes);
        }
    }
}

#[cfg(test)]
mod tests;
