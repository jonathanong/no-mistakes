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
    if let Some(class) = parenthesized_default_class(&export.declaration) {
        walk_default_class_with_scope(collector, class);
        return;
    }
    if default_expression_creates_own_scope(&export.declaration) {
        walk::walk_export_default_declaration(collector, export);
        return;
    }
    collector.record_exported_resource_root("default");
    if let Some(object) = default_object_expression(&export.declaration) {
        record_object_member_calls(
            collector,
            "default",
            "default",
            CallableId(export.span.start),
            object,
        );
        record_object_resource_scopes(collector, "default", object);
    }
    collector.push_function_scope(Some("default".to_string()), CallableId(export.span.start));
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
) -> Option<&'a oxc_ast::ast::Function<'a>> {
    default_expression(declaration).and_then(parenthesized_function_expression)
}

fn parenthesized_function_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a oxc_ast::ast::Function<'a>> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::FunctionExpression(function) => Some(function),
        _ => None,
    }
}

fn parenthesized_default_arrow<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a oxc_ast::ast::ArrowFunctionExpression<'a>> {
    default_expression(declaration).and_then(parenthesized_arrow_expression)
}

fn parenthesized_default_class<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a Class<'a>> {
    default_expression(declaration).and_then(|expression| {
        match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
            Expression::ClassExpression(class) => Some(class.as_ref()),
            _ => None,
        }
    })
}

fn parenthesized_arrow_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a oxc_ast::ast::ArrowFunctionExpression<'a>> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::ArrowFunctionExpression(arrow) => Some(arrow),
        _ => None,
    }
}

fn default_expression_creates_own_scope(declaration: &ExportDefaultDeclarationKind<'_>) -> bool {
    match default_expression(declaration).map(crate::codebase::ts_source::unwrap_ts_wrappers) {
        // Bare `export default () => …` is an expression; parenthesized functions
        // are unwrapped before this helper runs, and `export default function`
        // is a FunctionDeclaration rather than FunctionExpression.
        Some(Expression::ArrowFunctionExpression(_)) => true,
        _ => false,
    }
}

fn default_object_expression<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        _ => default_expression(declaration).and_then(object_expression),
    }
}

fn object_expression<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

fn default_expression<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a Expression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ParenthesizedExpression(expression) => {
            Some(&expression.expression)
        }
        ExportDefaultDeclarationKind::TSAsExpression(expression) => Some(&expression.expression),
        ExportDefaultDeclarationKind::TSNonNullExpression(expression) => {
            Some(&expression.expression)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(expression) => {
            Some(&expression.expression)
        }
        ExportDefaultDeclarationKind::TSTypeAssertion(expression) => Some(&expression.expression),
        _ => None,
    }
}

fn walk_default_arrow_with_scope<'a>(
    collector: &mut ImportCollector,
    arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
) {
    collector.push_function_scope(Some("default".to_string()), CallableId(arrow.span.start));
    collector.exported_functions.insert("default".to_string());
    collector.callable_scopes.insert("default".to_string());
    collector.add_type_parameter_names(arrow.type_parameters.as_deref());
    collector.add_formal_parameters(&arrow.params);
    walk_arrow_function_with_body_bindings(collector, arrow);
    collector.pop_function_scope(true);
}

fn walk_default_function_with_scope<'a>(
    collector: &mut ImportCollector,
    function: &oxc_ast::ast::Function<'a>,
    scope: &str,
) {
    let pushed_syntactic_caller = collector.push_syntactic_caller(function_name(function));
    collector.push_function_scope(Some(scope.to_string()), CallableId(function.span.start));
    collector.exported_functions.insert(scope.to_string());
    collector.callable_scopes.insert(scope.to_string());
    collector.add_type_parameter_names(function.type_parameters.as_deref());
    collector.add_formal_parameters(&function.params);
    walk_function_with_body_bindings(collector, function);
    collector.pop_function_scope(true);
    collector.pop_syntactic_caller(pushed_syntactic_caller);
}

fn walk_default_class_with_scope<'a>(collector: &mut ImportCollector, class: &Class<'a>) {
    let class_id = CallableId(class.span.start);
    let scope = class
        .id
        .as_ref()
        .map_or_else(|| "default".to_string(), |id| id.name.to_string());
    // Export resolution uses a synthetic default spelling for this class.
    collector.record_callable_binding_id("default", class_id);
    if scope != "default" {
        collector.push_callable_alias("default".to_string(), scope.clone(), class.span.start);
    }
    collector.callable_scope_ids.insert((class_id, scope.clone()));
    record_class_member_calls(collector, &scope, class_id, class);
    record_class_base_construction(collector, &scope, class_id, class);
    collector.record_exported_resource_root(&scope);
    record_class_resource_scopes(collector, &scope, class);
    collector.exported_functions.insert(scope.clone());
    collector.callable_scopes.insert(scope.clone());
    collector.class_scopes.insert(scope.clone());
    walk_class_with_scoped_methods(collector, &scope, class_id, class);
}
