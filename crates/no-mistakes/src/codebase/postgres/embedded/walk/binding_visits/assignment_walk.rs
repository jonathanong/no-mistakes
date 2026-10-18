use super::super::{assignments, resolve, ScopeVisitor};
use oxc_ast::ast::{self, AssignmentTarget};
use oxc_ast_visit::{walk, Visit};

impl<'a> ScopeVisitor<'a> {
    pub(in crate::codebase::postgres::embedded::walk) fn walk_assignment_expression_bindings(
        &mut self,
        assign: &oxc_ast::ast::AssignmentExpression<'a>,
    ) {
        if let AssignmentTarget::AssignmentTargetIdentifier(left) = &assign.left {
            if let ast::Expression::Identifier(right) =
                crate::codebase::ts_source::unwrap_ts_wrappers(&assign.right)
            {
                if assign.operator == ast::AssignmentOperator::Assign && left.name == right.name {
                    walk::walk_assignment_expression(self, assign);
                    return;
                }
            }
        }
        // Any assigned executor value may be rebound through its new owner.
        let tracked_alias = matches!(assign.left, AssignmentTarget::AssignmentTargetIdentifier(_))
            && assign.operator == ast::AssignmentOperator::Assign
            && matches!(
                crate::codebase::ts_source::unwrap_ts_wrappers(&assign.right),
                ast::Expression::Identifier(_)
            )
            && !self.receiver_origins(&assign.right).is_empty();
        let alias_origins =
            if let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assign.left {
                let mut origins = self.receiver_alias_origins(ident.name.as_str());
                for origin in self.receiver_origins(&assign.right) {
                    if !origins.contains(&origin) {
                        origins.push(origin);
                    }
                }
                origins
            } else {
                Vec::new()
            };
        if !tracked_alias {
            self.revoke_query_receiver(&assign.right);
        }
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
            // Keep all may-alias origins: a branch or later callback may
            // observe any prior assignment, including an opaque replacement.
            self.set_receiver_alias_origins(ident.name.as_str(), alias_origins);
        } else {
            resolve::functions::reassigned::for_each_assigned_name(&assign.left, &mut |name| {
                self.mark_dynamic(name);
            });
        }
    }
}
