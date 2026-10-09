use super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use crate::fx::FxHashSet;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn slot_write(
        &mut self,
        receiver: &Expr,
        index: usize,
        expression: &Expr,
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        let (depth, generic) = context;
        let receiver = self.expr(receiver, path, env, depth, generic).exposed();
        let value = self.expr(expression, path, env, depth, generic);
        let handled_value = self.handled_deletion_effect(expression, &value, path, *env);
        let stored = match value.clone().exposed() {
            Value::SlotDeletion => Value::Unknown,
            value => value,
        };

        let id = match receiver {
            Value::Arguments(id) => id,
            receiver => {
                self.invalidate_builders(&[receiver.clone(), stored.clone()]);
                self.opaque_callbacks(&[receiver, stored.clone()], depth);
                return Value::Evaluated(Box::new(stored), false);
            }
        };
        let Some(slot) = self
            .argument_objects
            .get_mut(&id)
            .and_then(|values| values.get_mut(index))
        else {
            self.invalidated_builders.insert(id);
            self.invalidate_mapped_freshness(&FxHashSet::from_iter([id]));
            self.invalidate_builders(std::slice::from_ref(&stored));
            self.opaque_callbacks(std::slice::from_ref(&stored), depth);
            return Value::Evaluated(Box::new(stored), false);
        };
        *slot = stored.clone();
        self.invalidated_builders.insert(id);
        if self.definite_deleted_argument_slots.contains(&(id, index)) {
            // A later write creates an argument property, but it does not
            // reconnect the deleted formal parameter. Preserve callback
            // effects from the new value without restoring that alias.
            self.opaque_callbacks(std::slice::from_ref(&stored), depth);
        } else {
            self.write_mapped_argument_slot(id, index, &stored);
        }
        self.invalidate_mapped_freshness(&FxHashSet::from_iter([id]));
        Value::Evaluated(Box::new(stored), handled_value)
    }
}

#[cfg(test)]
#[path = "slot_write/tests.rs"]
mod tests;
