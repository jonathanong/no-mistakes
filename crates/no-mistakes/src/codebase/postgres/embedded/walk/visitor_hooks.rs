use super::{imported, resolve, ScopeVisitor};
use oxc_ast::ast;
use oxc_ast_visit::walk;

impl<'a> ScopeVisitor<'a> {
    pub(super) fn walk_variable_declaration_bindings(
        &mut self,
        declaration: &oxc_ast::ast::VariableDeclaration<'a>,
    ) {
        let previous = self.declaring_const;
        let previous_plain = self.declaring_plain_const;
        self.declaring_plain_const = declaration.kind == ast::VariableDeclarationKind::Const;
        self.declaring_const = declaration.kind == ast::VariableDeclarationKind::Const
            || (self.executor_options.track_transaction_helpers
                && matches!(
                    declaration.kind,
                    ast::VariableDeclarationKind::Using | ast::VariableDeclarationKind::AwaitUsing
                ));
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
                let copied_query =
                    !crate::codebase::postgres::executor_detachment::executor_bindings(
                        &declarator.id,
                        &self.executor_options.executor_names,
                    )
                    .is_empty()
                        && self.detached_query_receiver(init);
                if !copied_query
                    && ((!tracked_alias && !self.declaring_const)
                        || (self.stable_executor_category(init).is_none()
                            && self.relative_executor_proof(init).is_none()
                            && !tracked_alias))
                {
                    self.revoke_query_receiver(init);
                }
            }
        }
        resolve::initialize_vars(declaration, self);
        walk::walk_variable_declaration(self, declaration);
        self.declaring_const = previous;
        self.declaring_plain_const = previous_plain;
    }

    pub(super) fn walk_closed_call(&mut self, call: &ast::CallExpression<'a>) {
        for argument in &call.arguments {
            if let Some(ast::Expression::Identifier(id)) = argument
                .as_expression()
                .map(crate::codebase::ts_source::unwrap_ts_wrappers)
            {
                self.invalidate_record_object(id.name.as_str());
            }
        }
        let reads = self.literal_objects.read_spans.len();
        self.exempt_closed_assertion(call);
        self.visit_finite_call(call);
        self.literal_objects.read_spans.truncate(reads);
    }

    pub(super) fn walk_classic_for_scope(&mut self, statement: &ast::ForStatement<'a>) {
        resolve::enter_classic_for(statement, self);
        self.with_append_loop(statement.span.start, |visitor| {
            walk::walk_for_statement(visitor, statement)
        });
        resolve::leave_classic_for(statement, self);
    }

    pub(super) fn walk_closed_await(&mut self, awaited: &ast::AwaitExpression<'a>) {
        if self.executor_options.track_transaction_helpers {
            if let ast::Expression::CallExpression(call) =
                crate::codebase::ts_source::unwrap_ts_wrappers(&awaited.argument)
            {
                self.awaited_lock_helper_calls.insert(call.span.start);
            }
        }
        walk::walk_await_expression(self, awaited);
    }

    pub(super) fn walk_closed_new(&mut self, expression: &ast::NewExpression<'a>) {
        let reads = self.literal_objects.read_spans.len();
        let members = self.literal_objects.member_uses.len();
        self.exempt_closed_set_input(expression);
        walk::walk_new_expression(self, expression);
        self.literal_objects.read_spans.truncate(reads);
        self.literal_objects.member_uses.truncate(members);
        self.revoke_namespace_arguments(&expression.callee, &expression.arguments);
    }

    pub(super) fn walk_runtime_export(&mut self, declaration: &ast::ExportNamedDeclaration<'a>) {
        if !declaration.export_kind.is_type() {
            for specifier in &declaration.specifiers {
                if !specifier.export_kind.is_type() {
                    walk::walk_export_specifier(self, specifier);
                }
            }
        }
    }

    pub(super) fn walk_executor_identifier(&mut self, id: &ast::IdentifierReference<'a>) {
        self.note_preproof_reference(id);
        self.note_executor_identifier(id);
        self.poison_literal_reference(id);
        if self.executor_options.track_transaction_helpers {
            self.poison_transaction_options(id);
            self.note_caller_transaction_escape(id);
        }
        self.escape_builder_reference(id);
    }
    pub(super) fn walk_executor_expression(&mut self, expression: &ast::Expression<'a>) {
        self.note_executor_member(expression);
        self.revoke_aggregate_values(expression);
        walk::walk_expression(self, expression);
    }
    pub(super) fn walk_computed_member(&mut self, member: &ast::ComputedMemberExpression<'a>) {
        let members = self.literal_objects.member_uses.len();
        self.use_closed_computed_member(member);
        let reads = self.literal_objects.read_spans.len();
        self.exempt_literal_read(&member.object);
        self.with_builder_receiver(&member.object, matches!(crate::codebase::ts_source::unwrap_ts_wrappers(&member.expression), ast::Expression::StringLiteral(name) if name.value == "text"), |visitor| {
            walk::walk_computed_member_expression(visitor, member)
        });
        self.literal_objects.read_spans.truncate(reads);
        self.literal_objects.member_uses.truncate(members);
    }

    pub(super) fn walk_program_scope(&mut self, program: &ast::Program<'a>) {
        self.push_scope();
        self.note_closed_imports(program);
        resolve::hoist_vars(&program.body, self);
        resolve::record_statements(&program.body, self);
        walk::walk_program(self, program);
        if !self.head_calls_only {
            imported::constants::snapshot_exports(self, program);
        }
        self.pop_scope();
    }
    pub(super) fn walk_literal_member(&mut self, member: &ast::StaticMemberExpression<'a>) {
        let members = self.literal_objects.member_uses.len();
        self.use_closed_static_member(member);
        let reads = self.literal_objects.read_spans.len();
        self.exempt_literal_read(&member.object);
        self.with_builder_receiver(&member.object, member.property.name == "text", |visitor| {
            walk::walk_static_member_expression(visitor, member)
        });
        self.literal_objects.read_spans.truncate(reads);
        self.literal_objects.member_uses.truncate(members);
    }
    pub(super) fn walk_update_binding(&mut self, expression: &oxc_ast::ast::UpdateExpression<'a>) {
        if let oxc_ast::ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(id) =
            &expression.argument
        {
            self.mark_dynamic(id.name.as_str());
        }
        walk::walk_update_expression(self, expression);
    }
    pub(super) fn walk_for_of_scope(&mut self, statement: &oxc_ast::ast::ForOfStatement<'a>) {
        self.push_scope();
        resolve::bind_for_statement_left(&statement.left, self);
        self.with_append_loop(statement.span.start, |visitor| {
            walk::walk_for_of_statement(visitor, statement)
        });
        self.pop_scope();
    }
}
