use super::{Environment, Evaluator, Value};
use crate::fx::FxHashSet;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn invalidate_mapped_freshness(
        &mut self,
        ids: &FxHashSet<u64>,
    ) {
        for id in ids {
            if let Some(bindings) = self.fresh_mapped_argument_bindings.remove(id) {
                for (env, names) in bindings {
                    if let std::collections::hash_map::Entry::Occupied(mut entry) =
                        self.fresh_mapped_parameters.entry(env)
                    {
                        entry.get_mut().retain(|name| !names.contains(name));
                        if entry.get().is_empty() {
                            entry.remove();
                        }
                    }
                }
            }
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
                if params.contains(name) {
                    ids.insert(*id);
                }
            }
        }
        self.invalidate_mapped_freshness(&ids);
        self.invalidated_builders.extend(ids);
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
        fresh.retain(|name| mappings.iter().any(|(_, params)| params.contains(name)));
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
            if let Some(index) = params.iter().rposition(|param| param == name) {
                if !self.disconnected_argument_slots.contains(&(*id, index)) {
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
            Value::Possible(possibilities)
        })
    }
}
