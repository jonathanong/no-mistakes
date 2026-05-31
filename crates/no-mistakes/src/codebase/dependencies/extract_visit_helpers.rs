impl ImportCollector {
    fn current_function(&self) -> Option<String> {
        self.function_stack.last().cloned()
    }

    fn push_symbol_reference(&mut self, name: String) {
        let Some(caller) = self.current_function() else {
            return;
        };
        if self.callee_shadows_import(&name) {
            return;
        }
        self.symbol_references.push(FunctionCall {
            caller: Some(caller),
            callee: name,
        });
    }

    fn add_formal_parameters(&mut self, params: &FormalParameters<'_>) {
        for param in &params.items {
            self.add_binding_names(&param.pattern);
        }
        if let Some(rest) = &params.rest {
            self.add_binding_names(&rest.rest.argument);
        }
    }

    fn add_type_parameter_names(&mut self, params: Option<&TSTypeParameterDeclaration<'_>>) {
        let Some(params) = params else { return };
        let Some(scope) = self.local_stack.last_mut() else {
            return;
        };
        for param in &params.params {
            scope.insert(param.name.name.to_string());
        }
    }

    fn add_function_binding_names(&mut self, pattern: &BindingPattern<'_>) {
        let Some(index) = self.function_scope_stack.last().copied() else {
            return;
        };
        let Some(scope) = self.local_stack.get_mut(index) else {
            return;
        };
        for name in binding_names(pattern) {
            scope.insert(name);
        }
    }

    fn add_binding_names(&mut self, pattern: &BindingPattern<'_>) {
        let Some(scope) = self.local_stack.last_mut() else {
            return;
        };
        for name in binding_names(pattern) {
            scope.insert(name);
        }
    }

    fn add_binding_name(&mut self, name: &str) {
        let Some(scope) = self.local_stack.last_mut() else {
            return;
        };
        scope.insert(name.to_string());
    }

    fn local_binding_shadows(&self, name: &str) -> bool {
        self.local_stack
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }

    fn callee_shadows_import(&self, callee: &str) -> bool {
        let binding = callee.split_once('.').map_or(callee, |(binding, _)| binding);
        self.local_binding_shadows(binding)
    }

    fn has_local_function_scope(&self, callee: &str) -> bool {
        let binding = callee.split_once('.').map_or(callee, |(binding, _)| binding);
        let Some(caller) = self.current_function() else {
            return self.known_function_scopes.contains(binding);
        };
        let mut scope = caller.as_str();
        loop {
            let candidate = format!("{scope}/{binding}");
            if self.known_function_scopes.contains(&candidate) {
                return true;
            }
            let Some((parent, _)) = scope.rsplit_once('/') else {
                return self.known_function_scopes.contains(binding);
            };
            scope = parent;
        }
    }
}

fn visit_variable_declarator_with_scope<'a>(
    collector: &mut ImportCollector,
    declarator: &VariableDeclarator<'a>,
) {
    let name = binding_identifier_name(&declarator.id).map(str::to_string);
    match declarator.init.as_ref() {
        Some(Expression::ArrowFunctionExpression(arrow)) => {
            push_variable_function_scope(collector, declarator, name);
            collector.add_type_parameter_names(arrow.type_parameters.as_deref());
            collector.add_formal_parameters(&arrow.params);
            walk::walk_arrow_function_expression(collector, arrow);
            collector.pop_function_scope(true);
        }
        Some(Expression::FunctionExpression(function)) => {
            let scope_name = name.or_else(|| function_name(function));
            push_variable_function_scope(collector, declarator, scope_name);
            collector.add_type_parameter_names(function.type_parameters.as_deref());
            collector.add_formal_parameters(&function.params);
            walk::walk_function(
                collector,
                function,
                oxc_syntax::scope::ScopeFlags::empty(),
            );
            collector.pop_function_scope(true);
        }
        Some(Expression::ObjectExpression(object))
            if name.is_some() && collector.function_stack.is_empty() =>
        {
            if let Some(name) = name.as_deref() {
                record_object_member_calls(collector, name, object);
            }
            walk::walk_variable_declarator(collector, declarator);
        }
        _ if name.is_some()
            && collector.function_stack.is_empty()
            && collector.export_depth > 0
            && declarator.init.is_some() =>
        {
            visit_exported_variable_declarator_reference(collector, declarator, name);
        }
        _ if name.is_some()
            && collector.function_stack.is_empty()
            && declarator.init.as_ref().is_some_and(is_simple_symbol_reference) =>
        {
            visit_exported_variable_declarator_reference(collector, declarator, name);
        }
        _ => walk::walk_variable_declarator(collector, declarator),
    }
}

fn visit_variable_declaration_with_bindings<'a>(
    collector: &mut ImportCollector,
    declaration: &VariableDeclaration<'a>,
) {
    if collector.current_function().is_none() {
        return;
    }
    for declarator in &declaration.declarations {
        if declaration.kind == VariableDeclarationKind::Var {
            collector.add_function_binding_names(&declarator.id);
        } else {
            collector.add_binding_names(&declarator.id);
        }
    }
}

fn visit_block_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    block: &BlockStatement<'a>,
) {
    let pushed = collector.push_lexical_scope();
    walk::walk_block_statement(collector, block);
    collector.pop_lexical_scope(pushed);
}

fn visit_catch_clause_with_scope<'a>(collector: &mut ImportCollector, clause: &CatchClause<'a>) {
    let pushed = collector.push_lexical_scope();
    if let Some(param) = &clause.param {
        collector.add_binding_names(&param.pattern);
    }
    walk::walk_catch_clause(collector, clause);
    collector.pop_lexical_scope(pushed);
}

fn visit_exported_type_alias_declaration<'a>(
    collector: &mut ImportCollector,
    declaration: &TSTypeAliasDeclaration<'a>,
) {
    let pushed = true;
    collector.push_function_scope(Some(declaration.id.name.to_string()));
    collector.add_type_parameter_names(declaration.type_parameters.as_deref());
    walk::walk_ts_type_alias_declaration(collector, declaration);
    collector.pop_function_scope(pushed);
}

fn visit_exported_interface_declaration<'a>(
    collector: &mut ImportCollector,
    declaration: &TSInterfaceDeclaration<'a>,
) {
    let pushed = true;
    collector.push_function_scope(Some(declaration.id.name.to_string()));
    collector.add_type_parameter_names(declaration.type_parameters.as_deref());
    walk::walk_ts_interface_declaration(collector, declaration);
    collector.pop_function_scope(pushed);
}
