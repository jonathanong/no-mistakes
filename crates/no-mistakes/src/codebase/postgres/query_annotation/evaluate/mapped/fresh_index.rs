use super::{Environment, Evaluator};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn index_mapped_freshness(
        &mut self,
        env: Environment,
    ) {
        if let Some(names) = self.fresh_mapped_parameters.get(&env) {
            for (id, params) in self.mapped_arguments.get(&env).into_iter().flatten() {
                for name in names {
                    if self.mapped_parameter_index(*id, params, name).is_none() {
                        continue;
                    }
                    self.fresh_mapped_argument_bindings
                        .entry(*id)
                        .or_default()
                        .entry(env)
                        .or_default()
                        .insert(name.clone());
                }
            }
        }
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn mark_mapped_fresh(
        &mut self,
        env: Environment,
        name: &str,
    ) {
        self.fresh_mapped_parameters
            .entry(env)
            .or_default()
            .insert(name.to_string());
        for (id, params) in self.mapped_arguments.get(&env).into_iter().flatten() {
            if let Some(index) = self.mapped_parameter_index(*id, params, name) {
                if !self.disconnected_argument_slots.contains(&(*id, index)) {
                    self.fresh_mapped_argument_bindings
                        .entry(*id)
                        .or_default()
                        .entry(env)
                        .or_default()
                        .insert(name.to_string());
                }
            }
        }
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn rebuild_fresh_mapped_argument_bindings(
        &mut self,
    ) {
        self.fresh_mapped_argument_bindings.clear();
        let environments = self
            .fresh_mapped_parameters
            .keys()
            .copied()
            .collect::<Vec<_>>();
        for env in environments {
            self.index_mapped_freshness(env);
        }
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn rebuild_binding_indexes(
        &mut self,
    ) {
        self.rebuild_captured_readers();
        self.rebuild_mapped_argument_owners();
        self.rebuild_fresh_mapped_argument_bindings();
    }
}
