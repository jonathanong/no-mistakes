use super::*;

#[path = "worklist.rs"]
mod worklist;
pub(in super::super) use worklist::refine_value_calls;
include!("refine_incoming.rs");
fn refine_site_in_place<'facts>(
    inputs: &GraphEdgeBuildInputs<'facts>,
    facts: &'facts dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes<'facts>,
    result: &mut ResolvedCallSite,
    data: ValueInputs<'_>,
    parameters_enabled: bool,
) -> FxHashSet<(std::path::PathBuf, CallableId)> {
    #[cfg(all(test, feature = "test-instrumentation"))]
    if data.cache.site_storage.original() {
        data.cache.site_storage.vector();
        let (mut sites, used) = vector_oracle::refine_iteration(
            inputs,
            facts,
            resolver,
            indexes,
            std::slice::from_ref(result),
            data,
            parameters_enabled,
        );
        *result = sites.remove(0);
        return used;
    }
    #[cfg(all(test, feature = "test-instrumentation"))]
    data.cache.site_storage.scalar();
    let mut flow = ValueFlowResolution {
        inputs,
        facts,
        resolver,
        indexes,
        incoming: data.incoming,
        supers: data.supers,
        replacements: data.replacements,
        replacement_read: false,
        foreign_acquisition: false,
        used_parameters: fx_set(),
        parameters_enabled,
        initializing: Vec::new(),
        cache: data.cache,
        visiting: fx_set(),
        calls_visiting: fx_set(), fields_visiting: fx_set(),
        default_origins: None,
        acquisition_context: None,
        invocation_read: None,
        invocation_start: None,
        executing_read: None,
        module_writer_phases: data.module_writer_phases,
        writer_dependencies: Default::default(),
        module_snapshot: None,
        member_initializer: None,
        member_read: None,
    };
    for site in std::iter::once(result) {
        if site.invocation == InvocationKind::Construct {
            continue;
        }
        let Some(index) = indexes.file(facts, &site.file) else {
            continue;
        };
        let call = index
            .values
            .sites
            .get(&site.offset)
            .and_then(|sites| sites.get(site.source_callee.as_str()))
            .and_then(|key| index.values.calls.get(key));
        let Some(call) = call else {
            continue;
        };
        if matches!(site.target, ResolvedCallTarget::Unknown) {
            let mut callee = &call.callee;
            while let ValueExpression::Member(base, _) = callee {
                callee = base;
            }
            if let ValueExpression::Named {
                name,
                scope: None | Some(0),
                ..
            } = callee
            {
                let head = name.split_once('.').map_or(name.as_str(), |(head, _)| head);
                if index.imported.get(head).is_some_and(|binding| binding.kind == crate::codebase::dependencies::extract::ImportedBindingKind::Namespace) { continue; }
            }
        }
        if call.bound {
            continue;
        }
        flow.replacement_read = false;
        let value = flow.value(&site.file, &call.callee);
        super::apply_value::apply_value(inputs, facts, indexes, site, value, flow.replacement_read);
    }
    flow.merge_writer_dependencies();
    flow.used_parameters
}

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "refine_vector_tests.rs"]
mod vector_oracle;

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "refinement_boundaries_tests.rs"]
mod boundary_tests;

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "refine_owned_oracle_tests.rs"]
mod owned_oracle;
// Existing scalar boundary tests retain their original owned-return entrypoint.
#[cfg(all(test, feature = "test-instrumentation"))]
fn refine_site<'facts>(
    inputs: &GraphEdgeBuildInputs<'facts>,
    facts: &'facts dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes<'facts>,
    base: &ResolvedCallSite,
    data: ValueInputs<'_>,
    parameters_enabled: bool,
) -> (
    ResolvedCallSite,
    FxHashSet<(std::path::PathBuf, CallableId)>,
) {
    let mut site = base.clone();
    let used = refine_site_in_place(
        inputs,
        facts,
        resolver,
        indexes,
        &mut site,
        data,
        parameters_enabled,
    );
    (site, used)
}
