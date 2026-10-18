use super::{Environment, Evaluator, Value};
use crate::fx::FxHashSet;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn invalidate_mapped_freshness(
        &mut self,
        ids: &FxHashSet<u64>,
    ) {
        let mut writes = Vec::new();
        for id in ids {
            if let Some(bindings) = self.fresh_mapped_argument_bindings.get(id) {
                for (env, names) in bindings {
                    writes.extend(names.iter().map(|name| (*env, name.clone())));
                }
            }
        }
        for (env, name) in writes {
            self.set_transaction_flag(super::super::transactions::Cell::Fresh(env, name), false);
        }
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn invalidate_opaque_mapped_targets(
        &mut self,
        targets: &[String],
        env: Environment,
    ) {
        let mut ids = FxHashSet::default();
        for name in targets {
            for (id, params) in self.mapped_arguments.get(&env).into_iter().flatten() {
                if self.mapped_parameter_index(*id, params, name).is_some() {
                    ids.insert(*id);
                }
            }
        }
        self.invalidate_mapped_freshness(&ids);
        for id in ids {
            self.set_transaction_flag(super::super::transactions::Cell::Invalidated(id), true);
        }
    }

    pub(super) fn inherit_freshness(
        &mut self,
        scope: Environment,
        captured: Environment,
        mappings: &[(u64, Vec<String>)],
    ) {
        let mut fresh = self
            .fresh_mapped_parameters
            .get(&captured)
            .cloned()
            .unwrap_or_default();
        fresh.retain(|name| {
            mappings
                .iter()
                .any(|(id, params)| self.mapped_parameter_index(*id, params, name).is_some())
        });
        if !fresh.is_empty() {
            self.fresh_mapped_parameters.insert(scope, fresh);
        }
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn mapped_parameter_value(
        &self,
        env: Environment,
        name: &str,
    ) -> Option<Value> {
        if !self.mapped_parameter_unknown(env, name) {
            return None;
        }
        let mut possibilities = Vec::new();
        for (id, params) in &self.mapped_arguments[&env] {
            if let Some(index) = self.mapped_parameter_index(*id, params, name) {
                if !self.disconnected_argument_slots.contains(&(*id, index)) {
                    // A possible deletion may have disconnected the formal
                    // before this property's later write; retain both outcomes.
                    if self.deleted_argument_slots.contains(&(*id, None))
                        || self.deleted_argument_slots.contains(&(*id, Some(index)))
                    {
                        if let Some(previous) = self.scopes[env].get(name) {
                            let previous = previous.clone().exposed();
                            if !matches!(previous, Value::Unknown)
                                && !possibilities.contains(&previous)
                            {
                                possibilities.push(previous);
                            }
                        }
                    }
                    let value = &self.argument_objects[id][index];
                    let value = value.clone().exposed();
                    if !matches!(value, Value::Unknown) && !possibilities.contains(&value) {
                        possibilities.push(value);
                    }
                }
            }
        }
        Some(if possibilities.is_empty() {
            Value::Unknown
        } else {
            Value::Possible(possibilities.into())
        })
    }
}
