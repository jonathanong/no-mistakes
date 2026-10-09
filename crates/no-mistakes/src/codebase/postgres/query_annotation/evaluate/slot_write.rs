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
        let previously_invalidated = self.invalidated_builders.contains(&id);
        let disconnected = self.disconnected_argument_slots.contains(&(id, index));
        self.definite_deleted_argument_slots.remove(&(id, index));
        self.deleted_argument_slots.remove(&(id, Some(index)));
        let Some(slot) = self
            .argument_objects
            .get_mut(&id)
            .and_then(|values| values.get_mut(index))
        else {
            self.argument_extra_slots
                .entry(id)
                .or_default()
                .insert(index, stored.clone());
            if disconnected {
                self.opaque_callbacks(std::slice::from_ref(&stored), depth);
            }
            return Value::Evaluated(Box::new(stored), handled_value);
        };
        *slot = stored.clone();
        if disconnected {
            // A later write creates an argument property, but it does not
            // reconnect the deleted formal parameter. Preserve callback
            // effects from the new value without restoring that alias.
            self.opaque_callbacks(std::slice::from_ref(&stored), depth);
        } else if !previously_invalidated {
            self.write_mapped_argument_slot(id, index, &stored);
        }
        if previously_invalidated {
            self.invalidate_mapped_freshness(&FxHashSet::from_iter([id]));
        }
        Value::Evaluated(Box::new(stored), handled_value)
    }
}

#[cfg(test)]
#[path = "slot_write/tests.rs"]
mod tests;
