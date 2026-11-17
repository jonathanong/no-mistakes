use super::{concat, Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::{Path, PathBuf};

mod finish;

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn tag_trusted(&self, tag: &str, path: &Path, env: Environment) -> bool {
        let file = &self.files[path];
        let name = if tag == "String.raw" { "String" } else { tag };
        let trusted = if tag == "String.raw" {
            !file.facts.raw_tag_reassigned && !file.module.imports.contains_key(name)
        } else {
            file.facts.trusted_tags.contains(tag)
        };
        if !trusted {
            return false;
        }
        // A configured tag's own module binding is that tag. CommonJS records
        // it as a global `const local = require(...)`. Nested scopes that
        // rebind the name, including parameters, are not the binding.
        if tag != "String.raw" {
            return !self.scopes[env].contains_key(name);
        }
        !self.scopes[env].contains_key(name) && !file.facts.globals.contains_key(name)
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
        let trusted_tag = self.tag_trusted(tag, path, *env);
        let values = effects
            .iter()
            .enumerate()
            .map(|(index, effect)| {
                if index == 0 && trusted_tag {
                    self.expr(effect, path, env, depth, generic)
                } else {
                    self.operand(effect, path, env, context)
                }
            })
            .collect::<Vec<_>>();
        let parameterized = trusted_tag && self.files[path].facts.parameterized_tags.contains(tag);
        let trusted_raw = tag == "String.raw" && trusted_tag;
        let interpolation_mutates = effects
            .iter()
            .skip(usize::from(trusted_raw))
            .chain(if trusted_raw { parts } else { &[] })
            .any(|effect| self.effect_can_mutate(effect, path, *env));
        if self.files[path].facts.precise_records && interpolation_mutates {
            // An interpolation getter or call can mutate a captured builder
            // before this tagged value is appended. Its generated bind marker
            // cannot certify the builder's prior SQL prefix.
            let builders = self
                .scopes
                .iter()
                .flat_map(|scope| scope.values().cloned())
                .collect::<Vec<_>>();
            self.invalidate_opaque_builders(&builders);
            return Value::Failure("opaque-tag-interpolation-effect");
        }
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
            let relation = matches!(&prefix, Value::Prefix(text, _, _)
                if crate::codebase::postgres::embedded::helpers::sentinel::in_relation_slot(text, text.len()));
            let value = if fragment {
                Value::Unknown
            } else if parameterized && index % 2 == 1 {
                match part {
                    Expr::Text(text) => {
                        let placeholder = text
                            .starts_with(crate::codebase::postgres::embedded::INTERNAL_PLACEHOLDER);
                        let invalid_position = placeholder
                            && matches!(&prefix, Value::Prefix(text, _, _) if text.ends_with('$'));
                        let text = if placeholder && !relation && !invalid_position {
                            format!("{}1", crate::codebase::postgres::embedded::INTERNAL_BIND)
                        } else {
                            text.clone()
                        };
                        Value::Prefix(text, !invalid_position, None)
                    }
                    _ => self.expr(part, path, env, depth, generic),
                }
            } else {
                self.expr(part, path, env, depth, generic)
            };
            if trusted_raw
                && self.files[path].facts.precise_records
                && !super::collections::string_element(&value)
                && !matches!(value, Value::Primitive)
            {
                self.invalidate_opaque_builders(&values);
                return Value::Failure("opaque-tag-interpolation-effect");
            }
            prefix = concat(prefix, value);
        }
        self.finish_tagged(tag, path, *env, &values, depth, prefix)
    }

    fn tag_helper(&self, expression: &Expr, path: &Path, env: Environment) -> bool {
        if let Expr::OpaqueWrite { children, .. } = expression {
            return children
                .last()
                .is_some_and(|value| self.tag_helper(value, path, env));
        }
        if let Expr::Tagged(tag, _, _) = expression {
            return tag != "String.raw" && self.tag_trusted(tag, path, env);
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
