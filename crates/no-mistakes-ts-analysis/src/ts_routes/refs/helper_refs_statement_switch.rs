fn collect_helper_refs_from_switch_statement<'a>(
    switch_stmt: &'a oxc_ast::ast::SwitchStatement<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    collect_helper_refs_from_expression(
        &switch_stmt.discriminant,
        source,
        file,
        router_bindings,
        helper_bindings,
        local_helpers,
        refs,
    );
    for case in &switch_stmt.cases {
        if let Some(test) = &case.test {
            collect_helper_refs_from_expression(
                test,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        let mut case_router_bindings = router_bindings.clone();
        let mut case_helper_bindings = helper_bindings.clone();
        collect_scope_router_bindings(&case.consequent, &mut case_router_bindings);
        collect_scope_helper_bindings(&case.consequent, &mut case_helper_bindings, local_helpers);
        for stmt in &case.consequent {
            collect_helper_refs_from_statement(
                stmt,
                source,
                file,
                &mut case_router_bindings,
                &mut case_helper_bindings,
                local_helpers,
                refs,
            );
        }
    }
}
