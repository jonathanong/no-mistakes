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
            && call.invocation
                == crate::codebase::dependencies::extract::InvocationKind::Construct
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
) -> FxHashMap<(usize, String), IndexedAlias> {
    aliases
        .iter()
        .map(|alias| {
            (
                (alias.binding_scope, alias.local.clone()),
                IndexedAlias {
                    target: alias.target.clone(),
                    declared_at: alias.declared_at,
                    invalidated_at: alias.invalidated_at,
                },
            )
        })
        .collect()
}

fn index_binding_declared_at(
    offsets: &[(usize, String, u32)],
) -> FxHashMap<(usize, String), u32> {
    offsets
        .iter()
        .map(|(scope, name, offset)| ((*scope, name.clone()), *offset))
        .collect()
}
