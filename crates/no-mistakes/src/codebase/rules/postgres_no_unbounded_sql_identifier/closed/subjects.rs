use super::*;

pub(super) fn guard_subject_holds(
    flow: &ValueFlowFacts,
    bindings: &Bindings<'_>,
    file: &std::path::Path,
    name: &str,
    scope: usize,
    offset: u32,
    guard: &FiniteIdentifierGuard,
) -> bool {
    match guard.subject {
        Some(GuardSubject::Binding {
            scope: guard_scope,
            declared_at,
        }) => bindings
            .value(file, scope, name, offset)
            .is_some_and(|binding| {
                binding.scope == guard_scope && binding.declared_at == declared_at
            }),
        Some(GuardSubject::Parameter {
            scope: guard_scope,
            owner,
            index,
        }) => {
            scope == guard_scope
                && bindings.value(file, scope, name, offset).is_none()
                && flow.parameters.iter().any(|parameter| {
                    parameter.owner == owner
                        && parameter.index == index
                        && parameter.scope == guard_scope
                        && parameter.name == name
                })
        }
        None => false,
    }
}

pub(super) fn written_since(
    flow: &ValueFlowFacts,
    guard: &FiniteIdentifierGuard,
    use_at: u32,
) -> bool {
    let from = match guard.kind {
        GuardKind::Assertion { call, .. } => call,
        _ => guard.from,
    };
    let scope = match guard.subject {
        Some(GuardSubject::Binding { scope, .. } | GuardSubject::Parameter { scope, .. }) => scope,
        None => return false,
    };
    if (!guard.members.is_empty() || matches!(guard.kind, GuardKind::Lookup))
        && flow
            .registry_mutations
            .iter()
            .any(|mutation| mutation.scope == scope && mutation.name == guard.name)
    {
        return true;
    }
    flow.identifier_writes.iter().any(|write| {
        write.scope == scope
            && write.name == guard.name
            && write.at > from
            && write.at < use_at
            && write
                .members
                .iter()
                .zip(&guard.members)
                .all(|(written, guarded)| written.as_ref().is_none_or(|written| written == guarded))
    })
}
