fn collect_helper_refs_from_var_declaration<'a>(
    var_decl: &'a oxc::ast::ast::VariableDeclaration<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    for decl in &var_decl.declarations {
        if let Some(init) = &decl.init {
            collect_helper_refs_from_expression(
                init,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
    }
}

fn collect_helper_refs_from_named_export<'a>(
    export: &'a oxc::ast::ast::ExportNamedDeclaration<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    match export.declaration.as_ref() {
        Some(oxc::ast::ast::Declaration::VariableDeclaration(var_decl)) => {
            collect_helper_refs_from_var_declaration(
                var_decl,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        Some(oxc::ast::ast::Declaration::FunctionDeclaration(func)) => {
            collect_helper_refs_from_function_body(
                func,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        _ => {}
    }
}

fn collect_helper_refs_from_default_export<'a>(
    export: &'a oxc::ast::ast::ExportDefaultDeclaration<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    match &export.declaration {
        oxc::ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
            collect_helper_refs_from_function_body(
                func,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        other => {
            if let Some(expr) = other.as_expression() {
                collect_helper_refs_from_expression(
                    expr,
                    source,
                    file,
                    router_bindings,
                    helper_bindings,
                    local_helpers,
                    refs,
                );
            }
        }
    }
}
