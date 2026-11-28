fn router_method_binding_name<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a str> {
    match pattern {
        BindingPattern::BindingIdentifier(id) => Some(id.name.as_str()),
        BindingPattern::AssignmentPattern(assign) => router_method_binding_name(&assign.left),
        _ => None,
    }
}

fn remove_shadowed_name(name: &str, bindings: &mut RouterBindings<'_>) {
    bindings.objects.remove(name);
    bindings.methods.remove(name);
    bindings.redirects.remove(name);
    if name == "fetch" {
        bindings.fetch_shadowed = true;
    }
}

fn remove_shadowed_function_binding(
    func: &oxc_ast::ast::Function,
    bindings: &mut RouterBindings<'_>,
) {
    if let Some(id) = &func.id {
        remove_shadowed_name(id.name.as_str(), bindings);
    }
}

fn remove_shadowed_class_binding(class: &oxc_ast::ast::Class, bindings: &mut RouterBindings<'_>) {
    if let Some(id) = &class.id {
        remove_shadowed_name(id.name.as_str(), bindings);
    }
}

fn remove_shadowed_binding(pattern: &BindingPattern, bindings: &mut RouterBindings<'_>) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            remove_shadowed_name(id.name.as_str(), bindings);
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                remove_shadowed_binding(&prop.value, bindings);
            }
            if let Some(rest) = &obj.rest {
                remove_shadowed_binding(&rest.argument, bindings);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for element in arr.elements.iter().flatten() {
                remove_shadowed_binding(element, bindings);
            }
            if let Some(rest) = &arr.rest {
                remove_shadowed_binding(&rest.argument, bindings);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            remove_shadowed_binding(&assign.left, bindings);
        }
    }
}

fn remove_shadowed_parameters(
    params: &oxc_ast::ast::FormalParameters,
    bindings: &mut RouterBindings<'_>,
) {
    for param in &params.items {
        remove_shadowed_binding(&param.pattern, bindings);
    }
    if let Some(rest) = &params.rest {
        remove_shadowed_binding(&rest.rest.argument, bindings);
    }
}
