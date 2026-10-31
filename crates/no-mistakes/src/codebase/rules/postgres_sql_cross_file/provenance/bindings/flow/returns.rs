use super::super::super::helpers::value_call_at;
use super::super::Bindings;
use super::Seen;
use crate::codebase::dependencies::extract::{CallableId, ValueExpression};
use crate::codebase::dependencies::graph::TsFactLookup;
use std::path::Path;

impl Bindings<'_> {
    pub(super) fn returned_field_unbounded(
        &self,
        file: &Path,
        start: u32,
        end: u32,
        key: &str,
        seen: &mut Seen,
    ) -> bool {
        if !seen.calls.insert((start, end, key.to_string())) {
            return false;
        }
        let Some(value_call) = value_call_at(self.facts, file, start, end) else {
            return false;
        };
        let ids = self.callable_ids(file, &value_call.callee);
        let Some(ts) = self.facts.get_ts_facts(file) else {
            return false;
        };
        ts.value_flow.returns.iter().any(|(id, value)| {
            ids.contains(id) && self.object_field_unbounded(file, value, key, seen)
        })
    }

    fn object_field_unbounded(
        &self,
        file: &Path,
        value: &ValueExpression,
        key: &str,
        seen: &mut Seen,
    ) -> bool {
        match value {
            ValueExpression::Object(fields, _) => fields
                .get(key)
                .is_some_and(|field| self.expr_unbounded(file, field, seen)),
            ValueExpression::Awaited(inner) => self.object_field_unbounded(file, inner, key, seen),
            ValueExpression::Named {
                name,
                scope,
                offset,
                ..
            } => self.named_field_unbounded(file, name, scope.unwrap_or(0), *offset, key, seen),
            ValueExpression::Either(left, right)
            | ValueExpression::Nullish(left, right)
            | ValueExpression::Default(left, right) => {
                self.object_field_unbounded(file, left, key, seen)
                    || self.object_field_unbounded(file, right, key, seen)
            }
            ValueExpression::Derived(parts) => parts
                .iter()
                .any(|part| self.object_field_unbounded(file, part, key, seen)),
            _ => false,
        }
    }

    fn named_field_unbounded(
        &self,
        file: &Path,
        name: &str,
        scope: usize,
        at: u32,
        key: &str,
        seen: &mut Seen,
    ) -> bool {
        let Some(binding) = self.value(file, scope, name, at) else {
            return false;
        };
        if !seen
            .bindings
            .insert((binding.scope, binding.name.clone(), binding.declared_at))
        {
            return false;
        }
        self.object_field_unbounded(file, &binding.value, key, seen)
    }

    fn callable_ids(&self, file: &Path, callee: &ValueExpression) -> Vec<CallableId> {
        let mut ids = Vec::new();
        match callee {
            ValueExpression::Callable(id) | ValueExpression::Super(id) => ids.push(*id),
            ValueExpression::Awaited(inner) => return self.callable_ids(file, inner),
            ValueExpression::Named {
                name,
                scope,
                offset,
                ..
            } => {
                if let Some(ValueExpression::Callable(id) | ValueExpression::Super(id)) = self
                    .value(file, scope.unwrap_or(0), name, *offset)
                    .map(|binding| &binding.value)
                {
                    ids.push(*id);
                }
                if let Some(ts) = self.facts.get_ts_facts(file) {
                    ids.extend(
                        ts.callable_bindings
                            .iter()
                            .filter(|(_, binding_name, _)| binding_name == name)
                            .map(|(_, _, id)| *id),
                    );
                }
            }
            _ => {}
        }
        ids.sort();
        ids.dedup();
        ids
    }
}
