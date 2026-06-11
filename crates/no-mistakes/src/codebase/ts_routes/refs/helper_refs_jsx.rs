fn collect_helper_refs_from_jsx_element<'a>(
    jsx_elem: &'a JSXElement<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    for attr_item in &jsx_elem.opening_element.attributes {
        let JSXAttributeItem::Attribute(attr) = attr_item else {
            continue;
        };
        let attr_name = match &attr.name {
            JSXAttributeName::Identifier(id) => id.name.as_str(),
            JSXAttributeName::NamespacedName(_) => continue,
        };
        let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value else {
            continue;
        };
        let Some(expr) = container.expression.as_expression() else {
            continue;
        };
        if attr_name == "href" || attr_name == "to" {
            push_helper_refs_from_expression(
                expr,
                source,
                file,
                attr.span.start as usize,
                helper_bindings,
                refs,
            );
        } else {
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

    for child in &jsx_elem.children {
        collect_helper_refs_from_jsx_child(
            child,
            source,
            file,
            router_bindings,
            helper_bindings,
            local_helpers,
            refs,
        );
    }
}

fn collect_helper_refs_from_jsx_child<'a>(
    child: &'a JSXChild<'a>,
    source: &str,
    file: &str,
    router_bindings: &mut RouterBindings<'a>,
    helper_bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
    refs: &mut Vec<RouteHelperRef>,
) {
    match child {
        JSXChild::Element(elem) => {
            collect_helper_refs_from_jsx_element(
                elem,
                source,
                file,
                router_bindings,
                helper_bindings,
                local_helpers,
                refs,
            );
        }
        JSXChild::Fragment(frag) => {
            for child in &frag.children {
                collect_helper_refs_from_jsx_child(
                    child,
                    source,
                    file,
                    router_bindings,
                    helper_bindings,
                    local_helpers,
                    refs,
                );
            }
        }
        JSXChild::ExpressionContainer(container) => {
            if let Some(expr) = container.expression.as_expression() {
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
        _ => {}
    }
}
