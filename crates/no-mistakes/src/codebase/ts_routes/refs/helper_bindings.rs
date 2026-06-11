#[derive(Clone, Default)]
struct RouteHelperBindings {
    identifiers: HashSet<String>,
    namespaces: HashSet<String>,
    aliases: HashMap<String, String>,
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

fn bound_helper_callee_name(callee: &str, bindings: &RouteHelperBindings) -> Option<String> {
    if callee
        .split_once('.')
        .is_some_and(|(namespace, _)| bindings.namespaces.contains(namespace))
    {
        return Some(callee.to_string());
    }
    if !bindings.identifiers.contains(callee) {
        return None;
    }
    Some(
        bindings
            .aliases
            .get(callee)
            .cloned()
            .unwrap_or_else(|| callee.to_string()),
    )
}

fn remove_shadowed_helper_name(name: &str, bindings: &mut RouteHelperBindings) {
    bindings.identifiers.remove(name);
    bindings.namespaces.remove(name);
    bindings.aliases.remove(name);
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
    _local_helpers: &HashSet<String>,
) {
    if let Some(id) = &func.id {
        remove_shadowed_helper_name(id.name.as_str(), bindings);
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
        if !binding_identifier_name(&decl.id).is_some_and(|name| local_helpers.contains(name)) {
            remove_shadowed_helper_binding(&decl.id, bindings);
        }
        let (Some(name), Some(init)) = (binding_identifier_name(&decl.id), &decl.init) else {
            if let Some(init) = &decl.init {
                register_namespace_destructured_helper_aliases(&decl.id, init, bindings);
            }
            continue;
        };
        if let Some(target) = helper_alias_target(init, bindings) {
            bindings.identifiers.insert(name.to_string());
            bindings.aliases.insert(name.to_string(), target);
        }
    }
}

fn helper_alias_target(expr: &Expression, bindings: &RouteHelperBindings) -> Option<String> {
    match expr {
        Expression::ParenthesizedExpression(paren) => helper_alias_target(&paren.expression, bindings),
        Expression::TSAsExpression(type_expr) => helper_alias_target(&type_expr.expression, bindings),
        Expression::TSSatisfiesExpression(type_expr) => {
            helper_alias_target(&type_expr.expression, bindings)
        }
        Expression::TSNonNullExpression(type_expr) => helper_alias_target(&type_expr.expression, bindings),
        Expression::TSTypeAssertion(type_expr) => helper_alias_target(&type_expr.expression, bindings),
        other => route_helper_callee_name_from_callee(other)
            .and_then(|callee| bound_helper_callee_name(&callee, bindings)),
    }
}
