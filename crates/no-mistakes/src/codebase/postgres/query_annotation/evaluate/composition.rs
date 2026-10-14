use super::{concat, Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn template(
        &mut self,
        parts: &[Expr],
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        let (depth, generic) = context;
        let mut prefix = Value::Prefix(String::new(), true, None);
        let mut handled = true;
        for part in parts {
            let value = self.expr(part, path, env, depth, generic);
            handled &= self.handled_deletion_effect(part, &value, path, *env);
            if self.files[path].facts.precise_records && !super::collections::string_element(&value)
            {
                // Unknown template coercion may execute a getter/toString hook.
                self.invalidate_live_builders();
                handled = false;
            }
            prefix = concat(prefix, value);
        }
        if self.files[path].facts.precise_records {
            Value::Evaluated(Box::new(prefix), handled)
        } else {
            prefix
        }
    }

    pub(super) fn append(
        &mut self,
        base: &Expr,
        tail: &Expr,
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        let (depth, generic) = context;
        let base_value = self.expr(base, path, env, depth, generic);
        let tail_value = self.expr(tail, path, env, depth, generic);
        if self.files[path].facts.precise_records
            && (!self.handled_deletion_effect(base, &base_value, path, *env)
                || !self.handled_deletion_effect(tail, &tail_value, path, *env))
        {
            self.invalidate_live_builders();
            return Value::Failure("opaque-append-operand-effect");
        }
        let base = base_value.exposed();
        let tail = tail_value;
        let base = if matches!(&base, Value::Prefix(_, _, Some(id)) if self.invalidated_builders.contains(id))
        {
            Value::Unknown
        } else {
            base
        };
        if self.files[path].facts.precise_records
            && !matches!(&base, Value::Prefix(_, _, Some(id)) if !self.invalidated_builders.contains(id))
        {
            self.invalidate_live_builders();
            self.opaque_callbacks(std::slice::from_ref(&base), depth);
            return Value::Failure("opaque-append-receiver");
        }
        let value = concat(base, tail);
        self.replace_builder(&value);
        value
    }

    pub(super) fn container(
        &mut self,
        expr: &Expr,
        children: &[Expr],
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        let (depth, generic) = context;
        let values: super::value::Values = children
            .iter()
            .map(|child| self.expr(child, path, env, depth, generic))
            .collect::<Vec<_>>()
            .into();
        match expr {
            Expr::Array(_) if self.files[path].facts.precise_records => {
                if children.iter().any(|child| matches!(child, Expr::Unknown)) {
                    Value::Failure("unknown-array-element")
                } else if children
                    .iter()
                    .any(|child| matches!(child, Expr::Spread(_)))
                {
                    Value::Failure("opaque-spread")
                } else {
                    let id = self.next_builder;
                    self.next_builder += 1;
                    self.argument_objects
                        .insert(id, values.into_iter().collect());
                    Value::Array(id)
                }
            }
            Expr::Array(_) | Expr::Container(_) => Value::Aggregate(values),
            Expr::Object(_) => Value::Object(values),
            Expr::Record(_, keys) => Value::Record(values, keys.clone()),
            _ => Value::References(values),
        }
    }
}
