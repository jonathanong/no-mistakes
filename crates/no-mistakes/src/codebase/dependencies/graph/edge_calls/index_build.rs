#[derive(Clone)]
struct IndexedAlias {
    target: String,
    declared_at: u32,
    invalidated_at: Option<u32>,
}
#[derive(Clone)]
struct IndexedCallableAssignment {
    scope: String,
    id: crate::codebase::dependencies::extract::CallableId,
    completed_at: u32,
}

fn index_class_members_by_id(
    members: &[(
        crate::codebase::dependencies::extract::CallableId,
        String,
        crate::codebase::dependencies::extract::CallableId,
    )],
) -> FxHashMap<
    crate::codebase::dependencies::extract::CallableId,
    FxHashMap<String, crate::codebase::dependencies::extract::CallableId>,
> {
    let mut by_class = fx_map();
    for (class_id, member, member_id) in members {
        by_class
            .entry(*class_id)
            .or_insert_with(fx_map)
            .insert(member.clone(), *member_id);
    }
    by_class
}

fn index_local_construct_bases(
    calls: &[crate::codebase::dependencies::extract::FunctionCall],
) -> FxHashMap<crate::codebase::dependencies::extract::CallableId, String> {
    let mut bases = fx_map();
    for call in calls {
        if !(call.is_callback
            && call.invocation == crate::codebase::dependencies::extract::InvocationKind::Construct
            && call.target_identity
                == crate::codebase::dependencies::extract::CallTargetIdentity::RepositoryFunction)
        {
            continue;
        }
        if let Some(id) = call.caller_id {
            bases.entry(id).or_insert_with(|| call.callee.clone());
        }
    }
    bases
}

fn index_callable_aliases(
    aliases: &[crate::codebase::dependencies::extract::CallableAlias],
) -> ScopedNameMap<IndexedAlias> {
    index_scoped_names(aliases.iter().map(|alias| {
        (
            (alias.binding_scope, alias.local.clone()),
            IndexedAlias {
                target: alias.target.clone(),
                declared_at: alias.declared_at,
                invalidated_at: alias.invalidated_at,
            },
        )
    }))
}

fn index_closed_callable_assignments(
    file: &crate::codebase::ts_source::facts::TsFileFacts,
) -> ScopedNameMap<IndexedCallableAssignment> {
    if file.value_flow.callable_assignments().is_empty() {
        return fx_map();
    }
    let Some(replacements) = file.value_flow.replacements.as_ref()
        .filter(|facts| facts.requires_receiver_proof()) else { return fx_map(); };
    let mut grouped: FxHashMap<(usize, String), Vec<&crate::codebase::dependencies::extract::CallableAssignment>> = fx_map();
    for write in file.value_flow.callable_assignments() {
        grouped.entry((write.scope, write.name.clone())).or_default().push(write);
    }
    let scopes: FxHashMap<_, _> = file.callable_scope_ids.iter().cloned().collect();
    index_scoped_names(grouped.into_iter().filter_map(|((scope, name), writes)| {
        let [write] = writes.as_slice() else { return None; };
        let id = write.callable?;
        if scope != 0 || write.owner.is_some() || !write.guaranteed
            || file.value_flow.callable_assignment_is_used(scope, &name)
            || file.exported_bindings.iter().any(|binding| binding.local == name)
            || file.callable_bindings.iter().any(|(binding_scope, binding, bound_id)| {
                *bound_id == id && file.value_flow.callable_value_uses.as_ref()
                    .and_then(|uses| uses.get(binding_scope))
                    .is_some_and(|uses| uses.contains(binding))
            })
            || file.function_calls.iter().any(|call| call.callee_binding_scope == Some(scope)
                && call.callee == name && call.caller_id.is_some())
            || !write.region_start.is_some_and(|start| replacements.regions.iter()
                .any(|region| region.start == start && region.owner.is_none() && region.may_repeat))
        { return None; }
        Some(((scope, name), IndexedCallableAssignment {
            scope: scopes.get(&id)?.clone(), id, completed_at: write.completed_at,
        }))
    }))
}

fn index_binding_declared_at(offsets: &[(usize, String, u32)]) -> ScopedNameMap<u32> {
    index_scoped_names(
        offsets
            .iter()
            .map(|(scope, name, offset)| ((*scope, name.clone()), *offset)),
    )
}

fn index_scope_ids_by_display(
    ids: &[(crate::codebase::dependencies::extract::CallableId, String)],
) -> FxHashMap<String, Vec<crate::codebase::dependencies::extract::CallableId>> {
    let mut by_display: FxHashMap<String, Vec<crate::codebase::dependencies::extract::CallableId>> =
        fx_map();
    for (id, scope) in ids {
        by_display.entry(scope.clone()).or_default().push(*id);
    }
    by_display
}
