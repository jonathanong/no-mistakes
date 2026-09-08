impl ImportCollector {
    fn record_const_callable_aliases(&mut self, declaration: &VariableDeclaration<'_>) {
        if declaration.kind != VariableDeclarationKind::Const || !self.is_function_or_module_scope()
        {
            return;
        }
        for declarator in &declaration.declarations {
            let (Some(local), Some(init)) = (
                binding_identifier_name(&declarator.id),
                declarator.init.as_ref(),
            ) else {
                continue;
            };
            let Some(target) = simple_callee_name(init) else {
                continue;
            };
            self.callable_aliases.push(CallableAliasBinding {
                alias: CallableAlias {
                    scope: self.current_function(),
                    local: local.to_string(),
                    target,
                },
                lexical_scope_depth: self.local_stack.len() - 1,
            });
        }
    }

    fn is_function_or_module_scope(&self) -> bool {
        match self.function_scope_stack.last().copied() {
            Some(function_scope) => self.local_stack.len() == function_scope + 1,
            // The program frame is lexical as well: a top-level block must not
            // expose a block-local alias as a module alias.
            None => self.local_stack.len() == 1,
        }
    }

    fn record_reassigned_callable_alias(&mut self, name: &str) {
        let Some(lexical_scope_depth) = self
            .local_stack
            .iter()
            .rposition(|scope| scope.contains(name))
        else {
            return;
        };
        let scope = self.current_function();
        self.reassigned_alias_bindings.extend(
            self.callable_aliases
                .iter()
                .filter(|binding| {
                    binding.alias.local == name
                        && binding.alias.scope == scope
                        && binding.lexical_scope_depth == lexical_scope_depth
                })
                .cloned(),
        );
    }

    fn call_target_identity(&self, callee: &str) -> CallTargetIdentity {
        let binding = callee
            .split_once('.')
            .map_or(callee, |(binding, _)| binding);
        if self.local_binding_shadows(binding) {
            return self
                .has_local_function_scope(callee)
                .then_some(CallTargetIdentity::RepositoryFunction)
                .unwrap_or(CallTargetIdentity::Unknown);
        }
        if self.imported_bindings.contains(binding)
            || self.predeclared_imported_bindings.contains(binding)
        {
            return CallTargetIdentity::ModuleExport;
        }
        if callee == binding || matches!(binding, "globalThis" | "window" | "self" | "global") {
            return CallTargetIdentity::Global;
        }
        CallTargetIdentity::Unknown
    }

    fn current_function(&self) -> Option<String> {
        self.function_stack.last().cloned()
    }

    fn push_value_symbol_reference(&mut self, name: String) {
        let caller = self.current_function();
        if self.callee_shadows_import(&name) {
            return;
        }
        self.symbol_references.push(FunctionCall {
            caller,
            callee: name,
            line: 0,
            offset: 0,
            is_callback: false,
            invocation: InvocationKind::Call,
            target_identity: CallTargetIdentity::Unknown,
            static_arg: None,
            static_cwd: None,
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

    fn add_var_binding_names(&mut self, pattern: &BindingPattern<'_>) {
        for name in binding_names(pattern) {
            self.add_var_binding_name(&name);
        }
    }

    // Kept as the narrower historical helper for its direct coverage test.
    fn add_var_binding_name(&mut self, name: &str) {
        let Some(index) = self
            .function_scope_stack
            .last()
            .copied()
            .or_else(|| (!self.local_stack.is_empty()).then_some(0))
        else {
            return;
        };
        let Some(scope) = self.local_stack.get_mut(index) else {
            return;
        };
        scope.insert(name.to_string());
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

    fn add_type_binding_name(&mut self, name: &str) {
        if self.type_local_stack.is_empty() {
            self.type_local_stack.push(HashSet::new());
        }
        if let Some(scope) = self.type_local_stack.last_mut() {
            scope.insert(name.to_string());
        }
    }

    fn local_binding_shadows(&self, name: &str) -> bool {
        self.local_stack
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }

    fn callee_shadows_import(&self, callee: &str) -> bool {
        let binding = callee
            .split_once('.')
            .map_or(callee, |(binding, _)| binding);
        self.local_binding_shadows(binding)
            && (self.imported_bindings.contains(binding)
                || self.predeclared_imported_bindings.contains(binding))
    }

    fn has_local_function_scope(&self, callee: &str) -> bool {
        let binding = callee
            .split_once('.')
            .map_or(callee, |(binding, _)| binding);
        // `api/run` can mean either an aggregate member or a lexical nested
        // function. A declared `function api` owns the latter spelling, so a
        // static `api.run()` must not be guessed as an aggregate dispatch.
        if callee.contains('.') && self.callable_scopes.contains(binding) {
            return false;
        }
        let Some(caller) = self.current_function() else {
            return self.callable_scopes.contains(binding)
                || self
                    .known_function_scopes
                    .contains(&binding.replace('.', "/"));
        };
        let mut scope = caller.as_str();
        loop {
            let candidate = format!("{scope}/{binding}");
            let member_candidate = format!("{scope}/{}", binding.replace('.', "/"));
            if self.callable_scopes.contains(&candidate)
                || self.callable_scopes.contains(&member_candidate)
            {
                return true;
            }
            let Some((parent, _)) = scope.rsplit_once('/') else {
                return self.callable_scopes.contains(binding)
                    || self.callable_scopes.contains(&binding.replace('.', "/"));
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
            walk::walk_function(collector, function, oxc_syntax::scope::ScopeFlags::empty());
            collector.pop_function_scope(true);
        }
        Some(Expression::ObjectExpression(object))
            if name.is_some() && collector.function_stack.is_empty() =>
        {
            if let Some(name) = name.as_deref() {
                record_object_member_calls(collector, name, object);
            }
            // Treat both inline `export const` and later `export { … }` named
            // object bindings as exported, so a registry written either way keeps
            // its dynamic-import edges reachable.
            let exported = collector.export_depth > 0
                || name
                    .as_deref()
                    .is_some_and(|name| collector.is_exported_top_level_name(name));
            if exported {
                if let Some(name) = name.as_deref() {
                    collector.record_exported_resource_root(name);
                    record_object_resource_scopes(collector, name, object);
                }
                visit_exported_variable_declarator_reference(collector, declarator, name);
            } else {
                if let Some(name) = name.as_deref() {
                    record_object_value_references(collector, name, object);
                    walk_object_values_with_parent_scope(collector, name, object);
                } else {
                    walk::walk_variable_declarator(collector, declarator);
                }
            }
        }
        Some(Expression::ClassExpression(class))
            if name.is_some() && collector.function_stack.is_empty() =>
        {
            if let Some(name) = name.as_deref() {
                record_class_member_calls(collector, name, class);
                if collector.is_exported_top_level_name(name) {
                    collector.record_exported_resource_root(name);
                    record_class_resource_scopes(collector, name, class);
                }
            }
            visit_exported_variable_declarator_reference(collector, declarator, name);
            walk::walk_variable_declarator(collector, declarator);
        }
        _ if name.is_some() && collector.function_stack.is_empty() && declarator.init.is_some() => {
            visit_exported_variable_declarator_reference(collector, declarator, name);
            walk::walk_variable_declarator(collector, declarator);
        }
        _ if collector.function_stack.is_empty() && declarator.init.is_some() => {
            visit_variable_declarator_references_for_bindings(collector, declarator);
            walk::walk_variable_declarator(collector, declarator);
        }
        _ => walk::walk_variable_declarator(collector, declarator),
    }
}

fn visit_variable_declaration_with_bindings<'a>(
    collector: &mut ImportCollector,
    declaration: &VariableDeclaration<'a>,
) {
    for declarator in &declaration.declarations {
        if declaration.kind == VariableDeclarationKind::Var {
            collector.add_var_binding_names(&declarator.id);
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
    predeclare_function_declarations(collector, &block.body);
    walk::walk_block_statement(collector, block);
    collector.pop_lexical_scope(pushed);
}

fn visit_switch_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    switch: &oxc_ast::ast::SwitchStatement<'a>,
) {
    // Case consequents share one lexical environment, while the discriminant
    // is evaluated outside it.
    collector.visit_expression(&switch.discriminant);
    let pushed = collector.push_lexical_scope();
    for case in &switch.cases {
        predeclare_function_declarations(collector, &case.consequent);
    }
    collector.visit_switch_cases(&switch.cases);
    collector.pop_lexical_scope(pushed);
}

fn visit_for_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::ForStatement<'a>,
) {
    let pushed = collector.push_lexical_scope();
    if let Some(init) = &statement.init {
        collector.visit_for_statement_init(init);
    }
    if let Some(test) = &statement.test {
        collector.visit_expression(test);
    }
    if let Some(update) = &statement.update {
        collector.visit_expression(update);
    }
    collector.visit_statement(&statement.body);
    collector.pop_lexical_scope(pushed);
}

fn visit_for_in_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::ForInStatement<'a>,
) {
    let pushed = collector.push_lexical_scope();
    collector.visit_for_statement_left(&statement.left);
    collector.visit_expression(&statement.right);
    collector.visit_statement(&statement.body);
    collector.pop_lexical_scope(pushed);
}

fn visit_for_of_statement_with_scope<'a>(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::ForOfStatement<'a>,
) {
    let pushed = collector.push_lexical_scope();
    collector.visit_for_statement_left(&statement.left);
    collector.visit_expression(&statement.right);
    collector.visit_statement(&statement.body);
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
