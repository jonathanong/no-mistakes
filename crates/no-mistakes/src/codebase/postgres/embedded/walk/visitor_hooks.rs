use super::{imported, resolve, ScopeVisitor};
use oxc_ast::ast;
use oxc_ast_visit::walk;

impl<'a> ScopeVisitor<'a> {
    pub(super) fn walk_closed_call(&mut self, call: &ast::CallExpression<'a>) {
        let reads = self.literal_objects.read_spans.len();
        self.poison_closed_arguments(call);
        self.exempt_closed_assertion(call);
        self.visit_finite_call(call);
        self.literal_objects.read_spans.truncate(reads);
    }
    pub(super) fn walk_closed_throw(&mut self, statement: &ast::ThrowStatement<'a>) {
        self.poison_closed_result(&statement.argument);
        walk::walk_throw_statement(self, statement);
        self.invalidate_exited_execution();
    }
    pub(super) fn walk_classic_for_scope(&mut self, statement: &ast::ForStatement<'a>) {
        resolve::enter_classic_for(statement, self);
        self.with_append_loop(statement.span.start, |visitor| {
            walk::walk_for_statement(visitor, statement)
        });
        resolve::leave_classic_for(statement, self);
    }
    pub(super) fn walk_closed_property(&mut self, property: &ast::PropertyDefinition<'a>) {
        if let Some(value) = &property.value {
            self.poison_closed_result(value);
        }
        walk::walk_property_definition(self, property);
    }
    pub(super) fn walk_closed_accessor(&mut self, property: &ast::AccessorProperty<'a>) {
        if let Some(value) = &property.value {
            self.poison_closed_result(value);
        }
        walk::walk_accessor_property(self, property);
    }
    pub(super) fn walk_closed_parameter(&mut self, parameter: &ast::FormalParameter<'a>) {
        if let Some(value) = &parameter.initializer {
            self.poison_closed_result(value);
        }
        walk::walk_formal_parameter(self, parameter);
    }
    pub(super) fn walk_closed_jsx(&mut self, container: &ast::JSXExpressionContainer<'a>) {
        if let Some(value) = container.expression.as_expression() {
            self.poison_closed_result(value);
        }
        walk::walk_jsx_expression_container(self, container);
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
    pub(super) fn walk_closed_yield(&mut self, yielded: &ast::YieldExpression<'a>) {
        if let Some(value) = &yielded.argument {
            self.poison_closed_result(value);
        }
        walk::walk_yield_expression(self, yielded);
    }

    pub(super) fn walk_closed_return(&mut self, statement: &ast::ReturnStatement<'a>) {
        if let Some(value) = &statement.argument {
            self.poison_closed_result(value);
        }
        super::flow::returned(self, statement);
    }
    pub(super) fn walk_closed_new(&mut self, expression: &ast::NewExpression<'a>) {
        let reads = self.literal_objects.read_spans.len();
        self.poison_closed_constructor(expression);
        self.exempt_closed_set_input(expression);
        walk::walk_new_expression(self, expression);
        self.literal_objects.read_spans.truncate(reads);
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
        }
        self.escape_builder_reference(id);
    }
    pub(super) fn walk_executor_expression(&mut self, expression: &ast::Expression<'a>) {
        self.note_executor_member(expression);
        self.revoke_aggregate_values(expression);
        walk::walk_expression(self, expression);
    }
    pub(super) fn walk_computed_member(&mut self, member: &ast::ComputedMemberExpression<'a>) {
        let reads = self.literal_objects.read_spans.len();
        self.exempt_literal_read(&member.object);
        self.with_builder_receiver(&member.object, matches!(crate::codebase::ts_source::unwrap_ts_wrappers(&member.expression), ast::Expression::StringLiteral(name) if name.value == "text"), |visitor| {
            walk::walk_computed_member_expression(visitor, member)
        });
        self.literal_objects.read_spans.truncate(reads);
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
        let reads = self.literal_objects.read_spans.len();
        self.exempt_literal_read(&member.object);
        self.with_builder_receiver(&member.object, member.property.name == "text", |visitor| {
            walk::walk_static_member_expression(visitor, member)
        });
        self.literal_objects.read_spans.truncate(reads);
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
        self.poison_closed_iterable(&statement.right);
        self.with_append_loop(statement.span.start, |visitor| {
            walk::walk_for_of_statement(visitor, statement)
        });
        self.pop_scope();
    }
}
