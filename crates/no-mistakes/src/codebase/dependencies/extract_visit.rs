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

    fn visit_ts_type_alias_declaration(&mut self, declaration: &TSTypeAliasDeclaration<'a>) {
        visit_ts_type_alias_declaration_with_scope(self, declaration);
    }

    fn visit_ts_interface_declaration(&mut self, declaration: &TSInterfaceDeclaration<'a>) {
        visit_ts_interface_declaration_with_scope(self, declaration);
    }

    fn visit_ts_enum_declaration(&mut self, declaration: &TSEnumDeclaration<'a>) {
        visit_ts_enum_declaration_with_scope(self, declaration);
    }

    fn visit_ts_module_block(&mut self, block: &TSModuleBlock<'a>) {
        visit_ts_module_block_with_depth(self, block);
    }

    fn visit_ts_namespace_declaration(&mut self, namespace: &TSNamespaceDeclaration<'a>) {
        visit_ts_namespace_declaration_with_path(self, namespace);
    }

    fn visit_binary_expression(&mut self, binary: &BinaryExpression<'a>) {
        visit_binary_expression_with_namespaces(self, binary);
    }

    fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
        visit_import_declaration_with_scope(self, import);
    }

    fn visit_export_named_declaration(&mut self, export: &ExportNamedDeclaration<'a>) {
        self.collect_local_export_specifiers(export);
    }

    fn visit_export_declaration(&mut self, export: &ExportDeclaration<'a>) {
        self.walk_inline_export_declaration(export);
    }

    fn visit_export_from_declaration(&mut self, export: &ExportFromDeclaration<'a>) {
        self.walk_sourced_export_declaration(export);
    }

    fn visit_export_all_declaration(&mut self, export: &ExportAllDeclaration<'a>) {
        visit_export_all_declaration_with_scope(self, export);
    }

    fn visit_export_default_declaration(&mut self, export: &ExportDefaultDeclaration<'a>) {
        self.record_default_value(export);
        visit_export_default_declaration_with_scope(self, export);
    }

    fn visit_import_expression(&mut self, import: &ImportExpression<'a>) {
        visit_import_expression_with_scope(self, import);
    }

    fn visit_ts_import_type(&mut self, import: &TSImportType<'a>) {
        visit_ts_import_type_with_scope(self, import);
    }

    fn visit_return_statement(&mut self, statement: &oxc_ast::ast::ReturnStatement<'a>) {
        self.visit_value_return(statement);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.visit_receiver_call(call);
    }

    fn visit_await_expression(&mut self, expression: &oxc_ast::ast::AwaitExpression<'a>) {
        self.invalidate_replacement_expression();
        self.record_unknown_fresh_call(expression.span.end);
        walk::walk_await_expression(self, expression);
        // Operand evaluation can create the first capture; completion must
        // invalidate it even when the pre-operand path was still cold.
        self.record_descriptor_call(expression.span.end);
    }

    fn visit_yield_expression(&mut self, expression: &oxc_ast::ast::YieldExpression<'a>) {
        self.invalidate_replacement_expression();
        self.record_unknown_fresh_call(expression.span.end);
        walk::walk_yield_expression(self, expression);
        self.record_descriptor_call(expression.span.end);
    }

    fn visit_unary_expression(&mut self, expression: &oxc_ast::ast::UnaryExpression<'a>) {
        if expression.operator == oxc_syntax::operator::UnaryOperator::Delete {
            self.invalidate_replacement_expression();
            if self.collect_value_flow {
                if let Some((name, path, start)) =
                    extract_value_flow::write_targets::from_expression(&expression.argument)
                {
                    self.record_reassigned_callable_alias_path(
                        &name,
                        start,
                        Some(&path),
                        expression.span.end,
                    );
                }
            }
        }
        walk::walk_unary_expression(self, expression);
    }

    fn visit_new_expression(&mut self, new: &NewExpression<'a>) {
        visit_new_expression_with_imports_and_calls(self, new);
    }

    fn visit_tagged_template_expression(&mut self, tagged: &TaggedTemplateExpression<'a>) {
        self.visit_receiver_tagged(tagged);
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        visit_identifier_reference_with_namespaces(self, identifier);
    }

    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        visit_static_member_expression_with_getters(self, member);
    }

    fn visit_computed_member_expression(&mut self, member: &ComputedMemberExpression<'a>) {
        visit_computed_member_expression_with_getters(self, member);
    }

    fn visit_ts_type_reference(&mut self, reference: &TSTypeReference<'a>) {
        visit_ts_type_reference_without_name_walk(self, reference);
    }

    fn visit_ts_type_query(&mut self, query: &oxc_ast::ast::TSTypeQuery<'a>) {
        self.walk_as_type_names(|collector| walk::walk_ts_type_query(collector, query));
    }

    fn visit_ts_class_implements(&mut self, implements: &oxc_ast::ast::TSClassImplements<'a>) {
        self.walk_as_type_names(|collector| walk::walk_ts_class_implements(collector, implements));
    }

    fn visit_ts_interface_heritage(&mut self, heritage: &oxc_ast::ast::TSInterfaceHeritage<'a>) {
        self.walk_as_type_names(|collector| walk::walk_ts_interface_heritage(collector, heritage));
    }

    fn visit_ts_type_parameter(&mut self, parameter: &TSTypeParameter<'a>) {
        visit_ts_type_parameter_without_name_walk(self, parameter);
    }

    fn visit_jsx_opening_element(&mut self, opening: &JSXOpeningElement<'a>) {
        visit_jsx_opening_element_with_value_reference(self, opening);
    }
}
