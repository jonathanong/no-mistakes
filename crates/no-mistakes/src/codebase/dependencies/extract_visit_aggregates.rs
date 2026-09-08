fn visit_class_with_scope<'a>(collector: &mut ImportCollector, class: &Class<'a>) {
    if let Some(name) = class.id.as_ref().map(|id| id.name.as_str()) {
        let scope = collector.callable_scope_name(name);
        let class_id = CallableId(class.span.start);
        collector.record_callable_binding_id(name, class_id);
        record_class_member_calls(collector, &scope, class_id, class);
        if collector.current_function().is_none() && collector.is_exported_top_level_name(name) {
            collector.record_exported_resource_root(name);
            record_class_resource_scopes(collector, name, class);
        }
        if collector.export_depth > 0 && collector.current_function().is_none() {
            collector.exported_functions.insert(scope.clone());
            collector.record_local_export_binding(name, &scope);
        }
        collector.callable_scopes.insert(scope.clone());
        collector.class_scopes.insert(scope.clone());
        walk_class_with_scoped_methods(collector, name, CallableId(class.span.start), class);
        return;
    }
    record_decorator_invocations(collector, &class.decorators);
    walk::walk_class(collector, class);
}

fn visit_export_default_declaration_with_scope<'a>(
    collector: &mut ImportCollector,
    export: &ExportDefaultDeclaration<'a>,
) {
    let default_local = match &export.declaration {
        ExportDefaultDeclarationKind::Identifier(identifier) => Some(identifier.name.to_string()),
        ExportDefaultDeclarationKind::FunctionDeclaration(function) => function_name(function),
        ExportDefaultDeclarationKind::ClassDeclaration(class) => class
            .id
            .as_ref()
            .map(|identifier| identifier.name.to_string()),
        _ => None,
    }
    .unwrap_or_else(|| "default".to_string());
    collector.record_local_export_binding(&default_local, "default");
    if matches!(
        &export.declaration,
        ExportDefaultDeclarationKind::Identifier(_)
    ) {
        collector.exported_functions.insert(default_local);
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
            collector.push_function_scope(Some("default".to_string()), CallableId(arrow.span.start));
            collector.exported_functions.insert("default".to_string());
            collector.callable_scopes.insert("default".to_string());
            collector.add_type_parameter_names(arrow.type_parameters.as_deref());
            collector.add_formal_parameters(&arrow.params);
            walk_arrow_function_with_body_bindings(collector, arrow);
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
            record_class_member_calls(collector, &scope, CallableId(class.span.start), class);
            collector.record_exported_resource_root(&scope);
            record_class_resource_scopes(collector, &scope, class);
            collector.exported_functions.insert(scope.clone());
            collector.callable_scopes.insert(scope.clone());
            collector.class_scopes.insert(scope.clone());
            walk_class_with_scoped_methods(collector, &scope, CallableId(class.span.start), class);
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
    collector.push_function_scope(Some(scope.clone()), CallableId(declaration.span.start));
    collector.exported_functions.insert(scope.clone());
    collector.exported_type_scopes.insert(scope);
    walk::walk_ts_enum_declaration(collector, declaration);
    collector.pop_function_scope(true);
}

fn record_class_member_calls(
    collector: &mut ImportCollector,
    class_name: &str,
    class_id: CallableId,
    class: &Class<'_>,
) {
    for element in &class.body.body {
        match element {
            ClassElement::MethodDefinition(method) => {
                let name = crate::codebase::ts_source::static_property_key_name(&method.key);
                if method.r#static || name == Some("constructor") {
                    record_member_call(collector, class_name, class_id, name);
                }
            }
            ClassElement::PropertyDefinition(property)
                if property.r#static
                    && matches!(
                        property.value,
                        Some(
                            Expression::FunctionExpression(_)
                                | Expression::ArrowFunctionExpression(_)
                        )
                    ) =>
            {
                record_member_call(
                    collector,
                    class_name,
                    class_id,
                    crate::codebase::ts_source::static_property_key_name(&property.key),
                );
            }
            _ => {}
        }
    }
}

fn record_object_member_calls(
    collector: &mut ImportCollector,
    object_binding: &str,
    object_scope: &str,
    object_id: CallableId,
    object: &ObjectExpression<'_>,
) {
    collector.record_callable_binding_id(object_binding, object_id);
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
                object_scope,
                object_id,
                crate::codebase::ts_source::static_property_key_name(&property.key),
            );
        }
    }
}

fn record_member_call(
    collector: &mut ImportCollector,
    parent: &str,
    parent_id: CallableId,
    name: Option<&str>,
) {
    if let Some(name) = name {
        collector.function_calls.push(FunctionCall {
            caller: Some(parent.to_string()),
            caller_id: Some(parent_id),
            syntactic_caller: collector.current_syntactic_caller(),
            callee: name.to_string(),
            line: 0,
            offset: 0,
            is_callback: true,
            invocation: InvocationKind::Membership,
            target_identity: CallTargetIdentity::RepositoryFunction,
            callee_binding_scope: None,
            static_arg: None,
            static_cwd: None,
        });
    }
}
