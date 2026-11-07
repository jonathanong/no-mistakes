use super::*;

pub(super) fn bound_argument(
    bindings: &Bindings<'_>,
    file: &Path,
    value: &ValueExpression,
    members: &[String],
    tags: &[TrustedSqlTag],
    state: &mut ProofState,
) -> bool {
    match value {
        ValueExpression::Undefined | ValueExpression::Null => members.is_empty(),
        ValueExpression::Derived(parts) if members.is_empty() => parts
            .iter()
            .all(|part| bound_argument(bindings, file, part, &[], tags, state)),
        ValueExpression::Awaited(inner) => {
            bound_argument(bindings, file, inner, members, tags, state)
        }
        ValueExpression::Nullish(left, right)
        | ValueExpression::Default(left, right)
        | ValueExpression::Either(left, right) => {
            bound_argument(bindings, file, left, members, tags, state)
                && bound_argument(bindings, file, right, members, tags, state)
        }
        ValueExpression::Call(start, end) if members.is_empty() => {
            value_call_at(bindings.facts, file, *start, *end)
                .is_some_and(|call| text_tag(bindings, file, call, tags, state))
        }
        ValueExpression::Member(inner, key) => {
            let mut path = vec![key.clone()];
            path.extend_from_slice(members);
            bound_argument(bindings, file, inner, &path, tags, state)
        }
        ValueExpression::Object(fields, _) if !members.is_empty() => fields
            .get(&members[0])
            .is_some_and(|value| bound_argument(bindings, file, value, &members[1..], tags, state)),
        ValueExpression::Named {
            name,
            scope,
            offset,
            ..
        } => {
            if super::super::env::alias_env_from_chain(bindings.facts, file, value, members)
                .is_some()
            {
                return true;
            }
            named_argument(
                bindings,
                file,
                scope.unwrap_or(0),
                name,
                *offset,
                members,
                tags,
                state,
            )
        }
        _ => false,
    }
}

fn named_argument(
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
        let key = ProofKey::Binding(scope, name.into(), binding.declared_at, members.to_vec());
        return memoized(state, key, |state| {
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
                    .any(|mutation| mutation.scope == scope && mutation.name == name))
                && bound_argument(bindings, file, &binding.value, members, tags, state)
        });
    }
    !ts.value_flow
        .bindings
        .iter()
        .any(|binding| binding.scope == scope && binding.name == name)
        && ts
            .value_flow
            .parameters
            .iter()
            .enumerate()
            .any(|(index, parameter)| {
                parameter.scope == scope
                    && parameter.name == name
                    && !state.opaque_parameter_defaults.contains(&index)
                    && parameter.invalidated_at.is_none_or(|write| write > offset)
                    && memoized(
                        state,
                        ProofKey::Parameter(scope, name.into(), members.to_vec()),
                        |state| {
                            super::callers::parameter_bound(
                                bindings, file, parameter, index, members, tags, state,
                            )
                        },
                    )
            })
}

pub(super) fn parameter_defaults(
    bindings: &Bindings<'_>,
    file: &Path,
    parameter: &ValueParameter,
    members: &[String],
    tags: &[TrustedSqlTag],
    state: &mut ProofState,
) -> bool {
    let path = |after| {
        parameter.projection[after..]
            .iter()
            .filter_map(|step| match step {
                ValueProjection::Member(member) | ValueProjection::Element(member) => {
                    Some(member.clone())
                }
                ValueProjection::Default(_, _) => None,
            })
            .chain(members.iter().cloned())
            .collect::<Vec<_>>()
    };
    (matches!(parameter.default, ValueExpression::Unknown)
        || bound_argument(bindings, file, &parameter.default, &path(0), tags, state))
        && parameter
            .projection
            .iter()
            .enumerate()
            .all(|(index, step)| match step {
                ValueProjection::Default(value, _) => {
                    matches!(value, ValueExpression::Unknown)
                        || bound_argument(bindings, file, value, &path(index + 1), tags, state)
                }
                _ => true,
            })
}
