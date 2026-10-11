use super::super::{scope, ScopeVisitor};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::{CallExpression, Expression};

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn parameter_helper_effect(
        &self,
        call: &CallExpression<'_>,
    ) -> Option<Vec<(String, Option<u32>)>> {
        let Expression::Identifier(name) = unwrap_ts_wrappers(&call.callee) else {
            return None;
        };
        if self.shadowed_locally(name.name.as_str()) {
            return None;
        }
        let helper = self.functions.parameter_builder(name.name.as_str())?;
        if helper.fragments.is_empty() {
            return None;
        }
        // Even an unsupported arity can mutate a builder argument before
        // returning. Capture its identity before other arguments execute.
        let argument = call
            .arguments
            .get(helper.parameter_index)?
            .as_expression()?;
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

    pub(in crate::codebase::postgres::embedded::walk) fn apply_parameter_helper_effect(
        &mut self,
        effect: Option<Vec<(String, Option<u32>)>>,
    ) {
        for (name, identity) in effect.into_iter().flatten() {
            self.apply_one_helper_effect(name, identity);
        }
    }

    fn apply_one_helper_effect(&mut self, name: String, identity: Option<u32>) {
        if let Some(identity) = identity {
            for binding in self.scopes.iter_mut().flat_map(|scope| scope.values_mut()) {
                if binding.builder_identity == Some(identity) {
                    scope::mark_binding_dynamic_keep_known_statement(binding);
                }
            }
        }
        // Branch reassignments can lose exact identity. Historical alias
        // edges then fail closed for unknown members of this connected group;
        // a known replacement object keeps its independent facts.
        let mut pending = vec![name];
        let mut seen = FxHashSet::default();
        while let Some(name) = pending.pop() {
            if seen.insert(name.clone()) {
                if let Some(aliases) = self.builder_aliases.get(&name) {
                    pending.extend(aliases.iter().cloned());
                }
                let affected = self.lookup(&name).is_some_and(|binding| {
                    identity.is_none() || binding.builder_identity.is_none()
                });
                if affected {
                    self.mark_dynamic_keep_known_statement(&name);
                }
            }
        }
    }
}
