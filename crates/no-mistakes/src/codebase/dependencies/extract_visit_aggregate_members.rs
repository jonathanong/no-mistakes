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

fn record_class_constructor_call(
    collector: &mut ImportCollector,
    class_name: &str,
    class: &Class<'_>,
) {
    let Some(constructor) = class.body.body.iter().find_map(|element| {
        let ClassElement::MethodDefinition(method) = element else {
            return None;
        };
        (crate::codebase::ts_source::static_property_key_name(&method.key)
            == Some("constructor"))
        .then_some(method.as_ref())
    }) else {
        return;
    };
    if !collector.collect_call_reachability || collector.suppress_call_reachability {
        return;
    }
    let scope = format!("{class_name}/constructor");
    collector.call_reachability.push(CallReachabilityFact {
        caller: Some(class_name.to_string()),
        callee: "constructor".to_string(),
        binding: CallBinding::Local { scope },
        line: import_line_at(&collector.line_starts, constructor.span.start as usize),
        invocation_kind: "construct",
    });
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
