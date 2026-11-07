use super::*;

pub(super) fn caller_argument(
    bindings: &Bindings<'_>,
    file: &Path,
    call: &ValueCall,
    index: usize,
    members: &[String],
    tags: &[TrustedSqlTag],
    state: &mut ProofState,
) -> bool {
    let origins = state.origins.clone();
    call.arguments.get(index).is_none_or(|value| {
        let origin = origins
            .as_ref()
            .and_then(|origins| origins.argument((call.offset, call.end), index));
        ordinary(bindings, file, value, origin, members, tags, state)
    })
}

fn ordinary(
    bindings: &Bindings<'_>,
    file: &Path,
    value: &ValueExpression,
    origin: Option<&TextOrigin>,
    members: &[String],
    tags: &[TrustedSqlTag],
    state: &mut ProofState,
) -> bool {
    match value {
        ValueExpression::Unknown => origin.is_some_and(|origin| origin.literal),
        ValueExpression::Undefined | ValueExpression::Null => true,
        ValueExpression::Derived(parts) if members.is_empty() => {
            parts.iter().enumerate().all(|(index, part)| {
                ordinary(
                    bindings,
                    file,
                    part,
                    origin
                        .and_then(|origin| origin.parts.get(index))
                        .and_then(Option::as_ref),
                    &[],
                    tags,
                    state,
                )
            })
        }
        ValueExpression::Awaited(inner) => {
            ordinary(bindings, file, inner, origin, members, tags, state)
        }
        ValueExpression::Either(left, right)
        | ValueExpression::Default(left, right)
        | ValueExpression::Nullish(left, right) => {
            ordinary(
                bindings,
                file,
                left,
                origin
                    .and_then(|origin| origin.parts.first())
                    .and_then(Option::as_ref),
                members,
                tags,
                state,
            ) && ordinary(
                bindings,
                file,
                right,
                origin
                    .and_then(|origin| origin.parts.get(1))
                    .and_then(Option::as_ref),
                members,
                tags,
                state,
            )
        }
        ValueExpression::Member(inner, key) => {
            let mut path = vec![key.clone()];
            path.extend_from_slice(members);
            ordinary(bindings, file, inner, origin, &path, tags, state)
        }
        ValueExpression::Object(fields, _) | ValueExpression::Array(fields)
            if !members.is_empty() =>
        {
            fields.get(&members[0]).is_some_and(|value| {
                ordinary(
                    bindings,
                    file,
                    value,
                    origin.and_then(|origin| origin.fields.get(&members[0])),
                    &members[1..],
                    tags,
                    state,
                )
            })
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
            (scope.unwrap_or(0), name, *offset),
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
    reference: (usize, &str, u32),
    members: &[String],
    tags: &[TrustedSqlTag],
    state: &mut ProofState,
) -> bool {
    let (scope, name, offset) = reference;
    let Some(ts) = bindings.facts.get_ts_facts(file) else {
        return false;
    };
    if let Some(binding) = bindings.value(file, scope, name, offset) {
        let origins = state.origins.clone();
        let origin = origins
            .as_ref()
            .and_then(|origins| origins.binding(scope, name, binding.declared_at));
        return origin_memoized(
            state,
            ProofKey::Binding(
                scope,
                name.to_string(),
                binding.declared_at,
                members.to_vec(),
            ),
            |state| {
                binding.immutable
                    && (!matches!(
                        binding.value,
                        ValueExpression::Call(_, _)
                            | ValueExpression::Object(_, _)
                            | ValueExpression::Array(_)
                    ) || !ts
                        .value_flow
                        .registry_mutations
                        .iter()
                        .any(|mutation| mutation.name == name && mutation.scope == scope))
                    && ordinary(bindings, file, &binding.value, origin, members, tags, state)
            },
        );
    }
    if ts
        .value_flow
        .bindings
        .iter()
        .any(|binding| binding.scope == scope && binding.name == name)
        || ts
            .value_flow
            .identifier_writes
            .iter()
            .any(|write| write.scope == scope && write.name == name)
    {
        return false;
    }
    ts.value_flow
        .parameters
        .iter()
        .enumerate()
        .find(|(_, parameter)| parameter.scope == scope && parameter.name == name)
        .is_none_or(|(index, parameter)| {
            parameter.invalidated_at.is_none_or(|write| write > offset)
                && parameter_bound(bindings, file, parameter, index, members, tags, state)
        })
}
