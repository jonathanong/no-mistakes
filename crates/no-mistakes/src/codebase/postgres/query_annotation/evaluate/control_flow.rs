use super::{concat, Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::{Expr, Step};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn binary(
        &mut self,
        op: &str,
        left: &Expr,
        right: &Expr,
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        let (depth, generic) = context;
        if !self.files[path].facts.precise_records {
            if op == "+" {
                return self.template(&[left.clone(), right.clone()], path, env, context);
            }
            let parts = vec![left.clone(), right.clone()];
            return self.container(&Expr::Children(parts.clone()), &parts, path, env, context);
        }
        let left = self.expr(left, path, env, depth, generic).exposed();
        let right = self.expr(right, path, env, depth, generic).exposed();
        match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => match op {
                "+" => a
                    .checked_add(*b)
                    .filter(|number| number.unsigned_abs() <= 9_007_199_254_740_991)
                    .map(Value::Number)
                    .unwrap_or(Value::Unknown),
                "===" | "==" => Value::Boolean(a == b),
                "!==" | "!=" => Value::Boolean(a != b),
                ">" => Value::Boolean(a > b),
                "<" => Value::Boolean(a < b),
                ">=" => Value::Boolean(a >= b),
                "<=" => Value::Boolean(a <= b),
                _ => Value::Unknown,
            },
            _ if op == "+" => concat(left, right),
            _ => Value::Unknown,
        }
    }

    // Continue each path independently so an early return never executes its suffix.
    pub(super) fn branch_steps(
        &mut self,
        test_and_arms: (&Expr, &[Step], &[Step]),
        suffix: &[Step],
        path: &Path,
        env: &Environment,
        context: (u8, bool),
        opaque: bool,
    ) -> Value {
        let (test, left, right) = test_and_arms;
        let (depth, generic) = context;
        let condition = self.expr(test, path, env, depth, generic).exposed();
        let arms = match condition {
            Value::Boolean(false) | Value::Number(0) => vec![right],
            Value::Boolean(true) | Value::Number(_) => vec![left],
            _ => vec![left, right],
        };
        let unsafe_condition = self.effect_can_mutate(test, path, *env);
        // Block-scoped bindings require scope exit semantics; do not flatten them
        // into the continuation and accidentally certify a shadowed SQL value.
        let binding = arms.iter().any(|arm| {
            arm.iter()
                .any(|step| matches!(step, Step::Bind(_, _) | Step::Hoisted(_, _) | Step::Var(_)))
        });
        let expressions = arms
            .into_iter()
            .map(|arm| {
                let mut steps = arm.to_vec();
                if binding || unsafe_condition || opaque {
                    steps.insert(0, Step::Unsupported);
                }
                steps.extend_from_slice(suffix);
                Expr::Flow(steps)
            })
            .collect::<Vec<_>>();
        let before = super::calls::async_suspension::event_lengths(&self.events);
        let result = self.alternatives(&expressions, path, env, depth, generic);
        let reason = if binding {
            Some("branch-local-binding")
        } else if unsafe_condition || opaque {
            Some("opaque-control-flow-effect")
        } else {
            None
        };
        if let Some(reason) = reason {
            for (key, events) in &mut self.events {
                for (_, value) in events
                    .iter_mut()
                    .skip(before.get(key).copied().unwrap_or(0))
                {
                    *value = Value::Failure(reason);
                }
            }
            Value::Failure(reason)
        } else {
            result
        }
    }
}
