fn walk_default_expression<'a>(
    collector: &mut ImportCollector,
    export: &ExportDefaultDeclaration<'a>,
) {
    if let Some(function) = parenthesized_default_function(&export.declaration) {
        walk_default_function_with_scope(collector, function, "default");
        return;
    }
    if let Some(arrow) = parenthesized_default_arrow(&export.declaration) {
        walk_default_arrow_with_scope(collector, arrow);
        return;
    }
    if default_expression_creates_own_scope(&export.declaration) {
        walk::walk_export_default_declaration(collector, export);
        return;
    }
    if let Some(object) = default_object_expression(&export.declaration) {
        record_object_member_calls(collector, "default", object);
    }
    collector.push_function_scope(Some("default".to_string()));
    // Flag the runtime import in the callback directly forming the default value —
    // e.g. `export default dynamic(() => import('./Foo'))` — as reachable through
    // the `default` binding instead of dropping it with its anonymous callback
    // scope. This keeps non-runtime (type) imports and, being depth-limited, does
    // not falsely keep deeper uninvoked nested imports.
    let saved_base_depth = collector.runtime_reachable_base_depth;
    collector.runtime_reachable_base_depth = Some(collector.function_stack.len());
    walk::walk_export_default_declaration(collector, export);
    collector.runtime_reachable_base_depth = saved_base_depth;
    collector.pop_function_scope(true);
}

fn parenthesized_default_function<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a oxc::ast::ast::Function<'a>> {
    let ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) = declaration else {
        return None;
    };
    parenthesized_function_expression(&parenthesized.expression)
}

fn parenthesized_function_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a oxc::ast::ast::Function<'a>> {
    match expression {
        Expression::FunctionExpression(function) => Some(function),
        Expression::ParenthesizedExpression(parenthesized) => {
            parenthesized_function_expression(&parenthesized.expression)
        }
        _ => None,
    }
}

fn parenthesized_default_arrow<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a oxc::ast::ast::ArrowFunctionExpression<'a>> {
    let ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) = declaration else {
        return None;
    };
    parenthesized_arrow_expression(&parenthesized.expression)
}

fn parenthesized_arrow_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a oxc::ast::ast::ArrowFunctionExpression<'a>> {
    match expression {
        Expression::ArrowFunctionExpression(arrow) => Some(arrow),
        Expression::ParenthesizedExpression(parenthesized) => {
            parenthesized_arrow_expression(&parenthesized.expression)
        }
        _ => None,
    }
}

fn default_expression_creates_own_scope(declaration: &ExportDefaultDeclarationKind<'_>) -> bool {
    match declaration {
        ExportDefaultDeclarationKind::FunctionExpression(_)
        | ExportDefaultDeclarationKind::ArrowFunctionExpression(_) => true,
        ExportDefaultDeclarationKind::ParenthesizedExpression(_) => false,
        _ => false,
    }
}

fn default_object_expression<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::ParenthesizedExpression(parenthesized) => {
            object_expression(&parenthesized.expression)
        }
        _ => None,
    }
}

fn object_expression<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::ParenthesizedExpression(parenthesized) => object_expression(&parenthesized.expression),
        _ => None,
    }
}

fn walk_default_arrow_with_scope<'a>(
    collector: &mut ImportCollector,
    arrow: &oxc::ast::ast::ArrowFunctionExpression<'a>,
) {
    collector.push_function_scope(Some("default".to_string()));
    collector.exported_functions.insert("default".to_string());
    collector.callable_scopes.insert("default".to_string());
    collector.add_type_parameter_names(arrow.type_parameters.as_deref());
    collector.add_formal_parameters(&arrow.params);
    walk::walk_arrow_function_expression(collector, arrow);
    collector.pop_function_scope(true);
}

fn walk_default_function_with_scope<'a>(
    collector: &mut ImportCollector,
    function: &oxc::ast::ast::Function<'a>,
    scope: &str,
) {
    collector.push_function_scope(Some(scope.to_string()));
    collector.exported_functions.insert(scope.to_string());
    collector.callable_scopes.insert(scope.to_string());
    collector.add_type_parameter_names(function.type_parameters.as_deref());
    collector.add_formal_parameters(&function.params);
    walk::walk_function(
        collector,
        function,
        oxc_syntax::scope::ScopeFlags::empty(),
    );
    collector.pop_function_scope(true);
}
