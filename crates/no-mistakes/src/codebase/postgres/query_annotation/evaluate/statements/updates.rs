use super::super::{concat, Environment, Evaluator, Value};
use super::effects;
use crate::codebase::postgres::query_annotation::Expr;
use std::path::{Path, PathBuf};
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn bind_step(
        &mut self,
        name: &str,
        expr: &Expr,
        path: &Path,
        env: &Environment,
        context: (u8, bool),
        state: (bool, &Value),
    ) {
        let (depth, generic) = context;
        let (supported, unsupported) = state;
        let may_mutate = self.files[path].facts.precise_records
            && (effects::contains_accessor_read(expr) || matches!(expr, Expr::Call { .. }))
            && self.effect_can_mutate(expr, path, *env)
            && !self.proven_append_only_effect(expr, path, *env, depth, generic);
        let live_before = may_mutate.then(|| self.live_builder_values());
        let value = if supported {
            self.expr(expr, path, env, depth, generic)
        } else {
            unsupported.clone()
        };
        let handled = self.handled_deletion_effect(expr, &value, path, *env);
        let value = value.exposed();
        self.write_captured_binding_evaluated(*env, name, &value, depth);
        if let Some(live_before) = live_before {
            // Known function bodies have already projected their
            // builder effects. An opaque result or accessor read
            // still cannot prove that earlier builders survived.
            if !handled
                && (effects::contains_accessor_read(expr)
                    || matches!(
                        value,
                        Value::Unknown | Value::Unsupported | Value::Failure(_)
                    ))
            {
                self.invalidate_opaque_builders(&live_before);
            }
        }
    }
    pub(super) fn append_step(
        &mut self,
        name: &str,
        expr: &Expr,
        path: &Path,
        env: &Environment,
        context: (u8, bool),
        state: (bool, &Value),
    ) -> bool {
        let (depth, generic) = context;
        let mut opaque_return = false;
        let (supported, unsupported) = state;
        let result = self.expr(expr, path, env, depth, generic);
        if self.files[path].facts.precise_records
            && !self.handled_deletion_effect(expr, &result, path, *env)
        {
            opaque_return = true;
            self.invalidate_live_builders();
        }
        let tail = result.exposed();
        let base = self.scopes[*env]
            .get(name)
            .cloned()
            .unwrap_or(Value::Unknown);
        if self.files[path].facts.precise_records
            && !matches!(&base, Value::Prefix(_, _, Some(id)) if !self.invalidated_builders.contains(id))
            && !matches!(&base, Value::Joined(values) if values.iter().all(|value| matches!(value, Value::Prefix(_, _, Some(id)) if !self.invalidated_builders.contains(id))))
        {
            opaque_return = true;
            self.invalidate_live_builders();
            self.opaque_callbacks(std::slice::from_ref(&base), depth);
        }
        let value = if supported {
            concat(base, tail)
        } else {
            unsupported.clone()
        };
        self.replace_builder(&value);
        self.write_captured_binding(*env, name, &value);

        opaque_return
    }
    pub(super) fn return_step(
        &mut self,
        expr: &Expr,
        path: &Path,
        env: &Environment,
        context: (u8, bool),
        opaque_return: bool,
    ) -> Value {
        let (depth, generic) = context;
        let result = self.expr(expr, path, env, depth, generic);
        if self.files[path].facts.precise_records {
            if matches!(result.clone().exposed(), Value::Failure(_)) {
                return result.exposed();
            }
            let handled = self.handled_deletion_effect(expr, &result, path, *env);
            if matches!(&result, Value::Evaluated(_, false))
                || opaque_return
                || (self.effect_can_mutate(expr, path, *env) && !handled)
            {
                return Value::Failure("opaque-return-effect");
            }
            return Value::Evaluated(Box::new(result.exposed()), true);
        }
        if opaque_return {
            Value::Unknown
        } else {
            result.exposed()
        }
    }
}
