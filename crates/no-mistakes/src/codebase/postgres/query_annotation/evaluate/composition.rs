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
        for part in parts {
            prefix = concat(prefix, self.expr(part, path, env, depth, generic));
        }
        prefix
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
        let base = self.expr(base, path, env, depth, generic).exposed();
        let tail = self.expr(tail, path, env, depth, generic);
        let base = if matches!(&base, Value::Prefix(_, _, Some(id)) if self.invalidated_builders.contains(id))
        {
            Value::Unknown
        } else {
            base
        };
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
        let values = children
            .iter()
            .map(|child| self.expr(child, path, env, depth, generic))
            .collect::<Vec<_>>()
            .into();
        match expr {
            Expr::Container(_) => Value::Aggregate(values),
            Expr::Object(_) => Value::Object(values),
            _ => Value::References(values),
        }
    }
}
