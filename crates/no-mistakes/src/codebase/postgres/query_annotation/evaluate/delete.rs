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
        let (depth, generic) = context;
        let values = children
            .iter()
            .map(|child| self.expr(child, path, env, depth, generic))
            .collect::<Vec<_>>();
        if let Some(Value::Arguments(id)) = values.first() {
            // Removing a slot does not mutate the builder it referenced.
            match key {
                DeleteKey::Index(index) => {
                    self.disconnect_mapped_slot(*id, index);
                    self.disconnected_argument_slots.insert((*id, index));
                    self.deleted_argument_slots.insert((*id, Some(index)));
                    self.definite_deleted_argument_slots.insert((*id, index));
                    if let Some(slot) = self
                        .argument_objects
                        .get_mut(id)
                        .and_then(|values| values.get_mut(index))
                    {
                        *slot = Value::Unknown;
                    }
                    if let Some(slots) = self.argument_extra_slots.get_mut(id) {
                        slots.remove(&index);
                    }
                }
                DeleteKey::Named => {}
                DeleteKey::Dynamic => {
                    self.deleted_argument_slots.insert((*id, None));
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
