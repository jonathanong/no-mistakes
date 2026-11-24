fn record_cjs_created_require_factory(
    collector: &mut ImportCollector,
    declaration: &VariableDeclaration<'_>,
) {
    if !collector.global_require_available() {
        return;
    }
    let scope = collector.current_lexical_scope_id();
    for declarator in &declaration.declarations {
        let Some(init) = declarator.init.as_ref() else {
            continue;
        };
        let expression = init.get_inner_expression();
        if let Expression::CallExpression(call) = expression {
            if !is_module_require_call(collector, call) {
                continue;
            }
            match &declarator.id {
                BindingPattern::BindingIdentifier(local) => {
                    collector
                        .created_requires
                        .scoped_namespaces
                        .entry(scope)
                        .or_default()
                        .insert(local.name.to_string());
                }
                BindingPattern::ObjectPattern(object) => {
                    for property in &object.properties {
                        if property.computed {
                            continue;
                        }
                        let BindingPattern::BindingIdentifier(local) = &property.value else {
                            continue;
                        };
                        if crate::codebase::ts_source::static_property_key_name(&property.key)
                            .as_deref()
                            == Some("createRequire")
                        {
                            collector
                                .created_requires
                                .scoped_factories
                                .entry(scope)
                                .or_default()
                                .insert(local.name.to_string());
                        }
                    }
                }
                _ => {}
            }
            continue;
        }
        let BindingPattern::BindingIdentifier(local) = &declarator.id else {
            continue;
        };
        if let Expression::StaticMemberExpression(member) = expression {
            if !member.optional && member.property.name == "createRequire" {
                if let Expression::CallExpression(call) = member.object.get_inner_expression() {
                    if is_module_require_call(collector, call) {
                        collector
                            .created_requires
                            .scoped_factories
                            .entry(scope)
                            .or_default()
                            .insert(local.name.to_string());
                    }
                }
            }
        }
    }
}

fn is_module_require_call(collector: &ImportCollector, call: &CallExpression<'_>) -> bool {
    if call.optional
        || !collector.global_require_available()
        || !is_require_callee(call.callee.get_inner_expression())
    {
        return false;
    }
    call.arguments
        .first()
        .and_then(Argument::as_expression)
        .and_then(static_import_specifier)
        .is_some_and(|specifier| specifier == "node:module" || specifier == "module")
}
