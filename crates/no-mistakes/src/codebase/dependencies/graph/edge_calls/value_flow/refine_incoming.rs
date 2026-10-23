pub(super) fn incoming_inputs<'facts>(
    sites: &[ResolvedCallSite],
    constructors: &ConstructorRoutes,
    indexes: &CallableResolutionIndexes<'facts>,
    facts: &'facts dyn TsFactLookup,
) -> ParameterInputs {
    let mut incoming: ParameterInputs = fx_map();
    for site in sites {
        if site.invocation == InvocationKind::Membership {
            continue;
        }
        for target in site.target_nodes() {
            let target = if site.invocation == InvocationKind::Construct {
                constructors::route(constructors, target).unwrap_or(target)
            } else {
                target
            };
            if let NodeId::Symbol {
                file,
                callable_id: Some(id),
                ..
            } = target
            {
                incoming
                    .entry((file.to_path_buf(), *id))
                    .or_default()
                    .push(ParameterInput::from_site(site, indexes, facts));
            }
        }
    }
    incoming
}
