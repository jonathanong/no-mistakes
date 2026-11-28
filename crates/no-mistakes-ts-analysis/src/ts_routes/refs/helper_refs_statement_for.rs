fn collect_helper_refs_from_for_init<'a>(
    for_stmt: &'a oxc_ast::ast::ForStatement<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    let Some(init) = &for_stmt.init else {
        return;
    };
    match init {
        ForStatementInit::VariableDeclaration(var_decl) => {
            collect_helper_refs_from_var_declaration(
                var_decl,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
            collect_router_bindings_from_var_decl(var_decl, router_bindings);
            remove_shadowed_helper_var_bindings(var_decl, helper_bindings, local_helpers);
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
