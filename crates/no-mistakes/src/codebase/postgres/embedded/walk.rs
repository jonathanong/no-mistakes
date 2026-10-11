use super::EmbeddedSqlKind;
use oxc_ast::ast::{
    AssignmentTarget, BlockStatement, CallExpression, Function, FunctionBody, FunctionType,
    Program, ReturnStatement,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;
mod flow;
pub(crate) mod resolve;
mod scope;
mod state;
mod variants;
pub(super) use state::{collect_calls, BindingState, ScopeVisitor};

impl<'a> Visit<'a> for ScopeVisitor<'a> {
    fn visit_program(&mut self, program: &Program<'a>) {
        self.push_scope();
        resolve::hoist_vars(&program.body, self);
        resolve::record_statements(&program.body, self);
        walk::walk_program(self, program);
        self.pop_scope();
    }

    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        self.push_scope();
        resolve::record_statements(&block.body, self);
        walk::walk_block_statement(self, block);
        self.pop_scope();
    }

    fn visit_catch_clause(&mut self, clause: &oxc_ast::ast::CatchClause<'a>) {
        self.push_scope();
        if let Some(param) = &clause.param {
            self.bind_param(&param.pattern, false);
        }
        walk::walk_catch_clause(self, clause);
        self.pop_scope();
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        self.push_scope();
        self.record_params(&function.params);
        // A named function expression shadows outer helpers only in its own
        // body. LocalFunctions does not resolve through this self-reference,
        // so record a shadow marker in the scope just pushed.
        if function.r#type == FunctionType::FunctionExpression {
            if let Some(id) = &function.id {
                self.bind_self_name(id.name.as_str());
            }
        }
        self.enter_function();
        walk::walk_function(self, function, flags);
        self.leave_function();
        self.pop_scope();
    }

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        resolve::hoist_vars(&body.statements, self);
        resolve::record_statements(&body.statements, self);
        walk::walk_function_body(self, body);
    }

    fn visit_static_block(&mut self, block: &oxc_ast::ast::StaticBlock<'a>) {
        // Class static blocks are also independent var ownership boundaries.
        self.push_scope();
        self.enter_function();
        resolve::hoist_vars(&block.body, self);
        resolve::record_statements(&block.body, self);
        walk::walk_static_block(self, block);
        self.leave_function();
        self.pop_scope();
    }

    fn visit_variable_declaration(&mut self, declaration: &oxc_ast::ast::VariableDeclaration<'a>) {
        resolve::initialize_vars(declaration, self);
        walk::walk_variable_declaration(self, declaration);
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

    fn visit_variable_declarator(&mut self, declaration: &oxc_ast::ast::VariableDeclarator<'a>) {
        self.refresh_builder_identity(declaration);
        let variants = declaration
            .init
            .as_ref()
            .and_then(|init| self.recover_variants(init));
        walk::walk_variable_declarator(self, declaration);
        self.initialize_variants(declaration, variants);
    }

    fn visit_update_expression(&mut self, expression: &oxc_ast::ast::UpdateExpression<'a>) {
        if let oxc_ast::ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(id) =
            &expression.argument
        {
            self.mark_dynamic(id.name.as_str());
        }
        walk::walk_update_expression(self, expression);
    }

    fn visit_assignment_expression(&mut self, assign: &oxc_ast::ast::AssignmentExpression<'a>) {
        let identity = self.assignment_builder_identity(assign);
        let variants = self.assigned_variants(assign);
        walk::walk_assignment_expression(self, assign);
        if let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assign.left {
            self.mark_dynamic(ident.name.as_str());
            self.set_variants(ident.name.as_str(), variants);
            self.set_builder_identity(ident.name.as_str(), identity);
        }
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let helper_effect = self.parameter_helper_effect(call);
        let append_effect = self.append_alias_effect(call);
        if self.suppress_nested_builder_fragments == 0 {
            if let Some(argument) = call.arguments.first() {
                if let Some(sql_text) = resolve::appended_builder_fragment(call, self) {
                    self.push_fragment(
                        crate::codebase::ts_source::byte_offset_to_line(
                            self.source,
                            argument.span().start as usize,
                        ),
                        Some(sql_text),
                        argument.as_expression(),
                        Some(call.span.start),
                    );
                } else if resolve::is_builder_append(call, self) {
                    self.push_fragment(
                        crate::codebase::ts_source::byte_offset_to_line(
                            self.source,
                            argument.span().start as usize,
                        ),
                        None,
                        argument.as_expression(),
                        Some(call.span.start),
                    );
                }
            }
        }
        let variants = self
            .appended_variants(call)
            .map(|(name, values)| (name.to_string(), values));
        resolve::apply_append(self, call);
        if let Some((name, values)) = variants {
            self.set_variants(&name, values);
        }
        resolve::record_executor_call(self, call);
        walk::walk_call_expression(self, call);
        self.apply_parameter_helper_effect(helper_effect);
        self.apply_append_alias_effect(append_effect);
    }

    fn visit_return_statement(&mut self, statement: &ReturnStatement<'a>) {
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
        walk::walk_try_statement(self, statement);
        self.invalidate_execution_variants();
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
