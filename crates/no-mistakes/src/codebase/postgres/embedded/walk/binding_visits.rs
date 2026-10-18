use super::{assignments, resolve, ScopeVisitor};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{self, AssignmentTarget};
use oxc_ast_visit::{walk, Visit};

impl<'a> ScopeVisitor<'a> {
    pub(super) fn walk_assignment_target_reference(
        &mut self,
        target: &ast::SimpleAssignmentTarget<'a>,
    ) {
        // A name write replaces its reference. A member write can mutate a
        // builder receiver, so its root reference must still escape.
        if matches!(target, ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(_)) {
            return;
        }
        walk::walk_simple_assignment_target(self, target);
    }

    pub(super) fn walk_variable_declarator_bindings(
        &mut self,
        declaration: &oxc_ast::ast::VariableDeclarator<'a>,
    ) {
        let executor = declaration
            .init
            .as_ref()
            .is_some_and(|init| self.proven_executor(init));
        self.refresh_builder_identity(declaration);
        let variants = declaration
            .init
            .as_ref()
            .and_then(|init| self.recover_variants(init));
        let exemptions = self.builder_reference_exemptions.len();
        if matches!(declaration.id, ast::BindingPattern::BindingIdentifier(_)) {
            if let Some(init) = &declaration.init {
                self.allow_builder_value(init);
            }
        }
        let outer = self.enter_iife(declaration);
        walk::walk_variable_declarator(self, declaration);
        self.builder_reference_exemptions.truncate(exemptions);
        self.initialize_variants(declaration, variants);
        self.initialize_executor_alias(declaration, executor);
        self.leave_iife(&declaration.id, outer);
    }

    pub(super) fn walk_assignment_expression_bindings(
        &mut self,
        assign: &oxc_ast::ast::AssignmentExpression<'a>,
    ) {
        let identity = self.assignment_builder_identity(assign);
        let variants = self.assigned_variants(assign);
        if matches!(
            assign.left,
            AssignmentTarget::ArrayAssignmentTarget(_)
                | AssignmentTarget::ObjectAssignmentTarget(_)
        ) {
            self.visit_expression(&assign.right);
            assignments::visit_target(self, &assign.left);
            return;
        }
        let exemptions = self.builder_reference_exemptions.len();
        if matches!(assign.left, AssignmentTarget::AssignmentTargetIdentifier(_))
            && assign.operator == ast::AssignmentOperator::Assign
        {
            self.allow_builder_value(&assign.right);
        }
        walk::walk_assignment_expression(self, assign);
        self.builder_reference_exemptions.truncate(exemptions);
        if let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assign.left {
            self.mark_dynamic(ident.name.as_str());
            self.set_variants(ident.name.as_str(), variants);
            self.set_builder_identity(ident.name.as_str(), identity);
        } else {
            resolve::functions::reassigned::for_each_assigned_name(&assign.left, &mut |name| {
                self.mark_dynamic(name);
            });
        }
    }
}
