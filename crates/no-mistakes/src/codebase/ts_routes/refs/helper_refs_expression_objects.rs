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
        match prop {
            ObjectPropertyKind::ObjectProperty(prop) if property_key_is_route_href(&prop.key) => {
                push_helper_refs_from_expression(
                    &prop.value,
                    source,
                    file,
                    prop.span.start as usize,
                    helper_bindings,
                    local_helpers,
                    refs,
                );
            }
            ObjectPropertyKind::ObjectProperty(prop) => {
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
            ObjectPropertyKind::SpreadProperty(spread) => {
                collect_helper_refs_from_expression(
                    &spread.argument,
                    source,
                    file,
                    router_bindings,
                    helper_bindings,
                    local_helpers,
                    refs,
                );
            }
        };
    }
}

fn collect_helper_refs_from_array_expression<'a>(
    array: &'a oxc::ast::ast::ArrayExpression<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    for element in &array.elements {
        let expr = match element {
            ArrayExpressionElement::SpreadElement(spread) => Some(&spread.argument),
            other => other.as_expression(),
        };
        let Some(expr) = expr else {
            continue;
        };
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

fn property_key_is_route_href(key: &PropertyKey<'_>) -> bool {
    match key {
        PropertyKey::StaticIdentifier(id) => matches!(id.name.as_str(), "href" | "to" | "pathname"),
        PropertyKey::StringLiteral(s) => matches!(s.value.as_str(), "href" | "to" | "pathname"),
        _ => false,
    }
}
