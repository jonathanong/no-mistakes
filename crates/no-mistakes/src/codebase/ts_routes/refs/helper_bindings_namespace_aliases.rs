fn register_namespace_destructured_helper_aliases(
    pattern: &BindingPattern<'_>,
    init: &Expression<'_>,
    bindings: &mut RouteHelperBindings,
) {
    let Expression::Identifier(namespace_id) = init else {
        return;
    };
    if !bindings.namespaces.contains(namespace_id.name.as_str()) {
        return;
    }
    let BindingPattern::ObjectPattern(obj) = pattern else {
        return;
    };
    for prop in &obj.properties {
        if let (PropertyKey::StaticIdentifier(imported), Some(local)) =
            (&prop.key, binding_identifier_name(&prop.value))
        {
            bindings.identifiers.insert(local.to_string());
            bindings.aliases.insert(
                local.to_string(),
                format!("{}.{}", namespace_id.name, imported.name),
            );
        }
    }
}
