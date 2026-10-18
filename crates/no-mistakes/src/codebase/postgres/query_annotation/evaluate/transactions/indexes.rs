use super::Cell;
use crate::codebase::postgres::query_annotation::evaluate::{Environment, Evaluator};
use std::path::{Path, PathBuf};
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn restore_transaction_index(
        &mut self,
        cell: &Cell,
    ) {
        match cell {
            Cell::Fresh(env, name) => self.sync_fresh_reverse(*env, name),
            Cell::Disconnected(id, index) => {
                if let Some((owner, name)) = self.mapped_slot_target(*id, *index) {
                    for env in self.captured_write_targets(owner, &name) {
                        self.sync_fresh_reverse(env, &name);
                    }
                }
            }
            _ => {}
        }
    }
    fn sync_fresh_reverse(&mut self, env: Environment, name: &str) {
        let fresh = self
            .fresh_mapped_parameters
            .get(&env)
            .is_some_and(|names| names.contains(name));
        for (id, params) in self.mapped_arguments.get(&env).into_iter().flatten() {
            let Some(index) = self.mapped_parameter_index(*id, params, name) else {
                continue;
            };
            if fresh && !self.disconnected_argument_slots.contains(&(*id, index)) {
                self.fresh_mapped_argument_bindings
                    .entry(*id)
                    .or_default()
                    .entry(env)
                    .or_default()
                    .insert(name.to_string());
            } else if let Some(bindings) = self.fresh_mapped_argument_bindings.get_mut(id) {
                if let Some(names) = bindings.get_mut(&env) {
                    names.remove(name);
                    if names.is_empty() {
                        bindings.remove(&env);
                    }
                }
                if bindings.is_empty() {
                    self.fresh_mapped_argument_bindings.remove(id);
                }
            }
        }
    }
}
