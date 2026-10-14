use super::*;

#[cfg(feature = "test-instrumentation")]
#[path = "replacement_incoming.rs"]
mod replacement_incoming;

type FunctionKey = (std::path::PathBuf, CallableId);

pub(in super::super::super) fn refine_value_calls(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
    pass: &mut CallPass,
) {
    let base = pass.sites.clone();
    let mut incoming = incoming_inputs(&base, &pass.constructor_routes, indexes, facts);
    let replacements = replacement_catalog::ReplacementCatalog::prepare(
        inputs, facts, resolver, indexes, pass, &incoming,
    );
    let mut pending: FxHashSet<usize> = base
        .iter()
        .enumerate()
        .filter(|(_, site)| {
            site.invocation != InvocationKind::Construct
                && (site.target_node.is_none()
                    || site.invocation == InvocationKind::Call
                        && replacements
                            .as_ref()
                            .and_then(|catalog| catalog.file(&site.file))
                            .is_some())
        })
        .map(|(index, _)| index)
        .collect();
    let mut dependencies: FxHashMap<FunctionKey, FxHashSet<usize>> = fx_map();
    let mut previous_dependencies: FxHashMap<usize, FxHashSet<FunctionKey>> = fx_map();
    let mut visits: FxHashMap<usize, usize> = fx_map();
    while !pending.is_empty() {
        let mut batch: Vec<_> = pending.drain().collect();
        batch.sort_unstable();
        for index in &batch {
            *visits.entry(*index).or_default() += 1;
        }
        let results: Vec<_> = batch
            .par_iter()
            .map(|index| {
                let enabled = visits[index] <= base.len() + 1;
                let (mut sites, used) = refine_iteration(
                    inputs,
                    facts,
                    resolver,
                    indexes,
                    &base[*index..=*index],
                    ValueInputs {
                        cache: &pass.parameter_cache,
                        incoming: &incoming,
                        supers: &pass.super_constructors,
                        replacements: replacements.as_ref(),
                    },
                    enabled,
                );
                (*index, sites.remove(0), used)
            })
            .collect();
        let mut changed = fx_set();
        for (index, site, used) in results {
            for key in previous_dependencies.remove(&index).into_iter().flatten() {
                if let Some(users) = dependencies.get_mut(&key) {
                    users.remove(&index);
                }
            }
            for key in &used {
                dependencies.entry(key.clone()).or_default().insert(index);
            }
            previous_dependencies.insert(index, used);
            if pass.sites[index].target != site.target
                || pass.sites[index].target_node != site.target_node
            {
                for key in function_keys(&pass.sites[index]) {
                    if let Some(arguments) = incoming.get_mut(&key) {
                        let previous = site_argument(&pass.sites[index], indexes, facts);
                        if let Some(position) =
                            arguments.iter().position(|argument| argument == &previous)
                        {
                            arguments.remove(position);
                        }
                    }
                    changed.insert(key);
                }
                for key in function_keys(&site) {
                    incoming
                        .entry(key.clone())
                        .or_default()
                        .push(site_argument(&site, indexes, facts));
                    changed.insert(key);
                }
            }
            pass.sites[index] = site;
        }
        if !changed.is_empty() {
            changed = pass.parameter_cache.affected(changed);
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
    replace_call_edges(&mut pass.edges, &base, &pass.sites, &processed, |site| {
        site_source(inputs, site)
    });
    #[cfg(feature = "test-instrumentation")]
    {
        let mut snapshot = replacements
            .as_ref()
            .map_or_else(ReplacementProbeSnapshot::default, |catalog| {
                catalog.snapshot()
            });
        replacement_incoming::capture(&mut snapshot, &incoming, &pass.sites, indexes, facts);
        super::super::super::value_flow_metrics::record_replacements(snapshot);
    }
}
fn replace_call_edges(
    edges: &mut Vec<Edge>,
    base: &[ResolvedCallSite],
    sites: &[ResolvedCallSite],
    processed: &[usize],
    source: impl Fn(&ResolvedCallSite) -> NodeId,
) {
    // Initial edges contain one entry per physical occurrence (before the
    // canonical adjacency dedupe). Remove exactly that occurrence's old Call
    // edges; identical calls and implicit constructor routes must survive.
    let mut removed: FxHashMap<Edge, usize> = fx_map();
    for index in processed {
        let site = &base[*index];
        for node in site.target_nodes() {
            *removed
                .entry((source(site), node.clone(), EdgeKind::Call))
                .or_default() += 1;
        }
    }
    if !removed.is_empty() {
        edges.retain(|edge| {
            if let Some(count) = removed.get_mut(edge) {
                if *count > 0 {
                    *count -= 1;
                    return false;
                }
            }
            true
        });
    }
    for index in processed {
        let site = &sites[*index];
        for node in site.target_nodes() {
            edges.push((source(site), node.clone(), EdgeKind::Call));
        }
    }
}
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

#[cfg(test)]
#[path = "replacement_edges_tests.rs"]
mod tests;
