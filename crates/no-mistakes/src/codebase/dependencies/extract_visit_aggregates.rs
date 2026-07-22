fn visit_method_definition_with_scope<'a>(
    collector: &mut ImportCollector,
    method: &MethodDefinition<'a>,
) {
    let name = crate::codebase::ts_source::static_property_key_name(&method.key);
    let keep_class_scope = collector.current_function().is_some_and(|scope| {
        collector.class_scopes.contains(&scope) && collector.exported_functions.contains(&scope)
    });
    let saved_function_stack =
        (!keep_class_scope).then(|| std::mem::take(&mut collector.function_stack));
    walk::walk_decorators(collector, &method.decorators);
    walk::walk_property_key(collector, &method.key);
    if let Some(saved_function_stack) = saved_function_stack {
        collector.function_stack = saved_function_stack;
    }
    let pushed = name.is_some();
    collector.push_function_scope(name.map(str::to_string));
    if let Some(scope) = collector.current_function() {
        collector.callable_scopes.insert(scope);
    }
    collector.add_type_parameter_names(method.value.type_parameters.as_deref());
    collector.add_formal_parameters(&method.value.params);
    walk::walk_function(
        collector,
        &method.value,
        oxc_syntax::scope::ScopeFlags::empty(),
    );
    collector.pop_function_scope(pushed);
}

fn visit_object_property_with_scope<'a>(
    collector: &mut ImportCollector,
    property: &ObjectProperty<'a>,
) {
    let name = crate::codebase::ts_source::static_property_key_name(&property.key);
    match &property.value {
        Expression::FunctionExpression(function) => {
            walk::walk_property_key(collector, &property.key);
            let pushed = name.is_some();
            collector.push_function_scope(name.map(str::to_string));
            if let Some(scope) = collector.current_function() {
                collector.callable_scopes.insert(scope);
            }
            collector.add_type_parameter_names(function.type_parameters.as_deref());
            collector.add_formal_parameters(&function.params);
            walk::walk_function(
                collector,
                function,
                oxc_syntax::scope::ScopeFlags::empty(),
            );
            collector.pop_function_scope(pushed);
        }
        Expression::ArrowFunctionExpression(arrow) => {
            walk::walk_property_key(collector, &property.key);
            let pushed = name.is_some();
            collector.push_function_scope(name.map(str::to_string));
            if let Some(scope) = collector.current_function() {
                collector.callable_scopes.insert(scope);
            }
            collector.add_type_parameter_names(arrow.type_parameters.as_deref());
            collector.add_formal_parameters(&arrow.params);
            walk::walk_arrow_function_expression(collector, arrow);
            collector.pop_function_scope(pushed);
        }
        _ => walk::walk_object_property(collector, property),
    }
}

fn visit_class_with_scope<'a>(collector: &mut ImportCollector, class: &Class<'a>) {
    if collector.current_function().is_none() {
        if let Some(name) = class.id.as_ref().map(|id| id.name.as_str()) {
            record_class_member_calls(collector, name, class);
            if collector.is_exported_top_level_name(name) {
                collector.record_exported_resource_root(name);
                record_class_resource_scopes(collector, name, class);
            }
            collector.push_function_scope(Some(name.to_string()));
            if collector.export_depth > 0 {
                collector.exported_functions.insert(name.to_string());
            }
            collector.callable_scopes.insert(name.to_string());
            collector.class_scopes.insert(name.to_string());
            walk::walk_class(collector, class);
            collector.pop_function_scope(true);
            return;
        }
    }
    walk::walk_class(collector, class);
}

fn visit_export_default_declaration_with_scope<'a>(
    collector: &mut ImportCollector,
    export: &ExportDefaultDeclaration<'a>,
) {
    if let ExportDefaultDeclarationKind::Identifier(identifier) = &export.declaration {
        collector.exported_functions.insert(identifier.name.to_string());
    }
    collector.export_depth += 1;
    match &export.declaration {
        ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
            let scope = function
                .id
                .as_ref()
                .map_or("default", |id| id.name.as_str());
            walk_default_function_with_scope(collector, function, scope);
            collector.export_depth -= 1;
        }
        ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow) => {
            collector.push_function_scope(Some("default".to_string()));
            collector.exported_functions.insert("default".to_string());
            collector.callable_scopes.insert("default".to_string());
            collector.add_type_parameter_names(arrow.type_parameters.as_deref());
            collector.add_formal_parameters(&arrow.params);
            walk::walk_arrow_function_expression(collector, arrow);
            collector.pop_function_scope(true);
            collector.export_depth -= 1;
        }
        ExportDefaultDeclarationKind::FunctionExpression(function) => {
            walk_default_function_with_scope(collector, function, "default");
            collector.export_depth -= 1;
        }
        ExportDefaultDeclarationKind::ClassDeclaration(class) => {
            let scope = class
                .id
                .as_ref()
                .map_or_else(|| "default".to_string(), |id| id.name.to_string());
            record_class_member_calls(collector, &scope, class);
            collector.record_exported_resource_root(&scope);
            record_class_resource_scopes(collector, &scope, class);
            collector.push_function_scope(Some(scope.clone()));
            collector.exported_functions.insert(scope.clone());
            collector.callable_scopes.insert(scope);
            if let Some(scope) = collector.current_function() {
                collector.class_scopes.insert(scope);
            }
            walk::walk_class(collector, class);
            collector.pop_function_scope(true);
            collector.export_depth -= 1;
        }
        _ => {
            walk_default_expression(collector, export);
            collector.export_depth -= 1;
        }
    }
}

fn visit_exported_enum_declaration<'a>(
    collector: &mut ImportCollector,
    declaration: &TSEnumDeclaration<'a>,
) {
    let scope = declaration.id.name.to_string();
    collector.push_function_scope(Some(scope.clone()));
    collector.exported_functions.insert(scope.clone());
    collector.exported_type_scopes.insert(scope);
    walk::walk_ts_enum_declaration(collector, declaration);
    collector.pop_function_scope(true);
}

fn record_class_member_calls(collector: &mut ImportCollector, class_name: &str, class: &Class<'_>) {
    for element in &class.body.body {
        if let ClassElement::MethodDefinition(method) = element {
            record_member_call(
                collector,
                class_name,
                crate::codebase::ts_source::static_property_key_name(&method.key),
            );
        }
    }
}

fn record_object_member_calls(
    collector: &mut ImportCollector,
    object_name: &str,
    object: &ObjectExpression<'_>,
) {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if matches!(
            property.value,
            Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
        ) {
            record_member_call(
                collector,
                object_name,
                crate::codebase::ts_source::static_property_key_name(&property.key),
            );
        }
    }
}

fn record_member_call(collector: &mut ImportCollector, parent: &str, name: Option<&str>) {
    if let Some(name) = name {
        collector.function_calls.push(FunctionCall {
            caller: Some(parent.to_string()),
            callee: name.to_string(),
            static_arg: None,
            static_cwd: None,
        });
    }
}
