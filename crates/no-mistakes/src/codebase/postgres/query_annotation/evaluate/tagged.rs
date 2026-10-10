use super::{concat, Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn tag_trusted(&self, tag: &str, path: &Path, env: Environment) -> bool {
        let file = &self.files[path];
        let name = if tag == "String.raw" { "String" } else { tag };
        let trusted = if tag == "String.raw" {
            !file.facts.raw_tag_reassigned && !file.imports.contains_key(name)
        } else {
            file.facts.trusted_tags.contains(tag)
        };
        trusted && !self.scopes[env].contains_key(name) && !file.facts.globals.contains_key(name)
    }
    pub(super) fn tagged(
        &mut self,
        tag: &str,
        parts: &[Expr],
        effects: &[Expr],
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        let (depth, generic) = context;
        let values = effects
            .iter()
            .map(|effect| self.expr(effect, path, env, depth, generic))
            .collect::<Vec<_>>();
        let mut prefix = Value::Prefix(String::new(), true, None);
        for (index, part) in parts.iter().enumerate() {
            // A trusted tag's odd parts are generated binds. A nested builder
            // instead splices SQL, whose leading text must stay unknown here.
            let fragment = tag != "String.raw"
                && index % 2 == 1
                && (values.get(1 + index / 2).is_some_and(has_builder)
                    || effects
                        .get(1 + index / 2)
                        .is_some_and(|effect| self.tag_helper(effect, path, *env)));
            let value = if fragment {
                Value::Unknown
            } else {
                self.expr(part, path, env, depth, generic)
            };
            prefix = concat(prefix, value);
        }
        let trusted = self.tag_trusted(tag, path, *env);
        let name = if tag == "String.raw" { "String" } else { tag };
        let legacy = self.files[path].facts.legacy_tag_spans.get(name).is_some_and(|start| {
            let local = self.scopes[*env].get(name);
            local.is_none() || matches!(local, Some(Value::Function(function, owner, _)) if owner == path && function.start == *start)
        });
        if !trusted && !legacy {
            self.invalidate_builders(&values);
            self.opaque_callbacks(&values, depth);
        }
        if trusted || legacy {
            if let Value::Prefix(_, _, id) = &mut prefix {
                if tag != "String.raw" || legacy {
                    *id = Some(self.next_builder);
                    self.next_builder += 1;
                }
            }
            // Reuse the existing local-tag contract only for this annotation
            // prefix; no complete-SQL fact is promoted for structural rules.
            if legacy
                && matches!(values.first(), Some(Value::Function(function, _, _)) if function.asynchronous)
            {
                Value::Promise(Box::new(prefix))
            } else {
                prefix
            }
        } else {
            Value::Unknown
        }
    }

    fn tag_helper(&self, expression: &Expr, path: &Path, env: Environment) -> bool {
        if let Expr::OpaqueWrite { children, .. } = expression {
            return children
                .last()
                .is_some_and(|value| self.tag_helper(value, path, env));
        }
        if let Expr::Tagged(tag, _, _) = expression {
            return self.tag_trusted(tag, path, env);
        }
        let Expr::Call { callee, .. } = expression else {
            return false;
        };
        let tag = match callee.as_ref() {
            Expr::Name(name) => Some(name.as_str()),
            Expr::Member(base, _) => match base.as_ref() {
                Expr::Name(name) => Some(name.as_str()),
                _ => None,
            },
            _ => None,
        };
        tag.is_some_and(|tag| self.tag_trusted(tag, path, env))
    }
}

fn has_builder(value: &Value) -> bool {
    match value {
        Value::Prefix(_, _, Some(_)) => true,
        Value::Evaluated(inner, _) => has_builder(inner),
        Value::Aggregate(values) | Value::Joined(values) | Value::Possible(values) => {
            values.iter().any(has_builder)
        }
        _ => false,
    }
}
