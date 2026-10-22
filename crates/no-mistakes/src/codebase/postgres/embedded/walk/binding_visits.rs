use super::{resolve, ScopeVisitor};
use oxc_ast::ast;
use oxc_ast_visit::walk;

impl<'a> ScopeVisitor<'a> {
    pub(super) fn walk_variable_declaration_bindings(
        &mut self,
        declaration: &oxc_ast::ast::VariableDeclaration<'a>,
    ) {
        let previous = self.declaring_const;
        self.declaring_const = declaration.kind == ast::VariableDeclarationKind::Const;
        for declarator in &declaration.declarations {
            if let Some(init) = &declarator.init {
                // A mutable alias or mixed const result can expose `.query`.
                let tracked_alias =
                    matches!(declarator.id, ast::BindingPattern::BindingIdentifier(_))
                        && matches!(
                            crate::codebase::ts_source::unwrap_ts_wrappers(init),
                            ast::Expression::Identifier(_)
                        )
                        && !self.receiver_origins(init).is_empty();
                if (!tracked_alias && declaration.kind != ast::VariableDeclarationKind::Const)
                    || (self.stable_executor_category(init).is_none()
                        && self.relative_executor_proof(init).is_none()
                        && !tracked_alias)
                {
                    self.revoke_query_receiver(init);
                }
            }
        }
        resolve::initialize_vars(declaration, self);
        walk::walk_variable_declaration(self, declaration);
        if declaration.kind == ast::VariableDeclarationKind::Const {
            for declarator in &declaration.declarations {
                resolve::bind_query_property(declarator, true, self);
            }
        }
        self.declaring_const = previous;
    }

    pub(super) fn walk_assignment_target_reference(
        &mut self,
        target: &ast::SimpleAssignmentTarget<'a>,
    ) {
        let wrapped = match target {
            ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(_) => return,
            ast::SimpleAssignmentTarget::TSAsExpression(value) => Some(&value.expression),
            ast::SimpleAssignmentTarget::TSSatisfiesExpression(value) => Some(&value.expression),
            ast::SimpleAssignmentTarget::TSNonNullExpression(value) => Some(&value.expression),
            ast::SimpleAssignmentTarget::TSTypeAssertion(value) => Some(&value.expression),
            _ => None,
        };
        let member = match wrapped {
            Some(expression) => {
                // Wrapped name writes also replace a reference without escaping it.
                let Some(member) = expression.get_inner_expression().as_member_expression() else {
                    return;
                };
                member
            }
            None => target.to_member_expression(),
        };
        self.walk_builder_write_member(member);
    }

    pub(super) fn walk_unary_write_expression(&mut self, unary: &ast::UnaryExpression<'a>) {
        if unary.operator.is_delete() {
            if let Some(member) = unary.argument.get_inner_expression().as_member_expression() {
                self.walk_builder_write_member(member);
                return;
            }
        }
        walk::walk_unary_expression(self, unary);
    }

    fn walk_builder_write_member(&mut self, member: &ast::MemberExpression<'a>) {
        match member {
            ast::MemberExpression::StaticMemberExpression(value)
                if value.property.name == "query" =>
            {
                self.revoke_mutated_query_member(&value.object);
            }
            ast::MemberExpression::ComputedMemberExpression(value)
                if query_key::may_write_query(&value.expression) =>
            {
                self.revoke_mutated_query_member(&value.object);
            }
            _ => {}
        }
        // The ordinary .text read projection must not exempt a write/delete
        // receiver. Walk each receiver/key once, including computed effects.
        match member {
            ast::MemberExpression::StaticMemberExpression(member) => {
                walk::walk_static_member_expression(self, member);
            }
            ast::MemberExpression::ComputedMemberExpression(member) => {
                walk::walk_computed_member_expression(self, member);
            }
            ast::MemberExpression::PrivateFieldExpression(member) => {
                walk::walk_private_field_expression(self, member);
            }
        }
    }

    pub(super) fn walk_variable_declarator_bindings(
        &mut self,
        declaration: &oxc_ast::ast::VariableDeclarator<'a>,
    ) {
        let executor = declaration
            .init
            .as_ref()
            .is_some_and(|init| self.proven_executor(init));
        let namespace = declaration
            .init
            .as_ref()
            .is_some_and(|init| self.namespace_expression(init));
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
        self.initialize_executor_alias(declaration, executor, namespace);
        if let ast::BindingPattern::BindingIdentifier(name) = &declaration.id {
            self.bind_undeclared_declaration(
                name.name.as_str(),
                oxc_span::GetSpan::span(&declaration.id).start,
            );
        }
        if let (ast::BindingPattern::BindingIdentifier(name), Some(init)) =
            (&declaration.id, &declaration.init)
        {
            if declaration.type_annotation.is_none() {
                self.bind_undeclared_alias(name.name.as_str(), init);
            }
            self.bind_receiver_alias(name.name.as_str(), init);
        }
        self.leave_iife(&declaration.id, outer);
    }
}

mod assignment_walk;
mod query_key;
