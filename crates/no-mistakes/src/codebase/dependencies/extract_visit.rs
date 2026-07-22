#[derive(Default)]
struct ImportCollector {
    source: String,
    imports: Vec<ExtractedImport>,
    function_calls: Vec<FunctionCall>,
    symbol_references: Vec<FunctionCall>,
    unknown_callers: Vec<Option<String>>,
    function_stack: Vec<String>,
    local_stack: Vec<HashSet<String>>,
    type_local_stack: Vec<HashSet<String>>,
    type_parameter_stack: Vec<HashSet<String>>,
    function_scope_stack: Vec<usize>,
    exported_functions: HashSet<String>,
    exported_resource_roots: HashSet<String>,
    exported_resource_scopes: HashSet<String>,
    collect_resource_roots: bool,
    exported_type_scopes: HashSet<String>,
    callable_scopes: HashSet<String>,
    class_scopes: HashSet<String>,
    export_depth: usize,
    has_unknown_top_level_call: bool,
    anonymous_scope_count: usize,
    known_function_scopes: HashSet<String>,
    imported_bindings: HashSet<String>,
    suppress_imports: bool,
    collect_suppressed_runtime_imports: bool,
    /// `function_stack` depth captured at the start of an exported binding
    /// initializer / default-export expression. A runtime (`import()`/`require()`)
    /// import exactly one function level below this depth — the callback directly
    /// forming the exported value, e.g. `dynamic(() => import('./Foo'))` — is
    /// flagged reachable. Deeper, uninvoked nested imports fall back to normal
    /// call-scope reachability so they are not falsely kept.
    runtime_reachable_base_depth: Option<usize>,
    later_exported_type_names: HashSet<String>,
}

impl<'a> Visit<'a> for ImportCollector {
    fn visit_statement(&mut self, statement: &Statement<'a>) {
        record_statement_type_binding(self, statement);
        walk::walk_statement(self, statement);
    }

    fn visit_function(
        &mut self,
        function: &oxc_ast::ast::Function<'a>,
        flags: oxc_syntax::scope::ScopeFlags,
    ) {
        let name = function_name(function);
        if self.current_function().is_some() {
            if let Some(name) = &name {
                self.add_binding_name(name);
            }
        }
        if name.is_some() {
            self.push_function_scope(name);
            if let Some(scope) = self.current_function() {
                self.callable_scopes.insert(scope.clone());
                if self.export_depth > 0 && self.function_stack.len() == 1 {
                    self.exported_functions.insert(scope.clone());
                }
            }
        } else {
            self.push_anonymous_function_scope();
        }
        self.add_type_parameter_names(function.type_parameters.as_deref());
        self.add_formal_parameters(&function.params);
        predeclare_function_body(self, function);
        walk::walk_function(self, function, flags);
        self.pop_function_scope(true);
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        self.push_anonymous_function_scope();
        self.add_type_parameter_names(arrow.type_parameters.as_deref());
        self.add_formal_parameters(&arrow.params);
        walk::walk_arrow_function_expression(self, arrow);
        self.pop_function_scope(true);
    }

    fn visit_method_definition(&mut self, method: &MethodDefinition<'a>) {
        visit_method_definition_with_scope(self, method);
    }

    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        visit_object_property_with_scope(self, property);
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        visit_variable_declarator_with_scope(self, declarator);
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        visit_class_with_scope(self, class);
    }

    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        visit_variable_declaration_with_bindings(self, declaration);
        walk::walk_variable_declaration(self, declaration);
    }

    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        visit_block_statement_with_scope(self, block);
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
        let kind = import_declaration_kind(import);
        let side_effect_only = import.specifiers.as_ref().is_none_or(|specifiers| specifiers.is_empty());
        self.push_with_side_effect(
            import.source.value.as_str(),
            kind,
            import.span.start as usize,
            side_effect_only,
            false,
        );
        self.record_imported_bindings(import);
    }

    fn visit_export_named_declaration(&mut self, export: &ExportNamedDeclaration<'a>) {
        if let Some(source) = &export.source {
            let kind = export_named_declaration_kind(export);
            self.push_reexport(source.value.as_str(), kind, export.span.start as usize);
        } else if !export.export_kind.is_type() {
            for specifier in &export.specifiers {
                if specifier.export_kind.is_type() {
                    continue;
                }
                if let Some(name) = module_export_name_name(&specifier.local) {
                    self.exported_functions.insert(name.to_string());
                }
            }
        }
        self.export_depth += 1;
        walk::walk_export_named_declaration(self, export);
        self.export_depth -= 1;
    }

    fn visit_export_all_declaration(&mut self, export: &ExportAllDeclaration<'a>) {
        let kind = if export.export_kind.is_type() {
            ImportKind::Type
        } else {
            ImportKind::Static
        };
        self.push_reexport(export.source.value.as_str(), kind, export.span.start as usize);
    }

    fn visit_export_default_declaration(&mut self, export: &ExportDefaultDeclaration<'a>) {
        visit_export_default_declaration_with_scope(self, export);
    }

    fn visit_import_expression(&mut self, import: &ImportExpression<'a>) {
        if let Some(specifier) = static_import_specifier(&import.source) {
            self.push(&specifier, ImportKind::Dynamic, import.span.start as usize);
        }
        walk::walk_import_expression(self, import);
    }

    fn visit_ts_import_type(&mut self, import: &TSImportType<'a>) {
        self.push(import.source.value.as_str(), ImportKind::Type, import.span.start as usize);
        walk::walk_ts_import_type(self, import);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        visit_call_expression_with_imports(self, call);
        walk::walk_call_expression(self, call);
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
