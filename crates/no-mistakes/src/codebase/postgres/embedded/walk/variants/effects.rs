mod bindings;
mod identities;
mod projection;

use super::super::{resolve::functions::reassigned::for_each_assigned_name, ScopeVisitor};
use oxc_ast::ast::{
    ArrowFunctionExpression, AssignmentExpression, CallExpression, ConditionalExpression,
    Expression, Function, LogicalExpression, SimpleAssignmentTarget, UpdateExpression,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

impl ScopeVisitor<'_> {
    pub(super) fn expression_mutates_builder(&self, expression: &Expression<'_>) -> bool {
        let mut effects = Effects {
            visitor: self,
            found: false,
            prefixes: None,
            identities: None,
            uncertain: false,
        };
        effects.visit_expression(expression);
        effects.found
    }

    pub(super) fn expression_builder_effects(&self, expression: &Expression<'_>) -> BuilderEffects {
        let mut effects = Effects {
            visitor: self,
            found: false,
            prefixes: Some(Vec::new()),
            identities: None,
            uncertain: false,
        };
        effects.identities = Some(identities::EffectIdentities::default());
        effects.visit_expression(expression);
        BuilderEffects {
            targets: effects
                .prefixes
                .unwrap()
                .into_iter()
                .map(|(name, identity, _)| (name, identity))
                .collect(),
            uncertain: effects.uncertain,
        }
    }

    pub(super) fn forget_effectful_sql_prefixes(&mut self, expression: &Expression<'_>) {
        let mut effects = Effects {
            visitor: self,
            found: false,
            prefixes: Some(Vec::new()),
            identities: None,
            uncertain: false,
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

pub(super) struct BuilderEffects {
    pub(super) targets: Vec<(String, Option<u32>)>,
    pub(super) uncertain: bool,
}

type PrefixEffect = (String, Option<u32>, Option<usize>);

struct Effects<'visitor, 'source> {
    visitor: &'visitor ScopeVisitor<'source>,
    found: bool,
    prefixes: Option<Vec<PrefixEffect>>,
    identities: Option<identities::EffectIdentities>,
    uncertain: bool,
}

impl<'a> Visit<'a> for Effects<'_, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let (effects, escape) = self.visitor.call_builder_effect(call);
        let (effects, appended) = self.project_call(call, effects, escape);
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
    fn visit_conditional_expression(&mut self, branch: &ConditionalExpression<'a>) {
        if self.identities.is_none() {
            walk::walk_conditional_expression(self, branch);
            return;
        }
        self.visit_expression(&branch.test);
        let before = self.identities.clone().unwrap();
        self.visit_expression(&branch.consequent);
        let left = self.identities.replace(before.clone()).unwrap();
        self.visit_expression(&branch.alternate);
        self.identities
            .as_mut()
            .unwrap()
            .join(self.visitor, &before, left);
    }
    fn visit_logical_expression(&mut self, branch: &LogicalExpression<'a>) {
        self.visit_expression(&branch.left);
        let before = self.identities.clone();
        self.visit_expression(&branch.right);
        if let Some(before) = before {
            self.identities
                .as_mut()
                .unwrap()
                .join(self.visitor, &before, before.clone());
        }
    }
    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        for_each_assigned_name(&assignment.left, &mut |name| self.collect_binding(name));
        if !self.found || self.prefixes.is_some() {
            walk::walk_assignment_expression(self, assignment);
        }
        if let Some(identities) = &mut self.identities {
            self.uncertain |= identities.assign(self.visitor, assignment);
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
