use super::*;

pub(super) fn ordinary(
    bindings: &Bindings<'_>,
    file: &Path,
    value: &ValueExpression,
    members: &[String],
    tags: &[TrustedSqlTag],
    state: &mut ProofState,
) -> bool {
    match value {
        ValueExpression::Unknown | ValueExpression::Undefined | ValueExpression::Null => true,
        ValueExpression::Derived(parts) if members.is_empty() => parts
            .iter()
            .all(|part| ordinary(bindings, file, part, &[], tags, state)),
        ValueExpression::Awaited(inner) => ordinary(bindings, file, inner, members, tags, state),
        ValueExpression::Either(left, right)
        | ValueExpression::Default(left, right)
        | ValueExpression::Nullish(left, right) => {
            ordinary(bindings, file, left, members, tags, state)
                && ordinary(bindings, file, right, members, tags, state)
        }
        ValueExpression::Member(inner, key) => {
            let mut path = vec![key.clone()];
            path.extend_from_slice(members);
            ordinary(bindings, file, inner, &path, tags, state)
        }
        ValueExpression::Object(fields, _) | ValueExpression::Array(fields)
            if !members.is_empty() =>
        {
            fields
                .get(&members[0])
                .is_none_or(|value| ordinary(bindings, file, value, &members[1..], tags, state))
        }
        ValueExpression::Call(start, end) if members.is_empty() => {
            value_call_at(bindings.facts, file, *start, *end)
                .is_some_and(|call| text_tag(bindings, file, call, tags, state))
        }
        ValueExpression::Named {
            name,
            scope,
            offset,
            ..
        } => named(
            bindings,
            file,
            scope.unwrap_or(0),
            name,
            *offset,
            members,
            tags,
            state,
        ),
        _ => false,
    }
}

fn named(
    bindings: &Bindings<'_>,
    file: &Path,
    scope: usize,
    name: &str,
    offset: u32,
    members: &[String],
    tags: &[TrustedSqlTag],
    state: &mut ProofState,
) -> bool {
    let Some(ts) = bindings.facts.get_ts_facts(file) else {
        return false;
    };
    if let Some(binding) = bindings.value(file, scope, name, offset) {
        return origin_memoized(
            state,
            ProofKey::Binding(
                scope,
                name.to_string(),
                binding.declared_at,
                members.to_vec(),
            ),
            |state| {
                (!matches!(
                    binding.value,
                    ValueExpression::Call(_, _)
                        | ValueExpression::Object(_, _)
                        | ValueExpression::Array(_)
                ) || !ts
                    .value_flow
                    .registry_mutations
                    .iter()
                    .any(|mutation| mutation.name == name && mutation.scope == scope))
                    && ordinary(bindings, file, &binding.value, members, tags, state)
            },
        );
    }
    ts.value_flow
        .parameters
        .iter()
        .enumerate()
        .find(|(_, parameter)| parameter.scope == scope && parameter.name == name)
        .is_none_or(|(index, parameter)| {
            parameter_bound(bindings, file, parameter, index, members, tags, state)
        })
}
