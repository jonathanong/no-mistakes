fn evaluate_helper_call<'a>(
    call: &'a oxc::ast::ast::CallExpression<'a>,
    defs: &HashMap<&'a str, HelperDef<'a>>,
    imported_helpers: &RouteHelperBindings,
    env: &HashMap<String, Vec<String>>,
    depth: usize,
) -> Vec<String> {
    let Some(callee) = route_helper_callee_name_from_callee(&call.callee) else {
        return vec!["*".to_string()];
    };
    let Some(def) = defs.get(callee.as_str()) else {
        if helper_callee_is_bound(&callee, imported_helpers) {
            return vec!["/**".to_string()];
        }
        return vec!["*".to_string()];
    };
    let mut provided = HashMap::new();
    for (param, arg) in def.params.items.iter().zip(call.arguments.iter()) {
        let Some(name) = binding_identifier_name(&param.pattern) else {
            continue;
        };
        let value = arg
            .as_expression()
            .map(|expr| evaluate_route_expression(expr, defs, imported_helpers, env, depth + 1))
            .unwrap_or_else(|| vec!["*".to_string()]);
        provided.insert(name.to_string(), value);
    }
    let values = evaluate_helper_def(def, defs, imported_helpers, &provided, depth + 1);
    if values.is_empty() {
        vec!["*".to_string()]
    } else {
        values
    }
}

fn evaluate_url_object_expression<'a>(
    obj: &'a oxc::ast::ast::ObjectExpression<'a>,
    defs: &HashMap<&'a str, HelperDef<'a>>,
    imported_helpers: &RouteHelperBindings,
    env: &HashMap<String, Vec<String>>,
    depth: usize,
) -> Vec<String> {
    obj.properties
        .iter()
        .find_map(|prop| {
            let ObjectPropertyKind::ObjectProperty(prop) = prop else {
                return None;
            };
            property_key_is_pathname(&prop.key).then(|| {
                evaluate_route_expression(&prop.value, defs, imported_helpers, env, depth + 1)
            })
        })
        .unwrap_or_else(|| vec!["*".to_string()])
}
