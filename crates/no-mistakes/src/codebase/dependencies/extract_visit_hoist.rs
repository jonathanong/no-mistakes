fn predeclare_function_body<'a>(
    collector: &mut ImportCollector,
    function: &oxc_ast::ast::Function<'a>,
) {
    if let Some(body) = &function.body {
        predeclare_function_declarations(collector, &body.statements);
    }
}

fn predeclare_function_declarations<'a>(
    collector: &mut ImportCollector,
    statements: &[Statement<'a>],
) {
    for statement in statements {
        if let Statement::FunctionDeclaration(function) = statement {
            if let Some(name) = function_name(function) {
                collector.add_binding_name(&name);
                let scope = collector
                    .current_function()
                    .map(|parent| format!("{parent}/{name}"))
                    .unwrap_or(name);
                collector.known_function_scopes.insert(scope);
            }
        }
    }
}
