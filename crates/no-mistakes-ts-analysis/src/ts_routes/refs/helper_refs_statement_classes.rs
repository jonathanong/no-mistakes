fn collect_helper_refs_from_class_body<'a>(
    class: &'a oxc_ast::ast::Class<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    for element in &class.body.body {
        if let ClassElement::MethodDefinition(method) = element {
            collect_helper_refs_from_function_body(
                &method.value,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        if let ClassElement::PropertyDefinition(prop) = element {
            if let Some(value) = &prop.value {
                collect_helper_refs_from_expression(
                    value,
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
