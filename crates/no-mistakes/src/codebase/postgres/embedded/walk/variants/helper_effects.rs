use super::super::{scope, ScopeVisitor};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{CallExpression, Expression};

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn call_builder_effect(
        &self,
        call: &CallExpression<'_>,
    ) -> (Vec<(String, Option<u32>)>, Option<bool>) {
        let helper = match unwrap_ts_wrappers(&call.callee) {
            Expression::Identifier(name) if !self.shadowed_locally(name.name.as_str()) => {
                self.functions.parameter_builder(name.name.as_str())
            }
            _ => None,
        };
        if let Some(helper) = helper {
            let effects = if helper.fragments.is_empty() {
                Vec::new()
            } else {
                self.parameter_helper_effect(call, helper.parameter_index)
                    .unwrap_or_default()
            };
            return (effects, None);
        }
        (
            self.escaped_builder_effect(call),
            Some(
                self.variant_guard_owner == Some(self.function_scopes.last().copied().unwrap_or(0)),
            ),
        )
    }

    pub(in crate::codebase::postgres::embedded::walk) fn parameter_helper_effect(
        &self,
        call: &CallExpression<'_>,
        parameter_index: usize,
    ) -> Option<Vec<(String, Option<u32>)>> {
        // Even an unsupported arity can mutate a builder argument before
        // returning. Capture its identity before other arguments execute.
        let argument = call.arguments.get(parameter_index)?.as_expression()?;
        Some(
            self.builder_result_names(argument)
                .into_iter()
                .filter_map(|name| {
                    let binding = self.lookup(&name)?;
                    Some((name, binding.builder_identity))
                })
                .collect(),
        )
    }

    pub(in crate::codebase::postgres::embedded::walk) fn apply_call_builder_effect(
        &mut self,
        effect: (Vec<(String, Option<u32>)>, Option<bool>),
    ) {
        let minimum_scope = effect
            .1
            .map_or(0, |_| self.function_scopes.last().copied().unwrap_or(0));
        for (name, identity) in effect.0 {
            self.apply_builder_effect(
                name,
                identity,
                None,
                effect.1 == Some(false),
                effect.1 == Some(true),
                minimum_scope,
            );
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn apply_builder_effect(
        &mut self,
        name: String,
        identity: Option<u32>,
        preserved_scope: Option<usize>,
        include_shadowed_aliases: bool,
        forget_prefix: bool,
        minimum_scope: usize,
    ) {
        // Branch reassignments can lose exact identity. Historical alias
        // edges then fail closed for unknown members of this connected group;
        // a known replacement object keeps its independent facts.
        let mut pending = vec![name.clone()];
        let mut seen = FxHashSet::default();
        while let Some(name) = pending.pop() {
            if seen.insert(name.clone()) {
                if let Some(aliases) = self.builder_aliases.get(&name) {
                    pending.extend(aliases.iter().cloned());
                }
            }
        }
        let visible: FxHashMap<_, _> = seen
            .iter()
            .filter_map(|name| {
                self.scopes
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, scope)| scope.contains_key(name))
                    .map(|(index, _)| (name.clone(), index))
            })
            .collect();
        // A function declaration is inspected without executing it. Give an
        // escaped captured builder a local snapshot so later body queries fail
        // closed without changing the enclosing execution's facts.
        let mut captured = Vec::new();
        for (index, scope) in self.scopes.iter().enumerate().take(minimum_scope) {
            for (bound_name, binding) in scope {
                let affected = identity
                    .is_some_and(|identity| binding.builder_identity == Some(identity))
                    || seen.contains(bound_name)
                        && (identity.is_none() || binding.builder_identity.is_none())
                        && visible.get(bound_name) == Some(&index);
                if affected
                    && self.scopes[index + 1..]
                        .iter()
                        .all(|scope| !scope.contains_key(bound_name))
                {
                    captured.push((bound_name.clone(), binding.clone()));
                }
            }
        }
        self.scopes[minimum_scope].extend(captured);
        for (index, scope) in self.scopes.iter_mut().enumerate().skip(minimum_scope) {
            for (bound_name, binding) in scope {
                let affected = identity
                    .is_some_and(|identity| binding.builder_identity == Some(identity))
                    || seen.contains(bound_name)
                        && (identity.is_none() || binding.builder_identity.is_none())
                        && (include_shadowed_aliases || visible.get(bound_name) == Some(&index));
                let preserved = preserved_scope == Some(index) && *bound_name == name;
                if affected && !preserved {
                    // Even an already-opaque snapshot can be mutated again
                    // before throwing; exception boundaries track every effect.
                    binding.builder_effect_revision =
                        binding.builder_effect_revision.wrapping_add(1);
                    // Legacy SQL and finite alternatives describe the same
                    // mutable object; neither may remain complete after escape.
                    scope::mark_binding_dynamic_keep_known_statement(binding);
                    if forget_prefix {
                        binding.sql = None;
                    }
                }
            }
        }
    }
}
