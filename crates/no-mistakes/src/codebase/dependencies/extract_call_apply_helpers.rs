fn normalize_function_prototype_call_apply(
    collector: &ImportCollector,
    mut callee: String,
) -> String {
    while let Some(receiver) = function_prototype_call_apply_receiver(collector, &callee) {
        callee = receiver;
    }
    callee
}

fn function_prototype_call_apply_receiver(
    collector: &ImportCollector,
    callee: &str,
) -> Option<String> {
    let (receiver, method) = callee.rsplit_once('.')?;
    if method != "call" && method != "apply" {
        return None;
    }
    if !is_function_prototype_receiver(collector, receiver) {
        return None;
    }
    if receiver_owns_member(collector, receiver, method) {
        return None;
    }
    Some(receiver.to_string())
}

fn is_function_prototype_receiver(collector: &ImportCollector, receiver: &str) -> bool {
    if receiver == "this" || receiver.contains("<unknown>") {
        return false;
    }
    let binding = receiver
        .split_once('.')
        .map_or(receiver, |(binding, _)| binding);
    if collector.imported_bindings.contains(binding)
        || collector.predeclared_imported_bindings.contains(binding)
    {
        return true;
    }
    let Some(scope) = collector.callee_binding_scope(receiver) else {
        return false;
    };
    collector.class_id_for_binding(scope, binding).is_none() || receiver.contains('.')
}

fn receiver_owns_member(collector: &ImportCollector, receiver: &str, method: &str) -> bool {
    let Some(scope) = collector.callee_binding_scope(receiver) else {
        return false;
    };
    if collector
        .indexed_callable_alias(scope, &format!("{receiver}.{method}"))
        .is_some()
    {
        return true;
    }
    let Some(id) = receiver_callable_id(collector, scope, receiver) else {
        return false;
    };
    owner_member_id(&collector.aggregate_callable_member_ids, id, method).is_some()
        || collector.has_object_getter_member(id, method)
        || collector.has_object_setter_member(id, method)
}

fn receiver_callable_id(
    collector: &ImportCollector,
    scope: usize,
    receiver: &str,
) -> Option<CallableId> {
    let mut parts = receiver.split('.');
    let first = parts.next()?;
    let mut id = collector
        .callable_binding_at(scope, first)
        .or_else(|| collector.class_id_for_binding(scope, first))?;
    for member in parts {
        id = owner_member_id(&collector.aggregate_callable_member_ids, id, member)
            .or_else(|| collector.class_member_id(id, member))?;
    }
    Some(id)
}
