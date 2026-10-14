use super::super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::{Path, PathBuf};
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn call_target(
        &mut self,
        callee: &Expr,
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> (Value, Option<Value>, bool) {
        let (depth, generic) = context;
        match callee {
            Expr::Index(object, _) | Expr::Member(object, _) => {
                let value = self.expr(object, path, env, depth, generic);
                let handled = self.handled_deletion_effect(object, &value, path, *env);
                let receiver = value.exposed();
                let target = match callee {
                    Expr::Index(_, index) => self.index(receiver.clone(), *index),
                    Expr::Member(_, name) => self.member(receiver.clone(), name, path),
                    _ => unreachable!(),
                };
                (target.exposed(), Some(receiver), handled)
            }
            other => {
                let value = self.expr(other, path, env, depth, generic);
                let handled = self.handled_deletion_effect(other, &value, path, *env);
                (value.exposed(), None, handled)
            }
        }
    }
}
