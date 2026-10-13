use super::*;

type FunctionKey = (std::path::PathBuf, CallableId);

pub(in super::super::super) fn refine_value_calls(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
    pass: &mut CallPass,
) {
    let base = pass.sites.clone();
    let mut incoming = incoming_inputs(&base, &pass.constructor_routes);
    let mut pending: FxHashSet<usize> = base
        .iter()
        .enumerate()
        .filter(|(_, site)| {
            site.target_node.is_none() && site.invocation != InvocationKind::Construct
        })
        .map(|(index, _)| index)
        .collect();
    let mut dependencies: FxHashMap<FunctionKey, FxHashSet<usize>> = fx_map();
    let mut previous_dependencies: FxHashMap<usize, FxHashSet<FunctionKey>> = fx_map();
    let mut edges: FxHashMap<usize, Vec<Edge>> = fx_map();
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
                let (mut sites, edges, used) = refine_iteration(
                    inputs,
                    facts,
                    resolver,
                    indexes,
                    &base[*index..=*index],
                    ValueInputs {
                        cache: &pass.parameter_cache,
                        incoming: &incoming,
                        supers: &pass.super_constructors,
                    },
                    enabled,
                );
                (*index, sites.remove(0), edges, used)
            })
            .collect();
        let mut changed = fx_set();
        for (index, site, resolved_edges, used) in results {
            for key in previous_dependencies.remove(&index).into_iter().flatten() {
                if let Some(users) = dependencies.get_mut(&key) {
                    users.remove(&index);
                }
            }
            for key in &used {
                dependencies.entry(key.clone()).or_default().insert(index);
            }
            previous_dependencies.insert(index, used);
            if pass.sites[index].target_node != site.target_node {
                if let Some(key) = function_key(&pass.sites[index]) {
                    if let Some(arguments) = incoming.get_mut(&key) {
                        arguments.retain(|argument| argument != &site_argument(&site));
                    }
                    changed.insert(key);
                }
                if let Some(key) = function_key(&site) {
                    incoming
                        .entry(key.clone())
                        .or_default()
                        .push(site_argument(&site));
                    changed.insert(key);
                }
            }
            pass.sites[index] = site;
            edges.insert(index, resolved_edges);
        }
        if !changed.is_empty() {
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
    let mut edges: Vec<_> = edges.into_iter().collect();
    edges.sort_unstable_by_key(|(index, _)| *index);
    for (_, edges) in edges {
        pass.edges.extend(edges);
    }
}
fn function_key(site: &ResolvedCallSite) -> Option<FunctionKey> {
    if site.invocation == InvocationKind::Membership {
        return None;
    }
    match &site.target_node {
        Some(NodeId::Symbol {
            file,
            callable_id: Some(id),
            ..
        }) => Some((file.to_path_buf(), *id)),
        _ => None,
    }
}
fn site_argument(site: &ResolvedCallSite) -> (std::path::PathBuf, u32, String) {
    (site.file.clone(), site.offset, site.source_callee.clone())
}
