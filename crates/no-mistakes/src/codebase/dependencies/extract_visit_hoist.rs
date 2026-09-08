fn predeclare_function_body<'a>(
    collector: &mut ImportCollector,
    function: &oxc_ast::ast::Function<'a>,
) {
    if let Some(body) = &function.body {
        predeclare_callable_statements(collector, &body.statements);
    }
}

fn predeclare_arrow_body<'a>(
    collector: &mut ImportCollector,
    arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
) {
    if let oxc_ast::ast::ArrowFunctionBody::FunctionBody(body) = &arrow.body {
        predeclare_callable_statements(collector, &body.statements);
    }
}

fn predeclare_callable_statements<'a>(
    collector: &mut ImportCollector,
    statements: &[Statement<'a>],
) {
    predeclare_function_declarations(collector, statements);
    predeclare_lexical_declarations(collector, statements);
    let mut vars = FunctionVarCollector::default();
    for statement in statements {
        vars.visit_statement(statement);
    }
    for name in vars.names {
        collector.add_predeclared_function_call_binding_name(&name);
    }
}

#[derive(Default)]
struct FunctionVarCollector {
    names: Vec<String>,
}

impl<'a> Visit<'a> for FunctionVarCollector {
    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        if declaration.kind == VariableDeclarationKind::Var {
            for declarator in &declaration.declarations {
                self.names.extend(binding_names(&declarator.id));
            }
            return;
        }
        walk::walk_variable_declaration(self, declaration);
    }

    fn visit_function(
        &mut self,
        _function: &oxc_ast::ast::Function<'a>,
        _flags: oxc_syntax::scope::ScopeFlags,
    ) {
    }

    fn visit_arrow_function_expression(
        &mut self,
        _arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
    }

    fn visit_class(&mut self, _class: &Class<'a>) {}
}

fn predeclare_lexical_declarations(
    collector: &mut ImportCollector,
    statements: &[Statement<'_>],
) {
    for statement in statements {
        match statement {
            Statement::VariableDeclaration(declaration)
                if declaration.kind != VariableDeclarationKind::Var =>
            {
                for declarator in &declaration.declarations {
                    collector.add_predeclared_call_binding_names(&declarator.id);
                }
            }
            Statement::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    collector.add_predeclared_call_binding_name(id.name.as_str());
                }
            }
            Statement::ExportDeclaration(export) => match &export.declaration {
                oxc_ast::ast::Declaration::VariableDeclaration(declaration)
                    if declaration.kind != VariableDeclarationKind::Var =>
                {
                    for declarator in &declaration.declarations {
                        collector.add_predeclared_call_binding_names(&declarator.id);
                    }
                }
                oxc_ast::ast::Declaration::ClassDeclaration(class) => {
                    if let Some(id) = &class.id {
                        collector.add_predeclared_call_binding_name(id.name.as_str());
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn predeclare_function_declarations<'a>(
    collector: &mut ImportCollector,
    statements: &[Statement<'a>],
) {
    for statement in statements {
        let function = match statement {
            Statement::FunctionDeclaration(function) => Some(function.as_ref()),
            Statement::ExportDeclaration(export) => {
                let oxc_ast::ast::Declaration::FunctionDeclaration(function) = &export.declaration
                else {
                    continue;
                };
                Some(function.as_ref())
            }
            _ => None,
        };
        if let Some(name) = function.and_then(function_name) {
            collector.add_binding_name(&name);
            let scope = collector
                .current_function()
                .map(|parent| format!("{parent}/{name}"))
                .unwrap_or(name);
            collector.known_function_scopes.insert(scope.clone());
            collector.callable_scopes.insert(scope);
        }
    }
}
