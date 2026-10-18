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
        let raw_receiver = self.expr(receiver, path, env, depth, generic);
        let handled_receiver = self.preserve_operand_effect(receiver, &raw_receiver, path, *env);
        let receiver = raw_receiver.exposed();
        let value = self.expr(expression, path, env, depth, generic);
        let handled_value =
            self.preserve_operand_effect(expression, &value, path, *env) && handled_receiver;
        let stored = match value.clone().exposed() {
            Value::SlotDeletion => Value::Unknown,
            value => value,
        };

        let id = match receiver {
            Value::Arguments(id) | Value::Array(id) => id,
            receiver => {
                self.invalidate_opaque_builders(&[receiver.clone(), stored.clone()]);
                self.opaque_callbacks(&[receiver, stored.clone()], depth);
                return Value::Evaluated(Box::new(stored), false);
            }
        };
        if self.conditional_collection_shape(id) {
            // A known slot write cannot recover a divergent array cardinality.
            self.invalidate_builders(&[Value::Array(id)]);
        }
        let previously_invalidated = self.invalidated_builders.contains(&id);
        let disconnected = self.disconnected_argument_slots.contains(&(id, index));
        let possibly_disconnected = self.deleted_argument_slots.contains(&(id, None))
            || self.deleted_argument_slots.contains(&(id, Some(index)));
        self.set_transaction_flag(super::transactions::Cell::Definite(id, index), false);
        // A definite deletion already froze the formal, so its index marker
        // can go away. A conditional marker stays: the next write must not
        // treat that formal as mapped again.
        if disconnected || !possibly_disconnected {
            self.set_transaction_flag(super::transactions::Cell::Deleted(id, Some(index)), false);
        }
        // This path assigned the property. A dynamic `(id, None)` marker
        // still covers indices that this write did not touch.
        self.set_transaction_flag(super::transactions::Cell::Recreated(id, index), true);
        if !self
            .argument_objects
            .get(&id)
            .is_some_and(|values| index < values.len())
        {
            self.write_sparse_slot(id, index, Some(stored.clone()));
            if previously_invalidated {
                self.opaque_callbacks(std::slice::from_ref(&stored), depth);
            }
            return Value::Evaluated(Box::new(stored), handled_value);
        }
        self.write_dense_slot(id, index, stored.clone());
        if previously_invalidated {
            self.opaque_callbacks(std::slice::from_ref(&stored), depth);
        } else if disconnected {
            // A later write creates an argument property, but it does not
            // reconnect the deleted formal parameter. A later escape will
            // visit any callbacks stored in the live property.
        } else if possibly_disconnected {
            self.write_possibly_disconnected_mapped_slot(id, index, &stored);
        } else {
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
