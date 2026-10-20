use super::*;

impl ImportCollector {
    pub(super) fn resolve_class_method(
        &mut self,
        call: &CallExpression<'_>,
    ) -> Option<(CallableId, Vec<ValueExpression>)> {
        if let Some(resolved) = prototype_call(self, call) {
            return Some(resolved);
        }
        let (object, method) = method_name(&call.callee)?;
        let (class_id, scope) = class_ref(self, object)?;
        let method = member_on_class(self, class_id, &method, scope)?;
        Some((method, call_arguments(self, call, 0)))
    }
}

fn prototype_call(
    collector: &ImportCollector,
    call: &CallExpression<'_>,
) -> Option<(CallableId, Vec<ValueExpression>)> {
    let Expression::StaticMemberExpression(member) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee)
    else {
        return None;
    };
    if member.property.name != "call" {
        return None;
    }
    let method = method_callable(collector, &member.object)?;
    Some((method, call_arguments(collector, call, 1)))
}

fn method_name<'a>(callee: &'a Expression<'a>) -> Option<(&'a Expression<'a>, String)> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(callee) {
        Expression::StaticMemberExpression(member) => {
            Some((&member.object, member.property.name.to_string()))
        }
        Expression::ComputedMemberExpression(member) => {
            let key = super::super::write_targets::static_key(&member.expression)?;
            Some((&member.object, key))
        }
        _ => None,
    }
}

fn class_ref(collector: &ImportCollector, object: &Expression<'_>) -> Option<(CallableId, usize)> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(object) {
        Expression::ThisExpression(_) => {
            let class_id = this_class(collector)?;
            Some((class_id, collector.current_lexical_scope_id()))
        }
        Expression::NewExpression(new) => {
            let name = simple_callee_name(&new.callee)?;
            let scope = collector.callee_binding_scope(&name).unwrap_or(0);
            let class_id = collector.callable_binding_at(scope, &name)?;
            Some((class_id, scope))
        }
        Expression::Identifier(identifier) => {
            let key = binding_key(collector, identifier.name.as_str());
            let class_id = collector.receiver_escapes.instances.get(&key).copied()?;
            Some((class_id, key.0))
        }
        Expression::StaticMemberExpression(member) => {
            let base = simple_callee_name(&member.object)?;
            let (scope, holder) = binding_key(
                collector,
                base.split_once('.').map_or(base.as_str(), |(head, _)| head),
            );
            let key = format!("{holder}.{}", member.property.name.as_str());
            let class_id = collector
                .receiver_escapes
                .instances
                .get(&(scope, key))
                .copied()?;
            Some((class_id, scope))
        }
        Expression::CallExpression(call) => {
            let name = simple_callee_name(&call.callee)?;
            let scope = collector.callee_binding_scope(&name).unwrap_or(0);
            let owner = collector.callable_binding_at(scope, &name)?;
            let class_id = collector.receiver_escapes.returns.get(&owner).copied()?;
            Some((class_id, scope))
        }
        _ => None,
    }
}

fn this_class(collector: &ImportCollector) -> Option<CallableId> {
    let stack = &collector.function_id_stack;
    let position = stack
        .iter()
        .rposition(|(id, _)| !collector.receiver_escapes.arrows.contains(id))?;
    let class_id = stack.get(position.checked_sub(1)?)?.0;
    is_class(collector, class_id).then_some(class_id)
}

fn is_class(collector: &ImportCollector, id: CallableId) -> bool {
    collector
        .callable_scope_ids
        .iter()
        .any(|(candidate, scope)| *candidate == id && collector.class_scopes.contains(scope))
}

fn member_on_class(
    collector: &ImportCollector,
    class_id: CallableId,
    method: &str,
    scope: usize,
) -> Option<CallableId> {
    if let Some(found) = own_member(collector, class_id, method) {
        return Some(found);
    }
    let base = collector.class_local_bases.get(&class_id)?;
    let base_id = collector.class_id_for_binding(scope, base)?;
    own_member(collector, base_id, method)
}

fn own_member(
    collector: &ImportCollector,
    class_id: CallableId,
    method: &str,
) -> Option<CallableId> {
    owner_member_id(&collector.aggregate_callable_member_ids, class_id, method).or_else(|| {
        collector
            .receiver_escapes
            .fields
            .get(&(class_id, method.to_string()))
            .copied()
    })
}

pub(super) fn method_callable(
    collector: &ImportCollector,
    object: &Expression<'_>,
) -> Option<CallableId> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(object) {
        Expression::Identifier(identifier) => collector
            .receiver_escapes
            .method_aliases
            .get(&binding_key(collector, identifier.name.as_str()))
            .copied(),
        Expression::StaticMemberExpression(member) => {
            let (class_id, scope) = class_ref(collector, &member.object)?;
            member_on_class(collector, class_id, member.property.name.as_str(), scope)
        }
        _ => None,
    }
}

pub(super) fn class_of_new(
    collector: &ImportCollector,
    expression: &Expression<'_>,
) -> Option<CallableId> {
    let Expression::NewExpression(new) = crate::codebase::ts_source::unwrap_ts_wrappers(expression)
    else {
        return None;
    };
    let name = simple_callee_name(&new.callee)?;
    let scope = collector.callee_binding_scope(&name).unwrap_or(0);
    collector.callable_binding_at(scope, &name)
}

pub(super) fn binding_key(collector: &ImportCollector, name: &str) -> (usize, String) {
    (
        collector
            .callee_binding_scope(name)
            .unwrap_or(collector.current_lexical_scope_id()),
        name.to_string(),
    )
}

fn call_arguments(
    collector: &ImportCollector,
    call: &CallExpression<'_>,
    skip: usize,
) -> Vec<ValueExpression> {
    call.arguments
        .iter()
        .skip(skip)
        .map(|argument| {
            argument
                .as_expression()
                .map_or(ValueExpression::Unknown, |expression| {
                    collector.value_expression(expression)
                })
        })
        .collect()
}
