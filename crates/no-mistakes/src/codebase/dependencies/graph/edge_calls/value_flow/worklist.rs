use super::*;

#[cfg(feature = "test-instrumentation")]
#[path = "replacement_incoming.rs"]
mod replacement_incoming;

include!("worklist_entry.rs");
#[path = "callback_parameter_cache.rs"]
mod callback_parameters;

pub(in super::super::super) fn refine_value_calls_with_incoming(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
    pass: &mut CallPass,
    data: WorklistInputs<'_>,
    mut expand: impl FnMut(&[ResolvedCallSite]) -> Vec<ResolvedCallSite>,
) {
    let incoming = data.incoming;
    let callbacks = callback_parameters::Caches::default();
    let replacements = replacement_catalog::ReplacementCatalog::prepare(
        inputs,
        facts,
        resolver,
        indexes,
        data.replacement_pass.unwrap_or(pass),
        incoming,
    );
    let admission = |site: &ResolvedCallSite| {
        site.invocation != InvocationKind::Construct
            && (site.target_node.is_none()
                || site.invocation == InvocationKind::Call
                    && replacements
                        .as_ref()
                        .and_then(|catalog| catalog.file(&site.file))
                        .is_some()
                    && known_callable_admission::needs_refinement(
                        facts,
                        indexes,
                        site,
                        #[cfg(feature = "test-instrumentation")]
                        &pass.parameter_cache.admission_probe,
                    ))
    };
    let pending = pass
        .sites
        .iter()
        .enumerate()
        .filter(|(_, site)| admission(site));
    #[cfg(feature = "test-instrumentation")]
    let pending = pending.inspect(|(_, site)| {
        pass.parameter_cache
            .admission_probe
            .admission(site.target_node.is_some())
    });
    let (mut admitted, mut base) = initial_sites(&pass.sites, pending.map(|(index, _)| index));
    let mut pending: FxHashSet<usize> = (0..base.len()).collect();
    let mut dependencies: FxHashMap<FunctionKey, FxHashSet<usize>> = fx_map();
    let mut previous_dependencies: FxHashMap<usize, FxHashSet<FunctionKey>> = fx_map();
    let mut visits: FxHashMap<usize, usize> = fx_map();
    loop {
        if pending.is_empty() {
            let offset = pass.sites.len();
            let added = expand(&pass.sites);
            if added.is_empty() {
                break;
            }
            pass.sites.extend(added);
            let (next, originals) = initial_sites(
                &pass.sites,
                (offset..pass.sites.len()).filter(|index| admission(&pass.sites[*index])),
            );
            let start = base.len();
            admitted.extend(next);
            base.extend(originals);
            pending.extend(start..base.len());
            continue;
        }
        let mut batch: Vec<_> = pending.drain().collect();
        batch.sort_unstable();
        for index in &batch {
            *visits.entry(*index).or_default() += 1;
        }
        // This projection belongs to one immutable incoming-call generation.
        let module_writer_phases = execution_phases::ModuleWriterPhases::default();
        let selected = selected_sites(&mut pass.sites, &batch, &admitted);
        let selected = selected.into_par_iter();
        #[cfg(all(test, feature = "test-instrumentation"))]
        let selected = selected.with_max_len(parameter_cache::scheduling::max_task_len(inputs.root));
        let results: Vec<_> = selected
            .map(|(index, site)| {
                let enabled = visits[&index] <= base.len() + 1;
                let previous = reset_resolution(site, &base[index]);
                let cache = callbacks.for_site(site, facts, indexes, &pass.parameter_cache);
                let used = refine_site_in_place(
                    inputs,
                    facts,
                    resolver,
                    indexes,
                    site,
                    ValueInputs {
                        module_writer_phases: &module_writer_phases,
                        cache,
                        incoming,
                        supers: &pass.super_constructors,
                        replacements: replacements.as_ref(),
                    },
                    enabled,
                );
                (index, previous, used)
            })
            .collect();
        let mut changed = fx_set();
        for (index, mut previous, used) in results {
            let site = &mut pass.sites[admitted[index]];
            for key in previous_dependencies.remove(&index).into_iter().flatten() {
                if let Some(users) = dependencies.get_mut(&key) {
                    users.remove(&index);
                }
            }
            for key in &used {
                dependencies.entry(key.clone()).or_default().insert(index);
            }
            previous_dependencies.insert(index, used);
            if previous.0 != site.target || previous.1 != site.target_node {
                swap_resolution(site, &mut previous);
                for key in function_keys(site) {
                    if let Some(arguments) = incoming.get_mut(&key) {
                        let previous = site_argument(site, indexes, facts);
                        if let Some(position) =
                            arguments.iter().position(|argument| argument == &previous)
                        {
                            arguments.remove(position);
                        }
                    }
                    changed.insert(key);
                }
                swap_resolution(site, &mut previous);
                for key in function_keys(site) {
                    incoming
                        .entry(key.clone())
                        .or_default()
                        .push(site_argument(site, indexes, facts));
                    changed.insert(key);
                }
            }
        }
        if !changed.is_empty() {
            changed = pass.parameter_cache.affected(changed);
            changed = callbacks.invalidate(&pass.parameter_cache, changed);
            pass.parameter_cache.clear();
        }
        for key in changed {
            if let Some(users) = dependencies.get(&key) {
                pending.extend(
                    users
                        .iter()
                        .copied()
                        .filter(|index| visits[index] <= base.len() + 1),
                );
            }
        }
    }
    #[cfg(feature = "test-instrumentation")]
    super::super::super::value_flow_metrics::record(visits.values().sum());
    let mut processed: Vec<_> = visits.into_keys().collect();
    processed.sort_unstable();
    replace_call_edges(
        &mut pass.edges,
        processed
            .iter()
            .map(|index| (&base[*index], &pass.sites[admitted[*index]])),
        |site| site_source(inputs, site),
    );
    #[cfg(all(test, feature = "test-instrumentation"))]
    pass.parameter_cache.site_storage.finish();
    #[cfg(feature = "test-instrumentation")]
    {
        let mut snapshot = replacements
            .as_ref()
            .map_or_else(ReplacementProbeSnapshot::default, |catalog| {
                catalog.snapshot()
            });
        pass.parameter_cache.admission_probe.capture(&mut snapshot);
        admission_mode::capture_edges(&mut snapshot, &pass.edges);
        replacement_incoming::capture(&mut snapshot, incoming, &pass.sites, indexes, facts);
        super::super::super::value_flow_metrics::record_replacements(snapshot);
        super::super::super::value_flow_metrics::record_callback_ownership_cache(
            callbacks.initialized(),
        );
    }
}
include!("worklist_edges.rs");
include!("site_ownership.rs");

include!("worklist_keys.rs");
include!("initial_sites.rs");
