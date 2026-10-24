impl<'a> Visit<'a> for ImportCollector {
    fn visit_statements(&mut self, statements: &oxc_allocator::Vec<'a, Statement<'a>>) {
        visit_statements_with_receiver_region(self, statements);
    }
    fn visit_expression(&mut self, expression: &Expression<'a>) {
        self.record_callable_value_use(expression);
        visit_expression_with_annex_b_initializer(self, expression);
    }

    fn visit_statement(&mut self, statement: &Statement<'a>) {
        visit_statement_with_type_binding(self, statement);
    }

    fn visit_if_statement(&mut self, statement: &oxc_ast::ast::IfStatement<'a>) {
        visit_if_with_execution(self, statement);
    }
    fn visit_logical_expression(&mut self, expression: &oxc_ast::ast::LogicalExpression<'a>) {
        visit_logical_with_execution(self, expression);
    }
    fn visit_conditional_expression(
        &mut self,
        expression: &oxc_ast::ast::ConditionalExpression<'a>,
    ) {
        visit_conditional_with_execution(self, expression);
    }
    fn visit_assignment_pattern(&mut self, pattern: &oxc_ast::ast::AssignmentPattern<'a>) {
        visit_binding_default_with_execution(self, pattern);
    }
    fn visit_assignment_target_with_default(
        &mut self,
        target: &oxc_ast::ast::AssignmentTargetWithDefault<'a>,
    ) {
        visit_target_default_with_execution(self, target);
    }
    fn visit_formal_parameter(&mut self, parameter: &oxc_ast::ast::FormalParameter<'a>) {
        visit_parameter_default_with_execution(self, parameter);
    }

    fn visit_function(
        &mut self,
        function: &oxc_ast::ast::Function<'a>,
        _flags: oxc_syntax::scope::ScopeFlags,
    ) {
        visit_function_with_value_flow_scope(self, function);
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        visit_arrow_function_with_value_flow_scope(self, arrow);
    }

    fn visit_method_definition(&mut self, method: &MethodDefinition<'a>) {
        visit_method_definition_with_scope(self, method);
    }

    fn visit_property_definition(&mut self, property: &PropertyDefinition<'a>) {
        visit_property_definition_with_decorators(self, property);
    }

    fn visit_accessor_property(&mut self, property: &AccessorProperty<'a>) {
        visit_accessor_property_with_decorators(self, property);
    }

    fn visit_static_block(&mut self, block: &StaticBlock<'a>) {
        visit_class_static_block_with_scope(self, block);
    }

    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        visit_object_property_with_scope(self, property);
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if self.export_depth > 0 && self.function_stack.is_empty() {
            for name in binding_names(&declarator.id) {
                self.record_local_export_binding(name.clone(), name);
            }
        }
        visit_variable_declarator_with_scope(self, declarator);
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        visit_class_with_scope(self, class);
    }

    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        visit_variable_declaration_with_values(self, declaration);
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        visit_assignment_expression_with_calls(self, assignment);
    }

    fn visit_update_expression(&mut self, update: &oxc_ast::ast::UpdateExpression<'a>) {
        visit_update_expression_with_calls(self, update);
    }

    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        visit_block_statement_with_scope(self, block);
    }

    fn visit_switch_statement(&mut self, switch: &oxc_ast::ast::SwitchStatement<'a>) {
        visit_switch_statement_with_scope(self, switch);
    }

    fn visit_for_statement(&mut self, statement: &oxc_ast::ast::ForStatement<'a>) {
        visit_for_statement_with_scope(self, statement);
    }

    fn visit_for_in_statement(&mut self, statement: &oxc_ast::ast::ForInStatement<'a>) {
        visit_for_in_statement_with_scope(self, statement);
    }

    fn visit_for_of_statement(&mut self, statement: &oxc_ast::ast::ForOfStatement<'a>) {
        visit_for_of_statement_with_scope(self, statement);
    }

    fn visit_while_statement(&mut self, statement: &oxc_ast::ast::WhileStatement<'a>) {
        with_repeat_execution(self, statement, visit_while_statement_per_item);
    }
    fn visit_do_while_statement(&mut self, statement: &oxc_ast::ast::DoWhileStatement<'a>) {
        with_repeat_execution(self, statement, visit_do_while_statement_per_item);
    }

    fn visit_try_statement(&mut self, statement: &oxc_ast::ast::TryStatement<'a>) {
        visit_try_with_receiver_execution(self, statement);
    }
    fn visit_labeled_statement(&mut self, statement: &oxc_ast::ast::LabeledStatement<'a>) {
        visit_label_with_receiver_execution(self, statement);
    }
    fn visit_catch_clause(&mut self, clause: &CatchClause<'a>) {
        visit_catch_clause_with_scope(self, clause);
    }

    visit_type_and_expression_methods!();

    visit_import_collector_tail!();
}
