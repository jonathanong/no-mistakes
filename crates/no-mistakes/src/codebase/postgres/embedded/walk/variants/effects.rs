use super::super::ScopeVisitor;
use super::ValueKind;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrowFunctionExpression, AssignmentExpression, AssignmentOperator, AssignmentTarget,
    CallExpression, Expression, Function, SimpleAssignmentTarget, UpdateExpression,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

impl ScopeVisitor<'_> {
    pub(super) fn expression_mutates_builder(&self, expression: &Expression<'_>) -> bool {
        self.builder_expression_effects(expression, false)
    }

    pub(super) fn expression_mutates_captured_builder(&self, expression: &Expression<'_>) -> bool {
        self.builder_expression_effects(expression, true)
    }

    fn builder_expression_effects(&self, expression: &Expression<'_>, captured: bool) -> bool {
        let mut effects = Effects {
            visitor: self,
            found: false,
            prefixes: None,
            captured,
        };
        effects.visit_expression(expression);
        effects.found
    }

    pub(super) fn forget_effectful_sql_prefixes(&mut self, expression: &Expression<'_>) {
        let mut effects = Effects {
            visitor: self,
            found: false,
            prefixes: Some(Vec::new()),
            captured: false,
        };
        effects.visit_expression(expression);
        let prefixes = effects.prefixes.unwrap();
        for (name, identity, scope) in prefixes {
            if let Some(scope) = scope {
                self.apply_builder_effect(name, identity, None, false, true, scope);
            } else {
                self.mark_dynamic(&name);
            }
        }
    }
}

type PrefixEffect = (String, Option<u32>, Option<usize>);

struct Effects<'visitor, 'source> {
    visitor: &'visitor ScopeVisitor<'source>,
    found: bool,
    prefixes: Option<Vec<PrefixEffect>>,
    captured: bool,
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
                prefixes.push((name.to_string(), None, None));
            }
        }
    }
}

impl<'a> Visit<'a> for Effects<'_, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let (effects, escape) = self.visitor.call_builder_effect(call);
        let appended = self.appended_builders(call);
        self.found |= !effects.is_empty() || !appended.is_empty();
        if let Some(prefixes) = &mut self.prefixes {
            let scope = escape.map_or(0, |_| {
                self.visitor.function_scopes.last().copied().unwrap_or(0)
            });
            prefixes.extend(
                effects
                    .into_iter()
                    .map(|(name, identity)| (name, identity, Some(scope))),
            );
            prefixes.extend(
                appended
                    .into_iter()
                    .map(|(name, identity)| (name, identity, Some(0))),
            );
        }
        if !self.found || self.prefixes.is_some() {
            walk::walk_call_expression(self, call);
        }
    }
    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if self.captured && identifier_rebinding(assignment) {
            // Earlier interpolations already hold the old object reference.
            // Rebinding its name is harmless, but the RHS may still mutate it.
            self.visit_expression(&assignment.right);
            return;
        }
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

fn identifier_rebinding(assignment: &AssignmentExpression<'_>) -> bool {
    if assignment.operator != AssignmentOperator::Assign {
        return false;
    }
    let expression = match &assignment.left {
        AssignmentTarget::AssignmentTargetIdentifier(_) => return true,
        AssignmentTarget::TSAsExpression(value) => &value.expression,
        AssignmentTarget::TSSatisfiesExpression(value) => &value.expression,
        AssignmentTarget::TSNonNullExpression(value) => &value.expression,
        AssignmentTarget::TSTypeAssertion(value) => &value.expression,
        _ => return false,
    };
    matches!(unwrap_ts_wrappers(expression), Expression::Identifier(_))
}
