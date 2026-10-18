use super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::{DeleteKey, Expr};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn deleted(
        &mut self,
        children: &[Expr],
        key: DeleteKey,
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        let depth = context.0;
        let values = children
            .iter()
            .map(|child| self.operand(child, path, env, context))
            .collect::<Vec<_>>();
        if let Some(Value::Arguments(id)) = values.first() {
            // Removing a slot does not mutate the builder it referenced.
            match key {
                DeleteKey::Index(index) => {
                    self.disconnect_mapped_slot(*id, index);
                    self.set_transaction_flag(
                        super::transactions::Cell::Disconnected(*id, index),
                        true,
                    );
                    self.set_transaction_flag(
                        super::transactions::Cell::Deleted(*id, Some(index)),
                        true,
                    );
                    self.set_transaction_flag(
                        super::transactions::Cell::Definite(*id, index),
                        true,
                    );
                    if self
                        .argument_objects
                        .get(id)
                        .is_some_and(|values| index < values.len())
                    {
                        self.write_dense_slot(*id, index, Value::Unknown);
                    }
                    self.write_sparse_slot(*id, index, None);
                }
                DeleteKey::Named => {}
                DeleteKey::Dynamic => {
                    self.set_transaction_flag(super::transactions::Cell::Deleted(*id, None), true);
                }
            }
            self.invalidate_builders(&values[1..]);
            if matches!(key, DeleteKey::Dynamic)
                && values.iter().skip(1).any(|value| {
                    !super::collections::string_element(value) && !matches!(value, Value::Primitive)
                })
            {
                // ToPropertyKey may execute an opaque conversion hook.
                self.invalidate_opaque_builders(&values[1..]);
            }
            self.opaque_callbacks(&values[1..], depth);
            // The complete receiver/key effects are handled precisely here.
            return Value::SlotDeletion;
        } else {
            self.invalidate_opaque_builders(&values);
            self.opaque_callbacks(&values, depth);
        }
        Value::Unknown
    }

    pub(super) fn operand(
        &mut self,
        expr: &Expr,
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        let value = self.expr(expr, path, env, context.0, context.1);
        if self.files[path].facts.precise_records && self.effect_can_mutate(expr, path, *env) {
            // Completed operand effects precede the next operand's executor events.
            self.preserve_operand_effect(expr, &value, path, *env);
        }
        value
    }

    pub(super) fn preserve_operand_effect(
        &mut self,
        expr: &Expr,
        value: &Value,
        path: &Path,
        env: Environment,
    ) -> bool {
        let handled = self.handled_deletion_effect(expr, value, path, env);
        if !handled {
            self.invalidate_live_builders();
        }
        handled
    }

    pub(super) fn handled_deletion_effect(
        &self,
        expr: &Expr,
        value: &Value,
        path: &Path,
        env: Environment,
    ) -> bool {
        match expr {
            _ if self
                .files
                .get(path)
                .is_some_and(|file| file.facts.precise_records)
                && matches!(value, Value::Evaluated(_, true)) =>
            {
                true
            }
            Expr::Array(parts)
                if self
                    .files
                    .get(path)
                    .is_some_and(|file| file.facts.precise_records) =>
            {
                let Value::Array(id) = value else {
                    return false;
                };
                !self.invalidated_builders.contains(id)
                    && self.argument_objects.get(id).is_some_and(|values| {
                        parts.len() == values.len()
                            && parts.iter().zip(values).all(|(part, value)| {
                                self.handled_deletion_effect(part, value, path, env)
                            })
                    })
            }
            Expr::Alternatives(_)
                if matches!(value, Value::Evaluated(_, true))
                    || !self.effect_can_mutate(expr, path, env) =>
            {
                true
            }
            Expr::Call { .. } | Expr::Sequence(_) | Expr::Discard(_) | Expr::SlotWrite { .. } => {
                matches!(value, Value::Evaluated(_, true))
            }
            Expr::Await(inner) => self.handled_deletion_effect(inner, value, path, env),
            Expr::Member(object, _) if self.static_namespace_read(object, path, env) => true,
            Expr::Delete(_, _) => matches!(value, Value::SlotDeletion),
            Expr::Children(parts)
            | Expr::Array(parts)
            | Expr::Container(parts)
            | Expr::Object(parts)
            | Expr::Record(parts, _)
            | Expr::Alternatives(parts) => {
                let (Value::Aggregate(values)
                | Value::References(values)
                | Value::Object(values)
                | Value::Record(values, _)) = value
                else {
                    return false;
                };
                parts.len() == values.len()
                    && parts
                        .iter()
                        .zip(values)
                        .all(|(part, value)| self.handled_deletion_effect(part, value, path, env))
            }
            Expr::Name(_) | Expr::Text(_) | Expr::Function(_) => true,
            // Configured executor calls and trusted SQL tags do not mutate a
            // local builder. Unknown calls and opaque expressions remain
            // conservative, even when another conditional arm deletes a slot.
            _ => !self.effect_can_mutate(expr, path, env),
        }
    }
}

#[cfg(test)]
#[path = "delete/tests.rs"]
mod tests;
