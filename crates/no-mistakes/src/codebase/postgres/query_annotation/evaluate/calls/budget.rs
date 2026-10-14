use super::{Environment, Evaluator, Expr, Value};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn guard_budget_callee(
        &mut self,
        target: &Value,
        callee: &Expr,
        path: &Path,
        env: Environment,
        depth: u8,
    ) {
        if self.files[path].facts.precise_records && depth_limited(target) {
            self.budget_callee_callbacks(callee, env, depth);
        }
    }

    fn budget_callee_callbacks(&mut self, mut callee: &Expr, env: Environment, depth: u8) {
        // An executed fetch that exhausted its budget cannot certify the
        // lexical prefix of callbacks retained by its supplied container.
        loop {
            match callee {
                Expr::Member(base, _) | Expr::Index(base, _) => callee = base,
                Expr::Name(name) => {
                    if let Some(value) = self.scopes[env].get(name).cloned() {
                        self.budget_callback_values(value, depth);
                    }
                    break;
                }
                _ => break,
            }
        }
    }
    fn budget_callback_values(&mut self, value: Value, depth: u8) {
        let callbacks = self.failed_scope_callbacks(value);
        self.opaque_callbacks(&[callbacks], depth);
    }
}

fn depth_limited(value: &Value) -> bool {
    match value {
        Value::Failure("evaluation-depth-limit") => true,
        Value::Aggregate(values) => values.iter().any(depth_limited),
        _ => false,
    }
}
