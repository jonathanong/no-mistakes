mod builder_ids;
#[cfg(test)]
mod tests;

use super::{Environment, Evaluator, Value};
use crate::fx::FxHashSet;
use std::path::PathBuf;

impl<F: Fn(&str, &std::path::Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn invalidate_live_builders(&mut self) {
        let values = self.live_builder_values();
        self.invalidate_opaque_builders(&values);
    }

    pub(super) fn invalidate_opaque_builders(&mut self, values: &[Value]) {
        // Opaque hooks can patch intrinsics even when no builder exists yet.
        self.set_transaction_flag(super::transactions::Cell::Trust, true);
        self.invalidate_builders(values);
    }

    pub(super) fn live_builder_values(&self) -> Vec<Value> {
        self.scopes
            .iter()
            .flat_map(|scope| scope.values().cloned())
            .collect()
    }

    pub(super) fn invalidate_builders(&mut self, values: &[Value]) {
        let ids = self.collect_builder_ids(values, std::mem::drop);
        self.invalidate_builder_ids(ids);
    }

    fn invalidate_builder_ids(&mut self, ids: FxHashSet<u64>) {
        // Most opaque calls carry no SQL builders. Do not scan every frame
        // (or detach shared snapshots) when there is nothing to invalidate.
        if ids.is_empty() {
            return;
        }
        self.invalidate_mapped_freshness(&ids);
        for id in &ids {
            self.set_transaction_flag(super::transactions::Cell::Invalidated(*id), true);
        }
        let mut writes = Vec::new();
        for cell in self.builder_locations(&ids) {
            if let super::transactions::Cell::Binding(env, name) = cell {
                if matches!(self.scopes[env].get(&name), Some(Value::Prefix(_, _, Some(id))) if ids.contains(id))
                {
                    writes.push((env, name));
                }
            }
        }
        for (env, name) in writes {
            self.write_binding(env, name, Value::Unknown);
        }
    }

    pub(super) fn invalidate_captured(&mut self, env: Environment, function: &super::Function) {
        let mut shadowed = function.params.iter().cloned().collect::<FxHashSet<_>>();
        shadowed.extend(function.self_name.iter().cloned());
        if !function.arrow {
            shadowed.insert("arguments".into());
        }
        for step in &function.body {
            match step {
                crate::codebase::postgres::query_annotation::Step::Bind(name, _)
                | crate::codebase::postgres::query_annotation::Step::Hoisted(name, _)
                | crate::codebase::postgres::query_annotation::Step::Var(name) => {
                    shadowed.insert(name.clone());
                }
                crate::codebase::postgres::query_annotation::Step::Reserve(names) => {
                    shadowed.extend(names.iter().cloned())
                }
                _ => {}
            }
        }
        let values = self.scopes[env]
            .iter()
            .filter(|(name, _)| !shadowed.contains(*name))
            .map(|(_, value)| value.clone())
            .collect::<Vec<_>>();
        self.invalidate_builders(&values);
        let names = self.scopes[env]
            .iter()
            .filter_map(|(name, value)| {
                (!shadowed.contains(name) && matches!(value, Value::Prefix(_, _, _)))
                    .then(|| name.clone())
            })
            .collect::<Vec<_>>();
        for name in names {
            self.write_binding(env, name, Value::Unknown);
        }
    }
}
mod replace;
