//! Whether a local binding's value flows from an unbounded identifier source.
use super::Bindings;
use crate::codebase::dependencies::extract::ValueExpression;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::fx::FxHashSet;
use std::path::Path;

pub(super) struct Seen {
    pub(super) bindings: FxHashSet<(usize, String, u32)>,
    pub(super) calls: FxHashSet<(u32, u32, String)>,
}

impl Bindings<'_> {
    pub(crate) fn flows_unbounded(&self, file: &Path, scope: usize, name: &str, at: u32) -> bool {
        let mut seen = Seen {
            bindings: FxHashSet::default(),
            calls: FxHashSet::default(),
        };
        let Some(binding) = self.value(file, scope, name, at) else {
            return self.parameter(file, name);
        };
        seen.bindings
            .insert((binding.scope, binding.name.clone(), binding.declared_at));
        self.expr_unbounded(file, &binding.value, &mut seen)
    }

    pub(crate) fn parameter(&self, file: &Path, name: &str) -> bool {
        self.facts.get_ts_facts(file).is_some_and(|ts| {
            ts.value_flow
                .parameters
                .iter()
                .any(|parameter| parameter.name == name)
        })
    }

    fn expr_unbounded(&self, file: &Path, value: &ValueExpression, seen: &mut Seen) -> bool {
        if self.env(file, value, &[]).is_some() {
            return true;
        }
        match value {
            ValueExpression::Named {
                name,
                scope,
                offset,
                ..
            } => self.named_unbounded(file, name, scope.unwrap_or(0), *offset, seen),
            ValueExpression::Member(inner, key) => self.member_unbounded(file, inner, key, seen),
            ValueExpression::Awaited(inner) => self.expr_unbounded(file, inner, seen),
            ValueExpression::Derived(parts) => parts
                .iter()
                .any(|part| self.expr_unbounded(file, part, seen)),
            ValueExpression::Either(left, right)
            | ValueExpression::Nullish(left, right)
            | ValueExpression::Default(left, right) => {
                self.expr_unbounded(file, left, seen) || self.expr_unbounded(file, right, seen)
            }
            _ => false,
        }
    }

    fn named_unbounded(
        &self,
        file: &Path,
        name: &str,
        scope: usize,
        at: u32,
        seen: &mut Seen,
    ) -> bool {
        let Some(binding) = self.value(file, scope, name, at) else {
            return self.parameter(file, name);
        };
        if !seen
            .bindings
            .insert((binding.scope, binding.name.clone(), binding.declared_at))
        {
            return false;
        }
        self.query_projection(file, scope, name, at)
            || self.expr_unbounded(file, &binding.value, seen)
    }

    fn member_unbounded(
        &self,
        file: &Path,
        inner: &ValueExpression,
        key: &str,
        seen: &mut Seen,
    ) -> bool {
        match inner {
            ValueExpression::Object(fields, _) => fields
                .get(key)
                .is_some_and(|field| self.expr_unbounded(file, field, seen)),
            ValueExpression::Awaited(next) => self.member_unbounded(file, next, key, seen),
            ValueExpression::Named {
                name,
                scope,
                offset,
                ..
            } => {
                let Some(binding) = self.value(file, scope.unwrap_or(0), name, *offset) else {
                    return false;
                };
                if !seen
                    .bindings
                    .insert((binding.scope, binding.name.clone(), binding.declared_at))
                {
                    return false;
                }
                self.member_unbounded(file, &binding.value, key, seen)
            }
            ValueExpression::Call(start, end) => {
                self.returned_field_unbounded(file, *start, *end, key, seen)
            }
            ValueExpression::Either(left, right)
            | ValueExpression::Nullish(left, right)
            | ValueExpression::Default(left, right) => {
                self.member_unbounded(file, left, key, seen)
                    || self.member_unbounded(file, right, key, seen)
            }
            _ => false,
        }
    }
}

mod returns;

#[cfg(test)]
mod tests;
