use oxc_ast::ast::{self, CallExpression};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;
mod assignments;
mod binding_visits;
mod calls;
mod executor_arguments;
mod executors;
mod flow;
mod iife;
mod imported;
mod namespace_imports;
mod reference_escapes;
pub(crate) mod resolve;
mod scope;
mod scope_visits;
mod state;
mod variants;
pub(super) use state::{collect_calls, BindingState, ScopeVisitor};

impl<'a> Visit<'a> for ScopeVisitor<'a> {
    fn visit_program(&mut self, program: &ast::Program<'a>) {
        self.push_scope();
        resolve::hoist_vars(&program.body, self);
        resolve::record_statements(&program.body, self);
        walk::walk_program(self, program);
        self.pop_scope();
    }

    fn visit_block_statement(&mut self, block: &ast::BlockStatement<'a>) {
        self.push_scope();
        resolve::record_statements(&block.body, self);
        walk::walk_block_statement(self, block);
        self.pop_scope();
    }

    fn visit_catch_clause(&mut self, clause: &oxc_ast::ast::CatchClause<'a>) {
        self.walk_catch_clause_scope(clause);
    }

    fn visit_function(&mut self, function: &ast::Function<'a>, flags: ScopeFlags) {
        self.walk_function_scope(function, flags);
    }

    fn visit_function_body(&mut self, body: &ast::FunctionBody<'a>) {
        resolve::hoist_vars(&body.statements, self);
        resolve::record_statements(&body.statements, self);
        walk::walk_function_body(self, body);
    }

    fn visit_class(&mut self, class: &oxc_ast::ast::Class<'a>) {
        self.walk_class_scope(class);
    }

    fn visit_static_block(&mut self, block: &oxc_ast::ast::StaticBlock<'a>) {
        self.walk_static_block_scope(block);
    }

    fn visit_variable_declaration(&mut self, declaration: &oxc_ast::ast::VariableDeclaration<'a>) {
        self.walk_variable_declaration_bindings(declaration);
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        self.push_scope();
        self.record_params(&arrow.params);
        self.enter_function();
        walk::walk_arrow_function_expression(self, arrow);
        self.leave_function();
        self.pop_scope();
    }

    fn visit_variable_declarator(&mut self, declaration: &ast::VariableDeclarator<'a>) {
        self.walk_variable_declarator_bindings(declaration);
    }

    fn visit_update_expression(&mut self, expression: &oxc_ast::ast::UpdateExpression<'a>) {
        if let oxc_ast::ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(id) =
            &expression.argument
        {
            self.mark_dynamic(id.name.as_str());
        }
        walk::walk_update_expression(self, expression);
    }

    fn visit_assignment_expression(&mut self, assign: &ast::AssignmentExpression<'a>) {
        self.walk_assignment_expression_bindings(assign);
    }

    fn visit_expression_statement(&mut self, statement: &oxc_ast::ast::ExpressionStatement<'a>) {
        self.note_statement_call(statement);
        walk::walk_expression_statement(self, statement);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.visit_sql_call(call);
    }

    fn visit_simple_assignment_target(&mut self, target: &ast::SimpleAssignmentTarget<'a>) {
        self.walk_assignment_target_reference(target);
    }

    fn visit_unary_expression(&mut self, unary: &ast::UnaryExpression<'a>) {
        self.walk_unary_write_expression(unary);
    }

    fn visit_tagged_template_expression(&mut self, template: &ast::TaggedTemplateExpression<'a>) {
        self.visit_builder_template(template);
    }
    // Erased types cannot retain a runtime builder reference.
    fn visit_ts_type(&mut self, _: &ast::TSType<'a>) {}
    fn visit_identifier_reference(&mut self, id: &ast::IdentifierReference<'a>) {
        self.escape_builder_reference(id);
    }
    fn visit_expression(&mut self, expression: &ast::Expression<'a>) {
        self.revoke_aggregate_values(expression);
        walk::walk_expression(self, expression);
    }
    fn visit_static_member_expression(&mut self, member: &ast::StaticMemberExpression<'a>) {
        self.with_builder_receiver(&member.object, member.property.name == "text", |visitor| {
            walk::walk_static_member_expression(visitor, member)
        });
    }
    fn visit_computed_member_expression(&mut self, member: &ast::ComputedMemberExpression<'a>) {
        self.with_builder_receiver(&member.object, matches!(crate::codebase::ts_source::unwrap_ts_wrappers(&member.expression), ast::Expression::StringLiteral(name) if name.value == "text"), |visitor| {
            walk::walk_computed_member_expression(visitor, member)
        });
    }

    fn visit_return_statement(&mut self, statement: &ast::ReturnStatement<'a>) {
        flow::returned(self, statement);
    }
    fn visit_throw_statement(&mut self, statement: &oxc_ast::ast::ThrowStatement<'a>) {
        walk::walk_throw_statement(self, statement);
        self.invalidate_execution_variants();
    }
    fn visit_break_statement(&mut self, statement: &oxc_ast::ast::BreakStatement<'a>) {
        walk::walk_break_statement(self, statement);
        self.invalidate_execution_variants();
    }
    fn visit_continue_statement(&mut self, statement: &oxc_ast::ast::ContinueStatement<'a>) {
        walk::walk_continue_statement(self, statement);
        self.invalidate_execution_variants();
    }
    fn visit_try_statement(&mut self, statement: &oxc_ast::ast::TryStatement<'a>) {
        flow::try_statement(self, statement);
    }

    fn visit_for_statement(&mut self, statement: &oxc_ast::ast::ForStatement<'a>) {
        resolve::enter_classic_for(statement, self);
        self.with_loop(|visitor| walk::walk_for_statement(visitor, statement));
        resolve::leave_classic_for(statement, self);
    }

    fn visit_for_in_statement(&mut self, statement: &oxc_ast::ast::ForInStatement<'a>) {
        self.push_scope();
        resolve::bind_for_statement_left(&statement.left, self);
        self.with_loop(|visitor| walk::walk_for_in_statement(visitor, statement));
        self.pop_scope();
    }

    fn visit_for_of_statement(&mut self, statement: &oxc_ast::ast::ForOfStatement<'a>) {
        self.push_scope();
        resolve::bind_for_statement_left(&statement.left, self);
        self.with_loop(|visitor| walk::walk_for_of_statement(visitor, statement));
        self.pop_scope();
    }

    fn visit_while_statement(&mut self, statement: &oxc_ast::ast::WhileStatement<'a>) {
        self.with_loop(|visitor| walk::walk_while_statement(visitor, statement));
    }

    fn visit_do_while_statement(&mut self, statement: &oxc_ast::ast::DoWhileStatement<'a>) {
        self.with_loop(|visitor| walk::walk_do_while_statement(visitor, statement));
    }

    fn visit_switch_statement(&mut self, statement: &oxc_ast::ast::SwitchStatement<'a>) {
        variants::switch::statement(self, statement);
    }
    fn visit_if_statement(&mut self, statement: &oxc_ast::ast::IfStatement<'a>) {
        variants::control::if_statement(self, statement);
    }
    fn visit_conditional_expression(&mut self, expr: &oxc_ast::ast::ConditionalExpression<'a>) {
        variants::control::conditional(self, expr);
    }
    fn visit_logical_expression(&mut self, expr: &oxc_ast::ast::LogicalExpression<'a>) {
        variants::control::logical(self, expr);
    }
}
