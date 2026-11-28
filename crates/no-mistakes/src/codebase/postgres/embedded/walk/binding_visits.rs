use super::{resolve, ScopeVisitor};
use oxc_ast::ast;
use oxc_ast_visit::walk;

impl<'a> ScopeVisitor<'a> {
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
        let object = match member {
            ast::MemberExpression::StaticMemberExpression(value) => Some(&value.object),
            ast::MemberExpression::ComputedMemberExpression(value) => Some(&value.object),
            ast::MemberExpression::PrivateFieldExpression(_) => None,
        };
        if let Some(ast::Expression::Identifier(id)) =
            object.map(|object| crate::codebase::ts_source::unwrap_ts_wrappers(object))
        {
            self.invalidate_record_object(id.name.as_str());
        }
        match member {
            ast::MemberExpression::StaticMemberExpression(value)
                if value.property.name == "query" =>
            {
                self.revoke_mutated_query_member(&value.object);
            }
            ast::MemberExpression::ComputedMemberExpression(value)
                if query_key::may_write_query(
                    &value.expression,
                    &self.executor_options.executor_names,
                ) =>
            {
                self.revoke_mutated_query_member(&value.object);
            }
            ast::MemberExpression::StaticMemberExpression(value)
                if self.executor_options.track_transaction_helpers
                    && matches!(value.property.name.as_str(), "commit" | "rollback") =>
            {
                self.mutate_transaction_receiver(&value.object);
            }
            ast::MemberExpression::ComputedMemberExpression(value)
                if self.executor_options.track_transaction_helpers
                    && query_key::may_write_boundary(&value.expression) =>
            {
                self.mutate_transaction_receiver(&value.object);
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
        let tracked_closed = matches!(&declaration.id,ast::BindingPattern::BindingIdentifier(name) if self.literal_objects.closed.has(&self.scopes,name.name.as_str()));
        let forwarded = declaration
            .init
            .as_ref()
            .filter(|_| {
                self.declaring_plain_const
                    && matches!(declaration.id, ast::BindingPattern::BindingIdentifier(_))
            })
            .and_then(|init| self.forwarded_executor_wrapper(init));
        let executor = forwarded.is_some()
            || declaration
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
        let literal_reads = self.literal_objects.read_spans.len();
        let member_reads = self.literal_objects.member_uses.len();
        if self.declaring_const && tracked_closed {
            if let Some(init) = &declaration.init {
                self.exempt_closed_const_alias(init);
            }
        }
        let outer = self.enter_iife(declaration);
        let saved_plain = self.arm_plain_binding(declaration);
        self.record_forwarded_body(declaration, forwarded);
        walk::walk_variable_declarator(self, declaration);
        self.restore_plain_binding(saved_plain);
        self.literal_objects.read_spans.truncate(literal_reads);
        self.literal_objects.member_uses.truncate(member_reads);
        self.builder_reference_exemptions.truncate(exemptions);
        self.initialize_variants(declaration, variants);
        self.initialize_executor_alias(declaration, executor, namespace);
        resolve::bind_query_property(declaration, self.declaring_const, self);
        self.initialize_detached_query(declaration, self.declaring_const);
        if self.executor_options.track_transaction_helpers {
            self.note_transaction_options(declaration);
            self.note_transaction_candidates(declaration);
        }
        if let ast::BindingPattern::BindingIdentifier(name) = &declaration.id {
            self.bind_undeclared_declaration(
                name.name.as_str(),
                oxc_span::GetSpan::span(&declaration.id).start,
            );
        }
        if let (ast::BindingPattern::BindingIdentifier(name), Some(init)) =
            (&declaration.id, &declaration.init)
        {
            self.refresh_builder_alias(name.name.as_str(), init);
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
