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
        let deleted = self.deleted_argument_slots.clone();
        let mut deleted_changed = deleted.clone();
        let mut returned = Vec::new();
        let mut changed = FxHashSet::default();
        for arm in arms {
            // Keep arm-created frames alive for callbacks returned from helpers.
            self.scopes[..scopes.len()].clone_from_slice(&scopes);
            self.modules.clone_from(&modules);
            self.invalidated_builders.clone_from(&original);
            self.deleted_argument_slots.clone_from(&deleted);
            returned.push(self.expr(arm, path, env, depth, generic));
            deleted_changed.extend(self.deleted_argument_slots.iter().copied());
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
        self.scopes[..scopes.len()].clone_from_slice(&scopes);
        self.modules = modules;
        self.invalidated_builders = original;
        self.deleted_argument_slots = deleted_changed;
        let values = changed
            .into_iter()
            .map(|id| Value::Prefix(String::new(), false, Some(id)))
            .collect::<Vec<_>>();
        self.invalidate_builders(&values);
        // Preserve possible callback captures for opaque consumers, while an
        // aggregate never proves the SQL prefix of a conditional return.
        Value::Aggregate(returned)
    }
}
