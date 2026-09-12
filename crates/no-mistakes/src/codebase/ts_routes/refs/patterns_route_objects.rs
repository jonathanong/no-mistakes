fn object_route_pattern(obj: &oxc_ast::ast::ObjectExpression) -> Option<String> {
    let mut pathname = None;
    let mut query_params = BTreeSet::new();
    for prop in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        if property_key_name(&prop.key).is_some_and(|name| name == "pathname") {
            pathname = extract_pattern_from_expression(&prop.value);
        } else if property_key_name(&prop.key).is_some_and(|name| name == "query") {
            collect_static_query_keys(&prop.value, &mut query_params);
        }
    }
    let mut pattern = pathname?;
    append_query_params(&mut pattern, query_params);
    Some(pattern)
}

fn property_key_name<'a>(key: &'a PropertyKey<'_>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(s) => Some(s.value.as_str()),
        _ => None,
    }
}

fn collect_static_query_keys(expr: &Expression<'_>, query_params: &mut BTreeSet<String>) {
    let Expression::ObjectExpression(obj) = expr else {
        return;
    };
    for prop in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            continue;
        };
        if let Some(name) = property_key_name(&prop.key) {
            query_params.insert(name.to_string());
        }
    }
}

fn append_query_params(pattern: &mut String, query_params: BTreeSet<String>) {
    for param in query_params {
        pattern.push(if pattern.contains('?') { '&' } else { '?' });
        pattern.push_str(&param);
    }
}
