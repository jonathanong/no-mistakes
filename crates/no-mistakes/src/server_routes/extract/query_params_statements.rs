fn collect_query_params_from_statement(
    statement: &Statement<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
    state: &mut QueryParamState,
) {
    match statement {
        Statement::ExpressionStatement(statement) => {
            collect_query_params_from_expression(&statement.expression, params, named_handlers, state);
        }
        Statement::VariableDeclaration(declaration) => {
            collect_query_params_from_variable_declaration(declaration, params, named_handlers, state);
        }
        Statement::BlockStatement(block) => {
            for statement in &block.body {
                collect_query_params_from_statement(statement, params, named_handlers, state);
            }
        }
        Statement::ReturnStatement(statement) => {
            if let Some(argument) = &statement.argument {
                collect_query_params_from_expression(argument, params, named_handlers, state);
            }
        }
        Statement::IfStatement(statement) => {
            collect_query_params_from_expression(&statement.test, params, named_handlers, state);
            collect_query_params_from_statement(&statement.consequent, params, named_handlers, state);
            if let Some(alternate) = &statement.alternate {
                collect_query_params_from_statement(alternate, params, named_handlers, state);
            }
        }
        Statement::ForStatement(statement) => {
            collect_query_params_from_for_statement(statement, params, named_handlers, state);
        }
        Statement::WhileStatement(statement) => {
            collect_query_params_from_expression(&statement.test, params, named_handlers, state);
            collect_query_params_from_statement(&statement.body, params, named_handlers, state);
        }
        Statement::ForOfStatement(statement) => {
            collect_query_params_from_expression(&statement.right, params, named_handlers, state);
            collect_query_params_from_statement(&statement.body, params, named_handlers, state);
        }
        Statement::ForInStatement(statement) => {
            collect_query_params_from_expression(&statement.right, params, named_handlers, state);
            collect_query_params_from_statement(&statement.body, params, named_handlers, state);
        }
        Statement::SwitchStatement(statement) => {
            collect_query_params_from_switch_statement(statement, params, named_handlers, state);
        }
        Statement::TryStatement(statement) => {
            collect_query_params_from_try_statement(statement, params, named_handlers, state);
        }
        Statement::ExportNamedDeclaration(export) => {
            collect_query_params_from_export_named_declaration(export, params, named_handlers);
        }
        _ => {}
    }
}

fn collect_query_params_from_variable_declaration(
    declaration: &oxc_ast::ast::VariableDeclaration<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
    state: &mut QueryParamState,
) {
    for declarator in &declaration.declarations {
        if let Some(init) = &declarator.init {
            if expression_is_query_object(init, &state.query_aliases) {
                if let BindingPattern::BindingIdentifier(identifier) = &declarator.id {
                    state.query_aliases.insert(identifier.name.as_str().to_string());
                } else {
                    collect_query_object_destructure_names(&declarator.id, params);
                }
            }
            collect_query_params_from_expression(init, params, named_handlers, state);
        }
    }
}

fn collect_query_params_from_for_statement(
    statement: &oxc_ast::ast::ForStatement<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
    state: &mut QueryParamState,
) {
    collect_query_params_from_for_init(statement.init.as_ref(), params, named_handlers, state);
    if let Some(test) = &statement.test {
        collect_query_params_from_expression(test, params, named_handlers, state);
    }
    if let Some(update) = &statement.update {
        collect_query_params_from_expression(update, params, named_handlers, state);
    }
    collect_query_params_from_statement(&statement.body, params, named_handlers, state);
}

fn collect_query_params_from_for_init(
    init: Option<&oxc_ast::ast::ForStatementInit<'_>>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
    state: &mut QueryParamState,
) {
    let Some(init) = init else {
        return;
    };
    if let oxc_ast::ast::ForStatementInit::VariableDeclaration(declaration) = init {
        collect_query_params_from_variable_declaration(declaration, params, named_handlers, state);
    } else if let Some(expression) = init.as_expression() {
        collect_query_params_from_expression(expression, params, named_handlers, state);
    }
}

fn collect_query_params_from_switch_statement(
    statement: &oxc_ast::ast::SwitchStatement<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
    state: &mut QueryParamState,
) {
    collect_query_params_from_expression(&statement.discriminant, params, named_handlers, state);
    for case in &statement.cases {
        if let Some(test) = &case.test {
            collect_query_params_from_expression(test, params, named_handlers, state);
        }
        for consequent in &case.consequent {
            collect_query_params_from_statement(consequent, params, named_handlers, state);
        }
    }
}

fn collect_query_params_from_try_statement(
    statement: &oxc_ast::ast::TryStatement<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
    state: &mut QueryParamState,
) {
    for statement in &statement.block.body {
        collect_query_params_from_statement(statement, params, named_handlers, state);
    }
    if let Some(handler) = &statement.handler {
        for statement in &handler.body.body {
            collect_query_params_from_statement(statement, params, named_handlers, state);
        }
    }
    if let Some(finalizer) = &statement.finalizer {
        for statement in &finalizer.body {
            collect_query_params_from_statement(statement, params, named_handlers, state);
        }
    }
}

fn collect_query_params_from_export_named_declaration(
    export: &oxc_ast::ast::ExportNamedDeclaration<'_>,
    params: &mut BTreeSet<String>,
    named_handlers: &HashMap<String, BTreeSet<String>>,
) {
    let Some(declaration) = &export.declaration else {
        return;
    };
    match declaration {
        oxc_ast::ast::Declaration::VariableDeclaration(declaration) => {
            let mut state = QueryParamState::default();
            collect_query_params_from_variable_declaration(declaration, params, named_handlers, &mut state);
        }
        oxc_ast::ast::Declaration::FunctionDeclaration(_) => {}
        _ => {}
    }
}
