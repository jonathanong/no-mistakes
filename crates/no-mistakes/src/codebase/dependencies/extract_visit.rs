impl<'a> Visit<'a> for ImportCollector {
    fn visit_statement(&mut self, statement: &Statement<'a>) {
        record_statement_type_binding(self, statement);
        walk::walk_statement(self, statement);
    }

    fn visit_function(
        &mut self,
        function: &oxc_ast::ast::Function<'a>,
        _flags: oxc_syntax::scope::ScopeFlags,
    ) {
        let name = function_name(function);
        let callable_id = name
            .as_deref()
            .and_then(|name| self.callable_binding_id(name))
            .unwrap_or(CallableId(function.span.start));
        let pushed_syntactic_caller = self.push_syntactic_caller(name.clone());
        if self.current_function().is_some() {
            if let Some(name) = &name {
                self.add_binding_name(name);
            }
        }
        if name.is_some() {
            if let Some(name) = &name {
                self.record_callable_binding_id(name, callable_id);
            }
            self.push_function_scope(name, callable_id);
            if let Some(scope) = self.current_function() {
                self.callable_scopes.insert(scope.clone());
                if self.export_depth > 0 && self.function_stack.len() == 1 {
                    self.exported_functions.insert(scope.clone());
                    self.record_local_export_binding(scope.clone(), scope);
                }
            }
        } else {
            self.push_anonymous_function_scope(callable_id);
        }
        self.add_type_parameter_names(function.type_parameters.as_deref());
        self.add_formal_parameters(&function.params);
        walk_function_with_body_bindings(self, function);
        self.pop_function_scope(true);
        self.pop_syntactic_caller(pushed_syntactic_caller);
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        self.push_anonymous_function_scope(CallableId(arrow.span.start));
        self.add_type_parameter_names(arrow.type_parameters.as_deref());
        self.add_formal_parameters(&arrow.params);
        walk::walk_arrow_function_expression(self, arrow);
        self.pop_function_scope(true);
    }

    fn visit_method_definition(&mut self, method: &MethodDefinition<'a>) {
        visit_method_definition_with_scope(self, method);
    }

    fn visit_property_definition(&mut self, property: &PropertyDefinition<'a>) {
        walk_decorators_as_invocations(self, &property.decorators);
        self.visit_property_key(&property.key);
        if let Some(type_annotation) = &property.type_annotation {
            self.visit_ts_type_annotation(type_annotation);
        }
        if let Some(value) = &property.value {
            self.visit_expression(value);
        }
    }

    fn visit_accessor_property(&mut self, property: &AccessorProperty<'a>) {
        walk_decorators_as_invocations(self, &property.decorators);
        self.visit_property_key(&property.key);
        if let Some(type_annotation) = &property.type_annotation {
            self.visit_ts_type_annotation(type_annotation);
        }
        if let Some(value) = &property.value {
            self.visit_expression(value);
        }
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
        visit_variable_declaration_with_bindings(self, declaration);
        self.record_callable_declaration_bindings(declaration);
        self.record_const_callable_aliases(declaration);
        walk::walk_variable_declaration(self, declaration);
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        for name in assignment_target_names(&assignment.left) {
            self.record_reassigned_callable_alias(&name);
        }
        walk::walk_assignment_expression(self, assignment);
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
        if self.function_stack.is_empty()
            && self.is_exported_top_level_name(declaration.id.name.as_str())
        {
            visit_exported_enum_declaration(self, declaration);
        } else {
            walk::walk_ts_enum_declaration(self, declaration);
        }
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
        visit_export_default_declaration_with_scope(self, export);
    }

    fn visit_import_expression(&mut self, import: &ImportExpression<'a>) {
        visit_import_expression_with_scope(self, import);
    }

    fn visit_ts_import_type(&mut self, import: &TSImportType<'a>) {
        visit_ts_import_type_with_scope(self, import);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        visit_call_expression_with_imports(self, call);
        walk::walk_call_expression(self, call);
    }

    fn visit_new_expression(&mut self, new: &NewExpression<'a>) {
        visit_new_expression_with_imports(self, new);
        walk::walk_new_expression(self, new);
    }

    fn visit_tagged_template_expression(&mut self, tagged: &TaggedTemplateExpression<'a>) {
        visit_tagged_template_expression_with_imports(self, tagged);
        walk::walk_tagged_template_expression(self, tagged);
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        self.push_value_symbol_reference(identifier.name.to_string());
        walk::walk_identifier_reference(self, identifier);
    }

    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if let Some(name) = simple_static_member_name(member) {
            self.push_value_symbol_reference(name);
        }
        walk::walk_static_member_expression(self, member);
    }

    fn visit_ts_type_reference(&mut self, reference: &TSTypeReference<'a>) {
        visit_ts_type_reference_without_name_walk(self, reference);
    }

    fn visit_ts_type_parameter(&mut self, parameter: &TSTypeParameter<'a>) {
        visit_ts_type_parameter_without_name_walk(self, parameter);
    }

    fn visit_jsx_opening_element(&mut self, opening: &JSXOpeningElement<'a>) {
        if let Some(name) = jsx_element_reference_name(&opening.name) {
            if name.chars().next().is_some_and(char::is_uppercase) {
                self.push_value_symbol_reference(name);
            }
        }
        walk::walk_jsx_opening_element(self, opening);
    }
}
