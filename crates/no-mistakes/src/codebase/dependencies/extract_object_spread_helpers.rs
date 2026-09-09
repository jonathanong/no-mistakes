fn recorded_call_callee(
    collector: &ImportCollector,
    call: &CallExpression<'_>,
) -> Option<String> {
    object_literal_spread_call_callee(collector, call)
        .or_else(|| simple_callee_name(&call.callee))
}

fn object_literal_spread_call_callee(
    collector: &ImportCollector,
    call: &CallExpression<'_>,
) -> Option<String> {
    let Expression::StaticMemberExpression(member) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee)
    else {
        return None;
    };
    let Expression::ObjectExpression(object) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&member.object)
    else {
        return None;
    };
    object_literal_member_target(collector, object, member.property.name.as_str())
}

fn object_literal_member_target(
    collector: &ImportCollector,
    object: &ObjectExpression<'_>,
    member: &str,
) -> Option<String> {
    let mut target = None;
    for property in &object.properties {
        match property {
            ObjectPropertyKind::SpreadProperty(spread) => {
                let (source, source_id) =
                    known_object_spread_source(collector, &spread.argument)?;
                if object_source_has_member(collector, source_id, &source, member) {
                    target = Some(format!("{source}.{member}"));
                }
            }
            ObjectPropertyKind::ObjectProperty(property) => {
                if crate::codebase::ts_source::static_property_key_name(&property.key) != Some(member)
                {
                    continue;
                }
                target = collector.callable_alias_target(&property.value);
            }
        }
    }
    target
}

fn known_object_spread_source(
    collector: &ImportCollector,
    argument: &Expression<'_>,
) -> Option<(String, CallableId)> {
    let Expression::Identifier(identifier) =
        crate::codebase::ts_source::unwrap_ts_wrappers(argument)
    else {
        return None;
    };
    let name = identifier.name.as_str();
    let binding_scope = collector.callee_binding_scope(name)?;
    if collector.has_reassigned_callable_at(binding_scope, name)
        || collector.class_id_for_binding(binding_scope, name).is_some()
    {
        return None;
    }
    let object_id = collector.callable_binding_at(binding_scope, name)?;
    Some((name.to_string(), object_id))
}

fn object_source_has_member(
    collector: &ImportCollector,
    source_id: CallableId,
    source: &str,
    member: &str,
) -> bool {
    owner_member_id(&collector.aggregate_callable_member_ids, source_id, member).is_some()
        || collector.has_object_getter_member(source_id, member)
        || collector.has_object_setter_member(source_id, member)
        || collector
            .callee_binding_scope(source)
            .and_then(|scope| collector.indexed_callable_alias(scope, &format!("{source}.{member}")))
            .is_some()
}

fn record_object_spread_property(
    collector: &mut ImportCollector,
    _object_binding: &str,
    object_scope: &str,
    object_id: CallableId,
    property: &ObjectPropertyKind<'_>,
) -> bool {
    let ObjectPropertyKind::SpreadProperty(spread) = property else {
        return false;
    };
    copy_known_object_spread(collector, object_scope, object_id, &spread.argument);
    true
}

fn copy_known_object_spread(
    collector: &mut ImportCollector,
    object_scope: &str,
    object_id: CallableId,
    argument: &Expression<'_>,
) -> bool {
    let Some((_, source_id)) = known_object_spread_source(collector, argument) else {
        collector.aggregate_callable_member_ids.remove(&object_id);
        collector.object_getter_member_ids.remove(&object_id);
        collector.object_setter_member_ids.remove(&object_id);
        return false;
    };
    let members = collector
        .aggregate_callable_member_ids
        .get(&source_id)
        .into_iter()
        .flatten()
        .map(|(member, id)| (member.clone(), *id))
        .collect::<Vec<_>>();
    for (member, member_id) in members {
        collector.record_aggregate_callable_member_id(object_id, &member, member_id);
        record_member_call(collector, object_scope, object_id, Some(&member));
    }
    if let Some(getters) = collector.object_getter_member_ids.get(&source_id).cloned() {
        for getter in getters {
            collector.insert_object_getter_member(object_id, &getter);
        }
    }
    if let Some(setters) = collector.object_setter_member_ids.get(&source_id).cloned() {
        for setter in setters {
            collector.insert_object_setter_member(object_id, &setter);
        }
    }
    true
}

fn spread_member_aliases(
    collector: &ImportCollector,
    argument: &Expression<'_>,
) -> Option<Vec<(String, String)>> {
    let (source, source_id) = known_object_spread_source(collector, argument)?;
    let mut members = collector
        .aggregate_callable_member_ids
        .get(&source_id)
        .into_iter()
        .flatten()
        .map(|(member, _)| member.clone())
        .collect::<FxHashSet<_>>();
    let prefix = format!("{source}.");
    members.extend(collector.callable_aliases.iter().filter_map(|alias| {
        alias
            .alias
            .local
            .strip_prefix(&prefix)
            .map(str::to_string)
    }));
    Some(
        members
            .into_iter()
            .map(|member| (member.clone(), format!("{source}.{member}")))
            .collect(),
    )
}
