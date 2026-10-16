use super::*;

#[path = "worklist.rs"]
mod worklist;
pub(in super::super) use worklist::refine_value_calls;
include!("refine_incoming.rs");
fn refine_site(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
    base: &ResolvedCallSite,
    data: ValueInputs<'_>,
    parameters_enabled: bool,
) -> (
    ResolvedCallSite,
    FxHashSet<(std::path::PathBuf, CallableId)>,
) {
    #[cfg(all(test, feature = "test-instrumentation"))]
    if data.cache.site_storage.original() {
        data.cache.site_storage.vector();
        let (mut sites, used) = vector_oracle::refine_iteration(
            inputs,
            facts,
            resolver,
            indexes,
            std::slice::from_ref(base),
            data,
            parameters_enabled,
        );
        return (sites.remove(0), used);
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
        calls_visiting: fx_set(),
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
    let mut result = base.clone();
    for site in std::iter::once(&mut result) {
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
    (result, flow.used_parameters)
}

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "refine_vector_tests.rs"]
mod vector_oracle;

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "refinement_boundaries_tests.rs"]
mod boundary_tests;
