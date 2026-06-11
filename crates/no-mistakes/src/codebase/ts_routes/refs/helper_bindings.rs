#[derive(Clone, Default)]
struct RouteHelperBindings {
    identifiers: HashSet<String>,
    namespaces: HashSet<String>,
}

fn collect_route_helper_bindings(
    helpers: &[RouteHelper],
    imports: &[RouteHelperImport],
) -> RouteHelperBindings {
    let mut bindings = RouteHelperBindings::default();
    for helper in helpers {
        bindings.identifiers.insert(helper.name.clone());
    }
    for import in imports {
        if import.imported == "*" {
            bindings.namespaces.insert(import.local.clone());
        } else {
            bindings.identifiers.insert(import.local.clone());
        }
    }
    bindings
}

fn helper_callee_is_bound(callee: &str, bindings: &RouteHelperBindings) -> bool {
    callee
        .split_once('.')
        .map(|(namespace, _)| bindings.namespaces.contains(namespace))
        .unwrap_or_else(|| bindings.identifiers.contains(callee))
}

fn remove_shadowed_helper_name(name: &str, bindings: &mut RouteHelperBindings) {
    bindings.identifiers.remove(name);
    bindings.namespaces.remove(name);
}

fn remove_shadowed_helper_binding(pattern: &BindingPattern, bindings: &mut RouteHelperBindings) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            remove_shadowed_helper_name(id.name.as_str(), bindings);
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                remove_shadowed_helper_binding(&prop.value, bindings);
            }
            if let Some(rest) = &obj.rest {
                remove_shadowed_helper_binding(&rest.argument, bindings);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for element in arr.elements.iter().flatten() {
                remove_shadowed_helper_binding(element, bindings);
            }
            if let Some(rest) = &arr.rest {
                remove_shadowed_helper_binding(&rest.argument, bindings);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            remove_shadowed_helper_binding(&assign.left, bindings);
        }
    }
}

fn remove_shadowed_helper_parameters(
    params: &oxc::ast::ast::FormalParameters,
    bindings: &mut RouteHelperBindings,
) {
    for param in &params.items {
        remove_shadowed_helper_binding(&param.pattern, bindings);
    }
    if let Some(rest) = &params.rest {
        remove_shadowed_helper_binding(&rest.rest.argument, bindings);
    }
}

fn remove_shadowed_helper_function_binding(
    func: &oxc::ast::ast::Function,
    bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
) {
    if let Some(id) = &func.id {
        let name = id.name.as_str();
        if !local_helpers.contains(name) {
            remove_shadowed_helper_name(name, bindings);
        }
    }
}

fn remove_shadowed_helper_class_binding(
    class: &oxc::ast::ast::Class,
    bindings: &mut RouteHelperBindings,
) {
    if let Some(id) = &class.id {
        remove_shadowed_helper_name(id.name.as_str(), bindings);
    }
}

fn remove_shadowed_helper_var_bindings(
    var_decl: &oxc::ast::ast::VariableDeclaration<'_>,
    bindings: &mut RouteHelperBindings,
    local_helpers: &HashSet<String>,
) {
    for decl in &var_decl.declarations {
        if binding_identifier_name(&decl.id)
            .map(|name| local_helpers.contains(name))
            .unwrap_or(false)
        {
            continue;
        }
        remove_shadowed_helper_binding(&decl.id, bindings);
    }
}
