fn collect_helper_refs_from_try_statement<'a>(
    try_stmt: &'a oxc::ast::ast::TryStatement<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    collect_helper_refs_from_block_statement(
        &try_stmt.block,
        source,
        file,
        router_bindings,
        helper_bindings,
        local_helpers,
        refs,
    );
    if let Some(handler) = &try_stmt.handler {
        collect_helper_refs_from_block_statement(
            &handler.body,
            source,
            file,
            router_bindings,
            helper_bindings,
            local_helpers,
            refs,
        );
    }
    if let Some(finalizer) = &try_stmt.finalizer {
        collect_helper_refs_from_block_statement(
            finalizer,
            source,
            file,
            router_bindings,
            helper_bindings,
            local_helpers,
            refs,
        );
    }
}
