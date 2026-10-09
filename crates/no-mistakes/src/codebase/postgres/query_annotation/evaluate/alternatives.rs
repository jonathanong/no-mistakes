//! Alternative arms share input facts, never each other's mutable effects.
use super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use crate::fx::FxHashSet;
use std::path::Path;

impl<F: Fn(&str, &Path) -> Option<std::path::PathBuf>> Evaluator<'_, F> {
    pub(super) fn alternatives(
        &mut self,
        arms: &[Expr],
        path: &Path,
        env: &Environment,
        depth: u8,
        generic: bool,
    ) -> Value {
        let scopes = self.scopes.clone();
        let modules = self.modules.clone();
        let original = self.invalidated_builders.clone();
        let mut changed = FxHashSet::default();
        for arm in arms {
            self.scopes.clone_from(&scopes);
            self.modules.clone_from(&modules);
            self.invalidated_builders.clone_from(&original);
            self.expr(arm, path, env, depth, generic);
            changed.extend(self.invalidated_builders.iter().copied());
            for (before, after) in scopes.iter().zip(&self.scopes) {
                for (name, value) in before {
                    if let Value::Prefix(text, complete, Some(id)) = value {
                        if !matches!(after.get(name), Some(Value::Prefix(other, done, Some(other_id))) if id == other_id && ((text == other && complete == done) || (text.trim_start().starts_with("/*") && other.trim_start().starts_with("/*"))))
                        {
                            changed.insert(*id);
                        }
                    }
                }
            }
        }
        self.scopes = scopes;
        self.modules = modules;
        self.invalidated_builders = original;
        let values = changed
            .into_iter()
            .map(|id| Value::Prefix(String::new(), false, Some(id)))
            .collect::<Vec<_>>();
        self.invalidate_builders(&values);
        Value::Unknown
    }
}
