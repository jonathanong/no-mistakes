fn record_object_resource_scopes(
    collector: &mut ImportCollector,
    parent: &str,
    object: &ObjectExpression<'_>,
) {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        let Some(name) = crate::codebase::ts_source::static_property_key_name(&property.key) else {
            continue;
        };
        let scope = format!("{parent}/{name}");
        match &property.value {
            Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => {
                collector.record_exported_resource_scope(scope);
            }
            Expression::ObjectExpression(nested) => {
                record_object_resource_scopes(collector, &scope, nested);
            }
            _ => {}
        }
    }
}

fn record_class_resource_scopes(
    collector: &mut ImportCollector,
    parent: &str,
    class: &Class<'_>,
) {
    for element in &class.body.body {
        match element {
            ClassElement::MethodDefinition(method) => {
                if let Some(name) =
                    crate::codebase::ts_source::static_property_key_name(&method.key)
                {
                    collector.record_exported_resource_scope(format!("{parent}/{name}"));
                }
            }
            ClassElement::PropertyDefinition(property)
                if matches!(
                    property.value,
                    Some(
                        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
                    )
                ) =>
            {
                if let Some(name) =
                    crate::codebase::ts_source::static_property_key_name(&property.key)
                {
                    collector.record_exported_resource_scope(format!("{parent}/{name}"));
                }
            }
            _ => {}
        }
    }
}
