fn collect_helper_refs_from_object_expression<'a>(
    obj: &'a oxc::ast::ast::ObjectExpression<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    for prop in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        collect_helper_refs_from_expression(
            &prop.value,
            source,
            file,
            router_bindings,
            helper_bindings,
            local_helpers,
            refs,
        );
    }
}
