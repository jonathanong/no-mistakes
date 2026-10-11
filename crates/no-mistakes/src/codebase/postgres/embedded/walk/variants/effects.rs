use super::super::ScopeVisitor;
use super::ValueKind;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrowFunctionExpression, AssignmentExpression, AssignmentTarget, CallExpression, Expression,
    Function, SimpleAssignmentTarget, UpdateExpression,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

impl ScopeVisitor<'_> {
    pub(super) fn expression_mutates_builder(&self, expression: &Expression<'_>) -> bool {
        let mut effects = Effects {
            visitor: self,
            found: false,
            prefixes: None,
        };
        effects.visit_expression(expression);
        effects.found
    }

    pub(super) fn forget_effectful_sql_prefixes(&mut self, expression: &Expression<'_>) {
        let mut effects = Effects {
            visitor: self,
            found: false,
            prefixes: Some(Vec::new()),
        };
        effects.visit_expression(expression);
        let prefixes = effects.prefixes.unwrap();
        for (name, identity, aliases) in prefixes {
            if aliases {
                self.apply_builder_effect(name, identity, None, false, true);
            } else {
                self.mark_dynamic(&name);
            }
        }
    }
}

struct Effects<'visitor, 'source> {
    visitor: &'visitor ScopeVisitor<'source>,
    found: bool,
    prefixes: Option<Vec<(String, Option<u32>, bool)>>,
}

impl Effects<'_, '_> {
    fn known_sql_binding(&self, name: &str) -> bool {
        self.visitor.lookup(name).is_some_and(|binding| {
            binding.sql_builder
                || binding.builder_identity.is_some()
                || binding
                    .variants
                    .as_ref()
                    .is_some_and(|values| values.iter().any(|value| value.value == ValueKind::Sql))
        })
    }
    fn appended_builders(&self, call: &CallExpression<'_>) -> Vec<(String, Option<u32>)> {
        let receiver = match unwrap_ts_wrappers(&call.callee) {
            Expression::StaticMemberExpression(member) if member.property.name == "append" => {
                &member.object
            }
            Expression::ComputedMemberExpression(member) if matches!(unwrap_ts_wrappers(&member.expression), Expression::StringLiteral(property) if property.value == "append") => {
                &member.object
            }
            _ => return Vec::new(),
        };
        self.visitor
            .builder_result_names(receiver)
            .into_iter()
            .filter(|name| self.known_sql_binding(name))
            .map(|name| {
                let identity = self
                    .visitor
                    .lookup(&name)
                    .and_then(|binding| binding.builder_identity);
                (name, identity)
            })
            .collect()
    }
    fn collect_binding(&mut self, name: &str) {
        if self.known_sql_binding(name) {
            self.found = true;
            if let Some(prefixes) = &mut self.prefixes {
                prefixes.push((name.to_string(), None, false));
            }
        }
    }
}

impl<'a> Visit<'a> for Effects<'_, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let effects = self
            .visitor
            .parameter_helper_effect(call)
            .unwrap_or_default();
        let effects = effects
            .into_iter()
            .chain(self.appended_builders(call))
            .collect::<Vec<_>>();
        self.found |= !effects.is_empty();
        if let Some(prefixes) = &mut self.prefixes {
            prefixes.extend(
                effects
                    .into_iter()
                    .map(|(name, identity)| (name, identity, true)),
            );
        }
        if !self.found || self.prefixes.is_some() {
            walk::walk_call_expression(self, call);
        }
    }
    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if let AssignmentTarget::AssignmentTargetIdentifier(id) = &assignment.left {
            self.collect_binding(id.name.as_str());
        }
        if !self.found || self.prefixes.is_some() {
            walk::walk_assignment_expression(self, assignment);
        }
    }
    fn visit_update_expression(&mut self, update: &UpdateExpression<'a>) {
        if let SimpleAssignmentTarget::AssignmentTargetIdentifier(id) = &update.argument {
            self.collect_binding(id.name.as_str());
        }
        if !self.found || self.prefixes.is_some() {
            walk::walk_update_expression(self, update);
        }
    }
    // Creating a callback does not execute mutations inside its body.
    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}
}
