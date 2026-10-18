use super::{Cell, Checkpoint, Saved};
use crate::codebase::postgres::query_annotation::evaluate::{Evaluator, Value};
use crate::fx::FxHashSet;
use std::path::{Path, PathBuf};
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn revoke_changed_freshness(
        &mut self,
        checkpoint: Checkpoint,
    ) {
        let changed = self.transactions.frames[checkpoint.0]
            .old
            .iter()
            .filter_map(|(cell, before)| {
                if let Cell::Binding(env, name) = cell {
                    if matches!(before, Saved::Value(value) if self.scopes[*env].get(name) != value.as_ref())
                        && self
                            .fresh_mapped_parameters
                            .get(env)
                            .is_some_and(|names| names.contains(name))
                    {
                        return Some((*env, name.clone()));
                    }
                }
                None
            })
            .collect::<Vec<_>>();
        for (env, name) in changed {
            self.set_transaction_flag(Cell::Fresh(env, name), false);
        }
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn taint_changed_bindings(
        &mut self,
        changed: &FxHashSet<u64>,
    ) {
        if changed.is_empty() {
            return;
        }
        let mut bindings = Vec::new();
        for cell in self.builder_locations(changed) {
            if let Cell::Binding(env, name) = cell {
                if matches!(self.scopes[env].get(&name), Some(Value::Prefix(_, _, Some(id))) if changed.contains(id))
                {
                    bindings.push((env, name));
                }
            }
        }
        for (env, name) in bindings {
            self.write_binding(env, name, Value::Unknown);
        }
        for id in changed {
            self.set_transaction_flag(Cell::Invalidated(*id), true);
        }
    }
}
