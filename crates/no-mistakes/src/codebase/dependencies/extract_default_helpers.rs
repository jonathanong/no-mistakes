fn walk_default_expression<'a>(
    collector: &mut ImportCollector,
    export: &ExportDefaultDeclaration<'a>,
) {
    if default_expression_creates_own_scope(&export.declaration) {
        walk::walk_export_default_declaration(collector, export);
        return;
    }
    collector.push_function_scope(Some("default".to_string()));
    walk::walk_export_default_declaration(collector, export);
    collector.pop_function_scope(true);
}

fn default_expression_creates_own_scope(declaration: &ExportDefaultDeclarationKind<'_>) -> bool {
    match declaration {
        ExportDefaultDeclarationKind::FunctionExpression(_)
        | ExportDefaultDeclarationKind::ArrowFunctionExpression(_) => true,
        ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            parenthesized_expression_creates_own_scope(&parenthesized.expression)
        }
        _ => false,
    }
}

fn parenthesized_expression_creates_own_scope(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => true,
        Expression::ParenthesizedExpression(parenthesized) => {
            parenthesized_expression_creates_own_scope(&parenthesized.expression)
        }
        _ => false,
    }
}

fn walk_default_function<'a>(
    collector: &mut ImportCollector,
    function: &oxc::ast::ast::Function<'a>,
) {
    collector.push_function_scope(Some("default".to_string()));
    collector.exported_functions.insert("default".to_string());
    collector.callable_scopes.insert("default".to_string());
    collector.add_type_parameter_names(function.type_parameters.as_deref());
    collector.add_formal_parameters(&function.params);
    walk::walk_function(
        collector,
        function,
        oxc_syntax::scope::ScopeFlags::empty(),
    );
    collector.pop_function_scope(true);
}
