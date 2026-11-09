fn function_keys(site: &ResolvedCallSite) -> impl Iterator<Item = FunctionKey> + '_ {
    site.target_nodes().filter_map(move |node| {
        if site.invocation == InvocationKind::Membership {
            return None;
        }
        match node {
            NodeId::Symbol {
                file,
                callable_id: Some(id),
                ..
            } => Some((file.to_path_buf(), *id)),
            _ => None,
        }
    })
}
fn site_argument(
    site: &ResolvedCallSite,
    indexes: &CallableResolutionIndexes,
    facts: &dyn TsFactLookup,
) -> ParameterInput {
    ParameterInput::from_site(site, indexes, facts)
}

fn site_source(inputs: &GraphEdgeBuildInputs<'_>, site: &ResolvedCallSite) -> NodeId {
    site.caller.as_deref().map_or_else(
        || NodeId::file_in(&inputs.interner, &site.file),
        |caller| NodeId::scoped_in(&inputs.interner, &site.file, caller, site.caller_id),
    )
}
