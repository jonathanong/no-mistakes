fn collect_var_helper_bindings_for_scope(
    stmts: &[Statement<'_>],
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
) {
    for stmt in stmts {
        if let Statement::VariableDeclaration(var_decl) = stmt {
            if var_decl.kind == VariableDeclarationKind::Var {
                remove_shadowed_helper_var_bindings(var_decl, helper_bindings, local_helpers);
            }
        }
    }
}
