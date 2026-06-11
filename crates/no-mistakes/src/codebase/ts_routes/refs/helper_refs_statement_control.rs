fn collect_helper_refs_from_block_statement<'a>(
    block: &'a oxc::ast::ast::BlockStatement<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    let mut scoped_bindings = router_bindings.clone();
    let mut scoped_helper_bindings = helper_bindings.clone();
    collect_router_bindings_for_scope(&block.body, &mut scoped_bindings);
    collect_scope_helper_bindings(&block.body, &mut scoped_helper_bindings, local_helpers);
    for stmt in &block.body {
        collect_helper_refs_from_statement(
            stmt,
            source,
            file,
            &mut scoped_bindings,
            &mut scoped_helper_bindings,
            local_helpers,
            refs,
        );
    }
    collect_var_helper_bindings_for_scope(&block.body, helper_bindings, local_helpers);
}

fn collect_helper_refs_from_if_statement<'a>(
    if_stmt: &'a oxc::ast::ast::IfStatement<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    collect_helper_refs_from_expression(
        &if_stmt.test,
        source,
        file,
        router_bindings,
        helper_bindings,
        local_helpers,
        refs,
    );
    collect_helper_refs_from_statement(
        &if_stmt.consequent,
        source,
        file,
        router_bindings,
        helper_bindings,
        local_helpers,
        refs,
    );
    if let Some(alt) = &if_stmt.alternate {
        collect_helper_refs_from_statement(
            alt,
            source,
            file,
            router_bindings,
            helper_bindings,
            local_helpers,
            refs,
        );
    }
}

fn collect_helper_refs_from_for_statement<'a>(
    for_stmt: &'a oxc::ast::ast::ForStatement<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    let mut scoped_router_bindings = router_bindings.clone();
    let mut scoped_helper_bindings = helper_bindings.clone();
    collect_helper_refs_from_for_init(
        for_stmt,
        source,
        file,
        &mut scoped_router_bindings,
        &mut scoped_helper_bindings,
        local_helpers,
        refs,
    );
    if let Some(test) = &for_stmt.test {
        collect_helper_refs_from_expression(
            test,
            source,
            file,
            &mut scoped_router_bindings,
            &mut scoped_helper_bindings,
            local_helpers,
            refs,
        );
    }
    if let Some(update) = &for_stmt.update {
        collect_helper_refs_from_expression(
            update,
            source,
            file,
            &mut scoped_router_bindings,
            &mut scoped_helper_bindings,
            local_helpers,
            refs,
        );
    }
    collect_helper_refs_from_statement(
        &for_stmt.body,
        source,
        file,
        &mut scoped_router_bindings,
        &mut scoped_helper_bindings,
        local_helpers,
        refs,
    );
}

fn collect_helper_refs_from_loop_body<'a>(
    parts: (
        &'a Expression<'a>,
        &'a Statement<'a>,
        Option<&'a ForStatementLeft<'a>>,
    ),
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    let (test_or_right, body, left) = parts;
    collect_helper_refs_from_expression(
        test_or_right,
        source,
        file,
        router_bindings,
        helper_bindings,
        local_helpers,
        refs,
    );
    let mut scoped_router_bindings = router_bindings.clone();
    let mut scoped_helper_bindings = helper_bindings.clone();
    if let Some(ForStatementLeft::VariableDeclaration(var_decl)) = left {
        collect_router_bindings_from_var_decl(var_decl, &mut scoped_router_bindings);
        remove_shadowed_helper_var_bindings(var_decl, &mut scoped_helper_bindings, local_helpers);
    }
    collect_helper_refs_from_statement(
        body,
        source,
        file,
        &mut scoped_router_bindings,
        &mut scoped_helper_bindings,
        local_helpers,
        refs,
    );
}

fn collect_helper_refs_from_do_while_statement<'a>(
    do_while_stmt: &'a oxc::ast::ast::DoWhileStatement<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    collect_helper_refs_from_statement(
        &do_while_stmt.body,
        source,
        file,
        router_bindings,
        helper_bindings,
        local_helpers,
        refs,
    );
    collect_helper_refs_from_expression(
        &do_while_stmt.test,
        source,
        file,
        router_bindings,
        helper_bindings,
        local_helpers,
        refs,
    );
}
