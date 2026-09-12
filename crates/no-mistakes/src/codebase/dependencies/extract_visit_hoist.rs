#[inline(never)]
fn walk_function_with_body_bindings<'a>(
    collector: &mut ImportCollector,
    function: &oxc_ast::ast::Function<'a>,
) {
    // Default parameters run in the parameter environment, before the body
    // lexical environment exists. Visit them before introducing body bindings.
    visit_type_parameter_constraints(collector, function.type_parameters.as_deref());
    walk::walk_formal_parameters(collector, &function.params);
    if let Some(return_type) = &function.return_type {
        walk::walk_ts_type_annotation(collector, return_type);
    }
    if let Some(body) = &function.body {
        predeclare_function_declarations(collector, &body.statements);
        walk::walk_function_body(collector, body);
    }
}

#[inline(never)]
fn walk_arrow_function_with_body_bindings<'a>(
    collector: &mut ImportCollector,
    arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
) {
    // Default parameters run before the body lexical environment exists.
    visit_type_parameter_constraints(collector, arrow.type_parameters.as_deref());
    walk::walk_formal_parameters(collector, &arrow.params);
    if let Some(return_type) = &arrow.return_type {
        walk::walk_ts_type_annotation(collector, return_type);
    }
    if let Some(body) = crate::ast::arrow_function_body(&arrow.body) {
        predeclare_function_declarations(collector, &body.statements);
        walk::walk_function_body(collector, body);
    } else if let Some(expression) = arrow.body.as_expression() {
        collector.visit_expression(expression);
    }
}

#[inline(never)]
fn predeclare_hoisted_var_bindings<'a>(
    collector: &mut ImportCollector,
    statements: &[Statement<'a>],
) {
    let mut names = fx_set();
    let mut visitor = HoistedVarBindingCollector { names: &mut names };
    for statement in statements {
        visitor.visit_statement(statement);
    }
    for name in names {
        collector.add_var_binding_name(&name);
    }
}

struct HoistedVarBindingCollector<'a> {
    names: &'a mut FxHashSet<String>,
}

impl<'ast> Visit<'ast> for HoistedVarBindingCollector<'_> {
    #[inline(never)]
    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'ast>) {
        if declaration.kind == VariableDeclarationKind::Var {
            for declarator in &declaration.declarations {
                self.names.extend(binding_names(&declarator.id));
            }
        }
        walk::walk_variable_declaration(self, declaration);
    }

    // Nested callables own their own `var` environments.
    #[inline(never)]
    fn visit_function(
        &mut self,
        _function: &oxc_ast::ast::Function<'ast>,
        _flags: oxc_syntax::scope::ScopeFlags,
    ) {
    }

    #[inline(never)]
    fn visit_arrow_function_expression(
        &mut self,
        _arrow: &oxc_ast::ast::ArrowFunctionExpression<'ast>,
    ) {
    }

    // Class static blocks own their own `var` environment, just like class
    // methods own their function environments. Neither can hoist into the
    // surrounding module or function.
    #[inline(never)]
    fn visit_class(&mut self, _class: &Class<'ast>) {}
}

#[inline(never)]
fn predeclare_function_declarations<'a>(
    collector: &mut ImportCollector,
    statements: &[Statement<'a>],
) {
    for statement in statements {
        let Statement::FunctionDeclaration(function) = statement else {
            continue;
        };
        if function.body.is_some() {
            if let Some(name) = function_name(function) {
                collector.record_callable_binding_id(&name, CallableId(function.span.start));
            }
        }
    }
    for statement in statements {
        match statement {
            Statement::FunctionDeclaration(function) => {
                if let Some(name) = function_name(function) {
                    collector.add_binding_name(&name);
                    if collector.callable_binding_id(&name).is_none() {
                        collector
                            .record_callable_binding_id(&name, CallableId(function.span.start));
                    }
                    let scope = collector.callable_scope_name(&name);
                    collector.known_function_scopes.insert(scope.clone());
                    collector.callable_scopes.insert(scope);
                }
            }
            // We only use this catalog to prove that a spelling is locally
            // bound. A call before a `let`/`const` declaration is a TDZ error
            // at runtime, but it must never be misclassified as a global API.
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    if declaration.kind == VariableDeclarationKind::Var {
                        collector.add_var_binding_names(&declarator.id);
                    } else {
                        collector.add_binding_names(&declarator.id);
                    }
                }
            }
            Statement::ClassDeclaration(class) => {
                if let Some(name) = class.id.as_ref() {
                    collector.add_binding_name(name.name.as_str());
                }
            }
            _ => {}
        }
    }
    predeclare_hoisted_var_bindings(collector, statements);
}

impl ImportCollector {
    #[inline(never)]
    fn record_callable_declaration_bindings(&mut self, declaration: &VariableDeclaration<'_>) {
        let binding_scope = if declaration.kind == VariableDeclarationKind::Var {
            self.var_scope_stack.last().copied().unwrap_or(0)
        } else {
            self.local_stack.len() - 1
        };
        let binding_scope = self.lexical_scope_ids[binding_scope];
        for declarator in &declaration.declarations {
            let callable_id = match declarator.init.as_ref() {
                Some(Expression::ArrowFunctionExpression(arrow)) => CallableId(arrow.span.start),
                Some(Expression::FunctionExpression(function)) => CallableId(function.span.start),
                Some(Expression::ObjectExpression(_)) => CallableId(declarator.span.start),
                _ => continue,
            };
            if let Some(name) = binding_identifier_name(&declarator.id) {
                self.insert_callable_binding_name_at(binding_scope, name.to_string());
                self.insert_callable_binding_at(binding_scope, name.to_string(), callable_id);
                self.insert_binding_declared_at(
                    binding_scope,
                    name.to_string(),
                    declarator.span.start,
                );
            }
        }
    }
}
