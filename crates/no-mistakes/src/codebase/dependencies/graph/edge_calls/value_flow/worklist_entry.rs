type FunctionKey = (std::path::PathBuf, CallableId);

pub(in super::super::super) struct WorklistInputs<'a> {
    pub(in super::super::super) incoming: &'a mut ParameterInputs,
    pub(in super::super::super) replacement_pass: Option<&'a CallPass>,
}

pub(in super::super::super) fn refine_value_calls(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
    pass: &mut CallPass,
) {
    #[cfg(all(test, feature = "test-instrumentation"))]
    if pass.parameter_cache.site_storage.cloning() || pass.parameter_cache.site_storage.original() {
        return owned_worklist::refine_value_calls(inputs, facts, resolver, indexes, pass);
    }
    let mut incoming = incoming_inputs(&pass.sites, &pass.constructor_routes, indexes, facts);
    refine_value_calls_with_incoming(
        inputs,
        facts,
        resolver,
        indexes,
        pass,
        WorklistInputs {
            incoming: &mut incoming,
            replacement_pass: None,
        },
        |_| Vec::new(),
    );
}
