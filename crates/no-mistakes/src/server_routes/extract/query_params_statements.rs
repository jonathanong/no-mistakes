fn collect_query_params_from_statement(statement: &Statement<'_>, params: &mut BTreeSet<String>) {
    match statement {
        Statement::ExpressionStatement(statement) => {
            collect_query_params_from_expression(&statement.expression, params);
        }
        Statement::VariableDeclaration(declaration) => {
            for declarator in &declaration.declarations {
                if let Some(init) = &declarator.init {
                    if expression_is_query_object(init) {
                        collect_query_object_destructure_names(&declarator.id, params);
                    }
                    collect_query_params_from_expression(init, params);
                }
            }
        }
        Statement::BlockStatement(block) => {
            for statement in &block.body {
                collect_query_params_from_statement(statement, params);
            }
        }
        Statement::ReturnStatement(statement) => {
            if let Some(argument) = &statement.argument {
                collect_query_params_from_expression(argument, params);
            }
        }
        Statement::IfStatement(statement) => {
            collect_query_params_from_expression(&statement.test, params);
            collect_query_params_from_statement(&statement.consequent, params);
            if let Some(alternate) = &statement.alternate {
                collect_query_params_from_statement(alternate, params);
            }
        }
        Statement::ForStatement(statement) => {
            if let Some(init) = &statement.init {
                if let oxc_ast::ast::ForStatementInit::VariableDeclaration(declaration) = init {
                    for declarator in &declaration.declarations {
                        if let Some(init) = &declarator.init {
                            collect_query_params_from_expression(init, params);
                        }
                    }
                } else if let Some(expression) = init.as_expression() {
                    collect_query_params_from_expression(expression, params);
                }
            }
            if let Some(test) = &statement.test {
                collect_query_params_from_expression(test, params);
            }
            if let Some(update) = &statement.update {
                collect_query_params_from_expression(update, params);
            }
            collect_query_params_from_statement(&statement.body, params);
        }
        Statement::WhileStatement(statement) => {
            collect_query_params_from_expression(&statement.test, params);
            collect_query_params_from_statement(&statement.body, params);
        }
        Statement::ForOfStatement(statement) => {
            collect_query_params_from_expression(&statement.right, params);
            collect_query_params_from_statement(&statement.body, params);
        }
        Statement::ForInStatement(statement) => {
            collect_query_params_from_expression(&statement.right, params);
            collect_query_params_from_statement(&statement.body, params);
        }
        Statement::SwitchStatement(statement) => {
            collect_query_params_from_expression(&statement.discriminant, params);
            for case in &statement.cases {
                if let Some(test) = &case.test {
                    collect_query_params_from_expression(test, params);
                }
                for consequent in &case.consequent {
                    collect_query_params_from_statement(consequent, params);
                }
            }
        }
        Statement::TryStatement(statement) => {
            for statement in &statement.block.body {
                collect_query_params_from_statement(statement, params);
            }
            if let Some(handler) = &statement.handler {
                for statement in &handler.body.body {
                    collect_query_params_from_statement(statement, params);
                }
            }
            if let Some(finalizer) = &statement.finalizer {
                for statement in &finalizer.body {
                    collect_query_params_from_statement(statement, params);
                }
            }
        }
        Statement::FunctionDeclaration(function) => {
            collect_query_params_from_optional_function_body(function.body.as_ref(), params);
        }
        Statement::ExportNamedDeclaration(export) => {
            if let Some(declaration) = &export.declaration {
                match declaration {
                    oxc_ast::ast::Declaration::VariableDeclaration(declaration) => {
                        for declarator in &declaration.declarations {
                            if let Some(init) = &declarator.init {
                                collect_query_params_from_expression(init, params);
                            }
                        }
                    }
                    oxc_ast::ast::Declaration::FunctionDeclaration(function) => {
                        collect_query_params_from_optional_function_body(
                            function.body.as_ref(),
                            params,
                        );
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}
